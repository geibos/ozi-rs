# Smoke: PLT export to file

Source: ADR-0020 (MVP scope), section Tracks — "export track as PLT."

## Preconditions

- App built and available at `target/debug/bundle/macos/ozi-rs.app`
- App launched via `appium_launch_session` (Mac2 driver)
- Fixture: GPX track imported (from Task 5a or fresh import; PLT fixture from Task 5b if available)
- Track visible in Tracks panel and on map
- Baseline point count recorded

## UI entry point

- **Export action**: Same entry point as GPX export (Task 6h), but selecting "Export as PLT" from the context menu
  1. Right-click on track row in Tracks panel → "Export as PLT" or "Export" → "PLT" submenu
  2. Track context menu → "Export" with format selector

- **Selector candidates**:
  - Track row: `//XCUIElementTypeGroup[contains(@label, "Track")]` → right-click
  - Context menu: `//XCUIElementTypeMenuItem[contains(@title, "Export")]` or `//XCUIElementTypeMenuItem[@title="Export as PLT"]`
  - Submenu: `//XCUIElementTypeMenuItem[@title="PLT"]` (if Export is a submenu)

## Steps and expected outcomes

1. **Action**: `appium_launch_session` (fresh or reuse from prior Task 6 session)
   **Expected**: Session active; app running with track visible.
   **Artifact**: Session ID.

2. **Action**: `appium_screenshot` showing baseline state
   **Expected**: Track displayed in Tracks panel and on map.
   **Artifact**: `.sisyphus/evidence/native-qa/appium_screenshot/screenshot_export_plt_baseline.png`

3. **Action**: **Right-click on the track row in Tracks panel**
   **Expected**: Context menu appears with export options.
   **Artifact**: `appium_screenshot` of context menu.

4. **Action**: If context menu has "Export as PLT" or "Export → PLT", **click on it**
   **Expected**: File save dialog opens.
   **Artifact**: n/a (captured in next screenshot).

5. **Action**: If "Export as PLT" is not directly visible, check if "Export" is a submenu:
   - Click/hover on "Export" to reveal submenu
   **Expected**: Submenu appears with format options (e.g., "GPX", "PLT", "WPT").
   **Artifact**: `appium_screenshot` of submenu.

6. **Action**: `appium_screenshot` of the file save dialog
   **Expected**: Dialog shows:
     - Default filename (e.g., `2018-09-26_Nizovskaya_500m.plt`)
     - Default location: `10-Tracks/` if bundle active, otherwise Documents
     - File type: PLT format selected or only option
   **Artifact**: `.sisyphus/evidence/native-qa/appium_screenshot/screenshot_export_plt_dialog.png`

7. **Action**: **Verify the default save location and filename**
   **Expected**: Location is reasonable per spec; filename has `.plt` extension.
   **Artifact**: Manual inspection; screenshot of path bar.

8. **Action**: **Accept the default location and filename** (or choose an alternative temp path)
   - Click "Save" or "Export" button
   **Expected**: Dialog closes; export begins.
   **Artifact**: n/a (captured in next screenshot).

9. **Action**: `appium_screenshot` after export dialog closes
   **Expected**: App returns to main view; file is being written to disk.
   **Artifact**: `.sisyphus/evidence/native-qa/appium_screenshot/screenshot_after_export_plt.png`

10. **Action**: **Verify the exported PLT file exists on disk**
    - Use bash: `ls -lah <export-path>/*.plt`
    **Expected**: File exists and has size >0 bytes.
    **Artifact**: Bash output showing file size.

11. **Action**: **Verify the exported file is valid PLT format**
    - Use bash: `head -5 <export-path>/*.plt` and `file <export-path>/*.plt`
    **Expected**: First line is PLT header (OziExplorer format marker) or file command identifies it as text.
      PLT files typically start with a header line like `OziExplorer Track Point File` or similar
    **Artifact**: `.sisyphus/evidence/native-qa/capture_logs/stdout.txt` (from bash output).

12. **Action**: `capture_logs` and search for "export" or "plt" markers
    **Expected**: Log lines showing export operation and file path.
    **Artifact**: `.sisyphus/evidence/native-qa/capture_logs/stdout.txt`

13. **Action**: **Compare exported point count with original** (optional):
    - Use bash: `grep -c "^-" <export-path>/*.plt` (PLT uses `-` to mark point entries, rough count)
    **Expected**: File contains expected data lines.
    **Artifact**: Bash output showing line count.

## Classification

- [ ] **works** — export menu item for PLT found and functional; file dialog opens with correct default location; exported file is valid PLT with correct point count
- [ ] **partial** — export works but default location incorrect; or exported file present but format may be incomplete
- [ ] **broken** — export menu item found but file dialog doesn't open or export fails
- [ ] **hidden** — "Export as PLT" option not found in menu (may indicate missing feature)
- [ ] **missing** — no export functionality for PLT format

**Expected classification: `works` or `hidden` (P2).**

Rationale: PLT export provides compatibility with OziExplorer format. Success criteria: (1) export menu accessible, (2) file dialog opens, (3) exported file is valid PLT format, (4) point count preserved.

## Evidence

- Appium session ID
- Fixture GPX: `/Users/sobieg/Projects/ozi-rs/example_data/2018-09-26_Nizovskaya/8-Android&iOS/2018-09-26_Nizovskaya_500m.gpx` (or PLT fixture if available)
- Screenshot (baseline): `.sisyphus/evidence/native-qa/appium_screenshot/screenshot_export_plt_baseline.png`
- Screenshot (context menu): `.sisyphus/evidence/native-qa/appium_screenshot/screenshot_export_plt_context_menu.png`
- Screenshot (export submenu, if present): `.sisyphus/evidence/native-qa/appium_screenshot/screenshot_export_plt_submenu.png`
- Screenshot (file dialog): `.sisyphus/evidence/native-qa/appium_screenshot/screenshot_export_plt_dialog.png`
- Screenshot (after export): `.sisyphus/evidence/native-qa/appium_screenshot/screenshot_after_export_plt.png`
- Exported PLT file: `<export-path>/2018-09-26_Nizovskaya_500m.plt`
- Bash verification: `ls -lah`, `head -5`, `file` outputs
- Logs: `.sisyphus/evidence/native-qa/capture_logs/stdout.txt`

## Known failure modes

- **"Export as PLT" menu item not present**: PLT export may not be implemented. Classify as `hidden` or `missing`.
- **Context menu doesn't appear**: Appium Mac2 limitation. Document and classify as `broken`.
- **File dialog opens but defaults to wrong location or format**: Check export handler for format/location logic.
- **Exported file is empty or doesn't parse as PLT**: Check logs for export errors; verify PLT writer implementation.
- **Point count in exported file differs from original**: Verify via grep or manual inspection; document any discrepancies.
- **Session crashes during export**: Check Appium logs. Classify as `broken`.

---

## References

- ADR-0020: MVP scope, Tracks section — "export track as PLT"
- PLT format: OziExplorer Track format (proprietary but widely supported in GPS/mapping apps)
- Task 5b: PLT track import smoke (for format reference)
- Task 6h: GPX export smoke (for comparison)
