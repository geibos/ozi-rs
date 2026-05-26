## 1. Single-listener consolidation (PR 1)

- [ ] 1.1 Move the `download-progress` listener from `src/routes/+page.svelte:79` into `src/routes/+layout.svelte`; keep the layout-level listener as the sole subscription
- [ ] 1.2 Move the `bundle-progress`, `bundle-file-ready`, and `projects-chunk` listeners from `+page.svelte` into `+layout.svelte`; their payloads now write into module-level stores instead of page-local `$state`
- [ ] 1.3 Promote `+page.svelte:bundleProgress` and `+page.svelte:currentDownload` to new stores in `src/lib/stores.ts` (e.g. `bundleProgress`, `currentDownload`); update `+page.svelte` to read from them
- [ ] 1.4 In `+layout.svelte:onDestroy`, reset the new stores so the next session starts clean
- [ ] 1.5 Remove the `loadProjects()` call from `+page.svelte:onMount`; keep the call in `+layout.svelte:onMount`
- [ ] 1.6 Keep the refresh-button path in `+page.svelte:handleRefresh` calling `loadProjects()` — that is the explicit user action

## 2. Debounced project filter (PR 1)

- [ ] 2.1 Add a `debouncedProjectFilter` local `$state` in `+page.svelte` and a `setTimeout(150ms)` updater driven by changes to `projectFilter`
- [ ] 2.2 Switch the `$derived filtered` to read from `debouncedProjectFilter` instead of `projectFilter`
- [ ] 2.3 Cancel the pending timeout on component teardown

## 3. Slice selectors in stores.ts (PR 2)

- [ ] 3.1 Add `activeMapRef = derived(appState, $s => $s?.active_map?.local_path ?? null)` to `src/lib/stores.ts`
- [ ] 3.2 Add `tracksFingerprint = derived(appState, $s => fingerprintTracks($s?.track_layers))` returning a stable string; design the fingerprint to include layer IDs, track IDs, track-point counts, and style fields that MapView actually renders against
- [ ] 3.3 Add `waypointsFingerprint = derived(appState, $s => fingerprintWaypoints($s?.waypoint_layers))` with the equivalent shape for waypoints (per-layer ID, per-waypoint ID, coordinates, symbol, visibility)
- [ ] 3.4 Add unit tests in `src/test/` pinning down each fingerprint function: same input ⇒ same output; relevant mutation ⇒ different output; irrelevant mutation (e.g., download progress) ⇒ unchanged output

## 4. MapView slice effects (PR 2)

- [ ] 4.1 Replace the broad `$effect($appState)` in `MapView.svelte:656` with three narrower `$effect` blocks, each reading one slice selector
- [ ] 4.2 The `activeMapRef` effect retains the existing `applyActiveMap` flow (it already de-dupes via `appliedMapPath`)
- [ ] 4.3 The `tracksFingerprint` effect calls `getTracksGeojson()` + `updateTracksLayer(map, geojson)` only when fingerprint differs from last applied; cache last-applied fingerprint locally in the component
- [ ] 4.4 The `waypointsFingerprint` effect calls the new incremental reconciler (step 5)

## 5. Incremental waypoint reconciler (PR 2)

- [ ] 5.1 Refactor `refreshWaypointMarkers` in `MapView.svelte` to a reconciler: compute incoming `Map<key, WaypointSnapshot>` from `$visibleWaypointLayers` + `getWaypoints(layerId)` calls; diff against the existing `waypointMarkers: Map<string, Marker>`
- [ ] 5.2 For each (layer_id, waypoint_id) present in incoming but absent locally, create a new marker (current `new maplibregl.Marker(...)` path)
- [ ] 5.3 For each key absent in incoming but present locally, `remove()` the marker and delete from the map
- [ ] 5.4 For each key in both: if coordinates differ, call `setLngLat`; if symbol or name differ, update marker element / popup text in place
- [ ] 5.5 Keep the old `clearWaypointMarkers()` + recreate path available behind an explicit debug-only entry point (e.g., bound to the dev console) but not on the hot path

## 6. Verification

- [ ] 6.1 Manual: open a bundle, start a download, type a 12-character filter string while the download runs; confirm the typing is smooth (no per-keystroke stutter)
- [ ] 6.2 Manual: open F3 FPS overlay during a download; confirm FPS does not collapse below ~30 during the download
- [ ] 6.3 Manual / instrumentation: log `get_tracks_geojson` invocations during a 30-second download; confirm count matches the number of actual track-state changes, not the number of progress events
- [ ] 6.4 Manual: add a single waypoint in a project of 50; confirm via DevTools that only one new `.waypoint-marker` element is added to the DOM (the others' elements are unchanged)
- [ ] 6.5 Static check: grep for `listen("state-changed"`, `listen("download-progress"`, `listen("bundle-progress"`, `listen("bundle-file-ready"`, `listen("projects-chunk"`; each pattern appears at most once
- [ ] 6.6 `just ci` passes (clippy, check, lint, test) including new fingerprint unit tests
- [ ] 6.7 `openspec validate consolidate-state-event-flow --strict` passes
