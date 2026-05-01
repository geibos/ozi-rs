# Smoke: Sort track points by timestamp

Source: ADR-0020 (MVP scope), section Tracks — "sort points by timestamp" (MVP-must per ADR-0020).

## Preconditions

- App built and available at `target/debug/bundle/macos/ozi-rs.app`
- App launched via `appium_launch_session` (Mac2 driver)
- Fixture: GPX track with timestamps (use `/Users/sobieg/Projects/ozi-rs/example_data/2018-09-26_Nizovskaya/8-Android&iOS/2018-09-26_Nizovskaya_500m.gpx`)
- Track imported; track-points panel open
- Baseline point order recorded (from AX tree or logs)

## UI entry point

- **Discovery goal**: Per ADR-0020, this is an MVP-must feature. UI entry point may be:
  1. A "Sort" button or menu in the track-points panel header
  2. A "Sort by Timestamp" menu item in a right-click context menu on the track
  3. A "Sort" option in the Tracks panel header (affects the active track)
  4. A toolbar button with a sort icon
  5. Or: the feature may be missing entirely (classify as `hidden` or `missing`)

- **Selector candidates**:
  - Sort button: `//XCUIElementTypeButton[contains(@title, "Sort")]`
  - Track context menu: `//XCUIElementTypeMenuItem[contains(@title, "Sort")]`
  - Panel header button: look for sort icon or label in AX tree

## Steps and expected outcomes

1. **Action**: `appium_launch_session` (fresh or reuse from prior Task 6 session)
   **Expected**: Session active; app running with track visible.
   **Artifact**: Session ID.

2. **Action**: `appium_screenshot` showing baseline state
   **Expected**: Track displayed; track-points panel visible with point list.
   **Artifact**: `.sisyphus/evidence/native-qa/appium_screenshot/screenshot_sort_baseline.png`

3. **Action**: Query AX tree to record initial point order (first 5 and last 5 points, with timestamps if visible)
   **Expected**: AX tree shows point sequence.
   **Artifact**: AX tree dump (baseline). Look for timestamp or position attributes.

4. **Action**: Examine the **track-points panel header** for a "Sort" button or menu
   **Expected**: Button or menu found, or confirmed absent.
   **Artifact**: `appium_screenshot` of panel header area (zoomed if needed).

5. **Action**: If Sort button/menu found, **click on it**
   **Expected**: A menu appears with options (e.g., "Sort by Timestamp", "Sort by Latitude", etc.) OR a sort dialog opens.
   **Artifact**: `appium_screenshot` showing sort menu or dialog.

6. **Action**: If menu appeared, **click on "Sort by Timestamp"** (or similar)
   **Expected**: Points are reordered by timestamp; backend updates the point list order.
   **Artifact**: n/a (captured in next screenshot).

7. **Action**: If no Sort button in panel header, try **right-clicking on the track row in Tracks panel**
   **Expected**: Context menu appears.
   **Artifact**: `appium_screenshot` of track context menu.

8. **Action**: If track context menu appeared, search for "Sort" item and **click on it**
   **Expected**: Sort menu or dialog appears; "Sort by Timestamp" option available.
   **Artifact**: `appium_screenshot` showing sort option.

9. **Action**: **Click on "Sort by Timestamp"** from the context menu
   **Expected**: Points are reordered.
   **Artifact**: n/a.

10. **Action**: `appium_screenshot` after sort
    **Expected**: Point order in the panel has changed. Timestamps should now be in ascending (or descending) order.
    **Artifact**: `.sisyphus/evidence/native-qa/appium_screenshot/screenshot_after_sort.png`

11. **Action**: Query AX tree after sort
    **Expected**: Point order in AX tree reflects the new timestamp-based order.
    **Artifact**: AX tree dump (post-sort). Compare first/last 5 points with baseline.

12. **Action**: `capture_logs` and search for "sort" markers
    **Expected**: Log lines showing sort operation or confirmation.
    **Artifact**: `.sisyphus/evidence/native-qa/capture_logs/stdout.txt`

13. **Action**: **Compare baseline and post-sort point order via visual inspection or logs**
    **Expected**: Points are re-sequenced; timestamps in chronological order.
    **Artifact**: Manual comparison of AX trees or log extracts.

## Classification

- [ ] **works** — sort UI found and functional; points reordered by timestamp; AX tree confirms new order
- [ ] **partial** — sort UI exists but sort doesn't actually change point order (backend issue); or sort works only if timestamps are present
- [ ] **broken** — sort UI found but sort action fails (error in logs)
- [ ] **hidden** — no sort UI found; backend may exist (check code for sort implementation)
- [ ] **missing** — no sort functionality evident

**Expected classification: `works` or `hidden` (P1 — MVP-must per ADR-0020).**

Rationale: ADR-0020 explicitly lists "sort by timestamp" as an MVP-must feature. If missing, this is a critical gap. Success criteria: (1) UI entry point found (button or menu), (2) sort action succeeds, (3) points visibly reordered by timestamp in ascending order.

## Evidence

- Appium session ID
- Fixture GPX: `/Users/sobieg/Projects/ozi-rs/example_data/2018-09-26_Nizovskaya/8-Android&iOS/2018-09-26_Nizovskaya_500m.gpx`
- Screenshot (baseline): `.sisyphus/evidence/native-qa/appium_screenshot/screenshot_sort_baseline.png`
- Screenshot (sort menu/dialog, if found): `.sisyphus/evidence/native-qa/appium_screenshot/screenshot_sort_menu.png`
- Screenshot (after sort): `.sisyphus/evidence/native-qa/appium_screenshot/screenshot_after_sort.png`
- AX tree (before sort): `.sisyphus/evidence/native-qa/appium/ax_tree_before_sort.json`
- AX tree (after sort): `.sisyphus/evidence/native-qa/appium/ax_tree_after_sort.json`
- Logs: `.sisyphus/evidence/native-qa/capture_logs/stdout.txt`

## Known failure modes

- **No sort UI found after searching panel header, context menu, and buttons**: Classify as `hidden`. Note the locations searched (panel header, track context menu, toolbar).
- **Sort UI exists but doesn't reorder points**: Backend sort may not be implemented or data binding broken. Check logs for sort errors.
- **Sort fails if timestamps missing from GPX**: May be a constraint. Classify as `partial` and note that sort requires timestamp field.
- **Sort uses incorrect order (descending instead of ascending, or alphabetical instead of chronological)**: Classify as `broken` and document the observed order.
- **Session crashes on sort**: Check Appium logs. Classify as `broken`.

---

## References

- ADR-0020: MVP scope, Tracks section — "sort by timestamp" (MVP-must)
- Task 6.1: Track points panel discovery smoke
- Timestamp requirement: GPX files must contain timestamp elements (`<time>`) for sort to be meaningful
