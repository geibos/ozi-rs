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

