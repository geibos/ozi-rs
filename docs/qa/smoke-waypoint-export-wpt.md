# Smoke: Export waypoints to WPT

Source: ADR-0022 (MVP critical gap assessment), section Waypoint Export — "export waypoints as WPT (native OziExplorer waypoint format)."

## Preconditions

- App built and available at `target/debug/bundle/macos/ozi-rs.app`
- App launched via `appium_launch_session` (Mac2 driver)
- At least 2-3 waypoints exist on map (for meaningful export)
- Active bundle with `10-Tracks/` folder or temp export location available
- Waypoints panel visible

## UI entry point

- **WPT export action**: Accessible via:
  1. Right-click on waypoint row in panel → "Export as WPT" menu item
  2. Right-click on map → "Export waypoints as WPT" context menu
  3. Waypoints panel → "Export" button with format selector (GPX/PLT/WPT)
  4. File menu → "Export Waypoints → WPT" (if menu present)

- **Selector candidates**:
  - Export menu: `//XCUIElementTypeMenuItem[contains(@title, "Export")]`
  - Format selector: `//XCUIElementTypePopUpButton[contains(@title, "format\|WPT")]`
  - Context menu: Right-click on waypoint or map area

## CRITICAL NOTE: WPT Export as MVP-Must Feature

WPT is the native OziExplorer waypoint format and is essential for SAR field operations compatibility. If this export option is NOT found in the UI, this is a critical MVP gap (per ADR-0022). Document as `missing` and flag as priority for implementation.

## Steps and expected outcomes

1. **Action**: `appium_launch_session` (fresh or reuse; must have 2-3 waypoints)
   **Expected**: Session active; app running with waypoints visible.
   **Artifact**: Session ID.

2. **Action**: `appium_screenshot` showing baseline state
   **Expected**: Waypoints visible in panel and on map.
   **Artifact**: `.sisyphus/evidence/native-qa/appium_screenshot/baseline-waypoint-export-wpt.png`

3. **Action**: Query AX tree to record waypoint count:
   - `curl -s http://127.0.0.1:4723/session/{sid}/source | jq . > /tmp/ax-tree-waypoints-wpt.json`
   **Expected**: AX tree contains waypoint rows (countable).
   **Artifact**: Bash output: `grep -c "waypoint\|Waypoint" /tmp/ax-tree-waypoints-wpt.json`

4. **Action**: **Search for WPT export UI entry point**:
   - Right-click on waypoint row looking for "Export as WPT" menu item
   - Right-click on map looking for WPT export option
   - Look in Export button menus for WPT format selector
   - Check File menu (if present) for WPT export option
   **Expected**: WPT export option found; if not found, document clearly as missing feature.
   **Artifact**: `appium_screenshot` of export menu showing WPT option (if found).

5. **Action**: If **NO WPT export found after checking all locations**:
   **Expected (negative case)**: No WPT export UI present.
   **Finding**: WPT export is missing from MVP. This is a **critical gap**.
   **Artifact**: Document in "Classification" section and "Finding" section below.
   **Next step**: Skip steps 6-13. Proceed to Classification.

6. **Action**: (ONLY IF WPT export found) If **"Export as WPT" option found**, **click on it**
   **Expected**: File save dialog opens (system file picker).
   **Artifact**: `appium_screenshot` of save dialog.

7. **Action**: (ONLY IF WPT export found) **Verify save dialog defaults**:
   - Filename: Should reflect waypoint data with `.wpt` extension (e.g., `waypoints.wpt`)
   - Location: If bundle active, should default to `10-Tracks/` or similar
   - File type: WPT format selected
   **Expected**: Dialog shows reasonable defaults.
   **Artifact**: `appium_screenshot` of save dialog with WPT format.

8. **Action**: (ONLY IF WPT export found) **Accept save**:
   - Click "Save" button in dialog
   **Expected**: Dialog closes; export begins.
   **Artifact**: n/a (captured in next screenshot).

9. **Action**: (ONLY IF WPT export found) `appium_screenshot` after export dialog closes
   **Expected**: App returns to main view; no error messages.
   **Artifact**: `.sisyphus/evidence/native-qa/appium_screenshot/screenshot-after-export-wpt.png`

