# Smoke: PLT track import

Source: ADR-0020 (MVP scope), section Tracks — "import PLT file."

## Preconditions

- App built and available at `target/debug/bundle/macos/ozi-rs.app`
- App launched indirectly via `appium_launch_session` (Mac2 driver)
- Fixture PLT files available:
  - Primary: `/Users/sobieg/Projects/ozi-rs/example_data/2018-09-26_Nizovskaya/_unpacked/topo_ozf2/5-Ozi(Win&Android)_Topo_EEKO/Data/2018-09-26_Nizovskaya_500m.plt` (79 track points)
  - Alternative: `/Users/sobieg/Projects/ozi-rs/example_data/2018-09-26_Nizovskaya/_unpacked/satell_ozf2/6-Ozi(Win&Android)_Satell/Data/2018-09-26_Nizovskaya_500m.plt`
- Active bundle with maps already loaded (from prior Task 4 smoke or pre-launch)

## UI entry point

- Selector candidate: `//XCUIElementTypeButton[@title="Import PLT"]` (visible in
  the AX tree as a sidebar button under the `TRACKS` section).
- Notes: Import PLT button reaches the user via the left sidebar of the main
  ozi-rs window, below "Import GPX" button. Confirmed present in AX tree on 2026-05-01.

## Steps and expected outcomes

1. **Action**: Use existing session from Task 5a (GPX import) or launch fresh via `appium_launch_session`
   **Expected**: Session active; app running with Tracks panel visible.
   **Artifact**: Session ID.

2. **Action**: `appium_screenshot` (baseline before PLT click)
   **Expected**: Screenshot captures the running app; Tracks panel shows any previously imported tracks.
   **Artifact**: `.sisyphus/evidence/native-qa/appium_screenshot/screenshot_plt_baseline.png`

3. **Action**: Query AX tree to confirm "Import PLT" button is visible and accessible
   **Expected**: AX tree contains button with `@title="Import PLT"` in TRACKS section.
   **Artifact**: AX tree dump (baseline).

4. **Action**: `appium_click` on `//XCUIElementTypeButton[@title="Import PLT"]`
   to trigger the file open dialog.
   **Expected**: File dialog appears within 1 s.
   **Artifact**: n/a (captured in next screenshot).

5. **Action**: Navigate file dialog to `/Users/sobieg/Projects/ozi-rs/example_data/2018-09-26_Nizovskaya/_unpacked/topo_ozf2/5-Ozi(Win&Android)_Topo_EEKO/Data/`
   and select `2018-09-26_Nizovskaya_500m.plt`.
   **Expected**: File is selected; "Open" or "Import" button becomes active.
   **Artifact**: `appium_screenshot` showing file selection.

6. **Action**: `appium_click` on "Open" button to confirm import.
   **Expected**: Dialog closes, import begins, AX tree updates within 2 s.
   **Artifact**: `appium_screenshot` after import starts.

7. **Action**: Wait ~2 s for import to complete, then query AX tree
   **Expected**: Tracks panel shows a new track row for the PLT file (name likely
   `2018-09-26_Nizovskaya_500m`), distinct from or in addition to any GPX track
   from Task 5a.
   **Artifact**: AX tree dump showing new or updated track row.

8. **Action**: `appium_screenshot` after import completed
   **Expected**: Visual confirmation that PLT track appears in Tracks panel.
   **Artifact**: `.sisyphus/evidence/native-qa/appium_screenshot/screenshot_after_plt_import.png`

9. **Action**: `capture_logs` and search for "plt" or "import" markers
   **Expected**: Log lines showing PLT import start/completion, no errors.
   **Artifact**: `.sisyphus/evidence/native-qa/capture_logs/stdout_plt.txt`

## Classification

- [ ] **works** — PLT track row appears in Tracks panel + visible on map polyline
- [ ] **partial** — only AX tree shows track row, visual rendering unclear
- [ ] **broken** — import fails silently or errors visible in logs
- [ ] **hidden** — fixture or UI element not accessible
- [ ] **missing**

**Expected classification: `works` (P2).**

Rationale: PLT import is a core MVP feature for OziExplorer compatibility. PLT
is the native OziExplorer format. Success criteria: (1) track row appears in Tracks
panel, (2) polyline visible on map, (3) no import errors in logs.

## Evidence

- Appium session ID
- Fixture PLT: `/Users/sobieg/Projects/ozi-rs/example_data/2018-09-26_Nizovskaya/_unpacked/topo_ozf2/5-Ozi(Win&Android)_Topo_EEKO/Data/2018-09-26_Nizovskaya_500m.plt`
- Baseline screenshot: `.sisyphus/evidence/native-qa/appium_screenshot/screenshot_plt_baseline.png`
- File selection screenshot: `.sisyphus/evidence/native-qa/appium_screenshot/screenshot_plt_file_select.png`
- Post-import screenshot: `.sisyphus/evidence/native-qa/appium_screenshot/screenshot_after_plt_import.png`
- AX tree (initial): `.sisyphus/evidence/native-qa/appium/ax_tree_plt_initial.json`
- AX tree (post-import): `.sisyphus/evidence/native-qa/appium/ax_tree_plt_post_import.json`
- Logs: `.sisyphus/evidence/native-qa/capture_logs/stdout_plt.txt`

## Known failure modes

- **File dialog does not appear**: Check Tauri file picker handler and `allow:file-open` permission.
- **Import silently fails**: Check logs for PLT parser errors. Run `cargo test --manifest-path src-tauri/Cargo.toml import_plt` to validate parser.
- **PLT track not distinguishable from GPX**: Both use same Tracks panel row structure. Inspect track properties (name, style) to confirm PLT was imported successfully.
- **Track appears but no polyline**: Check Tracks panel eye icon visibility toggle.

---

## References

- ADR-0020: MVP scope, Tracks section
- `docs/commands-reference.md`: `ImportTrack` command documentation
- `src-tauri/src/infrastructure/formats.rs`: PLT parser implementation
