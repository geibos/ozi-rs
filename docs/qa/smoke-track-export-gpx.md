# Smoke: GPX export to file

Source: ADR-0020 (MVP scope), section Tracks — "export track as GPX."

## Preconditions

- App built and available at `target/debug/bundle/macos/ozi-rs.app`
- App launched via `appium_launch_session` (Mac2 driver)
- Fixture: GPX track imported (from Task 5a or fresh import)
- Track visible in Tracks panel and on map
- Active bundle with `10-Tracks/` folder (used as export target when bundle active)
- Baseline point count recorded

## UI entry point

- **Export action**: Accessible via:
  1. Right-click on track row in Tracks panel → "Export as GPX" menu item
  2. Track context menu in Tracks panel → "Export" submenu with format options
  3. "Export" button in track-points panel (if open)
  4. File menu → "Export Track" (if file menu exists)

- **Selector candidates**:
  - Track row: `//XCUIElementTypeGroup[contains(@label, "Track")]` → right-click
  - Context menu: `//XCUIElementTypeMenuItem[contains(@title, "Export")]` or `//XCUIElementTypeMenuItem[@title="Export as GPX"]`
  - Export button: `//XCUIElementTypeButton[@title="Export"]`

## Steps and expected outcomes

1. **Action**: `appium_launch_session` (fresh or reuse from prior Task 6 session)
   **Expected**: Session active; app running with track visible.
   **Artifact**: Session ID.

2. **Action**: `appium_screenshot` showing baseline state
   **Expected**: Track displayed in Tracks panel and on map.
   **Artifact**: `.sisyphus/evidence/native-qa/appium_screenshot/screenshot_export_baseline.png`

3. **Action**: **Right-click on the track row in Tracks panel**
   **Expected**: Context menu appears with options (e.g., "Export", "Delete", "Style").
   **Artifact**: `appium_screenshot` of context menu.

4. **Action**: If context menu has "Export" or "Export as GPX", **click on it**
   **Expected**: File save dialog opens, allowing the user to choose a save location and filename.
   **Artifact**: n/a (captured in next screenshot).

5. **Action**: If context menu does not appear, try **right-clicking on the track name in the Tracks panel header** or look for an export button
   **Expected**: Alternative export entry point found or confirmed absent.
   **Artifact**: `appium_screenshot` of alternative location.

6. **Action**: `appium_screenshot` of the file save dialog
   **Expected**: Dialog shows:
     - Default filename (likely based on track name, e.g., `2018-09-26_Nizovskaya_500m.gpx`)
     - Default location: **if a bundle is active, should default to `10-Tracks/` folder within the bundle**
     - File type selector: GPX format selected or only option
   **Artifact**: `.sisyphus/evidence/native-qa/appium_screenshot/screenshot_export_dialog.png`

7. **Action**: **Verify the default save location**:
   - If bundle is active: confirm path is `<bundle>/10-Tracks/`
   - If no bundle: confirm path defaults to Documents or home directory
   **Expected**: Default location is reasonable and per spec.
   **Artifact**: Manual inspection of dialog path field; screenshot of path bar.

8. **Action**: **Accept the default location and filename** (or choose an alternative temp path for testing)
   - Click "Save" or "Export" button in the dialog
   **Expected**: Dialog closes; export begins.
   **Artifact**: n/a (captured in next screenshot).

9. **Action**: `appium_screenshot` after export dialog closes
   **Expected**: App returns to main view; file is being written to disk.
   **Artifact**: `.sisyphus/evidence/native-qa/appium_screenshot/screenshot_after_export.png`

10. **Action**: **Verify the exported file exists on disk**
    - Use bash: `ls -lah <export-path>/2018-09-26_Nizovskaya_500m.gpx` (or chosen filename)
    **Expected**: File exists and has size >0 bytes.
    **Artifact**: Bash output showing file size.

11. **Action**: **Verify the exported file is valid XML/GPX**
    - Use bash: `head -10 <export-path>/*.gpx`
    **Expected**: First line is XML declaration or GPX root element:
      `<?xml version="1.0" encoding="UTF-8"?>` or `<gpx ...>`
    **Artifact**: `.sisyphus/evidence/native-qa/capture_logs/stdout.txt` (from bash output).

12. **Action**: `capture_logs` and search for "export" or "gpx" markers
    **Expected**: Log lines showing export operation, file path, and confirmation.
    **Artifact**: `.sisyphus/evidence/native-qa/capture_logs/stdout.txt`

13. **Action**: **Compare exported point count with original** (optional):
    - Use bash: `grep -c "<trkpt" <export-path>/*.gpx` to count `<trkpt>` elements
    **Expected**: Count matches original point count (79 points in this fixture).
    **Artifact**: Bash output showing point count.

## Classification

- [ ] **works** — export menu item found and functional; file dialog opens with correct default location (`10-Tracks/` when active); exported file is valid GPX with correct point count
- [ ] **partial** — export works but default location incorrect; or exported file lacks some fields (timestamps, elevation)
- [ ] **broken** — export menu item found but file dialog doesn't open or export fails
- [ ] **hidden** — no export UI entry point found
- [ ] **missing** — no export functionality

**Expected classification: `works` (P2).**

Rationale: GPX export is a core MVP feature for data portability. Success criteria: (1) export menu accessible, (2) file dialog opens, (3) default location is `10-Tracks/` when bundle active, (4) exported file is valid GPX, (5) point count matches original.

## Evidence

- Appium session ID
- Fixture GPX: `/Users/sobieg/Projects/ozi-rs/example_data/2018-09-26_Nizovskaya/8-Android&iOS/2018-09-26_Nizovskaya_500m.gpx`
- Screenshot (baseline): `.sisyphus/evidence/native-qa/appium_screenshot/screenshot_export_baseline.png`
- Screenshot (context menu): `.sisyphus/evidence/native-qa/appium_screenshot/screenshot_export_context_menu.png`
- Screenshot (file dialog): `.sisyphus/evidence/native-qa/appium_screenshot/screenshot_export_dialog.png`
- Screenshot (after export): `.sisyphus/evidence/native-qa/appium_screenshot/screenshot_after_export.png`
- Exported GPX file: `<bundle>/10-Tracks/2018-09-26_Nizovskaya_500m.gpx` (or temp path)
- Bash verification: `ls -lah` and `head -10` and `grep -c` outputs
- Logs: `.sisyphus/evidence/native-qa/capture_logs/stdout.txt`

## Known failure modes

- **Right-click context menu does not appear**: Appium Mac2 context menu support may be limited. Document and classify as `broken`.
- **File dialog opens but defaults to wrong location**: Verify bundle is active before export. Check code for default path logic in export handler.
- **Exported file is empty or corrupt**: Check logs for export errors. Verify GPX writer implementation.
- **Point count in exported file differs from original**: May be due to segment breaks or filtering. Verify via grep count comparison.
- **File format wrong (not XML/GPX)**: Check export handler for format selection logic.
- **Session crashes during export**: Check Appium logs. Classify as `broken`.

---

## References

- ADR-0020: MVP scope, Tracks section — "export track as GPX"
- GPX specification: Open standard for GPS data (trkpt = track point)
- Task 5a: GPX track import smoke
- Task 6.1: Track points panel discovery smoke
