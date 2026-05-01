# Smoke: ZIP archive track import

Source: ADR-0020 (MVP scope), section Tracks — "import track bundle (ZIP)."

## Preconditions

- App built and available at `target/debug/bundle/macos/ozi-rs.app`
- App launched indirectly via `appium_launch_session` (Mac2 driver)
- Test ZIP archive containing multiple track files (GPX + PLT):
  - Location: `/Users/sobieg/Projects/ozi-rs/example_data/2018-09-26_Nizovskaya/test-tracks-bundle.zip`
  - Contents: 
    - `2018-09-26_Nizovskaya_500m.gpx` (79 points)
    - `2018-09-26_Nizovskaya_500m.plt` (79 points)
  - Note: Test ZIP will be created from existing fixtures before running this smoke
- Active bundle with maps already loaded

## UI entry point

- File picker triggered by "Import GPX" or "Import PLT" button (same as Tasks 5a/5b);
  ZIP detection and multi-file extraction handled by backend import handler.
- Notes: ZIP support is an enhancement to the basic import flow. No dedicated
  "Import ZIP" button is expected; instead, the file picker should accept `.zip`
  files and detect multi-track archives internally.

## Steps and expected outcomes

1. **Action**: Create test ZIP archive containing GPX and PLT fixtures (if not pre-existing)
   **Expected**: ZIP file created at `/Users/sobieg/Projects/ozi-rs/example_data/2018-09-26_Nizovskaya/test-tracks-bundle.zip`
   with both `.gpx` and `.plt` files.
   **Artifact**: ZIP file created (or confirmed pre-existing).

2. **Action**: `appium_launch_session` (fresh session or reuse from Task 5a/5b)
   **Expected**: Session active; app running with Tracks panel visible.
   **Artifact**: Session ID.

3. **Action**: `appium_screenshot` (baseline before ZIP import)
   **Expected**: Screenshot captures the running app; Tracks panel shows current tracks.
   **Artifact**: `.sisyphus/evidence/native-qa/appium_screenshot/screenshot_zip_baseline.png`

4. **Action**: Query AX tree to confirm import button accessible
   **Expected**: AX tree contains "Import GPX" or "Import PLT" button.
   **Artifact**: AX tree dump (baseline).

5. **Action**: `appium_click` on `//XCUIElementTypeButton[@title="Import GPX"]`
   (or "Import PLT") to trigger file dialog.
   **Expected**: File dialog appears within 1 s.
   **Artifact**: n/a (captured in next screenshot).

6. **Action**: Navigate file dialog to `/Users/sobieg/Projects/ozi-rs/example_data/2018-09-26_Nizovskaya/`
   and select `test-tracks-bundle.zip`.
   **Expected**: ZIP file is selectable and selected; "Open" or "Import" button becomes active.
   **Artifact**: `appium_screenshot` showing ZIP selection.

7. **Action**: `appium_click` on "Open" button to confirm import.
   **Expected**: Dialog closes, ZIP extraction and import begin, AX tree updates within 3 s.
   **Artifact**: `appium_screenshot` after import starts.

8. **Action**: Wait ~3 s for multi-file import to complete, then query AX tree
   **Expected**: Tracks panel shows **two new track rows** — one for GPX, one for PLT,
   both with visibility toggles (eye icons). Track names likely both
   `2018-09-26_Nizovskaya_500m` (same source, different formats).
   **Artifact**: AX tree dump showing both track rows.

9. **Action**: `appium_screenshot` after ZIP import completed
   **Expected**: Visual confirmation that both tracks appear in Tracks panel.
   **Artifact**: `.sisyphus/evidence/native-qa/appium_screenshot/screenshot_after_zip_import.png`

10. **Action**: `capture_logs` and search for "zip" or multi-file import markers
    **Expected**: Log lines showing ZIP detection, extraction, and dual-import completion, no errors.
    **Artifact**: `.sisyphus/evidence/native-qa/capture_logs/stdout_zip.txt`

## Classification

- [ ] **works** — both GPX and PLT tracks appear in Tracks panel + visible on map
- [ ] **partial** — only one track appears, or AX tree shows both but visual unclear
- [ ] **broken** — import fails silently or errors visible in logs, or only one file extracted
- [ ] **hidden** — ZIP import not implemented or UI does not support it
- [ ] **missing**

**Expected classification: `works` (P3).**

Rationale: ZIP import is an enhancement feature for convenience (importing a
whole SAR operation's tracks at once). Success criteria: (1) both tracks appear
in Tracks panel, (2) both polylines visible on map, (3) no import errors in logs.
If ZIP support is not yet implemented, classify as `hidden` with a note:
"ZIP import not yet implemented; backend file picker and handler do not distinguish
archive files from individual tracks."

## Evidence

- Appium session ID
- Test ZIP: `/Users/sobieg/Projects/ozi-rs/example_data/2018-09-26_Nizovskaya/test-tracks-bundle.zip`
- Baseline screenshot: `.sisyphus/evidence/native-qa/appium_screenshot/screenshot_zip_baseline.png`
- File selection screenshot: `.sisyphus/evidence/native-qa/appium_screenshot/screenshot_zip_file_select.png`
- Post-import screenshot: `.sisyphus/evidence/native-qa/appium_screenshot/screenshot_after_zip_import.png`
- AX tree (initial): `.sisyphus/evidence/native-qa/appium/ax_tree_zip_initial.json`
- AX tree (post-import): `.sisyphus/evidence/native-qa/appium/ax_tree_zip_post_import.json`
- Logs: `.sisyphus/evidence/native-qa/capture_logs/stdout_zip.txt`

## Known failure modes

- **ZIP file not shown in picker**: File picker filter may not include `.zip` files. Check Tauri file picker config in `commands/mod.rs`.
- **ZIP selected but import fails**: Backend may not detect or extract ZIP. Check `infrastructure/formats.rs` for ZIP handling; if not present, implement ZIP reader (likely using `zip` crate).
- **Only one track imported from ZIP**: Multi-file ZIP handling may have a bug. Check logs for parse errors on second file.
- **ZIP extraction succeeds but tracks don't appear**: Check import handler for correct path/name resolution after extraction.

---

## References

- ADR-0020: MVP scope, Tracks section
- `docs/commands-reference.md`: `ImportTrack` command documentation
- `src-tauri/src/infrastructure/formats.rs`: Track format handlers (GPX, PLT, potential ZIP reader)
