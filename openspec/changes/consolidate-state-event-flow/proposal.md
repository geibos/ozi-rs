## Why

The frontend's reactive event flow has three concrete symptoms a user feels:

1. **Filter typing lags.** Typing in the project filter in the bundle loader feels like every keystroke triggers a network round-trip, even though the filter is a pure client-side `Array.prototype.filter`. The illusion is real because, in parallel with typing, the app is fielding `state-changed`, `download-progress`, `bundle-progress`, `bundle-file-ready`, and `projects-chunk` events — each of which mutates a store, which re-derives `filtered`, which re-renders the list. And the `state-changed` listener cascades into a full `getTracksGeojson` IPC + complete `refreshWaypointMarkers` rebuild inside `MapView` (`MapView.svelte:656`).
2. **MapView shudders during normal use.** The `$effect` in `MapView.svelte:656` runs on every `$appState` change. Each run calls `getTracksGeojson()` over IPC and rebuilds every waypoint marker by clearing them all and recreating them. During a download or any flurry of `state-changed` events, this means the entire track GeoJSON is re-fetched and every marker is destroyed and recreated multiple times per second.
3. **Listeners are registered in two places.** `download-progress` is `listen()`-ed once in `+layout.svelte:37` and once in `+page.svelte:79`. Each event triggers the handler chain twice. `state-changed` is listened in `+layout.svelte:33` only, but `+page.svelte` separately runs `appState.refresh()` and `loadProjects()` on mount — duplicate kicks at startup.

These are not three independent bugs; they're three faces of one design choice: the frontend treats `$appState` as a coarse, opaque blob and replaces it whole-cloth on every backend event. There is no notion of "only the download progress changed" vs "a track was added".

## What Changes

- Tauri event listeners SHALL be registered in exactly one place per event type. `download-progress` SHALL no longer be `listen()`-ed in `+page.svelte`; the single layout-level listener is the source of truth. Same single-listener rule applies to `state-changed`, `bundle-progress`, `bundle-file-ready`, and `projects-chunk`.
- `MapView` SHALL NOT re-fetch track GeoJSON or rebuild waypoint markers on every `$appState` change. It SHALL maintain a local notion of "what tracks/waypoints/active-map are currently rendered" and only re-fetch / re-render when the relevant slice of state actually changes.
- `refreshWaypointMarkers` SHALL update markers incrementally: add new markers, remove gone markers, update coordinates of existing markers — instead of `clearWaypointMarkers()` + recreate-all on every call.
- The project filter input in the bundle loader SHALL debounce its `$derived` recomputation (≥120ms) so that fast typing produces at most one filter pass per debounce window.
- Startup IPC calls SHALL not be duplicated. `loadProjects()` SHALL be invoked exactly once at session start (from the layout, not also from the page).

## Capabilities

### New Capabilities
- _none_

### Modified Capabilities
- `ui-shell`: consolidate the Tauri-event listener topology to a single source per event; specify the MapView re-render contract; specify the debounced project filter; specify single-shot `loadProjects` at startup.

## Impact

- **Frontend**: `src/routes/+layout.svelte` (sole owner of event listeners), `src/routes/+page.svelte` (drops duplicate `download-progress` listener and duplicate `loadProjects` call; adds debounce wrapper around `projectFilter` derivation), `src/components/MapView.svelte` (adds slice-aware effects, incremental waypoint-marker reconciliation), `src/lib/stores.ts` (may grow derived stores or memoized selectors for the slices MapView reads).
- **Backend**: none. This is purely a frontend reactivity refactor.
- **Spec evidence**: smoke that typing a 12-character filter during an active download does not stutter; FPS overlay (F3) stays close to 60fps during downloads; counted IPC calls to `get_tracks_geojson` during a single waypoint-add operation is `1`, not `N`.
- **Risk**: medium. This refactor touches the central reactivity wiring. Possible regressions: stale markers after a state change is missed, duplicate markers after an incremental update bug. Mitigated by step-wise migration and per-step manual smoke checks (see tasks).
