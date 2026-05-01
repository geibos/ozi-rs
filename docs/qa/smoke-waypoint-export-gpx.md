# Smoke: Export waypoints to GPX

Source: ADR-0020 (MVP scope), section Waypoints — "export waypoints as GPX."

## Preconditions

- App built and available at `target/debug/bundle/macos/ozi-rs.app`
- App launched via `appium_launch_session` (Mac2 driver)
- At least 2-3 waypoints exist on map (for meaningful export)
- Active bundle with `10-Tracks/` folder (for default export location) or temp export location available
- Waypoints panel visible

## UI entry point

- **GPX export action**: Accessible via:
  1. Right-click on waypoint row in panel → "Export as GPX" menu item
  2. Right-click on map → "Export waypoints" context menu
  3. Waypoints panel → "Export" button (if present)
  4. File menu → "Export Waypoints" (if menu present)

- **Selector candidates**:
  - Export menu: `//XCUIElementTypeMenuItem[contains(@title, "Export")]`
  - Export button: `//XCUIElementTypeButton[contains(@title, "Export")]`
  - Context menu: Right-click on waypoint or map area

## Steps and expected outcomes

1. **Action**: `appium_launch_session` (fresh or reuse; must have 2-3 waypoints)
   **Expected**: Session active; app running with waypoints visible.
   **Artifact**: Session ID.

2. **Action**: `appium_screenshot` showing baseline state
   **Expected**: Waypoints visible in panel and on map.
   **Artifact**: `.sisyphus/evidence/native-qa/appium_screenshot/baseline-waypoint-export-gpx.png`

3. **Action**: Query AX tree to record waypoint count:
   - `curl -s http://127.0.0.1:4723/session/{sid}/source | jq . > /tmp/ax-tree-waypoints.json`
   **Expected**: AX tree contains waypoint rows (countable).
   **Artifact**: Bash output: `grep -c "waypoint\|Waypoint" /tmp/ax-tree-waypoints.json`

4. **Action**: **Discover export entry point**:
   - Try right-clicking on a waypoint row
   - Look for "Export" or "Export as GPX" menu item
   - If no context menu, look for Export button in panel header or toolbar
   **Expected**: Export option accessible.
   **Artifact**: `appium_screenshot` of export menu.

5. **Action**: If **"Export as GPX" option found**, **click on it**
   **Expected**: File save dialog opens (system file picker).
   **Artifact**: `appium_screenshot` of save dialog.

6. **Action**: **Verify save dialog defaults**:
   - Filename: Should reflect waypoint data (e.g., `waypoints.gpx` or date-based name)
   - Location: If bundle active, should default to `10-Tracks/` folder
   - File type: GPX format selected
   **Expected**: Dialog shows reasonable defaults.
   **Artifact**: `appium_screenshot` of save dialog with defaults.

7. **Action**: **Accept save** (or use a known temp location for testing):
   - Click "Save" button in dialog
   **Expected**: Dialog closes; export begins.
   **Artifact**: n/a (captured in next screenshot).

8. **Action**: `appium_screenshot` after export dialog closes
   **Expected**: App returns to main view; no error messages.
   **Artifact**: `.sisyphus/evidence/native-qa/appium_screenshot/screenshot-after-export-gpx.png`

9. **Action**: **Verify exported file exists**:
   - Bash: `test -f "/tmp/waypoints.gpx" && echo "File exists: OK" || echo "File not found"`
   - Or check bundle path: `ls -lah "/path/to/bundle/10-Tracks/"` (if applicable)
   **Expected**: File exists and has size >0 bytes.
   **Artifact**: Bash output showing file stat.

10. **Action**: **Verify GPX file format**:
    - Bash: `head -5 "/tmp/waypoints.gpx"`
    **Expected**: First line is XML declaration or GPX root element:
      `<?xml version="1.0" encoding="UTF-8"?>` or `<gpx ...>`
    **Artifact**: Bash output showing file header.

11. **Action**: **Verify waypoint count in exported file**:
    - Bash: `grep -c "<wpt" "/tmp/waypoints.gpx"`
    **Expected**: Count matches or is close to waypoint count from Step 3.
    **Artifact**: Bash output showing waypoint element count.

12. **Action**: `capture_logs` and search for "export\|gpx" markers
    **Expected**: Log lines showing GPX export operation.
    **Artifact**: `.sisyphus/evidence/native-qa/capture_logs/stdout.txt`

## Classification

- [ ] **works** — Export menu found; file dialog opens with correct defaults; exported file is valid GPX; waypoint count preserved
- [ ] **partial** — Export works but default location incorrect; or some waypoints missing from file
- [ ] **broken** — Export menu found but file dialog doesn't open or export fails
- [ ] **hidden** — No export UI entry point found
- [ ] **missing** — No waypoint export feature

**Expected classification: `works` (P2).**

Rationale: Waypoint export essential for data portability and SAR field coordination. Success criteria: (1) export menu accessible, (2) file dialog opens, (3) exported file is valid GPX, (4) waypoint count preserved.

## Evidence

- Appium session ID
- Screenshot (baseline): `.sisyphus/evidence/native-qa/appium_screenshot/baseline-waypoint-export-gpx.png`
- Screenshot (export menu): `.sisyphus/evidence/native-qa/appium_screenshot/screenshot-export-menu-gpx.png`
- Screenshot (save dialog): `.sisyphus/evidence/native-qa/appium_screenshot/screenshot-save-dialog-gpx.png`
- Screenshot (after export): `.sisyphus/evidence/native-qa/appium_screenshot/screenshot-after-export-gpx.png`
- Exported GPX file: `/tmp/waypoints.gpx` (or bundle path if applicable)
- AX tree (before export): `/tmp/ax-tree-waypoints.json` (bash verification only)
- Bash verification outputs: `file`, `head`, `grep -c` results
- Logs: `.sisyphus/evidence/native-qa/capture_logs/stdout.txt`

## Known failure modes

- **Export menu doesn't appear**: Right-click context menu support limited. Try finding Export button in toolbar.
- **File dialog opens but defaults to wrong location**: Check bundle active state before export. Verify default path logic in code.
- **Exported file is empty or missing XML declaration**: Check GPX writer implementation. Verify file write completion.
- **Waypoint count in exported file differs**: May be due to filtering or export options. Verify via grep count.
- **Export menu present but greyed out**: May require waypoint selection or active context. Try selecting waypoint first.
- **File format wrong (not XML/GPX)**: Check export handler for format selection logic.

---

## References

- ADR-0020: MVP scope, Waypoints section — "export waypoints as GPX"
- GPX specification: Open standard for GPS data (wpt = waypoint)
- Task 7: Waypoint feature audit
- Task 6h: Track GPX export (related pattern)
