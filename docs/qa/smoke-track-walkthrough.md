# Smoke: Track point walkthrough (next/previous navigation)

Source: ADR-0020 (MVP scope), section Tracks — "walk through track points."

## Preconditions

- App built and available at `target/debug/bundle/macos/ozi-rs.app`
- App launched via `appium_launch_session` (Mac2 driver)
- Fixture: GPX track with ≥10 points (use `/Users/sobieg/Projects/ozi-rs/example_data/2018-09-26_Nizovskaya/8-Android&iOS/2018-09-26_Nizovskaya_500m.gpx`, 79 points)
- Track imported; track-points panel open (from Task 6.1)

## UI entry point

- **Walkthrough controls**: Within the track-points panel, locate:
  1. "Next Point" button (or arrow icon pointing right)
  2. "Previous Point" button (or arrow icon pointing left)
  3. Alternatively: keyboard arrow keys (Up/Down) to navigate points
  4. Or: clickable list of points that updates a selected/highlighted point

- **Selector candidates**:
  - Next/Previous buttons: `//XCUIElementTypeButton[@title="Next"]` or `//XCUIElementTypeButton[@title="Previous"]`
  - Point list rows: `//XCUIElementTypeGroup[contains(@label, "Point")]` (clickable rows)
  - Keyboard: simulate `appium_key_press("Down")` or `appium_key_press("Right")`

## Steps and expected outcomes

1. **Action**: `appium_launch_session` (fresh or reuse from Task 6.1)
   **Expected**: Session active; app running with points panel open.
   **Artifact**: Session ID.

2. **Action**: `appium_screenshot` showing points panel baseline
   **Expected**: Points panel visible with list of ≥10 points; first point is selected/highlighted.
   **Artifact**: `.sisyphus/evidence/native-qa/appium_screenshot/screenshot_points_baseline.png`

3. **Action**: Query AX tree to identify next/previous button locations and keyboard-accessible structure
   **Expected**: AX tree reveals button titles, list structure, and keyboard navigation support.
   **Artifact**: AX tree dump.

4. **Action**: **Locate and click "Next Point" button** (or use Right arrow key):
   - If button exists: `appium_click` on the Next button
   - If keyboard nav: `appium_key_press("Down")` or `appium_key_press("Right")`
   **Expected**: Selection moves to next point in the list; row highlights change.
   **Artifact**: `appium_screenshot` after navigation.

5. **Action**: Verify the displayed information for the newly selected point:
   - Is latitude displayed?
   - Is longitude displayed?
   - Is timestamp displayed (if available in GPX)?
   - Is elevation displayed (if available in GPX)?
   **Expected**: At least 2+ of the above data fields visible for the selected point.
   **Artifact**: AX tree showing point details; screenshot showing info panel.

6. **Action**: Click/press "Next Point" **3-5 more times** to walk through multiple points
   **Expected**: Each click/press advances to the next point; info panel updates; no crashes.
   **Artifact**: `appium_screenshot` after 5th advance.

7. **Action**: Verify list position update (if list is scrollable):
   - Is the currently selected point visible in the list (scrolled into view)?
   - Does the selection highlight follow the point?
   **Expected**: List scrolls to keep selected point visible.
   **Artifact**: `appium_screenshot` showing scrolled list state.

8. **Action**: Click/press **"Previous Point"** button (or Left arrow key) to reverse direction
   **Expected**: Selection moves back one point; info panel updates with previous point's data.
   **Artifact**: `appium_screenshot` after previous.

9. **Action**: Continue previous navigation **2-3 times** to verify it works consistently
   **Expected**: No crashes; backward navigation works; info updates correctly.
   **Artifact**: `appium_screenshot` after 3rd reverse.

10. **Action**: `capture_logs` and search for navigation markers
    **Expected**: Log lines showing point selection changes, if any.
    **Artifact**: `.sisyphus/evidence/native-qa/capture_logs/stdout.txt`

## Classification

- [ ] **works** — next/previous buttons (or keyboard) work; info panel displays ≥2 data fields (lat/lon/timestamp/elevation); no crashes
- [ ] **partial** — navigation works but info panel incomplete (missing timestamp/elevation) OR keyboard nav works but buttons missing (or vice versa)
- [ ] **broken** — buttons present but don't advance selection; or selection advances but info panel doesn't update
- [ ] **hidden** — buttons or navigation mechanism not found
- [ ] **missing** — no UI entry for navigation

**Expected classification: `works` or `partial` (P2).**

Rationale: Walkthrough is a core editing feature. Success criteria: (1) navigation buttons/keys work, (2) selected point updates visually, (3) info panel displays essential data (lat/lon at minimum).

## Evidence

- Appium session ID
- Fixture GPX: `/Users/sobieg/Projects/ozi-rs/example_data/2018-09-26_Nizovskaya/8-Android&iOS/2018-09-26_Nizovskaya_500m.gpx`
- Screenshot (baseline): `.sisyphus/evidence/native-qa/appium_screenshot/screenshot_points_baseline.png`
- Screenshot (after next navigation): `.sisyphus/evidence/native-qa/appium_screenshot/screenshot_after_next.png`
- Screenshot (after previous navigation): `.sisyphus/evidence/native-qa/appium_screenshot/screenshot_after_previous.png`
- AX tree (points panel): `.sisyphus/evidence/native-qa/appium/ax_tree_walkthrough.json`
- Logs: `.sisyphus/evidence/native-qa/capture_logs/stdout.txt`

## Known failure modes

- **Next/Previous buttons not found**: Search AX tree for `XCUIElementTypeButton` with labels containing "next", "previous", "forward", "back", "arrow".
- **Selection advances but info panel doesn't update**: Data binding may be broken. Check React/Svelte reactivity in the component.
- **Keyboard navigation works but buttons missing**: Classify as `partial`. Note that keyboard is accessible but UI lacks visual buttons.
- **Info panel shows only lat/lon, no timestamp/elevation**: Classify as `partial` if GPX contains timestamp/elevation. Check if GPX parser is extracting these fields.
- **List scrolling doesn't follow selection**: May be an accessibility issue. Note as known limitation.

---

## References

- ADR-0020: MVP scope, Tracks section — "walk through points"
- Task 6.1: Track points panel discovery smoke
