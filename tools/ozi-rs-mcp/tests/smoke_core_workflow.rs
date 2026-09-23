//! E2E smoke gate: drives the REAL app through the core editing workflow.
//!
//! # One journey per Customer Journey
//!
//! The journeys here are named `smoke_cj<N>_<what>` after the entries in
//! `docs/customer-journeys.md`, so `just smoke cj3` runs one of them and the
//! naming says which part of the product a failure is about. Where a CJ has no
//! journey yet, this is what it is waiting for — written down rather than left
//! as an empty test that passes:
//!
//! | CJ | Journey | State |
//! |----|---------|-------|
//! | 1. Download maps before setting out | — | needs a fake catalogue server; the real one is a thousand pages and a live dependency |
//! | 2. Field start with no network | — | needs a launch with the link down and a cached bundle staged in a temp bundles root |
//! | 3. Collect a day's tracks | `smoke_cj3_layer_management` | layers only; the import half needs fixture `.gpx`/`.plt`/`.wpt`/`.zip` files and a file dialog the Mac2 driver can answer |
//! | 4. Clean up a track | — | needs a track in the project before the journey starts, which means loading a fixture `.ozp` at launch |
//! | 5. Plan the work | `smoke_cj5_draw_track` | drawing is covered end to end |
//! | 6. Hand the data out | — | blocked on the same file dialog as CJ-3's import half |
//! | 7. Do not lose the work | — | the close guard cannot be walked: dismissing a native quit dialog ends the Mac2 session |
//! | 8. Exchange projects between headquarters | — | needs two bundles root directories and a project file moved between them |
//!
//! The stand (`just stand`) walks all eight, and has found the defects in most
//! of them. It is not evidence for the desktop (ADR-0024); this file is, and
//! the gap above is the honest size of that evidence today.
//!
//! Scenario (every step asserts, nothing passes silently):
//!   1. Mac2 session launches the bundled app (`ru.lizaalert.ozi-rs`).
//!   2. Workspace renders (accessibility tree contains the Library tabs) —
//!      catches the "blank window" class of failure.
//!   3. Switch to the Tracks tab, enter drawing mode ("Create track") —
//!      `create_empty_track` IPC must succeed for the toggle to flip.
//!   4. Click 3 points on the map canvas, paced above the 220ms drawing
//!      debounce — each point is an `insert_track_point` IPC round-trip; the
//!      toggle label "Done (3 points)" is the end-to-end proof. This single
//!      assertion would have caught the BigInt-IPC bug that shipped silently.
//!   5. Esc cancels the draw (deletes the scratch track) so the user session
//!      is left unpolluted; the row must disappear.
//!
//! Preconditions (the test FAILS loudly when unmet — no silent skips; the
//! gate exists precisely because silent passes hid broken workflows):
//!   - `just build` artifact at target/debug/bundle/macos/ozi-rs.app
//!   - Appium server with the Mac2 driver at 127.0.0.1:4723
//!
//! Run via `just smoke` (the test is `#[ignore]`d so plain `cargo test`
//! stays GUI-free).

use std::{path::Path, process::Command, thread, time::Duration};

use ozi_rs_mcp::appium::{
    DEFAULT_APPIUM_SERVER_URL, META, SHIFT, appium_click_element_offsets_with_session_id,
    appium_click_with_session_id, appium_doctor, appium_launch_session_with_app_path,
    appium_page_source_with_session_id, appium_press_chord_with_session_id,
    appium_press_key_with_session_id, appium_stop_session_with_session_id,
};

const ESCAPE: char = '\u{E00C}';
const SOURCE_POLL_STEP: Duration = Duration::from_millis(500);
/// Pace map clicks above MapView's 220ms drawing debounce, which silently
/// swallows faster clicks (known frontend defect, tracked for Milestone 2).
const CLICK_PACING: Duration = Duration::from_millis(600);

