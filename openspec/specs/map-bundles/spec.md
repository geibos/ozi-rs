# map-bundles Specification

## Purpose
Covers the map bundle as a unit of storage separate from the project: what a bundle directory contains, where bundles live (the bundles root and its per-bundle subdirectories), how a local bundle is opened, how the active map inside a bundle is tracked and switched, and how the bundle catalog and per-map availability behave while a download is in flight. Fetching bundles from `maps.lizaalert.ru` is specified in `lizaalert-integration`; project files are specified in `project-persistence`.

### Decision history

- ADR-0002 (2026-03-29, accepted): map bundle (a directory of georeferenced rasters, downloaded or opened locally, shared by many projects) and project are distinct concepts, and the app keeps a configurable bundles root with one subdirectory per bundle; rationale: earlier versions lost track data when the map changed and had no home for several operations over the same area. Codified as: Map bundle is a directory containing one or more georeferenced raster maps; User can open a local bundle from a chosen directory; Bundles root directory is user-configurable; Active map is tracked per project and is switchable without unloading overlays; Bundles root defaults to Documents and holds one directory per bundle. Reality note: the root chosen via `set_bundles_root` is held in memory only and re-derived from the Documents folder at every start (`src-tauri/src/application/mod.rs:565-567`, `src-tauri/src/lib.rs:28-34`; `PersistedAppSession` has no such field), so the "persists across app restarts" clause of the user-configurable requirement is not met today.
- Legacy plan `docs/superpowers/plans/2026-04-12-production-bugs-fix.md` (executed): parallel per-file downloads and incremental map availability replaced the monolithic bundle load; rationale: users waited for whole multi-gigabyte bundles before any map could be opened. Codified as: Bundle catalog and cached maps remain interactive during an in-flight download; Per-map availability inside the active bundle streams into the UI live; Bundle download progress is observable from every surface that lists the downloading project's maps. Its "pre-created hidden bundle-loader window" (Task 5) is superseded: the loader is a component mounted on the `/` route (`src/routes/+page.svelte`), and `src/lib/windows.ts` no longer exists.
- Owner decision (2026-09-19): code is primary. The persistence promise in "Bundles root directory is user-configurable" stands and the in-memory-only implementation is fixed in `revive-ui-cycle` slice 0.3 (bundles root stored in the session file and restored at startup).

## Requirements
### Requirement: Map bundle is a directory containing one or more georeferenced raster maps

The system SHALL treat a map bundle as a filesystem directory that may contain SQLite MBTiles (`*.sqlitedb`) and/or OziExplorer raster maps (`*.map` paired with `*.ozf2`). A bundle MAY include a `10-Tracks/` subfolder for exported track files.

#### Scenario: Bundle directory layout

- **WHEN** the user opens a directory that contains a `*.sqlitedb` file
- **THEN** the system recognizes the directory as a map bundle and exposes the contained map(s) as candidate map layers

#### Scenario: Mixed-format bundle

- **WHEN** a bundle directory contains both an MBTiles file and an OZF2 + `.map` pair
- **THEN** the system exposes both maps as independent candidates and the user may select either as the active map

### Requirement: User can open a local bundle from a chosen directory

The system SHALL provide a UI affordance ("Maps…" / directory picker) that lets the user select a directory and load it as the active bundle without requiring network access.

#### Scenario: Local bundle open

- **WHEN** the user selects "Maps…" and picks a directory containing a recognized bundle layout
- **THEN** the available maps in that bundle become selectable, and any subsequently chosen active map renders in the map view

### Requirement: Bundles root directory is user-configurable

The system SHALL persist a user-configurable bundles root directory that LizaAlert downloads target by default and that the bundle browser uses as its initial path.

#### Scenario: Setting bundles root

- **WHEN** the user changes the bundles root directory in app settings
- **THEN** subsequent LizaAlert downloads place bundle data under the new root, and the new value persists across app restarts

### Requirement: Active map is tracked per project and is switchable without unloading overlays

The system SHALL remember which map within the active bundle is currently selected, and SHALL allow switching the active map without unloading project tracks or waypoints.

#### Scenario: Switching active map preserves overlays

- **WHEN** the user switches the active map from a Topo MBTiles map to an OZF2 Satellite map
- **THEN** all loaded tracks and waypoints remain present and rendered over the new base map

### Requirement: User can reveal active bundle in the OS file manager

The system SHALL expose a "Reveal bundle" action that opens the active bundle's directory in the host OS file manager (Finder on macOS, Explorer on Windows).

#### Scenario: Reveal active bundle

- **WHEN** the user invokes "Reveal bundle" with an active bundle loaded
- **THEN** the OS file manager opens at the bundle directory

### Requirement: Bundle catalog and cached maps remain interactive during an in-flight download

The system SHALL allow the user to interact with the bundle catalog and to open already-cached maps while another bundle's files are being downloaded. Only the specific map row whose file is being fetched right now SHALL remain non-interactive.

