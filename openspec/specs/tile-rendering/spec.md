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

The system SHALL register a `sqlite://` MapLibre protocol handler that resolves tile URLs of the form `sqlite://<path>/<base_zoom>/{z}/{x}/{y}` by querying the LizaAlert `.sqlitedb` tile database at `<path>` synchronously on each request. The database schema is `tiles(x, y, z, image)` plus `info(minzoom, maxzoom)`. The handler SHALL map the web zoom to the database zoom as `db_z = db_min + (base_zoom - web_z)` (database zoom 0 is the most detailed level, `base_zoom` is the web zoom that corresponds to it) and SHALL pass `x` and `y` through unchanged; no row inversion is applied. Web zooms outside `[base_zoom - (db_max - db_min), base_zoom]` SHALL be rejected as out of range.

#### Scenario: Topo MBTiles tile request

- **WHEN** MapLibre requests `sqlite://<bundle>/15/13/{x}/{y}` for a bundle whose `info` table reports `minzoom = 0`, `maxzoom = 4`
- **THEN** the backend runs `SELECT image FROM tiles WHERE x=? AND y=? AND z=2` with the requested `x`, `y` and returns the stored image bytes

#### Scenario: Zoom mapping at the range ends

- **WHEN** `base_zoom` is 15 and the bundle stores db zooms 0..4
- **THEN** web zoom 15 maps to db zoom 0, web zoom 11 maps to db zoom 4, and web zooms 10 and 16 are rejected

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

### Requirement: A picture with no calibration can be tied to the Earth

The system SHALL let the operator calibrate an image that has no `.map` beside
it, by naming at least two places in the picture whose latitude and longitude
are known, and SHALL write an OziExplorer `.map` file beside the image, named
after it, before opening the pair.

The file written SHALL be a valid OziExplorer `.map` — Windows-1251, CRLF, the
calibration points in degrees and decimal minutes with hemisphere letters, and
the four corner entries — so that the same folder opens in OziExplorer itself.

A calibration the system cannot solve SHALL be refused with the reason and no
file written: fewer than two points, or points that all share a latitude or all
share a longitude, or a coordinate that is not on the Earth.

#### Scenario: A screenshot becomes a map

- **WHEN** the operator chooses a picture, gives the coordinates of its
  top-left and bottom-right corners, and confirms
- **THEN** a `.map` is written beside the picture, the interface names the file
  it wrote, and the map opens

#### Scenario: What was written reads back as the same places

- **WHEN** a `.map` written by calibration is read by this application
- **THEN** each calibration point resolves to the coordinate the operator gave
  for it

#### Scenario: One point is not a calibration

- **WHEN** the operator gives one point only
- **THEN** the calibration is refused, saying that one point fixes a position
  but not the scale or the rotation, and no file is written

#### Scenario: Points on one parallel are not a calibration

- **WHEN** every point the operator gave shares a latitude
- **THEN** the calibration is refused, saying the points must not lie on one
  line, and no file is written

### Requirement: Coordinate fields read the notations a coordinator writes

Any field that takes a place SHALL accept decimal degrees, degrees with decimal
minutes, and degrees-minutes-seconds; with or without hemisphere letters; with
those letters in the Latin or the Cyrillic alphabet; before or after the
numbers; and in either order, so that `30.3609E 59.9311N` is the same place as
`59.9311, 30.3609`.

Text that does not hold exactly one place SHALL be refused rather than guessed
at, and the field SHALL show that it was not understood before anything is
submitted.

#### Scenario: The same place in four notations

- **WHEN** the operator enters `59.9311, 30.3609`, or `59°55'52"N 30°21'39"E`,
  or `N 59 55.87 E 30 21.65`, or `59.9311С 30.3609В`
- **THEN** each is read as the same place

#### Scenario: Half a place is refused

- **WHEN** the operator enters one number, or three
- **THEN** the field shows it was not understood and the action stays
  unavailable

### Requirement: Tiles are delivered as raw IPC byte responses, not over HTTP

Local tiles SHALL reach MapLibre through Tauri IPC only: `get_sqlite_tile`, `get_ozi_tile` and `get_ozi_tile_projected` SHALL return `tauri::ipc::Response` (raw bytes), SHALL be registered on the plain `tauri::generate_handler!` outside the typed `specta` bindings, and SHALL be invoked from the protocol handlers through the untyped `invokeIpc` wrapper. The application SHALL NOT start a local HTTP tile server.

#### Scenario: Byte commands bypass the typed bindings

- **WHEN** `src/lib/bindings.ts` is generated by the `typescript_bindings_are_up_to_date` test
- **THEN** it contains no `getSqliteTile`, `getOziTile` or `getOziTileProjected` entry, and `src/lib/api.ts` reaches those commands via `invokeIpc`

#### Scenario: No tile server dependency

- **WHEN** `grep -n 'tiny_http\|axum\|warp\|hyper\|actix' src-tauri/Cargo.toml` is run
- **THEN** it prints nothing

### Requirement: Missing or out-of-range tiles yield an empty response

When a requested tile does not exist — the `.sqlitedb` has no row for `(x, y, db_z)`, the web zoom is outside the bundle's range, or the Web Mercator tile does not intersect the OZF2 raster — the backend SHALL return `Err(String)` and the protocol handler (`sqlite-protocol.ts`, `ozi-protocol.ts`) SHALL resolve with an empty `ArrayBuffer` (length 0) instead of propagating the error. MapLibre thus renders nothing for that tile and does not surface an error. An OZF2 tile that only partially overlaps the raster SHALL be rendered with the uncovered pixels transparent.