/// Stops the WebDriver session and force-quits the app on every exit path —
/// the project verification protocol (CLAUDE.md) mandates that no session and
/// no app instance survive a QA turn, pass or fail.
struct SessionGuard {
    server_url: String,
    session_id: String,
}

impl Drop for SessionGuard {
    fn drop(&mut self) {
        let _ = appium_stop_session_with_session_id(&self.server_url, &self.session_id);
        kill_app_instances();
    }
}

fn kill_app_instances() {
    let _ = Command::new("pkill")
        .args(["-f", "ozi-rs.app/Contents/MacOS"])
        .status();
}

/// An abandoned `createSession` (client timeout, killed test run) wedges the
/// WebDriverAgent host: every subsequent session attempt then fails with
/// "Failed to launch" or hangs. Killing the runner is safe — Appium respawns
/// it on the next session, in seconds when the xcodebuild cache is warm.
fn kill_wedged_wda() {
    for pattern in ["WebDriverAgentRunner", "xcodebuild.*WebDriverAgentMac"] {
        let _ = Command::new("pkill").args(["-f", pattern]).status();
    }
}

/// The app opens in Russian and the language is switchable, so every label
/// this smoke waits for is matched in either language. The run broke when the
/// default became Russian: the journey was fine, the test was reading English.
const TRACKS_TAB: [&str; 2] = ["Треки", "Tracks"];
const WAYPOINTS_TAB: [&str; 2] = ["Точки", "Waypoints"];
const DRAW_TRACK: [&str; 2] = ["Нарисовать трек", "Draw a track"];
/// `shell.mapCanvas`. This one was left English-only when the rest of the
/// file was made bilingual, so the three map clicks failed with "no such
/// element" the first time the app under test happened to open in Russian —
/// which is its default. Both spellings, like everything else here.
const MAP_CANVAS: [&str; 2] = ["Холст карты", "Map canvas"];
const SCRATCH_TRACK_NAME: &str = "New Track";
/// `layers.menu`, `layers.new`, `layers.create`, `layers.delete` and the
/// default name a new track layer is given. Held against the dictionaries by
/// `src/test/smoke-label-contract.test.ts`.
const LAYER_MENU: [&str; 2] = ["Действия со слоем", "Layer actions"];
const LAYER_NEW: [&str; 2] = ["Новый слой…", "New layer…"];
const LAYER_CREATE: [&str; 2] = ["Создать", "Create"];
const LAYER_DELETE: [&str; 2] = ["Удалить слой", "Delete layer"];
/// The layer select's accessible name carries which layer is active.
///
/// It used to be the bare label «Слой треков», and the current value lived in
/// a span that WKWebView does not publish — so no assertion about *which*
/// layer is active was possible from outside. Saying what is selected is also
/// what an accessible name is for: a screen reader announcing "track layer"
/// and not which one is announcing half the control.
const LAYER_SELECT_WITH_NEW: [&str; 2] = [
    "Слой треков: Новый слой треков",
    "Track layer: New track layer",
];

fn contains_any(source: &str, needles: &[&str]) -> bool {
    needles.iter().any(|needle| source.contains(needle))
}

/// The drawing-mode toggle with its point count.
///
/// WKWebView publishes the button's `aria-label` and not the text inside it,
/// so this matches the label, not the visible "Готово (N)". Matching the
/// visible text is what silently broke this smoke once.
///
/// These strings are `tracksTab.finishTrack` in `src/lib/i18n.ts`, and a
/// change to either side breaks the gate — as it did on 2026-09-22, when the
/// label lost its "(N точек)" shape to get the Russian plural right and this
/// file went on looking for the old one. `src/test/smoke-label-contract.test.ts`
/// now fails first, and in seconds rather than after a build and a launch.
fn shows_point_count(source: &str, count: usize) -> bool {
    source.contains(&format!("Завершить трек · точек: {count}"))
        || source.contains(&format!("Finish the track · points: {count}"))
}

fn shows_any_point_count(source: &str) -> bool {
    source.contains("Завершить трек · точек:") || source.contains("Finish the track · points:")
}