10. **Action**: (ONLY IF WPT export found) **Verify exported file exists**:
    - Bash: `test -f "/tmp/waypoints.wpt" && echo "File exists: OK" || echo "File not found"`
    **Expected**: File exists and has size >0 bytes.
    **Artifact**: Bash output showing file stat.

11. **Action**: (ONLY IF WPT export found) **Verify WPT file format**:
    - Bash: `head -10 "/tmp/waypoints.wpt"`
    **Expected**: First line should be WPT header (OziExplorer waypoint format identifier).
    **Artifact**: Bash output showing file header.

12. **Action**: (ONLY IF WPT export found) **Verify waypoint count in exported file**:
    - Bash: `grep -c "^[A-Z0-9]" "/tmp/waypoints.wpt"` (heuristic for data rows)
    **Expected**: Count close to waypoint count from Step 3.
    **Artifact**: Bash output showing waypoint row count.

13. **Action**: (ONLY IF WPT export found) `capture_logs` and search for "export\|wpt" markers
    **Expected**: Log lines showing WPT export operation.
    **Artifact**: `.sisyphus/evidence/native-qa/capture_logs/stdout.txt`

## Classification

- [ ] **works** — Export as WPT menu found; file dialog opens with correct defaults; exported file is valid WPT; waypoint count preserved
- [ ] **partial** — Export works but default location incorrect; or some waypoints missing; or format selector hidden
- [ ] **broken** — Export menu found but file dialog doesn't open or export fails
- [ ] **hidden** — WPT export UI not found; feature may be in progress but not yet wired to UI
- [ ] **missing** — No WPT export feature at all (CRITICAL MVP GAP)

**Expected classification: `works` (P2).**

Rationale: WPT export is critical for OziExplorer compatibility and SAR field operations. **If missing, escalate as MVP-blocking issue.** Success criteria: (1) WPT export accessible, (2) file dialog opens, (3) exported file is valid WPT, (4) waypoint count preserved.

## Finding: WPT Export Status

**If WPT export was NOT found during discovery (steps 4-5):**

WPT export UI is not present in the current MVP build. This is a **critical gap** per ADR-0022 because:
- WPT is the native OziExplorer waypoint format
- SAR field teams expect to export waypoints in WPT format for compatibility with OziExplorer on mobile devices
- Without WPT export, field coordination is impaired

**Recommendation:**
- Escalate to engineering for MVP implementation prioritization
- Check if backend WPT writer exists (may be missing UI only)
- If backend exists: quick UI win to add menu option
- If backend missing: requires format writer implementation

## Evidence

- Appium session ID
- Screenshot (baseline): `.sisyphus/evidence/native-qa/appium_screenshot/baseline-waypoint-export-wpt.png`
- Screenshot (export menu): `.sisyphus/evidence/native-qa/appium_screenshot/screenshot-export-menu-wpt.png` (if found)
- Screenshot (save dialog with WPT): `.sisyphus/evidence/native-qa/appium_screenshot/screenshot-save-dialog-wpt.png` (if found)
- Screenshot (after export): `.sisyphus/evidence/native-qa/appium_screenshot/screenshot-after-export-wpt.png` (if found)
- Exported WPT file: `/tmp/waypoints.wpt` (if export succeeded)
- AX tree (before export): `/tmp/ax-tree-waypoints-wpt.json` (bash verification only)
- Bash verification outputs: `file`, `head`, `grep -c` results (if export found)
- Logs: `.sisyphus/evidence/native-qa/capture_logs/stdout.txt`

## Known failure modes

- **WPT export menu not found (expected case)**: WPT export feature not yet implemented. Classify as `missing` and flag as critical MVP gap.
- **Export menu shows only GPX/PLT options**: Confirm WPT not present. Check code for WPT format writer; may only need UI wiring.
- **File dialog opens but defaults to wrong location**: Check bundle active state.
- **Exported file is empty or wrong format**: WPT writer may have bugs. Check implementation.
- **File extension wrong**: Check export handler.

---

## References

- ADR-0022: MVP critical gap assessment — Waypoint export formats
- ADR-0020: MVP scope, Waypoints section
- OziExplorer: Third-party mapping software; WPT is native waypoint file format
- Task 7: Waypoint feature audit
- Task 6i: Track PLT export (related pattern)