A request to switch to a different project while a download is in progress SHALL be honored by cancelling the in-flight download and starting the requested one; already-downloaded files for the cancelled bundle SHALL remain on disk so the user can resume later by re-selecting that project.

#### Scenario: Selecting a different project mid-download

- **WHEN** a download for project A is in progress AND the user clicks project B in the bundle catalog
- **THEN** the download for A is cancelled, B starts downloading, and the partially-downloaded files for A remain on disk

#### Scenario: Opening a cached map mid-download

- **WHEN** a download is in progress for any bundle AND the user clicks a map row that is marked `cached` (already on disk)
- **THEN** that map opens in the workspace, the workspace becomes the active surface, and the original download continues unaffected

#### Scenario: Currently-fetching map row stays disabled

- **WHEN** a download is in progress and one of the maps inside the active bundle is the file being fetched right now
- **THEN** that specific map row is non-interactive and shows a progress badge; other map rows in the same bundle that are already cached remain interactive

#### Scenario: Resuming a previously cancelled download

- **WHEN** the user re-selects a project whose download was cancelled earlier and whose partial files are still on disk
- **THEN** the download resumes by fetching only the files that are not yet on disk

### Requirement: Per-map availability inside the active bundle streams into the UI live

The system SHALL surface a map within the active bundle as "available" (clickable, badged as cached) within the same event tick the backend completes the download of that map's file. The frontend SHALL NOT wait for the whole bundle to finish before flipping a freshly-downloaded map row from "downloading" to "cached".

The backend SHALL emit `state-changed` at the same point it emits `bundle-file-ready` so that the next `getAppState()` snapshot exposes the updated `downloaded` flag on `LizaMapPackageDto` for the just-completed map. The flag's underlying source (the per-`LizaMapPackage` `local_path`) MUST already be set before that emission.

#### Scenario: Single map finishes mid-download

- **WHEN** a multi-file bundle download is in progress AND the file backing one map (e.g. `10-Tracks/topo-A.sqlitedb`) finishes downloading while another file is still in flight
- **THEN** within the same tick the backend emits `bundle-file-ready` for that file it also emits `state-changed`; the next `AppStateDto.current_project.maps[i].downloaded` for that map is `true`; the corresponding map row in the bundle loader transitions from the blue `%` progress badge to the green `cached` badge and becomes clickable

#### Scenario: Clicking a just-finished map mid-download opens it

- **WHEN** a multi-file bundle download is in progress AND a map row whose file completed earlier in the same download now shows the `cached` badge AND the user clicks that row
- **THEN** the map opens in the workspace via the existing `openSelectedMap` path; the ongoing download of the remaining files continues uninterrupted; subsequent files completing in that bundle continue to flip their rows live

#### Scenario: All maps inside one bundle finish before non-map files

- **WHEN** a bundle contains two map files and one reference PDF AND both map files finish downloading before the PDF
- **THEN** both map rows flip to `cached` and become clickable as each finishes; the user can open either map without waiting for the PDF; once the PDF completes the bundle download is fully done with no further row state changes

### Requirement: Bundle download progress is observable from every surface that lists the downloading project's maps

While a bundle download is in flight, every UI surface that lists the maps belonging to the downloading project SHALL render an in-progress indicator on the affected rows, sourced from the `downloadingMaps` and `downloadProgress` stores in `src/lib/stores.ts`. This includes at minimum: the `BundleLoader` component's maps column (existing behaviour, unchanged) AND the Library Rail Maps tab (`src/components/library/MapsTab.svelte`, newly wired by this change).

The progress data SHALL be sourced from the existing layout-level listeners on `download-progress` and `bundle-progress` events (`src/routes/+layout.svelte`). No surface SHALL register its own `listen()` for either event type.

#### Scenario: Library Maps tab mirrors BundleLoader progress

- **WHEN** a bundle download is in progress for the current project AND the user has the Library Maps tab visible AND opens the bundle loader Sheet side-by-side
- **THEN** each map row in the Library Maps tab shows the same in-progress indicator as the corresponding row inside the bundle loader's maps column, both reflecting the same `downloadProgress` payload within one animation frame of each event arrival

#### Scenario: Status bar mirrors bundle-loader progress text

- **WHEN** a bundle download is in progress
- **THEN** the workspace status bar shows the same `bundleProgress` text that the bundle loader's status region shows, both sourced from the same `bundleProgress` store

#### Scenario: Per-map progress disappears when the map is cached

- **WHEN** a single file inside the bundle completes AND the backend emits `bundle-file-ready` AND `state-changed` AND `appState.refresh()` flips `currentProject.maps[i].downloaded` to true for that map
- **THEN** the in-progress indicator on the affected row is replaced by the `cached` badge AND the row no longer reads from `downloadProgress`

#### Scenario: Progress wiring does not duplicate event subscriptions

- **WHEN** the static count of `listen("download-progress", ...)`, `listen("bundle-progress", ...)`, and `listen("bundle-file-ready", ...)` registrations in the `src/` tree is taken
- **THEN** each event name appears exactly once across the entire frontend, in `src/routes/+layout.svelte`