fn poll_source_until<F: Fn(&str) -> bool>(
    server_url: &str,
    session_id: &str,
    timeout: Duration,
    what: &str,
    predicate: F,
) -> String {
    let deadline = std::time::Instant::now() + timeout;
    let mut last_source = String::new();
    let mut last_error = String::new();
    loop {
        match appium_page_source_with_session_id(server_url, session_id) {
            Ok(source) => {
                if predicate(&source) {
                    return source;
                }
                last_source = source;
            }
            Err(result) => {
                last_error = format!("{result:?}");
            }
        }
        if std::time::Instant::now() >= deadline {
            let excerpt: String = last_source.chars().take(2000).collect();
            panic!(
                "timed out waiting for: {what}\nlast page-source excerpt:\n{excerpt}\nlast error: {last_error}"
            );
        }
        thread::sleep(SOURCE_POLL_STEP);
    }
}

/// Click the first selector that works, answering whether any did.
///
/// WKWebView exposes web content with varying roles, and a control's text
/// lands in `title` with an empty `label` — so every lookup tries both, and a
/// caller that cares which one worked gets it back.
fn click_any_label(server: &str, sid: &str, labels: &[&str]) -> bool {
    for label in labels {
        let selector = format!("//*[@title=\"{label}\" or @label=\"{label}\"]");
        if appium_click_with_session_id(server, sid, Some(&selector)).ok {
            return true;
        }
    }
    false
}

/// A second journey through the packaged application: making, and then
/// unmaking, a layer.
///
/// Everything built between 2026-09-22 and 2026-09-23 — layer management,
/// the waypoint reader, a new search, drag and drop, the names on the map —
/// had been walked on the stand and never in the packaged application, where
/// the IPC is real. The stand proves how a screen behaves given an answer;
/// only this proves the answer comes back.
///
/// Layers are the one of those that needs no typing and no file dialog: the
/// name field arrives filled in, so the whole journey is clicks. That matters
/// because typing into the webview through Mac2 has never been made to work
/// (`appium_type_text` reached a keyboard shortcut instead of the field).
#[test]
#[ignore = "requires built app + Appium Mac2 server; run via `just smoke`"]
fn smoke_cj3_layer_management() {
    let workspace = Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .expect("workspace root");
    let app_bundle = workspace.join("target/debug/bundle/macos/ozi-rs.app");
    assert!(
        app_bundle.exists(),
        "app bundle missing at {} — run `just build` first",
        app_bundle.display(),
    );

    let doctor = appium_doctor();
    assert!(doctor.ok, "Appium Mac2 is not ready: {doctor:?}");

    kill_wedged_wda();
    kill_app_instances();
    thread::sleep(Duration::from_secs(2));

    let launch = appium_launch_session_with_app_path(
        true,
        DEFAULT_APPIUM_SERVER_URL,
        &app_bundle.to_string_lossy(),
    );
    assert!(launch.ok, "appium_launch_session failed: {launch:?}");
    let guard = SessionGuard {
        server_url: DEFAULT_APPIUM_SERVER_URL.to_owned(),
        session_id: launch.session_id.clone().expect("session id"),
    };
    let server = guard.server_url.as_str();
    let sid = guard.session_id.as_str();

    poll_source_until(
        server,
        sid,
        Duration::from_secs(20),
        "workspace tabs",
        |s| contains_any(s, &TRACKS_TAB) && contains_any(s, &WAYPOINTS_TAB),
    );

    assert!(
        click_any_label(server, sid, &TRACKS_TAB),
        "the Tracks tab SHALL be reachable"
    );
    poll_source_until(
        server,
        sid,
        Duration::from_secs(10),
        "the Tracks tab's controls",
        |s| contains_any(s, &LAYER_MENU),
    );
    println!("ok: Tracks tab open with the layer menu on it");

    // A layer, made from the menu. The name arrives filled in, so this is
    // the whole journey: menu → New layer… → Create.
    assert!(
        click_any_label(server, sid, &LAYER_MENU),
        "the layer menu SHALL open"
    );
    poll_source_until(
        server,
        sid,
        Duration::from_secs(5),
        "the layer menu's entries",
        |s| contains_any(s, &LAYER_NEW),
    );
    assert!(
        click_any_label(server, sid, &LAYER_NEW),
        "New layer… SHALL be clickable"
    );
    poll_source_until(
        server,
        sid,
        Duration::from_secs(5),
        "the name field, filled in",
        |s| contains_any(s, &LAYER_CREATE),
    );
    assert!(
        click_any_label(server, sid, &LAYER_CREATE),
        "Create SHALL be clickable"
    );

    // `create_track_layer` over real IPC, and the answer used to make the new
    // layer active — which is the whole reason the command returns an id.
    poll_source_until(
        server,
        sid,
        Duration::from_secs(10),
        "the new layer, active",
        |s| contains_any(s, &LAYER_SELECT_WITH_NEW),
    );
    println!("ok: layer created and made active (create_track_layer IPC works)");

    // And unmade, so the packaged application is left as it was found.
    assert!(
        click_any_label(server, sid, &LAYER_MENU),
        "the layer menu SHALL open again"
    );
    poll_source_until(
        server,
        sid,
        Duration::from_secs(5),
        "the layer menu's entries",
        |s| contains_any(s, &LAYER_DELETE),
    );
    assert!(
        click_any_label(server, sid, &LAYER_DELETE),
        "Delete layer SHALL be clickable"
    );
    poll_source_until(
        server,
        sid,
        Duration::from_secs(10),
        "the layer gone and another one active",
        |s| !contains_any(s, &LAYER_SELECT_WITH_NEW),
    );
    println!("ok: layer deleted, another made active (delete_track_layer IPC works)");
}

