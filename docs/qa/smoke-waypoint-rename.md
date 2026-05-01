# Smoke: Rename waypoint

Source: ADR-0020 (MVP scope), section Waypoints — "edit waypoint name."

## Preconditions

- App built and available at `target/debug/bundle/macos/ozi-rs.app`
- App launched via `appium_launch_session` (Mac2 driver)
- At least one waypoint exists with initial name (e.g., "Waypoint 1" or "WP-001")
- Waypoint name recorded before rename
- Waypoints panel visible

## UI entry point

- **Rename action**: Accessible via:
  1. Double-click on waypoint name field in waypoints panel
  2. Click on waypoint row → edit button (if present)
  3. Right-click on waypoint → "Rename" context menu
  4. Select waypoint → press Enter or F2 to edit

- **Selector candidates**:
  - Waypoint name field: `//XCUIElementTypeTextField[contains(@label, "name")]` or in panel row
  - Edit button: `//XCUIElementTypeButton[contains(@title, "Edit")]`
  - Context menu: `//XCUIElementTypeMenuItem[contains(@title, "Rename")]`

## Steps and expected outcomes

1. **Action**: `appium_launch_session` (fresh or reuse; must have at least one waypoint)
   **Expected**: Session active; app running with waypoint visible in panel.
   **Artifact**: Session ID.

2. **Action**: `appium_screenshot` showing baseline state
   **Expected**: Waypoint row visible in waypoints panel with current name.
   **Artifact**: `.sisyphus/evidence/native-qa/appium_screenshot/baseline-waypoint-rename.png`

3. **Action**: Query AX tree to record current waypoint name:
   - `curl -s http://127.0.0.1:4723/session/{sid}/source | jq . > /tmp/ax-tree-before-rename.json`
   **Expected**: AX tree contains waypoint row with name label.
   **Artifact**: Bash output: `grep -i "waypoint\|name" /tmp/ax-tree-before-rename.json | head -10`

4. **Action**: **Discover rename entry point**:
   - Try double-clicking on waypoint name field in panel
   - If no response, try right-clicking on waypoint row
   - If no context menu, look for edit/pencil button
   **Expected**: Field becomes editable (text cursor visible) or edit mode activated.
   **Artifact**: `appium_screenshot` of editable field.

5. **Action**: **Select all text** in name field (Cmd+A) and **type new name**
   - New name example: "Rendezvous Point A" or "RP-Alpha"
   **Expected**: Text field shows new name; label on map marker (if present) doesn't update yet.
   **Artifact**: `appium_screenshot` of typed text in field.

6. **Action**: **Confirm rename** — press Enter or click outside field
   **Expected**: Field loses focus; name is committed; map label updates (if visible); panel row shows new name.
   **Artifact**: `appium_screenshot` after confirmation.

7. **Action**: Query AX tree to verify name changed:
   - `curl -s http://127.0.0.1:4723/session/{sid}/source | jq . > /tmp/ax-tree-after-rename.json`
   **Expected**: AX tree waypoint row contains new name.
   **Artifact**: Bash verification: `grep -i "Rendezvous\|RP-Alpha" /tmp/ax-tree-after-rename.json | head -5`

8. **Action**: `capture_logs` and search for "rename" markers
   **Expected**: Log lines showing waypoint rename operation with old → new name.
   **Artifact**: `.sisyphus/evidence/native-qa/capture_logs/stdout.txt`

9. **Action**: **Test undo** (if undo feature exists):
   - Press Cmd+Z or use Edit menu
   **Expected**: Waypoint name reverts to original; panel updates.
   **Artifact**: `appium_screenshot` after undo.

## Classification

- [ ] **works** — Rename field accessible via double-click or edit button; text editable; confirm works; panel and map labels update; undo restores original name
- [ ] **partial** — Field editable but confirm doesn't persist; or map label doesn't update
- [ ] **broken** — Field interactive but text doesn't change or causes error
- [ ] **hidden** — No rename UI entry point found; field not editable
- [ ] **missing** — No waypoint rename feature

**Expected classification: `works` (P2).**

Rationale: Named markers essential for SAR operations. Success criteria: (1) name field editable, (2) confirm persists change, (3) labels update on map and panel.

## Evidence

- Appium session ID
- Screenshot (baseline): `.sisyphus/evidence/native-qa/appium_screenshot/baseline-waypoint-rename.png`
- Screenshot (field in edit mode): `.sisyphus/evidence/native-qa/appium_screenshot/screenshot-rename-edit.png`
- Screenshot (after confirm): `.sisyphus/evidence/native-qa/appium_screenshot/screenshot-rename-confirmed.png`
- AX tree (before rename): `/tmp/ax-tree-before-rename.json` (bash verification only)
- AX tree (after rename): `/tmp/ax-tree-after-rename.json` (bash verification only)
- Logs: `.sisyphus/evidence/native-qa/capture_logs/stdout.txt`

## Known failure modes

- **Double-click doesn't activate edit mode**: Try right-click context menu or find explicit edit button.
- **Text field not in panel**: Rename may be in separate dialog. Document and capture dialog screenshot.
- **Map label doesn't update after rename**: UI synchronization lag. Check logs for rename event.
- **Pressing Enter doesn't confirm**: Try clicking outside field or look for explicit Save/Confirm button.
- **Undo doesn't reverse rename**: Undo stack may not capture text field changes. Check implementation.

---

## References

- ADR-0020: MVP scope, Waypoints section — "edit waypoint name"
- Task 7: Waypoint feature audit
- Task 9.3: Undo/redo operations (related)
