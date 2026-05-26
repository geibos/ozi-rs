## ADDED Requirements

### Requirement: Library Tracks tab header exposes Import GPX, Import PLT, and Create Track affordances

The Tracks tab header inside `src/components/library/TracksTab.svelte` SHALL host, alongside the existing active-track-layer `Select`, three action buttons rendered as shadcn icon `Button`s with Lucide icons at `strokeWidth={1.5}`:

1. **Import GPX** — Lucide `Upload` icon; tooltip text `"Import GPX"`. Click SHALL open the native file dialog via `@tauri-apps/plugin-dialog`'s `open` with `multiple: false`, `directory: false`, and an extension filter `[{ name: "GPX", extensions: ["gpx"] }]`. When the user selects a path, the handler SHALL invoke `importGpx(path)` from `src/lib/api.ts`. Errors SHALL surface through `toast.error` (svelte-sonner) with `description: String(err)`, matching the error-handling shape of the existing export handlers in the same component.
2. **Import PLT** — Lucide `Upload` icon; tooltip text `"Import PLT"`. Click flow SHALL be identical to Import GPX except the extension filter SHALL be `[{ name: "PLT", extensions: ["plt"] }]` and the API call SHALL be `importPlt(path)`.
3. **Create Track** — Lucide `Pencil` icon; tooltip text `"Create track"`. Click flow SHALL toggle drawing mode by reading the `drawingModeActive` writable store from `src/lib/stores.ts`. When `$drawingModeActive === false`, the handler SHALL set `drawingTrackLayerId` to `$activeTrackLayerId`, call `createEmptyTrack($activeTrackLayerId)` to materialise the receiving track in the domain, and then set `drawingModeActive` to `true`. When `$drawingModeActive === true`, the handler SHALL set `drawingModeActive` to `false`. While `$drawingModeActive` is `true`, the button SHALL switch its icon to Lucide `Check`, switch its tooltip to `"Finish track"`, and surface a visible label `Done (N points)` adjacent to the icon — where `N` is the current point count of the in-progress drawing track, read from the same drawing-state plumbing the legacy `Sidebar.svelte` consumed. While drawing is active, the Import GPX and Import PLT buttons and the track-layer `Select` SHALL be `disabled`.

The three buttons SHALL all be `disabled` when `$activeTrackLayerId === null`. The entire icon-button row SHALL be hidden when the project contains zero track layers (mirroring the existing `{#if trackLayers.length > 0}` guard around the track-layer `Select`).

The buttons SHALL NOT introduce any new IPC commands; they reuse `importGpx`, `importPlt`, `createEmptyTrack` already declared in `src/lib/api.ts`, and the stores `drawingModeActive`, `drawingTrackLayerId`, `activeTrackLayerId` already declared in `src/lib/stores.ts`.

#### Scenario: Import GPX button surfaces a Tauri file dialog and loads a track

- **WHEN** the user is at `/project` with at least one track layer active AND the user clicks the Import GPX icon button in the Tracks tab header AND the user selects a `.gpx` file in the resulting file dialog
- **THEN** the application invokes `importGpx(path)` AND on success the Tracks tab refreshes to include the newly imported track AND on failure a `svelte-sonner` toast with `severity: error` displays the error message

#### Scenario: Import PLT button mirrors the GPX flow with a `.plt` filter

- **WHEN** the user clicks the Import PLT icon button in the Tracks tab header
- **THEN** the native file dialog opens with the extension filter restricting visible files to `.plt` AND selecting a file invokes `importPlt(path)`

#### Scenario: Create Track button enters drawing mode and shows live point count

- **WHEN** the user clicks the Create Track icon button while `$drawingModeActive === false` AND `$activeTrackLayerId !== null`
- **THEN** the handler sets `drawingTrackLayerId` to the current `$activeTrackLayerId`, calls `createEmptyTrack` against that layer, and flips `drawingModeActive` to `true` AND the button's icon switches to `Check` AND a label `Done (N points)` becomes visible next to the icon where `N` is the live point count of the drawing track AND the Import GPX, Import PLT, and track-layer `Select` controls become `disabled`

#### Scenario: Clicking Create Track again ends drawing mode

- **WHEN** the user clicks the Create Track button (now showing `Done (N points)`) while `$drawingModeActive === true`
- **THEN** `drawingModeActive` flips back to `false` AND the button reverts to its `Pencil` icon and `"Create track"` tooltip AND the Import GPX, Import PLT, and track-layer `Select` controls become enabled again

#### Scenario: Buttons hide when the project has no track layers

- **WHEN** the active project contains zero track layers
- **THEN** the entire icon-button row in the Tracks tab header SHALL be absent from the DOM, identical to the existing handling of the track-layer `Select`

#### Scenario: Buttons disable when no track layer is selected

- **WHEN** the project has at least one track layer but `$activeTrackLayerId === null`
- **THEN** the Import GPX, Import PLT, and Create Track buttons SHALL render but SHALL be `disabled`

### Requirement: Library Waypoints tab header exposes an Add Waypoint mode toggle

The Waypoints tab header inside `src/components/library/WaypointsTab.svelte` SHALL host, alongside the existing active-waypoint-layer `Select`, a single action button: **Add Waypoint** — a shadcn icon `Button` using the Lucide `MapPin` icon at `strokeWidth={1.5}` with tooltip text `"Add waypoint"`.

Click SHALL toggle the `addWaypointMode` writable store from `src/lib/stores.ts`. While `$addWaypointMode === true`, the button SHALL render in a pressed / active visual state distinguishable from the resting state (e.g. via `variant="default"` instead of `variant="ghost"`, or via a `data-state="on"` attribute the shadcn `Toggle` primitive applies). The button SHALL NOT call any IPC; the existing map-click handler in `MapView.svelte` reads `addWaypointMode` and is responsible for creating the waypoint at the clicked coordinate.

The button SHALL be `disabled` when `$activeWaypointLayerId === null`. The button SHALL be hidden when the project contains zero waypoint layers (mirroring the existing `{#if waypointLayers.length > 0}` guard around the waypoint-layer `Select`).

#### Scenario: Add Waypoint button toggles the addWaypointMode store

- **WHEN** the user clicks the Add Waypoint icon button in the Waypoints tab header while `$addWaypointMode === false`
- **THEN** `addWaypointMode` flips to `true` AND the button renders in its pressed / active visual state AND clicking again flips `addWaypointMode` back to `false` AND the button returns to its resting visual state

#### Scenario: Add Waypoint button is disabled when no waypoint layer is selected

- **WHEN** the project has at least one waypoint layer but `$activeWaypointLayerId === null`
- **THEN** the Add Waypoint button SHALL render but SHALL be `disabled`

#### Scenario: Add Waypoint button is hidden when the project has no waypoint layers

- **WHEN** the active project contains zero waypoint layers
- **THEN** the Add Waypoint button SHALL be absent from the DOM, identical to the existing handling of the waypoint-layer `Select`
