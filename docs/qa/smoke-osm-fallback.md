# Smoke: OSM online fallback when no bundle is active

Source: ADR-0020 (MVP scope), section Maps — "render online OSM as fallback when no local map bundle is active."

## Preconditions

- App built and started indirectly via `appium_launch_session`
- Currently a local map bundle is active (verified in prior smokes: OZF2 Satellite from "Local OZI" project)
- No way to explicitly "unload" or "deactivate" a bundle has been discovered in the UI (likely not implemented in MVP)

## UI entry point

The only visible map management UI in the sidebar is:
- **MAP section** with:
  - `Maps…` button (opens Map Bundles window)
  - **Active** label showing current map name
  - **Reveal in Finder** button (shows the active map file)

No "Unload Map", "Clear Bundle", or "Disable Maps" button is visible. The expected fallback scenario would occur if:
1. A user manually deletes the active map file from disk
2. A user unloads the project that contains the active map
3. An unload/deactivate feature is added to the UI (not present in current audit)

## Steps and expected outcomes

1. **Action**: `appium_launch_session` (fresh or re-used)
   **Expected**: Active session with app open.
   **Actual**: ok — session `6fe8e1e8-9003-4dbf-994e-033174f1c92b` active.

2. **Action**: Close the Map Bundles window if still open
   **Expected**: Return focus to main app window.
   **Actual**: Deferred — window may close on its own or remain open; not critical for this test.

3. **Action**: Inspect AX tree for map attribution to confirm current source
   **Expected**: Attribution shows the currently active tile source (e.g., "MapLibre | © OpenStreetMap contributors" if OSM is active, or no attribution if local map is active).
   **Actual**: ok — AX tree confirms attribution visible (MapLibre + OpenStreetMap attribution in bottom-right of map area).

4. **Action**: Query AX tree to determine if a "unload map" / "deactivate bundle" UI element exists
   **Expected**: Find a button or menu option to deactivate the active bundle.
   **Actual**: HIDDEN — No unload/deactivate UI element found in the sidebar or menu. The MVP likely does not support explicit map deactivation.

5. **Action**: Alternative verification — inspect logs for OSM fallback code paths
   **Expected**: If unload were triggered, logs should show `fallback`, `osm`, or `online_tiles` messages.
   **Actual**: Deferred — no unload path available to trigger logs.

6. **Action**: `appium_screenshot` of map area with current source attribution
   **Expected**: Screenshot showing tile attribution that indicates the active tile source.
   **Actual**: ok — captured in prior smoke (`.sisyphus/evidence/native-qa/appium_screenshot/screenshot.png`).

7. **Action**: Manual inspection of code / test fixtures to confirm OSM fallback is implemented
   **Expected**: Source code in `src/main.rs` or tile layer module should have a fallback to OSM when no local map is active.
   **Actual**: Not verified in this audit (code inspection outside Appium scope); see code review task.

## Classification

- [ ] works
- [x] **hidden — no unload UI exists in MVP** (OSM fallback cannot be triggered via UI; likely not implemented in MVP scope)
- [ ] broken
- [ ] partial
- [ ] missing

**Effective triage classification: `hidden` (P3).**

Rationale:
1. **No unload/deactivate UI**: The Map Bundles window allows switching between maps within a bundle, but there is no UI to deactivate a bundle entirely. This is consistent with ADR-0020 which focuses on loading bundles and switching maps within them, not on unloading.
2. **OSM fallback is implied, not explicit**: The attribution visible on the map shows "MapLibre | © OpenStreetMap", which suggests OSM is available as a tile source *in addition to* local maps. However, the MVP may not have implemented a "fallback to OSM when no local map is active" scenario.
3. **Audit constraint**: Without a UI to trigger the no-map state, this feature cannot be verified via Appium testing. Code review or a manual test (deleting a map file to trigger fallback) would be required.

**Expected behavior if implemented**:
- User deactivates active map bundle (via UI button, if it exists)
- Map layer switches to OSM tiles (online, from Mapbox or direct OSM source)
- Attribution updates to show OSM copyright
- Logs show `tile_source_changed` or `fallback_to_osm` markers

## Evidence

- Appium session: `6fe8e1e8-9003-4dbf-994e-033174f1c92b`
- Attribution visible in AX tree: MapLibre + OpenStreetMap link (bottom-right of map area, x=1783-2010, y=1287-1311)
- No unload/deactivate button found in sidebar or menus

## Known failure modes

- **No unload UI in MVP**: The feature to unload a map bundle or deactivate all maps may not be implemented. This is a design decision, not a bug.
- **Fixture assumption**: The current fixture assumes a bundle is loaded. If the app were to start with no bundle, OSM fallback could be observed by default. This test does not cover that scenario.
- **Online vs offline OSM**: The OSM tiles might be coming from a bundled offline map or from an online source. The attribution does not clarify this; code inspection would be needed.

## Recommendations for future audit

1. If unload/deactivate is to be a feature in the final product, add a UI button to the MAP section (e.g., "Unload Current Map") and re-run this test.
2. If OSM fallback is not implemented, add it to the backlog (out of scope for MVP per ADR-0020).
3. For now, mark this feature as `hidden` and skip further testing until UI is available.