#### Scenario: Absent SQLite tile

- **WHEN** MapLibre requests `sqlite://<bundle>/<base_zoom>/{z}/{x}/{y}` and the `tiles` table has no matching row
- **THEN** `get_sqlite_tile` returns `Err("tile not found")` and the protocol handler resolves with `{ data: ArrayBuffer(0) }`

#### Scenario: Web zoom outside the bundle range

- **WHEN** a bundle stores db zooms 0..4 with `base_zoom` 15 and MapLibre requests web zoom 10 or 16
- **THEN** `get_sqlite_tile` returns `Err("zoom out of range")` and the protocol handler resolves with an empty `ArrayBuffer`

#### Scenario: OZF2 tile outside the map

- **WHEN** MapLibre requests an `ozi://` tile whose bounding box lies entirely outside the OZF2 raster
- **THEN** `get_ozi_tile_projected` returns `Err("tile out of bounds")` and the protocol handler resolves with an empty `ArrayBuffer`

### Requirement: OZF2 map context is opened once per map path and reused

For `ozi://` requests the backend SHALL parse the `.map` file, build the georeference and open its raster tile source — OZF2 or an ordinary picture, which the adapter answers alike — once per `map_path`, and keep the result in a process-wide cache (`load_ozi_context` in `src-tauri/src/commands/tiles.rs`); subsequent tile requests for the same path SHALL reuse the cached context. Individual decoded tiles are not cached by the backend; MapLibre's own tile cache covers repeat requests.

#### Scenario: Second tile request reuses the context

- **WHEN** two `get_ozi_tile_projected` calls arrive for the same `map_path`
- **THEN** the `.map` file is read and parsed only for the first call; the second call is served from the cached `CachedOziMapContext`

#### Scenario: Different maps do not share context

- **WHEN** tile requests arrive for two different `map_path` values
- **THEN** each path has its own cached context and neither request returns tiles from the other map

### Requirement: OZF2 source level is the coarsest with one source pixel per output pixel

To render a 256×256 Web Mercator tile the backend SHALL project the tile's corners into OZF2 level-0 pixel space, take the larger of the horizontal and vertical span, and select the coarsest OZF2 level whose scale satisfies `span × scale / 256 ≥ 1` (at least one source pixel per output pixel). If no level satisfies the condition the finest level (index 0) SHALL be used. The overlapping OZF2 tiles of the chosen level SHALL be stitched and nearest-neighbour scaled into the 256×256 output.

#### Scenario: Zoomed-out tile picks a coarse level

- **WHEN** a raster has levels at scales 1, 1/2 and 1/4 and the requested tile spans 1200 level-0 pixels
- **THEN** the level at scale 1/4 is selected (1200 × 0.25 / 256 ≈ 1.17 ≥ 1)

#### Scenario: Zoomed-in tile falls back to the finest level

- **WHEN** the same raster receives a request whose tile spans 100 level-0 pixels
- **THEN** no level satisfies the condition and level 0 is selected, producing an upscaled (overzoomed) tile

### Requirement: OZF2 metadata command supplies source bounds and zoom range

`get_ozi_metadata(map_path)` SHALL return, for the given `.map` file, the list of OZF2 levels (index, dimensions, tile grid), the geographic `bounds` `[west, south, east, north]` derived from the georeference and level-0 dimensions, and zoom hints `native_zoom` (the Web Mercator zoom closest to the raster's level-0 resolution, capped at 22) and `min_zoom`. The frontend SHALL use these values as the `ozi://` raster source's `bounds`, `maxzoom` and `minzoom` so MapLibre requests tiles only inside the map's coverage and zoom range.

#### Scenario: Source is constrained by metadata

- **WHEN** an OZF2 map is activated in `MapView`
- **THEN** the `active-map` raster source is created with `maxzoom = native_zoom`, `minzoom = min_zoom` and `bounds` from `get_ozi_metadata`, and the map view fits to those bounds

#### Scenario: Metadata is computed in Rust

- **WHEN** the frontend needs the geographic extent of an OZF2 map
- **THEN** it reads `bounds` from `get_ozi_metadata` and performs no affine or Mercator math of its own

### Requirement: The map can draw a coordinate grid

The system SHALL be able to draw a latitude/longitude grid over the map,
turned on and off by the operator and remembered between sessions. The grid
SHALL be drawn beneath the tracks, the marks and the measurements.

Line spacing SHALL follow the zoom and SHALL always be a round quantity a
person can say aloud: a whole number of degrees, or 30, 20, 10, 5, 2 or 1
minutes, or the same in seconds. The spacing chosen SHALL be the coarsest that
still puts at least four lines across the view.

Each line SHALL be named where it meets the edge of the view — parallels at the
left, meridians at the top — in degrees, minutes and seconds with a hemisphere
letter, omitting the parts that are zero.

#### Scenario: Reading a sector off the map

- **WHEN** the grid is on over a view about two kilometres across
- **THEN** lines are drawn a minute apart and named `59°55'N`, `59°56'N` and so
  on

#### Scenario: The spacing follows the zoom

- **WHEN** the operator zooms from a fifty-metre view out to a ten-kilometre one
- **THEN** the spacing goes from seconds to minutes, and the number of lines on
  screen stays readable

#### Scenario: A line is named where it stands

- **WHEN** a parallel is drawn at 59.9375°
- **THEN** it is named `59°56'15"N`, and the name and the line are the same
  place however many steps were taken to reach it

#### Scenario: The grid is remembered

- **WHEN** the operator turns the grid on and reopens the application
- **THEN** the grid is on
