# Smoke: Douglas–Peucker simplification with tolerance slider

Source: ADR-0020 (MVP scope), section Tracks — "simplify track with Douglas–Peucker algorithm."

## Preconditions

- App built and available at `target/debug/bundle/macos/ozi-rs.app`
- App launched via `appium_launch_session` (Mac2 driver)
- Fixture: GPX track with ≥20 points (use `/Users/sobieg/Projects/ozi-rs/example_data/2018-09-26_Nizovskaya/8-Android&iOS/2018-09-26_Nizovskaya_500m.gpx`, 79 points)
- Track imported; polyline visible on map
- Baseline point count recorded

## UI entry point

- **Simplification UI**: May be accessible via:
  1. A "Simplify" button in the track-points panel header or context menu
  2. A "Simplify" menu item in the track right-click context menu (Tracks panel)
  3. A toolbar button with a "simplify" or "reduce points" icon
  4. A dedicated simplification panel/dialog

- **Tolerance slider**: Once simplification panel opens, look for:
  - A horizontal slider control labeled "Tolerance", "Epsilon", "Threshold", or similar
  - Numeric input field to enter tolerance value directly
  - A "Preview" button to trigger preview overlay on the map
  - An "Apply" or "OK" button to commit the simplification

- **Selector candidates**:
  - Simplify button: `//XCUIElementTypeButton[contains(@title, "Simplify")]`
  - Tolerance slider: `//XCUIElementTypeSlider[contains(@label, "Tolerance")]`
  - Preview button: `//XCUIElementTypeButton[@title="Preview"]`
  - Apply button: `//XCUIElementTypeButton[@title="Apply"]` or `//XCUIElementTypeButton[@title="OK"]`

## Steps and expected outcomes

1. **Action**: `appium_launch_session` (fresh or reuse from prior Task 6 session)
   **Expected**: Session active; app running with track visible.
   **Artifact**: Session ID.

2. **Action**: `appium_screenshot` showing baseline state
   **Expected**: Track polyline visible on map with full detail (baseline point count ~79).
   **Artifact**: `.sisyphus/evidence/native-qa/appium_screenshot/screenshot_simplify_baseline.png`

3. **Action**: Query AX tree to record baseline point count (from Tracks panel or logs)
   **Expected**: AX tree or logs show point count.
   **Artifact**: AX tree dump (baseline).

4. **Action**: Look for a **"Simplify" button or menu item**:
   - Check track-points panel header (if open from Task 6.1)
   - Check track context menu (right-click on track row)
   - Check Tracks panel header for tools menu
   **Expected**: Button or menu item found, or confirmed absent.
   **Artifact**: `appium_screenshot` of panel/menu area.

5. **Action**: If "Simplify" button/menu found, **click on it**
   **Expected**: Simplification panel or dialog opens, displaying:
     - A tolerance slider
     - A numeric input field
     - A "Preview" button (optional)
     - An "Apply" or "OK" button
   **Artifact**: `appium_screenshot` showing simplification panel.

6. **Action**: Locate the **tolerance slider** within the panel
   **Expected**: Slider visible with a range (e.g., 0.0–1000.0 meters or similar).
   **Artifact**: AX tree dump of panel; screenshot of slider area.

7. **Action**: **Drag the slider to a moderate value** (e.g., middle of the range, around 50–100 meters if using meters)
   **Expected**: Slider moves; tolerance value updates in the numeric field.
   **Artifact**: `appium_screenshot` showing slider position and value.

8. **Action**: **Click "Preview" button** (if present) to trigger preview overlay on the map
   **Expected**: Map now shows a preview of the simplified polyline (usually with a different color or overlay).
   **Artifact**: `appium_screenshot` of map showing preview overlay.

9. **Action**: **Compare preview polyline with original**: The preview should have fewer vertices (points) visible
   **Expected**: Preview polyline is smoother and has fewer visible turns.
   **Artifact**: Visual inspection or pixel-diff comparison of screenshots.

