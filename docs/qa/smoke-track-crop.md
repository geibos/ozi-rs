# Smoke: Crop track (by extent, time range, selected points)

Source: ADR-0020 (MVP scope), section Tracks — "crop track by map extent, time range, or selected points."

## Preconditions

- App built and available at `target/debug/bundle/macos/ozi-rs.app`
- App launched via `appium_launch_session` (Mac2 driver)
- Fixture: GPX track with ≥20 points and timestamps (use `/Users/sobieg/Projects/ozi-rs/example_data/2018-09-26_Nizovskaya/8-Android&iOS/2018-09-26_Nizovskaya_500m.gpx`)
- Track imported; polyline visible on map; track-points panel open
- Baseline point count and map extent recorded

## UI entry point

This smoke tests three separate crop modes. Each may have its own UI entry:

1. **Crop by map extent**:
   - Button/menu: "Crop to Visible Area", "Crop to Extent", or similar
   - Located: track context menu (right-click on track row)
   - Behavior: removes points outside current map view

2. **Crop by time range**:
   - UI: date/time picker dialog with "From" and "To" fields
   - Located: track context menu or track-points panel
   - Behavior: removes points outside specified time range

3. **Crop by selected points**:
   - UI: select points in panel (Ctrl+Click or checkbox per point), then "Crop to Selected" button/menu
   - Located: track context menu or panel toolbar
   - Behavior: keeps only selected points, removes all others

## Steps and expected outcomes

### Sub-test A: Crop by map extent

1. **Action**: `appium_screenshot` showing full track polyline on map
   **Expected**: Track covers multiple map areas; polyline spans a large region.
   **Artifact**: `.sisyphus/evidence/native-qa/appium_screenshot/screenshot_crop_extent_baseline.png`

2. **Action**: Record baseline point count via AX tree or logs
   **Expected**: Point count is ~79 (or fixture count).
   **Artifact**: AX tree dump (baseline).

3. **Action**: **Pan and zoom map to show only a portion of the polyline**
   - Pan: drag map to move visible area
   - Zoom: use mouse wheel or pinch to zoom in
   **Expected**: Only ~20% of the original polyline is now visible on screen.
   **Artifact**: `appium_screenshot` showing zoomed/panned view.

4. **Action**: **Right-click on the track row in Tracks panel** (or search for crop option in context menu)
   **Expected**: Context menu appears.
   **Artifact**: `appium_screenshot` of context menu.

5. **Action**: If menu has "Crop to Extent", "Crop to Visible Area", or similar, **click on it**
   **Expected**: Crop dialog or confirmation appears (or crop proceeds automatically).
   **Artifact**: `appium_screenshot` showing dialog (if any).

6. **Action**: **Confirm the crop** (if dialog requires confirmation)
   **Expected**: Points outside the visible map area are removed.
   **Artifact**: n/a (captured in next screenshot).

7. **Action**: `appium_screenshot` after crop by extent
   **Expected**: Polyline is now shorter; only the zoomed-in region remains.
   **Artifact**: `.sisyphus/evidence/native-qa/appium_screenshot/screenshot_crop_extent_result.png`

8. **Action**: Query AX tree after crop
   **Expected**: Point count has decreased (e.g., 79 → 20 points or similar).
   **Artifact**: AX tree dump (post-crop).

### Sub-test B: Crop by time range

1. **Action**: **Find time range picker UI** — may be in:
   - Track context menu (look for "Crop by Time", "Time Range Filter", etc.)
   - Track-points panel header
   - Dedicated crop dialog
   **Expected**: UI element found or confirmed absent.
   **Artifact**: `appium_screenshot` of UI area.

2. **Action**: If time range picker found, **click to open it**
   **Expected**: Dialog with date/time input fields appears (From, To).
   **Artifact**: `appium_screenshot` of time picker dialog.

