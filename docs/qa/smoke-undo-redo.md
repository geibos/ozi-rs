# Smoke: Undo and Redo

Source: ADR-0020 (MVP scope), UI Features section — "Undo (Ctrl+Z) and Redo (Ctrl+Y) keyboard shortcuts."

## Preconditions

- App built and available at `target/debug/bundle/macos/ozi-rs.app`
- App launched via `appium_launch_session` with tracks and waypoints loaded
- Interactive elements available: track points (from Task 6) and waypoints (from Task 7)
- Map zoom level allows precise interaction

## UI entry point

- **Undo action**: Accessible via:
  1. Keyboard shortcut: Cmd+Z (standard macOS)
  2. Edit menu → Undo
  3. Undo button in toolbar (if present)

- **Redo action**: Accessible via:
  1. Keyboard shortcut: Cmd+Shift+Z or Cmd+Y
  2. Edit menu → Redo
  3. Redo button in toolbar (if present)

- **Selector candidates**:
  - Edit menu: `//XCUIElementTypeMenuBarItem[contains(@label, 'Edit')]`
  - Undo button: `//XCUIElementTypeButton[contains(@label, 'Undo')]`
  - Redo button: `//XCUIElementTypeButton[contains(@label, 'Redo')]`

## Steps and expected outcomes

### Part A: Waypoint Move and Undo

1. **Action**: `appium_launch_session` with waypoint visible on map
   **Expected**: Session active; waypoint marker visible on map.
   **Artifact**: Session ID.

2. **Action**: `appium_screenshot` showing baseline waypoint position
   **Expected**: Waypoint marker visible with known position on map.
   **Artifact**: `.sisyphus/evidence/native-qa/appium_screenshot/baseline-undo-waypoint.png`

3. **Action**: Record baseline waypoint position from AX tree
   - `curl -s http://127.0.0.1:4723/session/{sid}/source | jq . > /tmp/ax-tree-undo-baseline.json`
   - `grep -i "waypoint\|lat\|lon" /tmp/ax-tree-undo-baseline.json | head -5`
   **Expected**: AX tree shows waypoint with position label or coordinates.
   **Artifact**: Bash output showing position data.

4. **Action**: Drag waypoint marker to a new position on map
   - Identify marker pixel position from baseline screenshot
   - Drag to new location (e.g., 50-100 pixels away)
   **Expected**: Waypoint moves visually; panel coordinates update.
   **Artifact**: `appium_screenshot` showing moved waypoint.

5. **Action**: Verify position changed in AX tree
   - `curl -s http://127.0.0.1:4723/session/{sid}/source | jq . > /tmp/ax-tree-undo-after-move.json`
   - `grep -i "waypoint\|lat\|lon" /tmp/ax-tree-undo-after-move.json | head -5`
   **Expected**: Coordinates differ from baseline.
   **Artifact**: Bash output showing new position.

6. **Action**: Undo the move operation
   - Press Cmd+Z keyboard shortcut
   **Expected**: Waypoint returns to original position; panel coordinates revert.
   **Artifact**: `appium_screenshot` showing undone waypoint position.

7. **Action**: Verify undo worked via AX tree
   - `curl -s http://127.0.0.1:4723/session/{sid}/source | jq . > /tmp/ax-tree-undo-after-undo.json`
   - `diff <(grep -i "lat\|lon" /tmp/ax-tree-undo-baseline.json) <(grep -i "lat\|lon" /tmp/ax-tree-undo-after-undo.json)`
   **Expected**: Diff is empty or minimal (position restored).
   **Artifact**: Bash diff output.

8. **Action**: Redo the move operation
   - Press Cmd+Shift+Z or Cmd+Y keyboard shortcut
   **Expected**: Waypoint moves to new position again; panel coordinates update.
   **Artifact**: `appium_screenshot` showing redone waypoint position.

9. **Action**: Verify redo worked via AX tree
   - `curl -s http://127.0.0.1:4723/session/{sid}/source | jq . > /tmp/ax-tree-undo-after-redo.json`
   - `grep -i "lat\|lon" /tmp/ax-tree-undo-after-redo.json`
   **Expected**: Coordinates match the moved position (step 5).
   **Artifact**: Bash output.

### Part B: Waypoint Deletion and Undo

10. **Action**: Create or use existing second waypoint on map
    **Expected**: 2+ waypoints visible in Waypoints panel and on map.
    **Artifact**: `appium_screenshot` showing multiple waypoints.

11. **Action**: Record waypoint count from AX tree
    - `grep -c "XCUIElementTypeCell" /tmp/ax-tree-undo-after-redo.json`
    **Expected**: At least 2 waypoint rows in Waypoints panel.
    **Artifact**: Bash count output.

12. **Action**: Delete a waypoint via context menu or Delete key
    - Right-click waypoint row in panel and select Delete
    - OR select row and press Delete key
    **Expected**: Waypoint disappears from panel and map.
    **Artifact**: `appium_screenshot` showing deleted waypoint gone.

13. **Action**: Verify deletion in AX tree
    - `curl -s http://127.0.0.1:4723/session/{sid}/source | jq . > /tmp/ax-tree-after-delete.json`
    - `grep -c "XCUIElementTypeCell" /tmp/ax-tree-after-delete.json`
    **Expected**: Waypoint count decreased by 1.
    **Artifact**: Bash count output.

14. **Action**: Undo the deletion
    - Press Cmd+Z
    **Expected**: Waypoint reappears in panel and on map with original position/style.
    **Artifact**: `appium_screenshot` showing restored waypoint.