#[test]
#[ignore = "requires built app + Appium Mac2 server; run via `just smoke`"]
fn smoke_cj5_draw_track() {
    let workspace = Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .expect("workspace root");

    // Precondition: bundled app exists.
    let app_bundle = workspace.join("target/debug/bundle/macos/ozi-rs.app");
    assert!(
        app_bundle.exists(),
        "app bundle missing at {} — run `just build` first",
        app_bundle.display(),
    );

    // Precondition: Appium + Mac2 ready.
    let doctor = appium_doctor();
    assert!(
        doctor.ok,
        "Appium Mac2 is not ready ({}). Start it with: appium --address 127.0.0.1 --port 4723",
        doctor.message.as_deref().unwrap_or("no diagnostic"),
    );

    // Launch by explicit appPath: the DEBUG bundle is not registered with
    // LaunchServices, so a bundleId launch hangs inside XCUITest. Clear any
    // stale instance first — Mac2 owns the app lifecycle for this session.
    // Recycle the WebDriverAgent host proactively: a host left over from a
    // previous session reliably rejects the next createSession (observed:
    // HTTP 500 after ~3 min), while a fresh one comes up in seconds when the
    // xcodebuild cache is warm.
    kill_wedged_wda();
    kill_app_instances();
    thread::sleep(Duration::from_secs(2));

    let mut launch = appium_launch_session_with_app_path(
        true,
        DEFAULT_APPIUM_SERVER_URL,
        &app_bundle.to_string_lossy(),
    );
    if !launch.ok {
        println!(
            "session creation failed ({:?}) — recycling a possibly wedged WebDriverAgent and retrying once",
            launch.error_kind,
        );
        kill_wedged_wda();
        kill_app_instances();
        thread::sleep(Duration::from_secs(3));
        launch = appium_launch_session_with_app_path(
            true,
            DEFAULT_APPIUM_SERVER_URL,
            &app_bundle.to_string_lossy(),
        );
    }
    assert!(launch.ok, "appium_launch_session failed twice: {launch:?}");
    let session_id = launch.session_id.clone().expect("session id on ok result");
    let guard = SessionGuard {
        server_url: DEFAULT_APPIUM_SERVER_URL.to_owned(),
        session_id: session_id.clone(),
    };
    let server = guard.server_url.as_str();
    let sid = guard.session_id.as_str();

    // 1. Workspace rendered: Library tabs present in the accessibility tree.
    poll_source_until(
        server,
        sid,
        Duration::from_secs(20),
        "workspace tabs",
        |s| contains_any(s, &TRACKS_TAB) && contains_any(s, &WAYPOINTS_TAB),
    );
    println!("ok: workspace rendered (tabs visible)");

    // 2. Switch to the Tracks tab. WKWebView exposes web content with varying
    //    element roles, so try role-specific XPath first, generic label last.
    // WKWebView maps web tab roles to XCUIElementTypeTab with the text in
    // `title` and an EMPTY `label` — match both attributes, title first.
    let tab_selectors = [
        "//XCUIElementTypeTab[@title=\"Треки\"]",
        "//XCUIElementTypeTab[@title=\"Tracks\"]",
        "//*[@title=\"Треки\" or @label=\"Треки\"]",
        "//*[@title=\"Tracks\" or @label=\"Tracks\"]",
    ];
    let mut tracks_tab_open = false;
    for selector in tab_selectors {
        let click = appium_click_with_session_id(server, sid, Some(selector));
        if !click.ok {
            continue;
        }
        let deadline = std::time::Instant::now() + Duration::from_secs(3);
        while std::time::Instant::now() < deadline {
            if appium_page_source_with_session_id(server, sid)
                .map(|s| contains_any(&s, &DRAW_TRACK) || shows_any_point_count(&s))
                .unwrap_or(false)
            {
                tracks_tab_open = true;
                break;
            }
            thread::sleep(SOURCE_POLL_STEP);
        }
        if tracks_tab_open {
            println!("ok: Tracks tab opened via {selector}");
            break;
        }
    }
    if !tracks_tab_open {
        // Dump the accessibility tree so the failure is diagnosable offline.
        let source = appium_page_source_with_session_id(server, sid).unwrap_or_default();
        let dump_dir = workspace.join(".sisyphus/evidence/native-qa/smoke_core_workflow");
        let _ = std::fs::create_dir_all(&dump_dir);
        let dump = dump_dir.join("last-source.xml");
        let _ = std::fs::write(&dump, &source);
        let tab_nodes: Vec<&str> = source
            .lines()
            .filter(|line| contains_any(line, &TRACKS_TAB))
            .map(str::trim)
            .collect();
        panic!(
            "could not open the Tracks tab: no selector produced the draw-track toggle.\n\
             full AX tree dumped to {}\nnodes containing \"Tracks\":\n{}",
            dump.display(),
            tab_nodes.join("\n"),
        );
    }

    // 3. Enter drawing mode. The toggle flips to "Done (0 points)" only after
    //    the create_empty_track IPC round-trip succeeds.
    let create = appium_click_with_session_id(
        server,
        sid,
        Some(
            "//*[@title=\"Нарисовать трек\" or @label=\"Нарисовать трек\" \
             or @title=\"Draw a track\" or @label=\"Draw a track\"]",
        ),
    );
    assert!(
        create.ok,
        "clicking the draw-track toggle failed: {create:?}"
    );
    poll_source_until(
        server,
        sid,
        Duration::from_secs(10),
        "drawing mode entered (point count 0)",
        |s| shows_point_count(s, 0),
    );
    println!("ok: drawing mode entered (create_empty_track IPC works)");

    // 4. Three paced clicks on the map canvas = three insert_track_point IPC
    //    round-trips. The toggle label counts only points the backend accepted.
    let map_selector = format!(
        "//*[{}]",
        MAP_CANVAS
            .iter()
            .map(|name| format!("@title=\"{name}\" or @label=\"{name}\""))
            .collect::<Vec<_>>()
            .join(" or "),
    );
    for offset in [(-120, -70), (10, 10), (110, 80)] {
        let click =
            appium_click_element_offsets_with_session_id(server, sid, &map_selector, &[offset]);
        assert!(click.ok, "map click at {offset:?} failed: {click:?}");
        thread::sleep(CLICK_PACING);
    }
    poll_source_until(
        server,
        sid,
        Duration::from_secs(10),
        "3 points registered (point count 3)",
        |s| shows_point_count(s, 3),
    );
    println!("ok: 3 track points inserted (insert_track_point IPC works end-to-end)");

    // 5. Esc cancels the draw and deletes the scratch track — leaves the
    //    user's session unpolluted and exercises delete_track IPC.
    let esc = appium_press_key_with_session_id(server, sid, ESCAPE);
    assert!(esc.ok, "pressing Escape failed: {esc:?}");
    poll_source_until(
        server,
        sid,
        Duration::from_secs(10),
        "drawing cancelled (toggle back to draw-track, scratch row gone)",
        |s| {
            contains_any(s, &DRAW_TRACK)
                && !shows_any_point_count(s)
                && !s.contains(SCRATCH_TRACK_NAME)
        },
    );
    println!("ok: draw cancelled, scratch track removed (delete_track IPC works)");

    drop(guard);
    println!("\n=== smoke_core_workflow: PASSED ===");
}

