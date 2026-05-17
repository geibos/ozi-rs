# tile-rendering Specification

## Purpose
TBD - created by archiving change bootstrap-current-state. Update Purpose after archive.
## Requirements
### Requirement: System serves MBTiles via the `sqlite://` custom tile protocol

The system SHALL register a `sqlite://` MapLibre protocol handler that resolves tile URLs of the form `sqlite://<path>/<base_zoom>/{z}/{x}/{y}` by querying the corresponding MBTiles SQLite database. The handler MUST account for MBTiles row-coordinate inversion.

#### Scenario: Topo MBTiles tile request

- **WHEN** MapLibre requests `sqlite://<bundle>/<base_zoom>/{z}/{x}/{y}` for a visible tile
- **THEN** the backend returns the tile image bytes from the MBTiles database, with row coordinates inverted as required by the MBTiles spec

### Requirement: System serves OZF2 rasters via the `ozi://` custom tile protocol

The system SHALL register an `ozi://` MapLibre protocol handler that resolves tile URLs of the form `ozi://<map_path>/{z}/{x}/{y}` by reading the underlying OZF2 raster and reprojecting tiles to Web Mercator. The handler SHALL return 256×256 PNG tiles.

#### Scenario: OZF2 satellite tile request

- **WHEN** MapLibre requests an `ozi://` tile for a visible viewport
- **THEN** the backend decodes the appropriate OZF2 source level, reprojects pixels to Web Mercator, and returns a 256×256 PNG

### Requirement: Coordinate math for OZI tiles is performed in Rust

The system SHALL compute all OZI georeferencing (affine transform between lat/lon and OZF2 pixel space) and Web Mercator reprojection in the Rust backend. The frontend SHALL receive ready-to-display tile bytes only; no calibration metadata for client-side tile computation SHALL be exposed via the tile protocol.

#### Scenario: No frontend georeferencing

- **WHEN** the frontend requests an `ozi://` tile
- **THEN** the response is opaque tile bytes; the frontend has no access to OZF2 calibration data through the tile protocol

### Requirement: OpenStreetMap is available as an online fallback

The system SHALL render OpenStreetMap raster tiles as a fallback when no local bundle is active and network is available. OSM attribution SHALL be displayed per OSM tile usage policy whenever OSM tiles are rendered.

#### Scenario: No bundle active

- **WHEN** the application starts with no active bundle and network is available
- **THEN** OpenStreetMap tiles render and the OSM attribution is visible in the map view

