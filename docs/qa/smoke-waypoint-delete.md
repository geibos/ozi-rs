# Smoke: Delete waypoint

Source: ADR-0020 (MVP scope), section Waypoints — "delete waypoint via UI."

## Preconditions

- App built and available at `target/debug/bundle/macos/ozi-rs.app`
- App launched via `appium_launch_session` (Mac2 driver)
- At least one waypoint exists on map and in waypoints panel
- Initial waypoint count recorded
- Map state and panel state documented

## UI entry point

- **Delete action**: Accessible via:
  1. Right-click on waypoint row in waypoints panel → "Delete" context menu
  2. Select waypoint row → press Delete or Backspace key
  3. Right-click on waypoint marker on map → "Delete" option
  4. "Delete" button in waypoints panel (if present)

- **Selector candidates**:
  - Waypoint row: `//XCUIElementTypeGroup[contains(@label, "Waypoint")]` → right-click
  - Context menu: `//XCUIElementTypeMenuItem[contains(@title, "Delete")]`
  - Delete button: `//XCUIElementTypeButton[contains(@title, "Delete")]` in panel

## Steps and expected outcomes

1. **Action**: `appium_launch_session` (fresh or reuse; must have at least one waypoint)
   **Expected**: Session active; app running with waypoint visible in panel and on map.
   **Artifact**: Session ID.

2. **Action**: `appium_screenshot` showing baseline state
   **Expected**: Waypoint row in panel; marker on map visible.
   **Artifact**: `.sisyphus/evidence/native-qa/appium_screenshot/baseline-waypoint-delete.png`

3. **Action**: Query AX tree to record current waypoint count:
   - `curl -s http://127.0.0.1:4723/session/{sid}/source | jq . > /tmp/ax-tree-before-delete.json`
   **Expected**: AX tree contains waypoint rows countable.
   **Artifact**: Bash output: `grep -c "waypoint\|Waypoint" /tmp/ax-tree-before-delete.json`

4. **Action**: **Discover delete entry point**:
   - Try right-clicking on waypoint row in panel
   - If context menu appears, look for "Delete" option
   - If no context menu, try pressing Delete key while row selected
   **Expected**: Delete option accessible or keyboard shortcut works.
   **Artifact**: `appium_screenshot` of context menu with Delete option.

5. **Action**: **Select waypoint row** by clicking on it (if needed for keyboard delete)
   **Expected**: Row highlighted or selected state visible in UI.
   **Artifact**: `appium_screenshot` of selected waypoint.

6. **Action**: **Delete waypoint** via context menu click or Delete key press
   **Expected**: Waypoint row disappears from panel; marker disappears from map; no error dialog.
   **Artifact**: `appium_screenshot` after delete action.

7. **Action**: Query AX tree to verify waypoint removed:
   - `curl -s http://127.0.0.1:4723/session/{sid}/source | jq . > /tmp/ax-tree-after-delete.json`
   **Expected**: Waypoint count in AX tree is one less than baseline.
   **Artifact**: Bash verification: `grep -c "waypoint\|Waypoint" /tmp/ax-tree-after-delete.json`

8. **Action**: **Visual verification** — map should no longer show marker at deleted location
   **Expected**: Map rendered without the deleted marker.
   **Artifact**: `appium_screenshot` of map after delete.

9. **Action**: `capture_logs` and search for "delete" markers
   **Expected**: Log lines showing waypoint deletion event and ID.
   **Artifact**: `.sisyphus/evidence/native-qa/capture_logs/stdout.txt`

10. **Action**: **Test undo** (if undo feature exists):
    - Press Cmd+Z or use Edit menu
    **Expected**: Deleted waypoint row reappears in panel; marker reappears on map.
    **Artifact**: `appium_screenshot` after undo.

## Classification

- [ ] **works** — Delete accessible via context menu or keyboard; waypoint removed from panel and map immediately; undo restores waypoint
- [ ] **partial** — Delete accessible but requires extra confirmation; or panel updates but map doesn't
- [ ] **broken** — Delete option found but action doesn't remove waypoint or causes error
- [ ] **hidden** — No delete UI entry point found
- [ ] **missing** — No waypoint delete feature

**Expected classification: `works` (P1).**

Rationale: Delete is essential editing operation in SAR context. Success criteria: (1) delete accessible via right-click, (2) waypoint removed from panel and map, (3) undo restores waypoint.

## Evidence

- Appium session ID
- Screenshot (baseline): `.sisyphus/evidence/native-qa/appium_screenshot/baseline-waypoint-delete.png`
- Screenshot (context menu): `.sisyphus/evidence/native-qa/appium_screenshot/screenshot-delete-menu.png`
- Screenshot (after delete): `.sisyphus/evidence/native-qa/appium_screenshot/screenshot-after-delete.png`
- Screenshot (map after delete): `.sisyphus/evidence/native-qa/appium_screenshot/screenshot-map-after-delete.png`
- AX tree (before delete): `/tmp/ax-tree-before-delete.json` (bash verification only)
- AX tree (after delete): `/tmp/ax-tree-after-delete.json` (bash verification only)
- Logs: `.sisyphus/evidence/native-qa/capture_logs/stdout.txt`

## Known failure modes

- **Context menu doesn't appear on right-click**: Mac2 driver may have limited context menu support. Try keyboard Delete key instead.
- **Waypoint row deleted from panel but marker remains on map**: Synchronization issue. Check logs for marker cleanup event.
- **Delete key doesn't work when row selected**: Keyboard handler may not be implemented. Check code for key event listener.
- **Error dialog appears**: Document error message and classify as `broken`.
- **Undo doesn't restore waypoint**: Undo stack may not capture delete operation. Check implementation.

---

## References

- ADR-0020: MVP scope, Waypoints section — "delete waypoint"
- Task 7: Waypoint feature audit
- Task 9.3: Undo/redo operations (related)
- Task 6c: Track point delete (related pattern)
