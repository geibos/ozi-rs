# tile-rendering Specification

## Purpose
Covers how raster maps reach the MapLibre canvas: LizaAlert `.sqlitedb` tile databases through the `sqlite://` protocol, OziExplorer OZF2 rasters through the `ozi://` protocol, and OpenStreetMap as the online fallback. It fixes where georeferencing and reprojection happen (Rust), how tile bytes cross the Tauri IPC boundary, how missing or out-of-range tiles are answered, what the backend caches, and how OZF2 metadata constrains the source's bounds and zoom range.

### Decision history

- ADR-0010 (2026-03-29, accepted, egui-era): render OZF2 rasters as multi-level tiles with viewport culling, a 256-entry LRU texture cache and a 4096 px oversized-tile guard; rationale: bound GPU memory and avoid driver crashes on huge textures. Codified as: "OZF2 source level is the coarsest with one source pixel per output pixel", "OZF2 map context is opened once per map path and reused" | LRU texture cache, culling and the 4096 px guard not codified because they were egui/glow constructs; `src-tauri/src/commands/tiles.rs` keeps no tile-level cache — MapLibre caches tiles client-side and the backend re-decodes on every request.
- ADR-0012 (2026-03-23, accepted, egui-era): serve SQLite tiles synchronously on the UI thread behind a 512-entry LRU cache with a `Missing` sentinel; rationale: sub-millisecond reads make a background loader unnecessary. Codified as: "System serves MBTiles via the `sqlite://` custom tile protocol" (synchronous query per request, zoom inversion), "Missing or out-of-range tiles yield an empty response" (the sentinel's job is now the protocol handler's empty `ArrayBuffer`) | the 512-entry LRU not codified because no cache exists: `get_sqlite_tile` opens a fresh `rusqlite::Connection` per request (`tiles.rs:65`).
- ADR-0016 (2026-03-30, accepted), tile-delivery part: MapLibre `addProtocol` handlers call Tauri commands that return binary blobs, avoiding a local HTTP tile server; rationale: no port, no extra process, reuse of the IPC boundary. Codified as: "System serves MBTiles via the `sqlite://` custom tile protocol", "System serves OZF2 rasters via the `ozi://` custom tile protocol", "Coordinate math for OZI tiles is performed in Rust", "Tiles are delivered as raw IPC byte responses, not over HTTP", "OZF2 metadata command supplies source bounds and zoom range".
- ADR-0006 (2026-03-29, accepted): OZF2 decoding lives in a separate crate behind the `ozi_raster.rs` adapter; rationale: isolate a reverse-engineered format. Codified in the `architecture` capability ("OZF2 rasters are decoded only through the `ozf2` crate adapter"); the crate is now the crates.io `ozf2 = "0.1"`, not a path sibling.
- bootstrap-current-state (2026-05, archived): captured the four base requirements of this capability (`sqlite://`, `ozi://`, Rust-side coordinate math, OSM fallback).

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

### Requirement: A cold launch with no link adds no remote basemap

When the machine reports that it has no network at all, the map SHALL NOT add
a remote raster source. The local maps of the project are what the operator
opened the application for, and a remote basemap offline renders as nothing
while still costing a request per visible tile.

#### Scenario: Launching in a field camp

- **WHEN** the map loads and the machine reports no network
- **THEN** no remote tile source is added and local maps render as before

### Requirement: Opening a map reports a failure and stays retryable

Applying the active map SHALL read its metadata before changing what is on the
screen, SHALL report a read that failed rather than dropping it, and SHALL
count the map as applied only once the read has succeeded, so that a failure
does not block a later attempt at the same map.

An apply that another map's apply has overtaken SHALL NOT change the map.

#### Scenario: Metadata that cannot be read

- **WHEN** the metadata of the map being opened cannot be read
- **THEN** the operator is told, the map on screen is left alone, and opening the same map again tries again

#### Scenario: Switching maps during the read

- **WHEN** a second map is chosen while the first one's metadata is still being read
- **THEN** only the second map is applied

### Requirement: A `.map` beside an ordinary picture opens like an OZF2 one

The system SHALL open an OziExplorer `.map` file whose raster is an ordinary
image — JPEG, PNG, TIFF, BMP, GIF or WebP — and SHALL serve its tiles through
the same `ozi://` protocol, with the same level and tile geometry, as a `.map`
whose raster is an OZF2 file. Nothing above the raster adapter SHALL need to
know which of the two it was given.

Levels SHALL be built by halving the picture until its shorter side reaches one
tile, and tiles SHALL be 256 px square except at the right and bottom edges,
where a tile is only as large as the pixels that are there.

A picture whose pixel count exceeds what the application will hold in memory
SHALL be refused before its pixels are read, with a message naming the file and
its size. `.ozfx3` SHALL remain refused.

#### Scenario: A scan opens

- **WHEN** the operator opens a `.map` file that names `sheet.png` beside it
- **THEN** the map opens and its tiles are served, exactly as for an OZF2 pair

#### Scenario: Edge tiles are only as big as what is there

- **WHEN** the picture is 600 px wide and a tile in the third column is
  requested
- **THEN** that tile is 88 px wide, and a tile in a fourth column is an error

#### Scenario: A picture that is not there names itself

- **WHEN** a `.map` names a raster that does not exist beside it
- **THEN** the failure names the missing file rather than reporting an
  unsupported format

#### Scenario: A picture too large to hold is refused by size

- **WHEN** the named picture has more pixels than the application will hold
- **THEN** the open fails with a message giving the file and its dimensions,
  and no attempt is made to decode it
