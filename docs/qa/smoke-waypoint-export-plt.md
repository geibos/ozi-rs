# Smoke: Export waypoints to PLT

Source: ADR-0020 (MVP scope), section Waypoints — "export waypoints as PLT (OziExplorer format)."

## Preconditions

- App built and available at `target/debug/bundle/macos/ozi-rs.app`
- App launched via `appium_launch_session` (Mac2 driver)
- At least 2-3 waypoints exist on map (for meaningful export)
- Active bundle with `10-Tracks/` folder (for default export location) or temp export location available
- Waypoints panel visible

## UI entry point

- **PLT export action**: Accessible via:
  1. Right-click on waypoint row in panel → "Export as PLT" menu item
  2. Right-click on map → "Export waypoints as PLT" context menu
  3. Waypoints panel → "Export" button with format selector (GPX/PLT/WPT)
  4. File menu → "Export Waypoints → PLT" (if menu present)

- **Selector candidates**:
  - Export menu: `//XCUIElementTypeMenuItem[contains(@title, "Export")]`
  - Format selector: `//XCUIElementTypePopUpButton[contains(@title, "format\|PLT")]`
  - Context menu: Right-click on waypoint or map area

## Steps and expected outcomes

1. **Action**: `appium_launch_session` (fresh or reuse; must have 2-3 waypoints)
   **Expected**: Session active; app running with waypoints visible.
   **Artifact**: Session ID.

2. **Action**: `appium_screenshot` showing baseline state
   **Expected**: Waypoints visible in panel and on map.
   **Artifact**: `.sisyphus/evidence/native-qa/appium_screenshot/baseline-waypoint-export-plt.png`

3. **Action**: Query AX tree to record waypoint count:
   - `curl -s http://127.0.0.1:4723/session/{sid}/source | jq . > /tmp/ax-tree-waypoints-plt.json`
   **Expected**: AX tree contains waypoint rows (countable).
   **Artifact**: Bash output: `grep -c "waypoint\|Waypoint" /tmp/ax-tree-waypoints-plt.json`

4. **Action**: **Discover export entry point**:
   - Try right-clicking on a waypoint row
   - Look for "Export" menu with format options (GPX/PLT/WPT)
   - If no context menu, look for Export button with format selector
   **Expected**: Export as PLT option accessible.
   **Artifact**: `appium_screenshot` of export menu with PLT option.

5. **Action**: If **"Export as PLT" option found**, **click on it**
   **Expected**: File save dialog opens (system file picker).
   **Artifact**: `appium_screenshot` of save dialog.

6. **Action**: If **only generic "Export" found**, **look for format selector**:
   - File type dropdown in save dialog should show "PLT (*.plt)" option
   - Or submenu under Export should have "Export as PLT"
   **Expected**: PLT format selectable.
   **Artifact**: `appium_screenshot` of format selector with PLT option.

7. **Action**: **Verify save dialog defaults**:
   - Filename: Should reflect waypoint data with `.plt` extension (e.g., `waypoints.plt`)
   - Location: If bundle active, should default to `10-Tracks/` folder
   - File type: PLT format selected
   **Expected**: Dialog shows reasonable defaults.
   **Artifact**: `appium_screenshot` of save dialog with PLT format.

8. **Action**: **Accept save** (or use a known temp location for testing):
   - Click "Save" button in dialog
   **Expected**: Dialog closes; export begins.
   **Artifact**: n/a (captured in next screenshot).

9. **Action**: `appium_screenshot` after export dialog closes
   **Expected**: App returns to main view; no error messages.
   **Artifact**: `.sisyphus/evidence/native-qa/appium_screenshot/screenshot-after-export-plt.png`

10. **Action**: **Verify exported file exists**:
    - Bash: `test -f "/tmp/waypoints.plt" && echo "File exists: OK" || echo "File not found"`
    - Or check bundle path: `ls -lah "/path/to/bundle/10-Tracks/"` (if applicable)
    **Expected**: File exists and has size >0 bytes.
    **Artifact**: Bash output showing file stat.

11. **Action**: **Verify PLT file format**:
    - Bash: `head -10 "/tmp/waypoints.plt"`
    **Expected**: First line should be PLT header (e.g., `OziExplorer` string or format identifier).
    **Artifact**: Bash output showing file header.

12. **Action**: **Verify waypoint count in exported file**:
    - Bash: `grep -c "^[A-Z0-9]" "/tmp/waypoints.plt"` (heuristic for data rows)
    - Or count non-header lines: `tail -n +2 "/tmp/waypoints.plt" | wc -l`
    **Expected**: Count close to waypoint count from Step 3.
    **Artifact**: Bash output showing waypoint row count.

13. **Action**: `capture_logs` and search for "export\|plt" markers
    **Expected**: Log lines showing PLT export operation.
    **Artifact**: `.sisyphus/evidence/native-qa/capture_logs/stdout.txt`

## Classification

- [ ] **works** — Export as PLT menu found; file dialog opens with correct defaults; exported file is valid PLT; waypoint count preserved
- [ ] **partial** — Export works but default location incorrect; or some waypoints missing; or format selector hidden in generic Export
- [ ] **broken** — Export menu found but file dialog doesn't open or export fails
- [ ] **hidden** — No PLT export UI entry point found (GPX export present but PLT absent)
- [ ] **missing** — No waypoint export feature at all

**Expected classification: `partial` (P2).**

Rationale: PLT export less critical than GPX (GPX is more portable) but important for OziExplorer compatibility. May be partial if only generic Export menu exists without specific PLT format selector. Success criteria: (1) PLT export accessible, (2) file dialog opens, (3) exported file is valid PLT, (4) waypoint count preserved.

## Evidence

- Appium session ID
- Screenshot (baseline): `.sisyphus/evidence/native-qa/appium_screenshot/baseline-waypoint-export-plt.png`
- Screenshot (export menu): `.sisyphus/evidence/native-qa/appium_screenshot/screenshot-export-menu-plt.png`
- Screenshot (save dialog with PLT): `.sisyphus/evidence/native-qa/appium_screenshot/screenshot-save-dialog-plt.png`
- Screenshot (after export): `.sisyphus/evidence/native-qa/appium_screenshot/screenshot-after-export-plt.png`
- Exported PLT file: `/tmp/waypoints.plt` (or bundle path if applicable)
- AX tree (before export): `/tmp/ax-tree-waypoints-plt.json` (bash verification only)
- Bash verification outputs: `file`, `head`, `grep -c` results
- Logs: `.sisyphus/evidence/native-qa/capture_logs/stdout.txt`

## Known failure modes

- **Export menu shows only GPX option**: PLT export not implemented. Classify as `hidden` or `missing`.
- **Export menu present but no specific PLT option**: Generic export may require format selector in dialog. Verify in save dialog file-type dropdown.
- **File dialog opens but defaults to wrong location**: Check bundle active state. Verify default path logic.
- **Exported file is empty or wrong format**: Check PLT writer implementation. May be not yet implemented (classify as `missing`).
- **Waypoint count in exported file differs**: May be due to filtering. Verify via line count.
- **File extension wrong (not .plt)**: Check export handler for extension selection.

---

## References

- ADR-0020: MVP scope, Waypoints section — "export waypoints as PLT"
- OziExplorer: Third-party mapping software; PLT is waypoint file format
- Task 7: Waypoint feature audit
- Task 6i: Track PLT export (related pattern)
