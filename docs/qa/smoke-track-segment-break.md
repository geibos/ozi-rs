# Smoke: Track segment break (split)

Source: ADR-0020 (MVP scope), section Tracks — "split track into segments."

## Preconditions

- App built and available at `target/debug/bundle/macos/ozi-rs.app`
- App launched via `appium_launch_session` (Mac2 driver)
- Fixture: GPX track with ≥10 points (use `/Users/sobieg/Projects/ozi-rs/example_data/2018-09-26_Nizovskaya/8-Android&iOS/2018-09-26_Nizovskaya_500m.gpx`)
- Track imported; track-points panel open OR polyline visible on map
- Initial track count and segment structure recorded

## UI entry point

- **Segment break action**: Accessible via:
  1. Right-click on a point row in the track-points panel → "Segment Break" or "Split" menu item
  2. Right-click on a vertex on the map polyline → "Segment Break" menu item
  3. Select a point and use a keyboard shortcut or button command

- **Selector candidates**:
  - Point row in panel: `//XCUIElementTypeGroup[contains(@label, "Point")]` → right-click
  - Context menu: `//XCUIElementTypeMenuItem[contains(@title, "Segment")]` or `//XCUIElementTypeMenuItem[@title="Split"]`
  - Map polyline vertex: right-click on polyline

## Steps and expected outcomes

1. **Action**: `appium_launch_session` (fresh or reuse from prior Task 6 session)
   **Expected**: Session active; app running with track visible.
   **Artifact**: Session ID.

2. **Action**: `appium_screenshot` showing baseline state
   **Expected**: Single track displayed; polyline continuous on map.
   **Artifact**: `.sisyphus/evidence/native-qa/appium_screenshot/screenshot_segment_baseline.png`

3. **Action**: Query AX tree to record initial track count
   **Expected**: AX tree shows single track entry in Tracks panel.
   **Artifact**: AX tree dump (baseline).

4. **Action**: **Right-click on a middle point in the track-points panel** (point index 5-10 of 79)
   **Expected**: Context menu appears with options (e.g., "Segment Break", "Split", "Delete").
   **Artifact**: `appium_screenshot` showing context menu (if it appears).

5. **Action**: If context menu appeared, **click on "Segment Break" or "Split" menu item**
   **Expected**: Track is split; the backend now treats the point as a segment boundary.
   **Artifact**: n/a (captured in next screenshot).

6. **Action**: If context menu did not appear, try **right-clicking on the polyline vertex on the map**
   **Expected**: Context menu appears on map.
   **Artifact**: `appium_screenshot` of map context menu (if it appears).

7. **Action**: If map context menu appeared, **click on "Segment Break" item**
   **Expected**: Track split.
   **Artifact**: n/a.

8. **Action**: `appium_screenshot` after segment break
   **Expected**: 
   - If the split was successful, the polyline on the map may now show:
     - A visual gap or different styling at the break point
     - Two segments rendered with different colors or line styles
   - Tracks panel may now show:
     - Original single track replaced with two track rows
     - Or: single track now has sub-segments visible in an expanded view
   **Artifact**: `.sisyphus/evidence/native-qa/appium_screenshot/screenshot_after_segment_break.png`

9. **Action**: Query AX tree after split
   **Expected**: AX tree now shows either:
     - Two separate track entries in the Tracks panel
     - Or: one track entry with expanded sub-segments
   **Artifact**: AX tree dump (post-split).

10. **Action**: `capture_logs` and search for "segment" or "split" markers
    **Expected**: Log lines showing segment break operation or track restructuring.
    **Artifact**: `.sisyphus/evidence/native-qa/capture_logs/stdout.txt`

## Classification

- [ ] **works** — segment break menu item found and functional; polyline updates to show gap or two segments; Tracks panel updates to reflect structure change
- [ ] **partial** — menu item found but polyline doesn't update visually; or split occurs in backend but UI not synchronized
- [ ] **broken** — menu item found but split action fails (error in logs or no visual change)
- [ ] **hidden** — no menu item found; backend may exist
- [ ] **missing** — no segment break functionality

**Expected classification: `works` or `partial` (P2).**

Rationale: Segment breaks are used to mark non-continuous sections of a track (e.g., helicopter jumps in SAR operations). Success criteria: (1) right-click menu accessible, (2) split action succeeds, (3) polyline or panel updates to reflect the segment boundary.

## Evidence

- Appium session ID
- Fixture GPX: `/Users/sobieg/Projects/ozi-rs/example_data/2018-09-26_Nizovskaya/8-Android&iOS/2018-09-26_Nizovskaya_500m.gpx`
- Screenshot (baseline): `.sisyphus/evidence/native-qa/appium_screenshot/screenshot_segment_baseline.png`
- Screenshot (context menu, if appeared): `.sisyphus/evidence/native-qa/appium_screenshot/screenshot_context_menu.png`
- Screenshot (after segment break): `.sisyphus/evidence/native-qa/appium_screenshot/screenshot_after_segment_break.png`
- AX tree (before split): `.sisyphus/evidence/native-qa/appium/ax_tree_before_split.json`
- AX tree (after split): `.sisyphus/evidence/native-qa/appium/ax_tree_after_split.json`
- Logs: `.sisyphus/evidence/native-qa/capture_logs/stdout.txt`

## Known failure modes

- **Right-click context menu does not appear**: Appium Mac2 context menu support may be limited. Document and classify as `broken`.
- **Split occurs in backend but polyline doesn't update**: Map layer may not be redrawing. Check map refresh logic.
- **Track count increases but no visual gap in polyline**: Backend split may be working, but visual rendering needs map redraw. Classify as `partial`.
- **Session crashes on segment break**: Check Appium logs for errors. Classify as `broken`.
- **Segment break disabled for first or last point**: Document as `partial` if there are constraints on where breaks can be inserted.

---

## References

- ADR-0020: MVP scope, Tracks section — "split track"
- Task 6.1: Track points panel discovery smoke
- SAR context: Segment breaks mark gaps in continuous movement (e.g., helicopter jumps, vehicle transfers)
