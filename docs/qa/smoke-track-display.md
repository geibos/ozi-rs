# Smoke: Multiple track display and visibility toggle

Source: ADR-0020 (MVP scope), section Tracks — "display and toggle visibility of multiple tracks."

## Preconditions

- App built and available at `target/debug/bundle/macos/ozi-rs.app`
- At least two tracks already imported into the active project (from Tasks 5a/5b
  or pre-loaded from a previous session).
- Tracks panel visible in left sidebar with multiple track rows (at least 2).
- Active bundle with maps already loaded (for visual rendering on map).

## UI entry point

- Tracks panel in left sidebar: shows list of imported tracks, each with:
  - Track name (e.g., `2018-09-26_Nizovskaya_500m`)
  - Visibility toggle (eye icon, clickable)
  - Potential style controls (color swatch, line-width, etc.)
- Selector candidates:
  - Track row: `//XCUIElementTypeGroup[contains(@value, "2018-09-26_Nizovskaya_500m")]` (or similar name)
  - Eye icon: `//XCUIElementTypeButton[@title="Hide"]` or `//XCUIElementTypeButton[@title="Show"]` (state-dependent)

## Steps and expected outcomes

1. **Action**: Use session from Task 5 (GPX + PLT or ZIP import) or launch fresh via `appium_launch_session`
   **Expected**: Session active; app running with Tracks panel showing at least 2 tracks.
   **Artifact**: Session ID.

2. **Action**: `appium_screenshot` (baseline with both tracks visible)
   **Expected**: Screenshot shows Tracks panel with multiple track rows, each with visibility toggle (eye icon).
   Also shows map view with polylines from all visible tracks rendered.
   **Artifact**: `.sisyphus/evidence/native-qa/appium_screenshot/screenshot_multi_baseline.png`

3. **Action**: Query AX tree to inspect Tracks panel structure and identify visibility toggles
   **Expected**: AX tree contains:
   - Multiple Group or Button elements representing track rows (one per imported track)
   - Each track row contains a visibility toggle with `@title` like "Hide" or "Show"
   - Eye icon button accessible via XPath
   **Artifact**: AX tree dump showing track rows.

4. **Action**: Identify first track visibility toggle button and its current state
   **Expected**: AX tree reveals button title: "Hide" (if track is currently visible) or "Show" (if hidden).
   **Artifact**: AX tree element XPath for toggle (e.g., `//XCUIElementTypeGroup[contains(@label,"track1")]//XCUIElementTypeButton[@title="Hide"]`).

5. **Action**: `appium_click` on the visibility toggle of the first track to hide it
   **Expected**: Button title changes from "Hide" to "Show" within 1 s; map polyline for that track disappears.
   **Artifact**: `appium_screenshot` after clicking (showing one track polyline gone from map).

6. **Action**: Query AX tree to confirm toggle button state changed
   **Expected**: AX tree now shows button title as "Show" for the toggled track.
   **Artifact**: AX tree dump (post-toggle).

7. **Action**: `appium_click` on the same track's visibility toggle again to show it
   **Expected**: Button title changes back to "Hide"; map polyline re-appears.
   **Artifact**: `appium_screenshot` after re-showing (both polylines visible again).

8. **Action**: Toggle visibility of the second track (hide it)
   **Expected**: Second track polyline disappears; first track remains visible (assuming it was re-shown in step 7).
   **Artifact**: `appium_screenshot` showing only first track polyline on map.

9. **Action**: Query AX tree to inspect final visibility state
   **Expected**: AX tree shows correct toggle states for both tracks.
   **Artifact**: AX tree dump (final state).

10. **Action**: `capture_logs` and search for "visibility" or "toggle" markers
    **Expected**: Log lines showing track visibility state changes, no errors.
    **Artifact**: `.sisyphus/evidence/native-qa/capture_logs/stdout_display.txt`

## Classification

- [ ] **works** — multiple tracks display simultaneously; visibility toggle shows/hides polylines correctly
- [ ] **partial** — tracks display but toggle only works for one track, or visual feedback delayed
- [ ] **broken** — toggle not clickable, or polylines don't change visibility, or only one track renders
- [ ] **hidden**
- [ ] **missing**

**Expected classification: `works` (P2).**

Rationale: Multi-track display and visibility control are essential MVP features
for SAR workflow (comparing multiple search patterns, managing visibility for map
clarity). Success criteria: (1) both tracks visible initially, (2) eye icon toggle
changes state, (3) polyline visibility on map responds to toggle, (4) toggling is
responsive (no lag >500ms).

## Evidence

- Appium session ID
- Baseline screenshot: `.sisyphus/evidence/native-qa/appium_screenshot/screenshot_multi_baseline.png`
- After first toggle (hide): `.sisyphus/evidence/native-qa/appium_screenshot/screenshot_first_hide.png`
- After toggle back (show): `.sisyphus/evidence/native-qa/appium_screenshot/screenshot_first_show.png`
- After second toggle (hide): `.sisyphus/evidence/native-qa/appium_screenshot/screenshot_second_hide.png`
- AX tree (initial): `.sisyphus/evidence/native-qa/appium/ax_tree_display_initial.json`
- AX tree (post-toggles): `.sisyphus/evidence/native-qa/appium/ax_tree_display_final.json`
- Logs: `.sisyphus/evidence/native-qa/capture_logs/stdout_display.txt`

## Known failure modes

- **Eye icon not clickable**: May be disabled or not exposed in AX tree. Check if visibility toggle is a Button vs. static icon. If static, verify backend is setting the correct accessibility role.
- **Toggle clicks don't change visibility on map**: Map layer may not be listening to track visibility changes. Check `src/components/MapView.svelte` for reactive subscription to track visibility state.
- **Polylines don't disappear immediately**: Rendering may be batched or debounced. Check MapLibre layer visibility config in `src/lib/maplibre/` for update timing.
- **Only one track visible despite multiple imports**: Check `Tracks` store in `src/lib/stores.ts` to ensure all imported tracks are listed; verify layer creation logic in `MapView.svelte`.

---

## References

- ADR-0020: MVP scope, Tracks section
- `docs/commands-reference.md`: `SetTrackVisibility` command
- `src/components/Sidebar.svelte`: Tracks panel layout and toggle buttons
- `src/components/MapView.svelte`: Map layer management and visibility handling
- `src/lib/stores.ts`: Tracks store and visibility state
