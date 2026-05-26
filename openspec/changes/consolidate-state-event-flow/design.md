## Context

The frontend talks to the backend through a small set of Tauri events:

- `state-changed` — fired after most mutations; frontend calls `getAppState()` to pull the new `AppStateDto`.
- `download-progress` — per-file byte progress during a LizaAlert download.
- `bundle-progress`, `bundle-file-ready` — aggregate bundle download progress.
- `projects-chunk` — streamed chunks of the LizaAlert project catalog.

Today (`+layout.svelte:21`, `+page.svelte:63`):

- `state-changed` is listened in the layout (single).
- `download-progress` is listened in the layout AND in the page (double).
- `bundle-progress`, `bundle-file-ready`, `projects-chunk` are listened in the page (single).
- The layout AND the page both call `loadProjects()` on mount (double).

`MapView.svelte` runs an `$effect` (`MapView.svelte:656`) on every `$appState` change that:

1. Calls `getTracksGeojson()` over IPC.
2. Calls `refreshWaypointMarkers()`, which clears every waypoint marker and recreates them all.

Because `state-changed` is sometimes fired by progress mutations (active download bookkeeping), and the layout's listener calls `appState.refresh()`, this `$effect` runs on download events too. The user typing in the filter feels lag because the same JS event loop is doing IPC + DOM rebuilds in the background.

## Goals / Non-Goals

**Goals:**
- Each Tauri event has exactly one listener.
- `MapView` re-renders only the slice of state that actually changed.
- The bundle-loader filter debounces input so fast typing doesn't drive the filter on every keystroke.
- `loadProjects()` is called once per session start, not twice.

**Non-Goals:**
- Backend changes. We will not introduce typed `state-changed` payloads, version vectors, or per-mutation events. Treat backend as fixed.
- Re-architecting the store layer. We keep Svelte `writable` / `derived` and add helpers around them; we do not replace with Redux-like state management.
- Visual performance work outside the events-and-renders path (e.g., MapLibre tile cache tuning). That's a separate change.
- Fixing track-not-rendering or point-not-placing functional bugs — those belong to change B.

## Decisions

### Decision 1: One owner per event, layout owns lifetime

The root layout (`+layout.svelte`) SHALL be the sole owner of every Tauri event listener that drives store updates: `state-changed`, `download-progress`, `bundle-progress`, `bundle-file-ready`, `projects-chunk`. The page-level component (`+page.svelte`) SHALL only consume the stores those listeners feed, never re-listen.

The currently-page-local state in `+page.svelte` (`bundleProgress`, `currentDownload`) SHALL become module-level stores in `stores.ts`, written by the layout's listeners and read by `+page.svelte`. This converts a private-component state into shared, single-writer state.

**Rationale**: The layout outlives every page; route changes destroy and recreate the page, which is exactly the wrong place to own long-lived IPC plumbing. Today's setup happens to work because the active-download lifetime mostly overlaps with the `/` route, but it's fragile to navigation between `/` and `/project`.

**Alternatives considered**:
- _Keep listeners on the page that uses them, but de-duplicate_: still couples listener lifetime to a route. Rejected.
- _Move listeners into `stores.ts` initialization_: stores then have implicit module-load side effects, which is hard to reason about. Rejected.

### Decision 2: MapView reacts to slices, not the blob

Introduce a small number of slice selectors in `stores.ts`:

- `tracksFingerprint`: a string derived from `track_layers` describing identity-and-shape (e.g., concatenated `${layer.id}:${layer.track_ids.join(",")}` plus a hash of relevant style fields). It changes if and only if a track was added, removed, renamed, restyled, or had its geometry edited.
- `waypointsFingerprint`: same idea for waypoints.
- `activeMapRef`: the `local_path` of the active map (already partially implemented via `appliedMapPath` in MapView; promote to a store).

`MapView`'s big `$effect($appState)` SHALL be split into three effects, each subscribing to one fingerprint. Each effect compares the new fingerprint to the last applied and runs the corresponding refresh path only on change.

