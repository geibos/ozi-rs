# map-bundles Specification

## Purpose
TBD - created by archiving change bootstrap-current-state. Update Purpose after archive.
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

