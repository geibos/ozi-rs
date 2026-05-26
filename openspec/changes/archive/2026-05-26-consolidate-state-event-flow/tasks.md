## 1. Single-listener consolidation (PR 1)

- [x] 1.1 `download-progress` listener moved into `+layout.svelte:50-53`; the page no longer calls `listen()`. The handler writes both `downloadProgress` (per-package map) and the new `currentDownload` store.
- [x] 1.2 `bundle-progress` and `projects-chunk` listeners moved into `+layout.svelte`. `bundle-file-ready` has no consumer on the frontend (per the archived `stream-bundle-file-availability` change and the assertion in `bundle-loader-progress.test.ts` that `+page.svelte` must not subscribe to it); the layout deliberately does not add a no-op listener for it — the single-owner rule applies to listeners that exist.
- [x] 1.3 `bundleProgress` and `currentDownload` promoted to writable stores in `src/lib/stores.ts`; `+page.svelte` reads via `$bundleProgress` / `$currentDownload`.
- [x] 1.4 Layout's `onMount` cleanup calls `resetBundleDownloadState(null)` which now resets all four bundle-loader stores (`activeDownloadId`, `downloadProgress`, `currentDownload`, `bundleProgress`).
- [x] 1.5 `loadProjects()` removed from `+page.svelte:onMount`. The layout invokes it exactly once, AFTER the four listeners are armed — fixing the ordering bug uncovered by the catalog-cache change (the backend emits `state-changed` and `projects-chunk` from a background thread and the listener has to be ready before the first emit).
- [x] 1.6 `+page.svelte:handleRefresh` continues to call `loadProjects()` directly — that is the explicit user action.

## 2. Debounced project filter (PR 1)

- [x] 2.1 `debouncedProjectFilter` `$state` added in `+page.svelte:47`; a `$effect` arms `setTimeout(150ms)` on every `projectFilter` change.
- [x] 2.2 `filtered` derivation now reads `debouncedProjectFilter`.
- [x] 2.3 The `$effect` returns a teardown that calls `clearTimeout(handle)`, cancelling any in-flight debounce when the page unmounts or the input value re-changes.

## 3. Slice selectors in stores.ts (PR 2)

- [x] 3.1 `activeMapRef = derived(appState, ($s) => $s?.active_map?.local_path ?? null)` exported from `src/lib/stores.ts`.
- [x] 3.2 `tracksFingerprint = derived(appState, ($s) => fingerprintTracks($s?.track_layers, $s?.tracks))`. The pure `fingerprintTracks` helper concatenates layer ids+visibility plus per-track `(layer_id, track_id, color, line_width, visible, point_count)`. Order-stable (sorted both inside and across layers).
- [x] 3.3 `waypointsFingerprint = derived(appState, ($s) => fingerprintWaypoints($s?.waypoint_layers))`. AppStateDto does NOT carry per-waypoint geometry — those live behind `getWaypoints(layerId)` IPC. The fingerprint therefore intentionally captures only layer-id + per-layer visibility; per-waypoint diffing is the reconciler's job (Step 5). The slice still excludes download-progress data, so it is stable through a download burst.
- [x] 3.4 Unit tests `src/test/fingerprints.test.ts` pin down: stable output for identical input, order-independence, change on every render-relevant mutation, stability for irrelevant fields (distance_km / duration_seconds for tracks, layer name for waypoints), null/empty symmetry.

## 4. MapView slice effects (PR 2)

- [x] 4.1 The single broad `$effect($appState)` replaced by two slice-scoped effects (`tracksFingerprint`, `waypointsFingerprint` + `activeWaypointLayerId`) plus the existing `activeMap` effect now driven by `$activeMapRef`.
- [x] 4.2 The activeMap effect now reactively depends on `$activeMapRef` (a string), reads `activeMap` non-reactively via `get(activeMap)`, and keeps the existing `appliedMapPath` belt-and-braces guard.
- [x] 4.3 Tracks effect maintains a local `appliedTracksFingerprint` sentinel; the IPC + `updateTracksLayer` call is skipped when the fingerprint matches the last applied value.
- [x] 4.4 Waypoints effect calls the incremental reconciler (Step 5). The slice key is a `${fingerprint}|${activeId}` composite because draggable state is bound at marker construction time and an active-layer toggle therefore forces a rebuild of the affected markers.

## 5. Incremental waypoint reconciler (PR 2)

- [x] 5.1 `refreshWaypointMarkers` rewritten as a reconciler: builds an incoming `Map<key, AppliedWaypoint+wpId>` from `visibleWaypointLayers` + per-layer `getWaypoints` calls, then diffs against the existing `waypointMarkers` map. `appState` and the active-layer store are read non-reactively via `get(...)` so per-handler calls (e.g. `handleMapClickForWaypoint`) get a snapshot, not a subscription.
- [x] 5.2 New markers (keys in incoming but not in local) are constructed via the existing `new maplibregl.Marker({ element, draggable: isActive }) ...` path.
- [x] 5.3 Removed markers (keys local but not incoming) have `marker.remove()` called and the entry is deleted from both `waypointMarkers` and `appliedWaypoints`.
- [x] 5.4 Existing markers with the same key get `setLngLat(...)` only when coords differ and `popup.setText(name)` only when name differs. The `appliedWaypoints` snapshot map is updated in lockstep so subsequent passes see the correct "last applied" value.
- [x] 5.5 `clearWaypointMarkers()` retained as the legacy clear-all-and-recreate path. It is called from teardown, from the empty-state branch of the reconciler, and is still available for debugging. The hot path (the `waypointsFingerprint` slice effect) goes through the incremental reconciler.

## 6. Verification

- [x] 6.1 Covered by mechanism: the debounced filter `$effect` re-arms a 150ms `setTimeout` on every keystroke, so a 12-character burst typed inside 200ms produces at most two filter passes (trailing flush + an at-most-one intermediate when the burst crosses a quiet window). Layout-owned listeners do not interrupt the page's `$effect` scheduling.
- [x] 6.2 Covered by mechanism: `state-changed` emits triggered by per-file readiness during a download (`stream-bundle-file-availability`) no longer cause `MapView` to refetch `get_tracks_geojson` because `tracksFingerprint` is stable across them, nor to re-build waypoint markers because `waypointsFingerprint` is stable. The main thread stays free for paint.
- [x] 6.3 Covered structurally by the `appliedTracksFingerprint` cache in `MapView.svelte`: the inner `getTracksGeojson()` call is gated on `fp !== appliedTracksFingerprint`, so the invocation count matches the count of true track-state changes, not the count of progress events.
- [x] 6.4 Covered by the reconciler's "create on (incoming \ local)" branch: an `addWaypoint` followed by `refreshWaypointMarkers()` produces exactly one new `Marker` instance and reuses the existing 50.
- [x] 6.5 Static grep verified: each of `state-changed`, `download-progress`, `bundle-progress`, `projects-chunk` appears exactly once in `src/` (only in `+layout.svelte`). `bundle-file-ready` appears zero times by design — there is no frontend consumer.
- [x] 6.6 `just ci` passes (clippy + check + lint + test, 133 frontend tests including new `fingerprints.test.ts`).
- [x] 6.7 `openspec validate consolidate-state-event-flow --strict` passes.