15. **Action**: Verify undo restored waypoint
    - `curl -s http://127.0.0.1:4723/session/{sid}/source | jq . > /tmp/ax-tree-after-undo-delete.json`
    - `grep -c "XCUIElementTypeCell" /tmp/ax-tree-after-undo-delete.json`
    **Expected**: Waypoint count restored to original.
    **Artifact**: Bash count output.

### Part C: Track Point Deletion and Undo

16. **Action**: Open track points panel (from Task 6a discovery)
    - Right-click track in Tracks panel and select "Edit Points" or similar
    - OR double-click track row to open points panel
    **Expected**: Track points panel opens with list of track points.
    **Artifact**: `appium_screenshot` showing points panel.

17. **Action**: Record track point count
    - Query AX tree for track points list
    - `grep -c "XCUIElementTypeCell" /tmp/ax-tree-track-points.json`
    **Expected**: ≥2 track points in panel.
    **Artifact**: Bash count output.

18. **Action**: Delete a track point
    - Select point row and press Delete
    - OR right-click and select Delete
    **Expected**: Point removed from panel and polyline updated on map.
    **Artifact**: `appium_screenshot` showing point removed and polyline redrawn.

19. **Action**: Undo the deletion
    - Press Cmd+Z
    **Expected**: Point reappears in panel and polyline restored.
    **Artifact**: `appium_screenshot` showing restored point.

20. **Action**: Verify undo worked
    - Query AX tree for point count
    - `grep -c "XCUIElementTypeCell" /tmp/ax-tree-after-undo-point-delete.json`
    **Expected**: Point count restored.
    **Artifact**: Bash count output.

### Part D: Redo After Delete

21. **Action**: Redo the point deletion
    - Press Cmd+Shift+Z or Cmd+Y
    **Expected**: Point removed again; polyline updated.
    **Artifact**: `appium_screenshot` showing point removed via redo.

22. **Action**: Verify redo
    - Query AX tree for point count
    **Expected**: Point count matches deleted state.
    **Artifact**: Bash count output.

### Part E: Multi-step Undo

23. **Action**: Create new waypoint, move it, delete it (3 actions)
    **Expected**: 3 operations recorded in undo stack.
    **Artifact**: `appium_screenshot` after each action.

24. **Action**: Press Cmd+Z three times
    - First undo: delete undone (waypoint reappears)
    - Second undo: move undone (waypoint returns to original position)
    - Third undo: create undone (waypoint disappears completely)
    **Expected**: Each Cmd+Z reverses one action.
    **Artifact**: 3 screenshots showing progressive undo.

25. **Action**: Press Cmd+Y three times
    - First redo: create redone (waypoint appears)
    - Second redo: move redone (waypoint moves)
    - Third redo: delete redone (waypoint disappears)
    **Expected**: Each Cmd+Y reapplies one action.
    **Artifact**: 3 screenshots showing progressive redo.

## Classification

- [ ] **works** — Undo (Cmd+Z) and Redo (Cmd+Shift+Z/Cmd+Y) both function correctly for waypoint move, deletion, and track point operations; each undo/redo is a single step; multi-step undo/redo works correctly
- [ ] **partial** — Undo/Redo work for some actions (e.g., move) but not others (e.g., delete); OR undo exists but redo missing
- [ ] **broken** — Undo/Redo shortcuts exist but don't reverse actions; OR undo stack corrupted (wrong state restored)
- [ ] **hidden** — No Undo/Redo menu items or keyboard shortcuts found
- [ ] **missing** — Undo/Redo feature not implemented

**Expected classification: `works` (P1).**

Rationale: Undo/Redo are critical for SAR workflow usability. Success criteria: (1) Cmd+Z/Cmd+Shift+Z reachable, (2) all action types support undo, (3) each undo is a single step (drag events coalesced), (4) redo works correctly.

## Evidence

- Appium session ID
- Screenshot (baseline waypoint): `.sisyphus/evidence/native-qa/appium_screenshot/baseline-undo-waypoint.png`
- Screenshot (after move): after-move.png
- Screenshot (after undo move): after-undo-move.png
- Screenshot (after redo move): after-redo-move.png
- Screenshot (multiple waypoints): multiple-waypoints.png
- Screenshot (after delete): after-delete.png
- Screenshot (after undo delete): after-undo-delete.png
- Screenshot (track points panel): track-points-panel.png
- Screenshot (after point delete): after-point-delete.png
- Screenshot (after undo point delete): after-undo-point-delete.png
- Screenshots (multi-step undo sequence): undo-step-1/2/3.png
- Screenshots (multi-step redo sequence): redo-step-1/2/3.png
- AX tree (baseline): `/tmp/ax-tree-undo-baseline.json`
- AX tree (after move): `/tmp/ax-tree-undo-after-move.json`
- AX tree (after undo): `/tmp/ax-tree-undo-after-undo.json`
- Logs: `.sisyphus/evidence/native-qa/capture_logs/stdout.txt`

## Known failure modes

- **Cmd+Z doesn't work**: Mac2 driver may not support keyboard shortcuts properly. Try Edit menu → Undo instead.
- **Undo greyed out**: Undo stack empty or no undoable action recorded. Ensure move/delete action completed before undo.
- **Undo reverts wrong action**: Undo stack corrupted. Check logs for operation sequence.
- **Redo doesn't work**: Redo stack may not be maintained. Check implementation for redo history pruning.
- **Undo coalesces drag events into multiple steps**: Expected behavior may be single step for whole drag. Check implementation for event batching.
- **Session crashes during undo**: State restoration may corrupt app memory. Check logs for exceptions.

---

## References

- ADR-0020: MVP scope, UI Features section — "Undo (Ctrl+Z) and Redo (Ctrl+Y)"
- Task 6: Track points panel and operations (point deletion related)
- Task 7: Waypoint operations (move, delete related)
- Task 9: Project-level features audit