/// The moment captured into a folder, walked in the packaged application.
///
/// This is the one thing about the report that no other test can reach. The
/// unit tests cover the folder's contents and the rectangle arithmetic; the
/// stand covers the toast and the note box. Neither has a window, and a report
/// whose whole point is a picture of the window is not proved without one.
///
/// It asserts the folder and the three files that need no permission, and
/// only *reports* on `screenshot.png`, because whether macOS has been given
/// Screen Recording for this bundle is a fact about the machine and not about
/// the code. The application is built to survive that refusal — the folder
/// still appears — and this test is built the same way, so a missing grant
/// does not turn the gate red for something no commit can fix.
///
/// The folder it makes is removed afterwards: the operator's reports are
/// theirs, and a test's leftovers in that directory are noise in the thing
/// they are meant to hand over.
#[test]
#[ignore = "requires built app + Appium Mac2 server; run via `just smoke`"]
fn smoke_report_capture_moment() {
    let workspace = Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .expect("workspace root");
    let app_bundle = workspace.join("target/debug/bundle/macos/ozi-rs.app");
    assert!(
        app_bundle.exists(),
        "app bundle missing at {} — run `just build` first",
        app_bundle.display(),
    );

    let reports_root = std::path::PathBuf::from(std::env::var("HOME").expect("HOME"))
        .join("Documents")
        .join("ozi-rs-отчёты");
    let before = report_folders(&reports_root);

    let doctor = appium_doctor();
    assert!(doctor.ok, "Appium Mac2 is not ready: {doctor:?}");

    kill_wedged_wda();
    kill_app_instances();
    thread::sleep(Duration::from_secs(2));

    let launch = appium_launch_session_with_app_path(
        true,
        DEFAULT_APPIUM_SERVER_URL,
        &app_bundle.to_string_lossy(),
    );
    assert!(launch.ok, "appium_launch_session failed: {launch:?}");
    let guard = SessionGuard {
        server_url: DEFAULT_APPIUM_SERVER_URL.to_owned(),
        session_id: launch.session_id.clone().expect("session id"),
    };
    let server = guard.server_url.as_str();
    let sid = guard.session_id.as_str();

    // The window has to be there before it is worth photographing.
    poll_source_until(
        server,
        sid,
        Duration::from_secs(20),
        "workspace tabs",
        |s| contains_any(s, &TRACKS_TAB) && contains_any(s, &WAYPOINTS_TAB),
    );
    println!("ok: the workspace is up");

    let chord = appium_press_chord_with_session_id(server, sid, &[SHIFT, META], 'd');
    assert!(chord.ok, "pressing Shift+Cmd+D failed: {chord:?}");

    // `screencapture` is a second process, and on a first run macOS may be
    // deciding about a permission, so this waits rather than looking once.
    let folder = poll_for_new_report(&reports_root, &before, Duration::from_secs(25));
    let folder = folder.unwrap_or_else(|| {
        panic!(
            "no new folder appeared under {} within 25s — Shift+Cmd+D did not reach \
             the application, or save_report failed. The toast text is in the app log.",
            reports_root.display()
        )
    });
    println!("ok: the folder appeared at {}", folder.display());

    for name in ["diagnostics.txt", "state.json", "about.txt"] {
        let file = folder.join(name);
        let body = std::fs::read_to_string(&file)
            .unwrap_or_else(|e| panic!("{} is not readable: {e}", file.display()));
        assert!(!body.trim().is_empty(), "{} is empty", file.display());
    }

    // Not just present — the right file. A folder of four empty files would
    // pass a presence check and tell a reader nothing.
    let diagnostics = std::fs::read_to_string(folder.join("diagnostics.txt")).expect("read");
    assert!(
        diagnostics.contains("Последние сообщения приложения"),
        "diagnostics.txt is not the diagnostics file: {diagnostics}"
    );
    let state = std::fs::read_to_string(folder.join("state.json")).expect("read");
    assert!(
        state.contains("\"project_name\"") && state.contains("\"diagnostics\""),
        "state.json is not the application state: {state}"
    );
    let about = std::fs::read_to_string(folder.join("about.txt")).expect("read");
    assert!(
        about.contains("ozi-rs ") && about.contains("macos"),
        "about.txt does not name the build and the machine: {about}"
    );
    println!("ok: diagnostics, state and build are all in it and all say what they should");

    // The picture, reported and not required — see the doc comment.
    let shot = folder.join("screenshot.png");
    match std::fs::metadata(&shot) {
        Ok(meta) if meta.len() > 0 => {
            // Since the preflight went in, a file here means the system said
            // yes — the wallpaper-instead-of-window case writes nothing.
            println!("ok: screenshot.png is there, {} bytes", meta.len());
        }
        Ok(_) => println!(
            "NOTE: screenshot.png is empty — screencapture ran and wrote nothing. \
             Check Screen Recording for ozi-rs in System Settings › Privacy."
        ),
        Err(_) => println!(
            "NOTE: no screenshot.png — macOS has not been given Screen Recording for \
             this bundle, and the application declined to write the wallpaper picture \
             it would otherwise have got. The report is complete otherwise, which is \
             the designed behaviour; allow it in System Settings › Privacy › Screen \
             Recording and restart ozi-rs to get the picture too."
        ),
    }

    // Kept when asked, because a run that produced the wrong picture looks
    // exactly like a run that produced the right one from here: the assertions
    // above cannot tell a window from a wallpaper, only a human eye can, and
    // it needs the file to still be there.
    if std::env::var_os("OZI_KEEP_REPORT").is_some() {
        println!("kept for inspection: {}", folder.display());
    } else {
        let _ = std::fs::remove_dir_all(&folder);
    }

    drop(guard);
    println!("\n=== smoke_report_capture_moment: PASSED ===");
}

/// The report folders that exist right now, as a sorted list of paths.
fn report_folders(root: &Path) -> Vec<std::path::PathBuf> {
    let Ok(entries) = std::fs::read_dir(root) else {
        return Vec::new();
    };
    let mut found: Vec<std::path::PathBuf> = entries
        .filter_map(Result::ok)
        .map(|e| e.path())
        .filter(|p| p.is_dir())
        .collect();
    found.sort();
    found
}

/// Wait for a folder that was not there before, answering it once it is.
fn poll_for_new_report(
    root: &Path,
    before: &[std::path::PathBuf],
    timeout: Duration,
) -> Option<std::path::PathBuf> {
    let deadline = std::time::Instant::now() + timeout;
    loop {
        if let Some(fresh) = report_folders(root)
            .into_iter()
            .find(|p| !before.contains(p))
        {
            return Some(fresh);
        }
        if std::time::Instant::now() >= deadline {
            return None;
        }
        thread::sleep(SOURCE_POLL_STEP);
    }
}
