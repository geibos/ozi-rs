# Smoke: Track points panel discovery

Source: ADR-0020 (MVP scope), section Tracks — "track-points detail panel."

## Preconditions

- App built and available at `target/debug/bundle/macos/ozi-rs.app`
- App launched via `appium_launch_session` (Mac2 driver)
- Fixture: GPX track pre-imported from Task 5 (or use `/Users/sobieg/Projects/ozi-rs/example_data/2018-09-26_Nizovskaya/8-Android&iOS/2018-09-26_Nizovskaya_500m.gpx`)
- Active bundle with track displayed on map

## UI entry point

- **Discovery goal**: User reports they cannot find the track-points detail panel. Determine:
  1. Is there a button in the track row (Tracks panel) to open the panel?
  2. Is there a right-click context menu on the track row?
  3. Does double-clicking the track name open the panel?
  4. Is the panel accessible via a separate toolbar button or menu?

- **Selector candidates**:
  - Track row in Tracks panel: `//XCUIElementTypeGroup[contains(@label, "Track")]` or similar
  - Double-click on track name in Tracks panel
  - Right-click context menu on track row
  - Dedicated "Points" or "Edit Points" button (if present)

## Steps and expected outcomes

1. **Action**: `appium_launch_session` (fresh session)
   **Expected**: Mac2 WebDriver session created; app started.
   **Artifact**: Session ID recorded.

2. **Action**: Import a GPX track via the "Import GPX" button in Tracks panel (from prior Task 5, or perform fresh import).
   **Expected**: Track row appears in Tracks panel; polyline visible on map.
   **Artifact**: `appium_screenshot` showing Tracks panel with imported track.

3. **Action**: Query AX tree to list all elements in the Tracks panel, specifically looking for:
   - Sub-buttons or labels under the track row (e.g., "View Points", "Edit", "Details")
   - Any "Points" panel or modal listed in the tree
   **Expected**: AX tree reveals structure of track row and available sub-actions.
   **Artifact**: AX tree dump (initial).

4. **Action**: **Right-click on the track row** (right-click on track name or row).
   **Expected**: Context menu appears with options (e.g., "Delete", "Export", "View Points", "Edit").
   **Artifact**: `appium_screenshot` showing context menu (if it appears).

5. **Action**: If context menu appeared and contains "View Points" or similar, `appium_click` on that item.
   **Expected**: Points panel opens, showing list of track points with columns (e.g., lat, lon, timestamp, elevation).
   **Artifact**: `appium_screenshot` showing points panel.

6. **Action**: If context menu did not appear, try **double-clicking on the track name/row**.
   **Expected**: Points panel opens (or a detail panel of any kind).
   **Artifact**: `appium_screenshot` showing panel.

7. **Action**: If neither right-click nor double-click worked, search Tracks panel header and toolbar for a dedicated button:
   - Look for buttons labeled "Points", "Edit", "Details", or an icon (e.g., list icon, pencil icon)
   - Check if there's a menu or dropdown in the Tracks panel header
   **Expected**: Button or menu found, or confirmed absent.
   **Artifact**: `appium_screenshot` of Tracks panel header/toolbar area.

8. **Action**: If points panel successfully opened in any of the above steps, take a screenshot showing:
   - Panel title (if any)
   - List of points with visible columns (lat, lon, timestamp, elevation, etc.)
   - Any controls (next/previous, delete, etc.)
   **Expected**: Points panel clearly visible with at least 3+ rows of data.
   **Artifact**: `.sisyphus/evidence/native-qa/appium_screenshot/screenshot_points_panel.png`

9. **Action**: Query AX tree with points panel open.
   **Expected**: AX tree shows the structure of the points panel, confirming it is a distinct UI element.
   **Artifact**: AX tree dump (panel open).

10. **Action**: `capture_logs` and search for "points" or "panel" markers.
    **Expected**: Log lines showing panel lifecycle events or confirmations.
    **Artifact**: `.sisyphus/evidence/native-qa/capture_logs/stdout.txt`

## Classification

- [ ] **works** — points panel opens via button, right-click menu, or double-click; displays point list with ≥3 columns
- [ ] **partial** — points panel exists but only accessible via non-obvious method or missing columns
- [ ] **broken** — points panel accessible but crashes or fails to load point data
- [ ] **hidden** — UI entry point not found; backend may exist (check code for PointsPanel component)
- [ ] **missing** — no UI entry and no evidence of backend implementation

**Expected classification: `works` or `hidden` (P2).**

Rationale: User explicitly reports inability to find this panel. Discovery is the primary goal. If the panel is missing from the UI, classify as `hidden` and recommend backend/UI implementation.

## Evidence

- Appium session ID
- Fixture GPX: `/Users/sobieg/Projects/ozi-rs/example_data/2018-09-26_Nizovskaya/8-Android&iOS/2018-09-26_Nizovskaya_500m.gpx`
- Screenshot (Tracks panel baseline): `.sisyphus/evidence/native-qa/appium_screenshot/screenshot_baseline.png`
- Screenshot (context menu, if appeared): `.sisyphus/evidence/native-qa/appium_screenshot/screenshot_context_menu.png`
- Screenshot (points panel, if opened): `.sisyphus/evidence/native-qa/appium_screenshot/screenshot_points_panel.png`
- AX tree (initial): `.sisyphus/evidence/native-qa/appium/ax_tree_initial.json`
- AX tree (panel open, if successful): `.sisyphus/evidence/native-qa/appium/ax_tree_panel_open.json`
- Logs: `.sisyphus/evidence/native-qa/capture_logs/stdout.txt`

## Known failure modes

- **Right-click does not open menu**: Appium Mac2 context menu support may be limited. Try double-click instead; if both fail, classify as `hidden` and note "right-click context menus unsupported in Appium Mac2."
- **Double-click registers as single click**: May need to use Appium's `double_click` action explicitly.
- **Points panel opens but shows no data**: Check logs for parse errors or data-binding issues. Verify GPX import succeeded (Task 5).
- **Points panel listed in AX tree but invisible on screen**: Likely a CSS visibility issue (z-index, overflow hidden, etc.). Classify as `broken` and note the UI hierarchy mismatch.
- **No UI entry point found after 3+ attempts**: Classify as `hidden`. Code review of `src-tauri/src/ui/` or frontend components may reveal if the panel is implemented but unreachable.

---

## References

- ADR-0020: MVP scope, Tracks section
- User report: "Cannot find track-points panel"
- Task 5: GPX track import smoke (Task 5a)
