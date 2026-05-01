# Smoke: Create new track by drawing on the map

Source: ADR-0020 (MVP scope), section Tracks — "draw track by clicking on map."

## Preconditions

- App built and available at `target/debug/bundle/macos/ozi-rs.app`
- App launched via `appium_launch_session` (Mac2 driver)
- No track currently selected (or clear selection before starting)
- Map visible with clear view of an area to draw on
- Baseline Tracks panel state recorded (list should be empty or non-empty depending on prior imports)

## UI entry point

- **Drawing mode toggle**: Accessible via:
  1. A "Draw Track" or "Create Track" button in the Tracks panel header or toolbar
  2. A pencil icon button in the toolbar
  3. A right-click context menu on the map → "Draw Track" or "Create Track"
  4. A keyboard shortcut (if supported)
  5. Or: drawing mode may be toggled on by a dedicated mode toggle in the sidebar

- **Selector candidates**:
  - Draw Track button: `//XCUIElementTypeButton[contains(@title, "Draw")]` or `//XCUIElementTypeButton[@title="Create Track"]`
  - Pencil icon: `//XCUIElementTypeButton[@label="pencil"]` or similar
  - Map right-click: context menu with "Draw Track" item
  - Sidebar toggle: `//XCUIElementTypeToggle[@label="Drawing Mode"]` or similar

## Steps and expected outcomes

1. **Action**: `appium_launch_session` (fresh or reuse from prior Task 6 session)
   **Expected**: Session active; app running with map visible.
   **Artifact**: Session ID.

2. **Action**: `appium_screenshot` showing baseline state
   **Expected**: Map visible; Tracks panel visible on sidebar.
   **Artifact**: `.sisyphus/evidence/native-qa/appium_screenshot/screenshot_draw_baseline.png`

3. **Action**: Query AX tree to record baseline Tracks panel state (number of existing tracks)
   **Expected**: AX tree shows Tracks panel content.
   **Artifact**: AX tree dump (baseline).

4. **Action**: **Look for a "Draw Track", "Create Track", or pencil-icon button**:
   - Check Tracks panel header
   - Check toolbar area
   - Check sidebar for mode toggle
   **Expected**: Button or toggle found, or confirmed absent.
   **Artifact**: `appium_screenshot` of panel header/toolbar area.

5. **Action**: **Click on the draw/create button to toggle drawing mode ON**
   **Expected**: 
     - Button or toggle changes state (e.g., highlighted, pressed, color changed)
     - Map cursor may change (e.g., to a crosshair or pencil)
     - A hint or label may appear indicating "drawing mode active"
   **Artifact**: `appium_screenshot` showing drawing mode indicator.

6. **Action**: **Click on 3 different points on the map to create waypoints along the new track**
   - First point: click somewhere in the middle of the visible map area
   - Second point: click at a different location (e.g., 50 pixels away)
   - Third point: click at yet another location (e.g., 100 pixels away in a different direction)
   **Expected**: Visual markers (dots or vertices) appear at each click location; a polyline starts to form connecting the points.
   **Artifact**: `appium_screenshot` showing 3 points connected by a polyline.

7. **Action**: **Double-click to finish drawing**
   **Expected**: Drawing mode is closed; polyline is finalized.
   **Artifact**: n/a (captured in next screenshot).

8. **Action**: `appium_screenshot` after drawing is complete
   **Expected**: 
     - Polyline is now static (no longer in editing mode)
     - A new track row appears in the Tracks panel
     - The new track shows a default name (e.g., "New Track", "Track 1", or a timestamp-based name)
   **Artifact**: `.sisyphus/evidence/native-qa/appium_screenshot/screenshot_after_draw.png`

9. **Action**: Query AX tree after drawing completes
   **Expected**: Tracks panel shows a new track entry (one more track than baseline).
   **Artifact**: AX tree dump (post-draw).

10. **Action**: **Verify the polyline is visible on the map with 3 vertices**
    **Expected**: Polyline clearly shows the 3 points you clicked, connected by line segments.
    **Artifact**: `appium_screenshot` of map showing the new track.

11. **Action**: **Verify the new track row has a default name** (or prompt to name):
    - If a naming dialog appeared during/after drawing, verify you can enter a name
    - If no naming prompt, verify a default name is generated
    **Expected**: Track has a readable name.
    **Artifact**: `appium_screenshot` of Tracks panel showing new track row.

12. **Action**: `capture_logs` and search for "draw" or "create" markers
    **Expected**: Log lines showing track creation, point insertion, or drawing mode toggle.
    **Artifact**: `.sisyphus/evidence/native-qa/capture_logs/stdout.txt`

## Classification

- [ ] **works** — drawing mode toggle found and functional; points can be placed on map; polyline connects them; new track appears in Tracks panel with correct point count
- [ ] **partial** — drawing mode works but points don't connect (polyline not visible); or track appears but with wrong point count
- [ ] **broken** — drawing mode toggle found but clicking on map doesn't create points or polyline doesn't render
- [ ] **hidden** — no drawing mode toggle found
- [ ] **missing** — no track drawing functionality

**Expected classification: `works` or `partial` (P2).**

Rationale: Track drawing is a core feature for creating tracks on-the-fly in the field. Success criteria: (1) drawing mode toggle accessible, (2) map clicks create waypoints, (3) polyline connects waypoints, (4) new track appears in Tracks panel, (5) correct point count.

## Evidence

- Appium session ID
- Screenshot (baseline): `.sisyphus/evidence/native-qa/appium_screenshot/screenshot_draw_baseline.png`
- Screenshot (drawing mode enabled): `.sisyphus/evidence/native-qa/appium_screenshot/screenshot_draw_mode_enabled.png`
- Screenshot (after 3 points clicked): `.sisyphus/evidence/native-qa/appium_screenshot/screenshot_draw_3_points.png`
- Screenshot (after drawing complete): `.sisyphus/evidence/native-qa/appium_screenshot/screenshot_after_draw.png`
- Screenshot (Tracks panel with new track): `.sisyphus/evidence/native-qa/appium_screenshot/screenshot_tracks_panel_new_track.png`
- AX tree (baseline): `.sisyphus/evidence/native-qa/appium/ax_tree_draw_baseline.json`
- AX tree (after draw): `.sisyphus/evidence/native-qa/appium/ax_tree_draw_after.json`
- Logs: `.sisyphus/evidence/native-qa/capture_logs/stdout.txt`

## Known failure modes

- **Drawing mode toggle not found**: Check Tracks panel header, toolbar, and sidebar. Classify as `hidden` if not found after searching 3+ locations.
- **Drawing mode toggles but map clicks don't create points**: Click handler may not be wired for drawing mode. Check map click handler in code.
- **Points are created but no polyline visible**: Line rendering may be disabled. Check map layer for track polyline visibility.
- **Polyline visible but new track doesn't appear in Tracks panel**: Data binding between map drawing and Tracks panel may be broken. Classify as `broken`.
- **Track appears with wrong point count**: Count waypoint objects in new track via AX tree or logs. May be off-by-one or missing the double-click as a point.
- **Double-click doesn't finish drawing**: May need to use a different finishing action (e.g., button click, keyboard shortcut, right-click).
- **Session crashes during drawing**: Check Appium logs. Classify as `broken`.

---

## References

- ADR-0020: MVP scope, Tracks section — "draw track on map"
- Task 5a: GPX track import smoke (for track structure context)
- Task 6.1: Track points panel discovery smoke (for new track inspection)
