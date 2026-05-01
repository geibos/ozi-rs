# Smoke: Track point delete via context menu

Source: ADR-0020 (MVP scope), section Tracks — "delete track point."

## Preconditions

- App built and available at `target/debug/bundle/macos/ozi-rs.app`
- App launched via `appium_launch_session` (Mac2 driver)
- Fixture: GPX track with ≥5 points (use `/Users/sobieg/Projects/ozi-rs/example_data/2018-09-26_Nizovskaya/8-Android&iOS/2018-09-26_Nizovskaya_500m.gpx`)
- Track imported; track-points panel open OR polyline visible on map
- Baseline point count recorded (via AX tree or visual inspection)

## UI entry point

- **Delete action**: Accessible via:
  1. Right-click on a point row in the track-points panel → "Delete" context menu item
  2. Right-click on a point marker/vertex on the map polyline → "Delete" context menu item
  3. Select a point and press Delete key
  4. Select a point and click a "Delete" button in the info panel

- **Selector candidates**:
  - Point row in panel: `//XCUIElementTypeGroup[contains(@label, "Point")]` → right-click
  - Context menu: `//XCUIElementTypeMenuItem[@title="Delete"]` or similar
  - Keyboard: `appium_key_press("Delete")` after point selection

## Steps and expected outcomes

1. **Action**: `appium_launch_session` (fresh or reuse from prior Task 6 session)
   **Expected**: Session active; app running with track visible.
   **Artifact**: Session ID.

2. **Action**: `appium_screenshot` showing baseline state
   **Expected**: Track displayed on map with polyline visible; track-points panel (if open) shows point list.
   **Artifact**: `.sisyphus/evidence/native-qa/appium_screenshot/screenshot_delete_baseline.png`

3. **Action**: Query AX tree to count initial points in the track
   **Expected**: AX tree shows point count or list size.
   **Artifact**: AX tree dump (baseline).

4. **Action**: **Right-click on a point row in the track-points panel** (middle point, not first or last)
   **Expected**: Context menu appears with options (e.g., "Delete", "Edit", "Jump to Map").
   **Artifact**: `appium_screenshot` showing context menu (if it appears).

5. **Action**: If context menu appeared, **click on "Delete" menu item**
   **Expected**: Point removed from the list; polyline on map updates to reflect removal.
   **Artifact**: n/a (captured in next screenshot).

6. **Action**: If context menu did not appear, try **right-clicking on the polyline vertex on the map** (a point along the track)
   **Expected**: Context menu appears on map.
   **Artifact**: `appium_screenshot` of map context menu (if it appears).

7. **Action**: If map context menu appeared, **click on "Delete" item**
   **Expected**: Point deleted; polyline updates.
   **Artifact**: n/a.

8. **Action**: If neither right-click worked, try **selecting a point and pressing the Delete key**:
   - `appium_click` on a point row to select it
   - `appium_key_press("Delete")`
   **Expected**: Point deleted.
   **Artifact**: n/a.

9. **Action**: `appium_screenshot` after deletion
   **Expected**: 
   - Points panel shows one fewer point in the list
   - Polyline on map has one fewer vertex (visible as a shorter or modified path)
   - Track name in Tracks panel may show updated point count
   **Artifact**: `.sisyphus/evidence/native-qa/appium_screenshot/screenshot_after_delete.png`

10. **Action**: Query AX tree after deletion
    **Expected**: Point count decreased by exactly 1; deleted point no longer in list.
    **Artifact**: AX tree dump (post-delete).

11. **Action**: Measure polyline length or point count visually/via logs
    **Expected**: Polyline shorter or visibly modified; point count in logs reflects deletion.
    **Artifact**: `capture_logs` output showing point count change.

## Classification

- [ ] **works** — context menu or keyboard delete works; point removed from panel and polyline updated on map
- [ ] **partial** — delete works but polyline doesn't update (or vice versa); or UI shows success but data inconsistency
- [ ] **broken** — delete action fails silently or throws error visible in logs
- [ ] **hidden** — no delete UI entry point found; backend may exist
- [ ] **missing** — no delete functionality apparent

**Expected classification: `works` (P1).**

Rationale: Point deletion is an essential editing operation. Success criteria: (1) context menu or keyboard shortcut available, (2) point removed from panel list, (3) polyline on map updates synchronously, (4) no errors in logs.

## Evidence

- Appium session ID
- Fixture GPX: `/Users/sobieg/Projects/ozi-rs/example_data/2018-09-26_Nizovskaya/8-Android&iOS/2018-09-26_Nizovskaya_500m.gpx`
- Screenshot (baseline): `.sisyphus/evidence/native-qa/appium_screenshot/screenshot_delete_baseline.png`
- Screenshot (context menu, if appeared): `.sisyphus/evidence/native-qa/appium_screenshot/screenshot_context_menu.png`
- Screenshot (after delete): `.sisyphus/evidence/native-qa/appium_screenshot/screenshot_after_delete.png`
- AX tree (before delete): `.sisyphus/evidence/native-qa/appium/ax_tree_before_delete.json`
- AX tree (after delete): `.sisyphus/evidence/native-qa/appium/ax_tree_after_delete.json`
- Logs: `.sisyphus/evidence/native-qa/capture_logs/stdout.txt`

## Known failure modes

- **Right-click context menu does not appear**: Appium Mac2 context menu support may be limited (see Task 6.1). Document and classify as `broken`, note "right-click context menus unsupported in Appium Mac2."
- **Point removed from panel but polyline doesn't update**: Likely a map layer sync issue. Check map redraw logic in code.
- **Delete works but undo is not available**: Document as `broken` if Delete cannot be undone (Ctrl+Z should work per ADR-0020).
- **Permission denied or "Cannot delete first/last point" error**: If error message appears, classify as `partial` and note the constraint.
- **Session crashes on delete**: Check Appium logs for segfault or panic. Classify as `broken`.

---

## References

- ADR-0020: MVP scope, Tracks section — "delete track point"
- Task 6.1: Track points panel discovery smoke
- Task 9.3: Undo/redo smoke (for undo verification)