10. **Action**: **Click "Apply" or "OK" button** to commit the simplification
    **Expected**: Simplification panel closes; map updates to show the simplified polyline permanently.
    **Artifact**: n/a (captured in next screenshot).

11. **Action**: `appium_screenshot` after simplification applied
    **Expected**: Polyline on map is now simplified; fewer points visible compared to baseline.
    **Artifact**: `.sisyphus/evidence/native-qa/appium_screenshot/screenshot_after_simplify.png`

12. **Action**: Query AX tree after simplification
    **Expected**: Point count in Tracks panel or logs shows reduction (e.g., 79 → 30 points).
    **Artifact**: AX tree dump (post-simplify).

13. **Action**: `capture_logs` and search for "simplify" or "douglas" markers
    **Expected**: Log lines showing simplification operation and point count reduction.
    **Artifact**: `.sisyphus/evidence/native-qa/capture_logs/stdout.txt`

14. **Action**: **Measure the reduction in point count**
    **Expected**: Point count has decreased by at least 20% (e.g., 79 → ≤63 points).
    **Artifact**: Manual count or log extraction.

## Classification

- [ ] **works** — simplify UI found and functional; slider works; preview shows reduced points; apply commits changes; point count decreases
- [ ] **partial** — slider moves and preview works, but apply doesn't reduce point count; or preview missing but apply works
- [ ] **broken** — UI found but simplification fails (no point reduction or error in logs)
- [ ] **hidden** — no simplify UI found; backend may exist
- [ ] **missing** — no simplification functionality

**Expected classification: `works` or `partial` (P2).**

Rationale: Simplification is a useful feature for reducing storage/bandwidth for large tracks. Success criteria: (1) simplify button/menu accessible, (2) tolerance slider present and adjustable, (3) preview shows simplified polyline, (4) apply reduces point count meaningfully.

## Evidence

- Appium session ID
- Fixture GPX: `/Users/sobieg/Projects/ozi-rs/example_data/2018-09-26_Nizovskaya/8-Android&iOS/2018-09-26_Nizovskaya_500m.gpx`
- Screenshot (baseline): `.sisyphus/evidence/native-qa/appium_screenshot/screenshot_simplify_baseline.png`
- Screenshot (simplify panel): `.sisyphus/evidence/native-qa/appium_screenshot/screenshot_simplify_panel.png`
- Screenshot (preview overlay): `.sisyphus/evidence/native-qa/appium_screenshot/screenshot_simplify_preview.png`
- Screenshot (after simplify applied): `.sisyphus/evidence/native-qa/appium_screenshot/screenshot_after_simplify.png`
- AX tree (before simplify): `.sisyphus/evidence/native-qa/appium/ax_tree_before_simplify.json`
- AX tree (after simplify): `.sisyphus/evidence/native-qa/appium/ax_tree_after_simplify.json`
- Logs: `.sisyphus/evidence/native-qa/capture_logs/stdout.txt`

## Known failure modes

- **No simplify UI found**: Classify as `hidden`. Note all locations searched (panel header, context menu, toolbar).
- **Slider moves but tolerance value doesn't update**: UI binding issue. Check React/Svelte state management.
- **Preview button missing but apply works**: Classify as `partial`. User must estimate tolerance without preview.
- **Apply doesn't reduce point count**: Backend simplification may not be implemented. Check logs for Douglas–Peucker algorithm invocation.
- **Simplification fails for certain tolerance values**: May be edge-case handling. Classify as `partial` and document constraints.
- **Session crashes during simplification**: Check Appium logs. Classify as `broken`.

---

## References

- ADR-0020: MVP scope, Tracks section — "simplify track"
- Douglas–Peucker algorithm: Standard polyline simplification; reduces points while preserving shape
- Task 6.1: Track points panel discovery smoke
