## ADDED Requirements

### Requirement: Tauri event listeners are owned by the root layout, one per event type

The frontend SHALL register exactly one `listen()` subscription per Tauri event type that drives store updates (`state-changed`, `download-progress`, `bundle-progress`, `bundle-file-ready`, `projects-chunk`). All such subscriptions SHALL live in `src/routes/+layout.svelte` and SHALL write into module-level stores in `src/lib/stores.ts`. Page-level components SHALL consume those stores; they SHALL NOT re-register `listen()` for the same event types.

#### Scenario: Single listener per event type

- **WHEN** a static scan of the `src/` tree counts `listen(<EVENT_NAME>, ...)` registrations grouped by event name
- **THEN** each of `state-changed`, `download-progress`, `bundle-progress`, `bundle-file-ready`, `projects-chunk` appears exactly once

#### Scenario: Page-level state does not duplicate layout listeners

- **WHEN** the bundle-loader page (`+page.svelte`) mounts
- **THEN** it does not call `listen()` for any of the layout-owned event types; instead it reads from stores fed by the layout's listeners

### Requirement: MapView re-renders only the slice of state that changed

`MapView.svelte` SHALL subscribe to three independent slice indicators — `activeMapRef`, `tracksFingerprint`, `waypointsFingerprint` — derived from `AppState` in `src/lib/stores.ts`. Each indicator SHALL change only when its corresponding domain slice changes. `MapView` SHALL run its `applyActiveMap` / `getTracksGeojson + updateTracksLayer` / waypoint-marker reconciliation paths only when the respective indicator changes from its last applied value.

#### Scenario: Download progress does not refresh tracks

- **WHEN** a bundle download is in progress and `download-progress` events are arriving at 5+ events per second
- **THEN** the IPC command `get_tracks_geojson` is invoked at most once for the entire download (matching the count of true track-state changes), not once per progress event

#### Scenario: Adding a waypoint does not rebuild every marker

- **WHEN** the user adds a single waypoint in a project that already has 50 waypoints across two layers
- **THEN** at most one new MapLibre `Marker` instance is constructed; the existing 50 markers are not destroyed and recreated

#### Scenario: Switching the active map does not touch waypoint markers

- **WHEN** the user switches the active map within the same bundle
- **THEN** the `activeMapRef`-driven effect updates the tile source and the waypoint markers are not destroyed or recreated

### Requirement: Waypoint markers are reconciled incrementally on the map

The waypoint-rendering path SHALL maintain its internal `Map<string, Marker>` (keyed by `${layerId}:${waypointId}`) and reconcile changes by:

- creating markers only for keys present in the incoming state and absent locally,
- removing markers only for keys present locally and absent in the incoming state,
- updating the coordinates / symbol / name only for existing markers whose corresponding fields differ.

A full clear-and-recreate path MAY exist as an explicit debugging affordance but SHALL NOT run on normal state changes.

#### Scenario: Waypoint coordinate update reuses the existing marker

- **WHEN** the user drags a waypoint and the new coordinates arrive via state update
- **THEN** the marker's `setLngLat(...)` is called on the existing marker instance; no new `Marker` is constructed and the old one is not removed

#### Scenario: Waypoint deletion removes one marker

- **WHEN** the user deletes a waypoint from a layer of three waypoints
- **THEN** exactly one marker is removed and the other two markers remain unchanged

### Requirement: Bundle-loader project filter debounces input

The project filter `<input>` in `src/routes/+page.svelte` SHALL feed a debounced state with a delay of at least 120 milliseconds. The `$derived` filtered list SHALL read from the debounced state, not from the raw input value.

#### Scenario: Rapid typing produces at most one filter pass per debounce window

- **WHEN** the user types six characters into the filter input within 200 milliseconds
- **THEN** the filter pass runs no more than twice in that window — once for the trailing debounce flush and at most one intermediate

### Requirement: Startup `loadProjects()` runs exactly once per session

The application SHALL invoke `loadProjects()` exactly once per session start. The invocation SHALL be initiated from the root layout's `onMount`. The bundle-loader page SHALL NOT independently call `loadProjects()` at mount time. The user-facing refresh button SHALL remain a valid additional entry point for re-running `loadProjects()`.

#### Scenario: Cold start fires loadProjects once

- **WHEN** the application starts and the user reaches the bundle loader for the first time without clicking refresh
- **THEN** the IPC command `load_projects` is invoked exactly once

#### Scenario: User refresh fires loadProjects again

- **WHEN** the user clicks the refresh button in the bundle loader
- **THEN** `load_projects` is invoked once more, in addition to the startup invocation
