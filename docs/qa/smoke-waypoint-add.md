# Smoke: Add waypoint

Source: ADR-0020 (MVP scope), section Waypoints — "add waypoint via UI or map click."

## Preconditions

- App built and available at `target/debug/bundle/macos/ozi-rs.app`
- App launched via `appium_launch_session` (Mac2 driver)
- Active bundle with waypoints panel visible (if present in UI)
- Initial waypoint count recorded

## UI entry point

- **Add waypoint action**: Accessible via:
  1. "Add Waypoint" button in WAYPOINTS sidebar panel (if panel present)
  2. Map click to add marker (if drawing mode enabled)
  3. Right-click on map → "Add Waypoint" context menu (if present)
  4. Waypoints panel → "+" button or "New" button

- **Selector candidates**:
  - Add button: `//XCUIElementTypeButton[contains(@title, "Add")]` in waypoints panel
  - Map area: `//XCUIElementTypeWindow[contains(@title, "map")]`
  - Context menu: `//XCUIElementTypeMenuItem[contains(@title, "Add Waypoint")]`

## Steps and expected outcomes

1. **Action**: `appium_launch_session` (fresh or reuse from prior session)
   **Expected**: Session active; app running with waypoints panel visible or map visible.
   **Artifact**: Session ID.

2. **Action**: `appium_screenshot` showing baseline state
   **Expected**: Current UI state with any visible waypoints or empty waypoints panel.
   **Artifact**: `.sisyphus/evidence/native-qa/appium_screenshot/baseline-waypoint-add.png`

3. **Action**: **Discover Add Waypoint entry point** — Look for:
   - Button labeled "Add Waypoint" or "+" in waypoints panel
   - "Add Waypoint" menu item in context menu
   - Drawing mode toggle button
   **Expected**: At least one entry point found and documented.
   **Artifact**: `appium_screenshot` of waypoints panel or menu.

4. **Action**: If "Add Waypoint" button found in panel, **click on it**
   **Expected**: App enters waypoint-add mode (cursor changes, map is clickable, or dialog opens).
   **Artifact**: `appium_screenshot` after clicking Add button.

5. **Action**: If button not found, try **right-clicking on the map area**
   **Expected**: Context menu appears with "Add Waypoint" option or similar.
   **Artifact**: `appium_screenshot` of context menu.

6. **Action**: If drawing mode toggle found, **click it to enable**
   **Expected**: Map enters clickable state for adding waypoints.
   **Artifact**: `appium_screenshot` of map in drawing mode.

7. **Action**: **Click on the map** at a known location (center of visible area)
   **Expected**: Waypoint marker appears on map; new waypoint row appears in waypoints panel.
   **Artifact**: `appium_screenshot` of map with new marker.

8. **Action**: **Query AX tree** to verify waypoint row added:
   - `curl -s http://127.0.0.1:4723/session/{sid}/source | jq . > /tmp/ax-tree-after-add.json`
   **Expected**: AX tree contains new waypoint element with label or row entry.
   **Artifact**: Bash verification output confirming element count increased.

9. **Action**: `capture_logs` and search for "waypoint" or "add" markers
   **Expected**: Log lines showing waypoint creation event.
   **Artifact**: `.sisyphus/evidence/native-qa/capture_logs/stdout.txt`

## Classification

- [ ] **works** — Add Waypoint UI entry point found and functional; map click creates waypoint; panel row appears immediately
- [ ] **partial** — Entry point found but adding waypoint requires extra steps (dialog, confirmation); or panel doesn't update immediately
- [ ] **broken** — Entry point found but action fails to create waypoint or update panel
- [ ] **hidden** — No Add Waypoint UI entry point found; backend may exist
- [ ] **missing** — No waypoints feature at all

**Expected classification: `works` (P2).**

Rationale: Waypoint management is core MVP feature for SAR marker placement. Success criteria: (1) add entry point accessible, (2) map click creates marker, (3) panel updates with new row.

## Evidence

- Appium session ID
- Screenshot (baseline): `.sisyphus/evidence/native-qa/appium_screenshot/baseline-waypoint-add.png`
- Screenshot (after add button click): `.sisyphus/evidence/native-qa/appium_screenshot/screenshot-add-button.png`
- Screenshot (after map click): `.sisyphus/evidence/native-qa/appium_screenshot/screenshot-map-click.png`
- AX tree (after add): `/tmp/ax-tree-after-add.json` (bash verification only)
- Logs: `.sisyphus/evidence/native-qa/capture_logs/stdout.txt`

## Known failure modes

- **Add Waypoint button not found**: Waypoints feature may not be implemented or UI may be hidden. Classify as `hidden`.
- **Map click does nothing**: Drawing mode may need to be enabled first. Check logs for error messages.
- **Waypoint created but panel not updated**: Synchronization issue between backend and UI. Check AX tree to confirm backend state.
- **Waypoint appears off-map or at wrong coordinates**: Coordinate system mismatch. Note position and verify against click location.
- **Session crashes when clicking map**: Check Appium logs for native exception or unhandled event.

---

## References

- ADR-0020: MVP scope, Waypoints section — "add waypoint via UI or map click"
- Task 7: Waypoint feature audit
