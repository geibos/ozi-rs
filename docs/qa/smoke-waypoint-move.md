# Smoke: Move waypoint

Source: ADR-0020 (MVP scope), section Waypoints — "move waypoint to new position."

## Preconditions

- App built and available at `target/debug/bundle/macos/ozi-rs.app`
- App launched via `appium_launch_session` (Mac2 driver)
- At least one waypoint exists on map (from Task 7a or manual creation)
- Waypoint position recorded before move
- Map zoom level allows precise interaction

## UI entry point

- **Move action**: Accessible via:
  1. Drag waypoint marker on map to new location
  2. Edit coordinates in waypoints panel text field
  3. Keyboard arrows to nudge position (if supported)

- **Selector candidates**:
  - Waypoint marker: `//XCUIElementTypeButton[contains(@label, "Waypoint")]` or position-based click
  - Coordinates field: `//XCUIElementTypeTextField[contains(@label, "Lat")]` or similar in waypoints panel

## Steps and expected outcomes

1. **Action**: `appium_launch_session` (fresh or reuse; must have at least one waypoint)
   **Expected**: Session active; app running with waypoint visible on map.
   **Artifact**: Session ID.

2. **Action**: `appium_screenshot` showing baseline state
   **Expected**: Waypoint marker visible on map with known position.
   **Artifact**: `.sisyphus/evidence/native-qa/appium_screenshot/baseline-waypoint-move.png`

3. **Action**: Query AX tree to record current waypoint position:
   - `curl -s http://127.0.0.1:4723/session/{sid}/source | jq . > /tmp/ax-tree-before-move.json`
   **Expected**: AX tree contains waypoint row with lat/lon or position label.
   **Artifact**: Bash output: `grep -i "lat\|lon" /tmp/ax-tree-before-move.json | head -5`

4. **Action**: **Locate waypoint marker on map** — Use baseline screenshot to identify pixel position
   **Expected**: Marker is visible and clickable.
   **Artifact**: Screenshot coordinates recorded.

5. **Action**: **Drag waypoint marker** to a new position on the map
   - Appium call: `appium_click(selector="<waypoint-marker-selector>")` then drag
   - Or use pixel-based drag from known marker position to new location
   **Expected**: Marker moves to new position; panel coordinates update.
   **Artifact**: `appium_screenshot` after drag operation.

6. **Action**: Query AX tree to verify position changed:
   - `curl -s http://127.0.0.1:4723/session/{sid}/source | jq . > /tmp/ax-tree-after-move.json`
   **Expected**: Waypoint coordinates in AX tree differ from baseline.
   **Artifact**: Bash verification: `diff <(grep -i "lat\|lon" /tmp/ax-tree-before-move.json) <(grep -i "lat\|lon" /tmp/ax-tree-after-move.json)`

7. **Action**: **Verify undo** (if undo feature exists):
   - Check Edit menu or keyboard shortcut (Cmd+Z)
   - Undo should restore original position
   **Expected**: Waypoint returns to original position; counted as single undo step.
   **Artifact**: `appium_screenshot` after undo.

8. **Action**: `capture_logs` and search for "move" or "drag" markers
   **Expected**: Log lines showing waypoint move operation and coordinate delta.
   **Artifact**: `.sisyphus/evidence/native-qa/capture_logs/stdout.txt`

## Classification

- [ ] **works** — Waypoint marker draggable; position updates on map and panel; undo works as single step
- [ ] **partial** — Waypoint draggable but position doesn't sync to panel; or undo doesn't coalesce moves
- [ ] **broken** — Drag action registered but marker doesn't move or position corrupts
- [ ] **hidden** — No UI to drag waypoint; coordinate edit fields not found
- [ ] **missing** — Waypoint move feature absent

**Expected classification: `partial` (P2).**

Rationale: Drag is fundamental SAR workflow. May be partial if undo coalescing not implemented. Success criteria: (1) drag moves marker, (2) coordinates sync, (3) single undo step (coalesce multiple drag events).

## Evidence

- Appium session ID
- Screenshot (baseline): `.sisyphus/evidence/native-qa/appium_screenshot/baseline-waypoint-move.png`
- Screenshot (after move): `.sisyphus/evidence/native-qa/appium_screenshot/screenshot-after-move.png`
- AX tree (before move): `/tmp/ax-tree-before-move.json` (bash verification only)
- AX tree (after move): `/tmp/ax-tree-after-move.json` (bash verification only)
- Logs: `.sisyphus/evidence/native-qa/capture_logs/stdout.txt`

## Known failure modes

- **Drag has no effect**: Mac2 driver may not support drag/drop. Try pixel-based click-hold-release sequence instead.
- **Position moves on map but not in panel**: Synchronization lag. Wait 100ms and retry AX tree query.
- **Coordinates jump to wrong position**: Coordinate system issue or snapping behavior. Document observed offset.
- **Undo restores wrong position**: Undo stack corrupted or drag events not properly coalesced. Check logs for event sequence.
- **Session crashes during drag**: Check Appium logs for native exception.

---

## References

- ADR-0020: MVP scope, Waypoints section — "move waypoint to new position"
- Task 7: Waypoint feature audit
- Task 9.3: Undo/redo operations (related)
