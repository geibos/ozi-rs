# Smoke: GPX track import

Source: ADR-0020 (MVP scope), section Tracks — "import GPX file."

## Preconditions

- App built and available at `target/debug/bundle/macos/ozi-rs.app`
- App launched indirectly via `appium_launch_session` (Mac2 driver)
- Fixture: `/Users/sobieg/Projects/ozi-rs/example_data/2018-09-26_Nizovskaya/8-Android&iOS/2018-09-26_Nizovskaya_500m.gpx` (79 track points)
- Active bundle with maps already loaded (from prior Task 4 smoke or pre-launch)

## UI entry point

- Selector candidate: `//XCUIElementTypeButton[@title="Import GPX"]` (visible in
  the AX tree as a sidebar button under the `TRACKS` section).
- Notes: Import GPX button reaches the user via the left sidebar of the main
  ozi-rs window, below "Track layer" popup. Confirmed present in AX tree on 2026-05-01.

## Steps and expected outcomes

1. **Action**: `appium_launch_session` (fresh session)
   **Expected**: Mac2 WebDriver session created; app started for `ru.lizaalert.ozi-rs`.
   **Artifact**: Session ID recorded.

2. **Action**: `appium_screenshot` (baseline before any click)
   **Expected**: Screenshot captures the running app with Tracks panel visible on sidebar.
   **Artifact**: `.sisyphus/evidence/native-qa/appium_screenshot/screenshot_baseline.png`

3. **Action**: Query AX tree via `GET /session/{sid}/source` to inspect Tracks section
   **Expected**: AX tree contains "TRACKS" section with buttons: "Import GPX", "Import PLT", "Create Track", etc.
   **Artifact**: AX tree dump for later inspection (initial state).

4. **Action**: `appium_click` on `//XCUIElementTypeButton[@title="Import GPX"]`
   to trigger the file open dialog.
   **Expected**: File dialog appears within 1 s, allowing navigation to GPX files.
   **Artifact**: n/a (dialog state captured in next screenshot).

5. **Action**: Navigate file dialog to `/Users/sobieg/Projects/ozi-rs/example_data/2018-09-26_Nizovskaya/8-Android&iOS/`
   and select `2018-09-26_Nizovskaya_500m.gpx`.
   **Expected**: File is selected; "Open" or "Import" button becomes active.
   **Artifact**: `appium_screenshot` showing file selection.

6. **Action**: `appium_click` on "Open" button to confirm import.
   **Expected**: Dialog closes, import begins, AX tree updates within 2 s.
   **Artifact**: `appium_screenshot` after import starts.

7. **Action**: Wait ~2 s for import to complete, then query AX tree
   **Expected**: Tracks panel now shows a new track row with name (likely
   `2018-09-26_Nizovskaya_500m` or similar), visible in AX tree as a group or
   button with the track name and visibility toggle (eye icon).
   **Artifact**: AX tree dump showing new track row.

8. **Action**: `appium_screenshot` after import completed
   **Expected**: Visual confirmation that track appears in Tracks panel.
   **Artifact**: `.sisyphus/evidence/native-qa/appium_screenshot/screenshot_after_import.png`

9. **Action**: `capture_logs` and search for "gpx" or "import" markers
   **Expected**: Log lines showing import start/completion, no errors.
   **Artifact**: `.sisyphus/evidence/native-qa/capture_logs/stdout.txt`

## Classification

- [ ] **works** — track row appears in Tracks panel + visible on map polyline
- [ ] **partial** — only AX tree shows track row, visual rendering unclear
- [ ] **broken** — import fails silently or errors visible in logs
- [ ] **hidden**
- [ ] **missing**

**Expected classification: `works` (P2).**

Rationale: GPX import is a core MVP feature. The file picker and import handler
are well-established in prior codebase versions. Success criteria: (1) track row
appears in Tracks panel, (2) polyline visible on map, (3) no import errors in logs.

## Evidence

- Appium session ID
- Fixture GPX: `/Users/sobieg/Projects/ozi-rs/example_data/2018-09-26_Nizovskaya/8-Android&iOS/2018-09-26_Nizovskaya_500m.gpx`
- Baseline screenshot: `.sisyphus/evidence/native-qa/appium_screenshot/screenshot_baseline.png`
- File selection screenshot: `.sisyphus/evidence/native-qa/appium_screenshot/screenshot_file_select.png`
- Post-import screenshot: `.sisyphus/evidence/native-qa/appium_screenshot/screenshot_after_import.png`
- AX tree (initial): `.sisyphus/evidence/native-qa/appium/ax_tree_initial.json`
- AX tree (post-import): `.sisyphus/evidence/native-qa/appium/ax_tree_post_import.json`
- Logs: `.sisyphus/evidence/native-qa/capture_logs/stdout.txt`

## Known failure modes

- **File dialog does not appear**: F8 click fix confirmed in commit 191fd39; if click succeeds but dialog doesn't open, check Tauri file picker handler in `commands/mod.rs` and ensure `allow:file-open` permission is set.
- **Import silently fails (no track row)**: Check logs for parse errors. GPX format may be malformed. Run `cargo test --manifest-path src-tauri/Cargo.toml import_gpx` to validate parser.
- **Track row appears but no polyline on map**: Map layer visibility may be toggled off. Check Tracks panel eye icon; click to toggle if needed.
- **Session terminates after click**: Mac2 driver may close session on error. Check Appium logs and retry with fresh launch.

---

## References

- ADR-0020: MVP scope, Tracks section
- `docs/commands-reference.md`: `ImportTrack` command documentation
- `src-tauri/src/infrastructure/formats.rs`: GPX parser implementation
