## 1. Backend invariant

- [x] 1.1 In `src-tauri/src/domain/project.rs`, change `Project::default()` (and any other empty-construction path) to populate one default `TrackLayer` named `"Tracks"` and one default `WaypointLayer` named `"Waypoints"`
- [x] 1.2 Add a unit test in `domain/project.rs` asserting that `Project::default()` has `track_layers().len() == 1` and `waypoint_layers().len() == 1` with the expected names
- [x] 1.3 Audit other construction paths (bundle-driven new project, etc.) and ensure each produces a project that satisfies the invariant; add a fixture-based unit test per path

## 2. Load-path normalization

- [x] 2.1 In the `.ozp` deserialization path (`src-tauri/src/infrastructure/...`), after deserializing into `Project`, check `track_layers().len()` and `waypoint_layers().len()`; if either is zero, append the corresponding default layer
- [x] 2.2 Add a unit test that loads a hand-crafted `.ozp` with zero layers and asserts the returned project has the default layers appended
- [x] 2.3 Add a unit test that loads a `.ozp` with existing layers (one track, two waypoint) and asserts no defaults are appended
- [x] 2.4 Add a regression test asserting that loading-then-saving a legacy `.ozp` produces a file whose layer set matches the normalized in-memory state

## 3. Frontend cleanup (now that nulls are impossible during edit flows)

- [x] 3.1 In `src/components/Sidebar.svelte:toggleTrackDrawingMode`, remove `if (layerId === null) return;` and treat `$activeTrackLayerId` as non-null when `$appState` reports a current project
- [x] 3.2 In `src/components/MapView.svelte:handleMapClickForWaypoint`, remove `if (layerId === null) return;` (same reasoning)
- [x] 3.3 Verify `src/lib/stores.ts:syncActiveLayer` still gracefully handles the empty array (still possible during the brief window before `appState` settles); do not weaken that guard
- [x] 3.4 Update `src/lib/types.ts` if `active_track_layer_id` / `active_waypoint_layer_id` DTO fields gain stronger non-null semantics during an open project (keep the type `number | null` but document the invariant inline)

## 4. Verification

- [ ] 4.1 Manual: launch the app on a fresh bundle, click "Add Waypoint", click on the map, confirm a waypoint marker appears
- [ ] 4.2 Manual: on the same fresh bundle, click "Create Track", click three points on the map, double-click to finish, confirm the new track appears in Tracks panel
- [ ] 4.3 Manual: load a known legacy `.ozp` (if available) and confirm the workspace opens with at least one track and one waypoint layer visible in the layer selectors
- [x] 4.4 `just test` passes (Rust unit tests added in 1.2, 2.2, 2.3, 2.4)
- [x] 4.5 `just ci` passes (clippy, check, lint, test)
- [x] 4.6 `openspec validate auto-create-default-layers --strict` passes
