//! E2E smoke gate: drives the REAL app through the core editing workflow.
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
    DEFAULT_APPIUM_SERVER_URL, appium_click_element_offsets_with_session_id,
    appium_click_with_session_id, appium_doctor, appium_launch_session_with_app_path,
    appium_page_source_with_session_id, appium_press_key_with_session_id,
    appium_stop_session_with_session_id,
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

#[test]
#[ignore = "requires built app + Appium Mac2 server; run via `just smoke`"]
fn smoke_core_workflow_draw_track() {
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
    poll_source_until(server, sid, Duration::from_secs(20), "workspace tabs", |s| {
        s.contains("Tracks") && s.contains("Waypoints")
    });
    println!("ok: workspace rendered (tabs visible)");

    // 2. Switch to the Tracks tab. WKWebView exposes web content with varying
    //    element roles, so try role-specific XPath first, generic label last.
    // WKWebView maps web tab roles to XCUIElementTypeTab with the text in
    // `title` and an EMPTY `label` — match both attributes, title first.
    let tab_selectors = [
        "//XCUIElementTypeTab[@title=\"Tracks\"]",
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
                .map(|s| s.contains("Create track") || s.contains("Done ("))
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
            .filter(|line| line.contains("Tracks"))
            .map(str::trim)
            .collect();
        panic!(
            "could not open the Tracks tab: no selector produced the \"Create track\" toggle.\n\
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
        Some("//*[@title=\"Create track\" or @label=\"Create track\"]"),
    );
    assert!(create.ok, "clicking \"Create track\" failed: {create:?}");
    poll_source_until(
        server,
        sid,
        Duration::from_secs(10),
        "drawing mode entered (Done (0 points))",
        |s| s.contains("Done (0 points)"),
    );
    println!("ok: drawing mode entered (create_empty_track IPC works)");

    // 4. Three paced clicks on the map canvas = three insert_track_point IPC
    //    round-trips. The toggle label counts only points the backend accepted.
    let map_selector = "//*[@title=\"Map canvas\" or @label=\"Map canvas\"]";
    for offset in [(-120, -70), (10, 10), (110, 80)] {
        let click =
            appium_click_element_offsets_with_session_id(server, sid, map_selector, &[offset]);
        assert!(click.ok, "map click at {offset:?} failed: {click:?}");
        thread::sleep(CLICK_PACING);
    }
    poll_source_until(
        server,
        sid,
        Duration::from_secs(10),
        "3 points registered (Done (3 points))",
        |s| s.contains("Done (3 points)"),
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
        "drawing cancelled (toggle back to Create track, scratch row gone)",
        |s| s.contains("Create track") && !s.contains("Done (") && !s.contains("New Track"),
    );
    println!("ok: draw cancelled, scratch track removed (delete_track IPC works)");

    drop(guard);
    println!("\n=== smoke_core_workflow: PASSED ===");
}