3. **Action**: **Set "From" date to partition the track** (e.g., set From to 50% through the track's time span)
   **Expected**: From date is entered or selected.
   **Artifact**: n/a.

4. **Action**: **Set "To" date to another point** (e.g., 75% through the time span)
   **Expected**: To date is entered or selected.
   **Artifact**: n/a.

5. **Action**: **Click "Apply" or "OK" to confirm crop**
   **Expected**: Points outside the time range are removed.
   **Artifact**: n/a (captured in next screenshot).

6. **Action**: `appium_screenshot` after crop by time
   **Expected**: Polyline is shorter; only points within the time range remain.
   **Artifact**: `.sisyphus/evidence/native-qa/appium_screenshot/screenshot_crop_time_result.png`

7. **Action**: Query AX tree after crop
   **Expected**: Point count has decreased.
   **Artifact**: AX tree dump (post-crop).

### Sub-test C: Crop by selected points

1. **Action**: **Open track-points panel** (from Task 6.1)
   **Expected**: Point list visible.
   **Artifact**: `appium_screenshot` of panel.

2. **Action**: **Select 3–5 points in the panel**:
   - May use Ctrl+Click per point, or checkboxes per row, or click range selection
   **Expected**: Points are visually marked as selected (highlighted, checkbox ticked, etc.).
   **Artifact**: `appium_screenshot` showing selected points.

3. **Action**: **Right-click on a selected point** or look for "Crop to Selected" button/menu
   **Expected**: Context menu appears or button found.
   **Artifact**: `appium_screenshot` of menu/button.

4. **Action**: **Click "Crop to Selected" menu item** (if present)
   **Expected**: Crop dialog or confirmation appears.
   **Artifact**: `appium_screenshot` of dialog (if any).

5. **Action**: **Confirm the crop**
   **Expected**: Only selected points remain in the track.
   **Artifact**: n/a (captured in next screenshot).

6. **Action**: `appium_screenshot` after crop by selection
   **Expected**: Polyline shows only the selected points; others removed.
   **Artifact**: `.sisyphus/evidence/native-qa/appium_screenshot/screenshot_crop_selection_result.png`

7. **Action**: Query AX tree after crop
   **Expected**: Point count matches number of selected points (e.g., 3–5).
   **Artifact**: AX tree dump (post-crop).

## Classification

- [ ] **works** — all three crop modes (extent, time, selection) are accessible and functional; points removed correctly in each case
- [ ] **partial** — one or two modes work; one mode missing or broken
- [ ] **broken** — crop UI found but crop actions fail (no points removed or error in logs)
- [ ] **hidden** — no crop UI found; backend may exist
- [ ] **missing** — no crop functionality

**Expected classification: `works` or `partial` (P2).**

Rationale: Crop is a useful feature for focusing on specific track segments. Success criteria: (1) at least one crop mode accessible and functional, (2) points visibly removed from polyline after crop, (3) AX tree confirms point count reduction.

## Evidence

- Appium session ID
- Fixture GPX: `/Users/sobieg/Projects/ozi-rs/example_data/2018-09-26_Nizovskaya/8-Android&iOS/2018-09-26_Nizovskaya_500m.gpx`
- **Crop by extent**:
  - Screenshot (baseline): `.sisyphus/evidence/native-qa/appium_screenshot/screenshot_crop_extent_baseline.png`
  - Screenshot (after crop): `.sisyphus/evidence/native-qa/appium_screenshot/screenshot_crop_extent_result.png`
  - AX tree (before): `.sisyphus/evidence/native-qa/appium/ax_tree_crop_extent_before.json`
  - AX tree (after): `.sisyphus/evidence/native-qa/appium/ax_tree_crop_extent_after.json`
- **Crop by time**:
  - Screenshot (time picker dialog, if present): `.sisyphus/evidence/native-qa/appium_screenshot/screenshot_crop_time_picker.png`
  - Screenshot (after crop): `.sisyphus/evidence/native-qa/appium_screenshot/screenshot_crop_time_result.png`
  - AX tree (before): `.sisyphus/evidence/native-qa/appium/ax_tree_crop_time_before.json`
  - AX tree (after): `.sisyphus/evidence/native-qa/appium/ax_tree_crop_time_after.json`
- **Crop by selection**:
  - Screenshot (selected points): `.sisyphus/evidence/native-qa/appium_screenshot/screenshot_crop_selection_before.png`
  - Screenshot (after crop): `.sisyphus/evidence/native-qa/appium_screenshot/screenshot_crop_selection_result.png`
  - AX tree (before): `.sisyphus/evidence/native-qa/appium/ax_tree_crop_selection_before.json`
  - AX tree (after): `.sisyphus/evidence/native-qa/appium/ax_tree_crop_selection_after.json`
- Logs: `.sisyphus/evidence/native-qa/capture_logs/stdout.txt`

## Known failure modes

- **One crop mode present, others missing**: Classify as `partial`. Document which modes are present/missing.
- **Crop UI exists but doesn't remove points**: Backend may not be implemented. Check logs for crop operation markers.
- **Time range picker missing**: May be an MVP gap if timestamps are required. Classify as `missing` for that sub-test.
- **Selection mechanism unclear or unavailable**: May not support point selection (classify as `missing` for that sub-test).
- **Crop removes wrong points**: Range or filter logic may be inverted. Classify as `broken`.
- **Session crashes during crop**: Check Appium logs. Classify as `broken`.

---

## References

- ADR-0020: MVP scope, Tracks section — "crop track"
- Task 6.1: Track points panel discovery smoke
- Task 6.2: Point walkthrough smoke (for selection context)