**Rationale**: Svelte's `derived` already memoizes by `===` on primitives. Returning a string fingerprint gives us free shallow equality at the store layer, which is the cheapest correct way to skip work.

**Trade-off**: Fingerprints must include every field MapView actually re-renders against. Forgetting a field means stale rendering. We mitigate this with explicit unit tests on the selector functions (separate from MapView itself).

**Alternatives considered**:
- _Backend emits typed state-changed payloads_: clean, but requires Rust changes and is out of scope per Non-Goals.
- _Deep structural equality in MapView_: O(N) per state change, defeats the purpose.

### Decision 3: Waypoint markers reconcile incrementally

`refreshWaypointMarkers()` SHALL be rewritten as a reconciler:

```text
incoming set = { (layer_id, waypoint_id) → coords, name, symbol, isActive }
current set  = { (layer_id, waypoint_id) → marker }

for each key in incoming \ current: create marker
for each key in current \ incoming: remove marker
for each key in incoming ∩ current: if coords / symbol / name differ → update; else no-op
```

Marker construction (DOM element, event handlers) only runs on the additions. The existing `waypointMarkers: Map<string, Marker>` keyed by `${layerId}:${waypointId}` (`MapView.svelte:53`) is already the right shape — only the update logic changes.

**Rationale**: Today's clear-all-and-recreate is O(N) DOM-destroy + O(N) DOM-create + O(N) MapLibre marker setup on every state change. Reconciliation is O(N) compare + O(Δ) DOM work, which is closer to free for normal edit operations.

### Decision 4: Filter debounces by 150ms

`projectFilter` SHALL feed into a `debouncedProjectFilter` state via a 150ms `setTimeout`. The `$derived filtered` SHALL read from the debounced value, not the raw input.

150ms is the lower edge of perceptual instantaneity for typing-then-seeing-results; values below 100ms feel "live", values above 250ms feel "the app is slow".

**Alternative considered**: react to keypress with `requestIdleCallback` for filtering. Rejected because the bundle catalog can be hundreds of entries and we want predictable timing, not opportunistic.

### Decision 5: Single startup `loadProjects()`

The layout SHALL call `loadProjects()` exactly once during its `onMount`. The page SHALL NOT call `loadProjects()` from its `onMount`. The page SHALL still call `loadProjects()` via the user-facing refresh button — that path is intentional.

## Risks / Trade-offs

- **Risk**: A new mutation type lands later, MapView's fingerprint doesn't include the relevant field, and the render becomes stale. **Mitigation**: each fingerprint selector lives in `stores.ts` next to a unit test that pins down its inputs.
- **Risk**: Incremental waypoint reconciliation has a subtle bug (duplicate marker, missed removal). **Mitigation**: keep the existing `clearWaypointMarkers()` as an opt-in "force full rebuild" entry point, callable from console for debugging. Don't reach for it in the normal path.
- **Risk**: Moving page-local state to stores leaks more state into the layout's lifetime. **Mitigation**: reset the relevant stores in the layout's `onDestroy` (which only fires on app close).
- **Trade-off**: Adding fingerprint selectors is more code than the current "refresh everything" approach. Pays for itself the first time the user opens a project with a few hundred waypoints.

## Open Questions

- Q: Should the fingerprint include track-point counts so that a point-drag invalidates `tracksFingerprint`? **A (tentative)**: yes, but represented as the integer count, not the points themselves. Drag coalesces (per `undo-redo` spec) so this triggers once per drag, not once per intermediate.
- Q: Do we need any backend telemetry to confirm the new wiring doesn't miss events? **A**: no — manual smoke is enough at this scale. Telemetry would be a separate ADR.

## Migration Plan

Land in two PRs to make regressions easy to bisect:

1. **PR 1**: deduplicate listeners + single `loadProjects` + debounce filter. Pure cleanup, no behavior change.
2. **PR 2**: MapView slice effects + incremental waypoint reconciler. Behavioral but well-localized.
