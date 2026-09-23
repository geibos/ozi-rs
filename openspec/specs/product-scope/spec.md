# product-scope Specification

## Purpose
What the product includes and excludes, independent of implementation status:
the LizaAlert search-and-rescue workflow (open a bundle → import or draw tracks
→ edit → place waypoints → save `.ozp` → export GPX/PLT/WPT), the field tools a
crew reaches for on the map, the supported platforms and the explicit
non-goals. Behaviour lives in the other capabilities; this one answers "is X
something we build at all?".

Decision history:

- `how-far-is-that` (2026-09-22): three on-map tools — a distance measured by
  clicking, a geodesic radius ring and a waypoint placed by bearing and
  distance; rationale: a search crew takes these measurements constantly, and
  without them they measure by eye. Reachable from the command palette rather
  than the toolbar, because `ui-shell` still requires the mode chips that would
  otherwise be the place for them. Codified as the three requirements below;
  their rendered pixels were verified on the stand on 2026-09-22
  (`docs/progress/2026-09-22-verification/`).
- The scope statements drawn from ADR-0020, ADR-0022 and ADR-0023 — the MVP
  workflow, WPT export in and WPT import out, no map printing — arrive with
  `codify-architecture-decisions`, together with the owner's 2026-09-19
  decision that macOS is primary while they are the only user.

## Requirements

### Requirement: Distance can be measured on the map

The operator SHALL be able to measure a distance on the map by clicking points,
and SHALL see the running total while they do. This is the measurement a search
crew takes constantly, and without it they measure by eye.

The tool SHALL be reachable from the command palette, SHALL be dismissable with
Escape, and SHALL take a click whole while it is active so that measuring does
not also select or place something.

Measured points SHALL NOT be part of the project: they are scratch, discarded
when the tool is switched off. A measurement worth keeping is a track.

A measured distance SHALL be shown in metres below a kilometre.

The points clicked SHALL be drawn on the map, joined in the order they were
clicked and distinguishable from anything belonging to the project, so that the
operator can see where each click landed and whether it landed at all. The last
point SHALL be removable without ending the measurement.

#### Scenario: Measuring a leg

- **WHEN** the operator turns the tool on and clicks two points on the map
- **THEN** the distance between them is shown, in metres if it is under a kilometre

#### Scenario: A misclick

- **WHEN** the operator removes the last measured point
- **THEN** it is gone from the map, the total is recomputed, and the measurement continues

#### Scenario: Finishing

- **WHEN** the operator presses Escape while measuring
- **THEN** the tool is off and nothing it measured remains

#### Scenario: A click while measuring

- **WHEN** the operator clicks the map while measuring
- **THEN** a point is added and no track is selected and no waypoint is placed

### Requirement: A radius ring can be drawn on the map

The operator SHALL be able to draw a ring of a chosen radius around a chosen
point, by placing a centre and then setting the radius, and SHALL see the
radius while they do. A search works in rings around a last known position.

The ring SHALL be geodesic: every point on it SHALL be the stated distance from
the centre on the ground, not in screen pixels. A ring drawn flat is wrong
everywhere but the equator and worse the further north the search is.

The ring SHALL be scratch, like a measurement, and SHALL NOT be part of the
project.

Only one on-map tool SHALL be listening for clicks at a time, and switching
tools SHALL discard what the previous one held.

#### Scenario: A ring around the last known position

- **WHEN** the operator places a centre and then sets a radius
- **THEN** a ring of that radius is drawn, and every point on it is that distance from the centre

#### Scenario: Switching tools

- **WHEN** the operator turns on one on-map tool while another is active
- **THEN** the other is off and what it held is discarded

### Requirement: A waypoint can be placed by bearing and distance

The operator SHALL be able to place a waypoint by choosing a point on the map
and entering a bearing and a distance from it. A position given over a radio
arrives as an azimuth and a range, not as somewhere to point at.

Where the waypoint will land SHALL be shown on the map before it is placed, so
that a misheard bearing is caught before it becomes a mark, and the placement
SHALL NOT be offered until there is a distance to place at.

#### Scenario: A position given over the radio

- **WHEN** the operator chooses a point, enters a bearing and a distance, and confirms
- **THEN** a waypoint is placed at that bearing and distance from the chosen point

#### Scenario: Before confirming

- **WHEN** a bearing and a distance have been entered
- **THEN** the resulting position is shown on the map before the waypoint exists

### Requirement: Length and area are measured by separate tools

The system SHALL provide measuring a length and measuring an area as two
tools, not as one readout carrying both numbers.

Measuring a length is what a crew does most often — how long is this ride,
how far from the road to the stream — and for an open path the enclosed area
is not a small number but a meaningless one. A number shown in a place where
people read numbers gets read.

The length tool SHALL report the length of the clicked path and nothing else.
The area tool SHALL report what the points enclose, with the perimeter beside
it, because a sector is described by both: "прочесать 2.4 км², обойти по
кромке 6 км". Its outline SHALL be drawn closed, so the shape agrees with the
number.

#### Scenario: Measuring how far it is

- **WHEN** the operator measures a path with the length tool
- **THEN** the readout carries the length alone

#### Scenario: Measuring a sector

- **WHEN** the operator clicks round a sector with the area tool
- **THEN** the readout carries the enclosed area and the perimeter, and the outline on the map is closed

#### Scenario: Too few points to enclose anything

- **WHEN** fewer than three points have been placed with the area tool
- **THEN** no area is claimed

### Requirement: A radius ring reports the ground it covers

The radius ring SHALL report the area it encloses beside its radius.

A ring is drawn to say "everything within five hundred metres of the last
known position", and the next question a coordinator asks is how much ground
that is.

#### Scenario: A ring around the last known position

- **WHEN** the operator sets a ring's radius
- **THEN** the readout carries both the radius and the enclosed area

### Requirement: Product scope is the LizaAlert SAR track-handling workflow

The product SHALL cover one end-to-end workflow for a LizaAlert search-and-rescue volunteer: open a map bundle, import or draw tracks, edit and visualise them, place waypoints, save the operation as a project, and export the results in formats consumed by handheld navigators. Work that does not trace back to a stage of this workflow SHALL NOT be scheduled without a recorded product decision (see the `documentation` capability).

#### Scenario: Every workflow stage has a registered IPC command

- **WHEN** the `tauri_specta::collect_commands!` block in `src-tauri/src/lib.rs` is listed
- **THEN** it contains at least one command for each stage: bundle open (`open_local_bundle`, `load_project`), track import or drawing (`import_gpx`, `import_plt`, `create_empty_track`), track editing (`move_track_point`, `delete_track_point`), waypoint placement (`add_waypoint`), project persistence (`save_project`, `load_project_file`) and export (`export_gpx`, `export_track_plt`, `export_wpt_waypoints`)

### Requirement: Maps in scope: LizaAlert bundles, local folders, MBTiles, OziExplorer rasters, OSM fallback

The product SHALL include opening a LizaAlert map bundle from the online catalog and from a previously downloaded local folder, switching between the maps of one bundle (typically Topo and Satellite) without restarting the application, rendering SQLite/MBTiles tile maps and OziExplorer rasters, and falling back to OpenStreetMap online tiles when no local map is active. Opening a bundle directly by URL is part of the declared scope.

An OziExplorer raster is a `.map` beside either an OZF2 file or an ordinary picture; both are in scope, because the pair a headquarters is handed is far more often the second. A picture that arrives with no `.map` at all SHALL be calibratable in the product rather than sent back to OziExplorer to be prepared. Both are specified in `tile-rendering`.

#### Scenario: Map commands are registered

- **WHEN** the command registration in `src-tauri/src/lib.rs` is listed (`tauri_specta::collect_commands!` plus the raw-bytes `tauri::generate_handler!` tile block)
- **THEN** it contains `load_projects`, `load_project`, `open_local_bundle`, `open_selected_map`, `set_bundles_root`, `tiles::get_ozi_metadata`, and the tile block contains `tiles::get_sqlite_tile` and `tiles::get_ozi_tile_projected`

#### Scenario: OSM fallback source is configured

- **WHEN** `src/components/MapView.svelte` is inspected
- **THEN** it declares a raster source pointing at `https://tile.openstreetmap.org/{z}/{x}/{y}.png` with OpenStreetMap attribution

#### Scenario: Open-by-URL is tracked as scope, not dropped

- **WHEN** `docs/feature-status.md` and `docs/requirements.md` are read
- **THEN** the feature-status matrix has a row for opening a LizaAlert bundle by URL and `docs/requirements.md` § "MVP Non-Goals" does not list it

### Requirement: Tracks in scope: import, multi-track display, per-track styling

The product SHALL include importing GPX and OziExplorer PLT tracks, including ZIP archives that contain them; displaying many tracks at once, each with its own colour, line width and visibility; and loading tracks with tens of thousands of points without blocking the interface.

#### Scenario: Import and display commands are registered

- **WHEN** the specta command registry in `src-tauri/src/lib.rs` is listed
- **THEN** it contains `import_gpx`, `import_plt`, `import_tracks_directory`, `get_tracks_geojson`, `set_track_color`, `set_track_line_width` and `toggle_track_visible`

#### Scenario: ZIP archives are selectable in the import picker

- **WHEN** the import file picker in `src/components/library/TracksTab.svelte` is inspected
- **THEN** its extension filter includes `zip` alongside `gpx` and `plt`

### Requirement: Tracks in scope: point-level editing, cleanup, drawing

The product SHALL include walking through track points one by one with per-point attributes, deleting points, splitting a track into segments and joining segments back, sorting points by timestamp, Douglas–Peucker simplification with a tolerance preview, cropping a track to the current map extent, to a time range or to selected points, and drawing a new track directly on the map.

#### Scenario: Editing and cleanup commands are registered

- **WHEN** the specta command registry in `src-tauri/src/lib.rs` is listed
- **THEN** it contains `move_track_point`, `delete_track_point`, `insert_track_point`, `split_segment`, `join_segments`, `sort_track_points`, `simplify_track`, `get_simplified_preview`, `crop_track_to_extent`, `crop_track_to_time`, `create_empty_track` and `get_track_detail`

#### Scenario: Cleanup actions are reachable from the Track Inspector

- **WHEN** `src/components/inspector/TrackInspector.svelte` and `src/components/inspector/TrackSegmentsTable.svelte` are inspected
- **THEN** they call `sortTrackPoints`, `cropTrackToExtent`, `cropTrackToTime`, `splitSegment` and `joinSegments` from `$lib/api`

#### Scenario: Declared cleanup work not yet built stays in scope

- **WHEN** `docs/requirements.md` § "MVP Non-Goals" and `docs/roadmap.md` § "Deferred (post-1.0)" are read
- **THEN** neither crop-by-selected-points nor next/previous point walkthrough appears there

### Requirement: Waypoints in scope: place, edit, customise, show many, export

The product SHALL include adding waypoints by clicking on the map, moving, renaming and deleting them, choosing a waypoint symbol and visual appearance, showing many waypoints at once with per-waypoint visibility, and exporting waypoints to GPX and to OziExplorer WPT.

#### Scenario: Waypoint commands are registered

- **WHEN** the specta command registry in `src-tauri/src/lib.rs` is listed
- **THEN** it contains `add_waypoint`, `move_waypoint`, `rename_waypoint`, `delete_waypoint`, `set_waypoint_symbol`, `toggle_waypoint_visible`, `get_waypoints` and `export_wpt_waypoints`

### Requirement: WPT waypoints go both ways

The product SHALL export all waypoints of the active waypoint layer to the OziExplorer WPT format with a WGS-84 header, offered alongside GPX (tracks and waypoints) and PLT (tracks), and SHALL default the destination to the active bundle's track folder when one is known.

It SHALL also import that format. The scope was written when import was deferred and waypoint import stayed GPX-based, which was wrong about the field: the headquarters next door runs OziExplorer itself and hands over `.wpt`. Writing a format this application cannot read makes an exchange that only works in one direction. The behaviour is specified in `track-import` ("OziExplorer waypoint files import").

#### Scenario: WPT exporter and default-path command exist

- **WHEN** the specta command registry and `src-tauri/src/infrastructure/export/` are listed
- **THEN** `export_wpt_waypoints` and `get_wpt_export_default_path` are registered and `wpt.rs` exists in the export module

#### Scenario: The importer exists too

- **WHEN** the specta command registry and `src-tauri/src/infrastructure/import/` are listed
- **THEN** `import_wpt` is registered and `wpt.rs` exists in the import module

### Requirement: On-map field tools in scope: distance, area, radius circle, projection

The product scope SHALL include four on-map tools: measuring a distance, measuring an area, drawing a circle of an explicit radius centred on a point or on the cursor, and placing a waypoint by projection (azimuth plus distance) from a selected point. Once a tool ships it SHALL be reachable from the main workspace without developer-console workarounds.

Distance and area are separate tools, not one tool that guesses: a coordinator measuring how far a crew still has to walk wants a length, and a coordinator sizing a sector wants an area, and being handed the other one costs a click and a moment of doubt at the worst time. A circle SHALL state both its radius and the area it covers, for the same reason (owner decision, 2026-09-23).

#### Scenario: Tools are tracked as MVP scope

- **WHEN** `docs/requirements.md` § "MVP Non-Goals", `docs/roadmap.md` § "Deferred (post-1.0)" and `docs/feature-status.md` are read
- **THEN** none of the four tools appears under the non-goal or deferred headings, and the feature-status matrix has one row per tool

#### Scenario: A shipped tool is a first-class command

- **WHEN** one of the tools is implemented
- **THEN** its mutation is a command in the specta registry and its entry point lives in the workspace (Library, Inspector, mode chips or command palette), not in the developer console

### Requirement: Project in scope: .ozp save/load, recent list, undo/redo, name warning

The product SHALL include saving and loading a project as a JSON `.ozp` file, a persisted list of recently used items for quick re-open, undo/redo through the delta-based command stack, and OK-standard (`YYYYMMDD_Callsign`) track-name validation that only warns.

#### Scenario: Project commands are registered

- **WHEN** the specta command registry in `src-tauri/src/lib.rs` is listed
- **THEN** it contains `save_project`, `load_project_file`, `undo` and `redo`

#### Scenario: Recent list is persisted under a versioned key

- **WHEN** `src/lib/recentFiles.ts` is read
- **THEN** it exports `RECENT_FILES_KEY` with the value `ozi:recent-files:v1`

#### Scenario: Name validation never blocks a workflow

- **WHEN** a track is renamed to a name that does not match the OK standard
- **THEN** rename, save and export still succeed and the interface shows only a warning

### Requirement: Export in scope: GPX and PLT tracks, GPX and WPT waypoints

The product SHALL export tracks to GPX and to OziExplorer PLT, and waypoints to GPX and to OziExplorer WPT. Export dialogs SHALL suggest the active bundle's `10-Tracks/` folder when a bundle is active. PLT is a track-only format; OziExplorer waypoints travel through WPT.

#### Scenario: Export commands are registered

- **WHEN** the specta command registry in `src-tauri/src/lib.rs` is listed
- **THEN** it contains `export_gpx`, `export_track_plt`, `export_wpt_waypoints`, `get_track_export_default_path` and `get_wpt_export_default_path`

#### Scenario: One exporter module per format

- **WHEN** `src-tauri/src/infrastructure/export/` is listed
- **THEN** it contains `gpx.rs`, `plt.rs` and `wpt.rs` and no other format module

### Requirement: Platforms in scope: macOS now, Windows next, Linux buildable

macOS SHALL be the primary target while the project's only field user works on macOS (owner decision 2026-09-19); Windows SHALL remain a supported release target and becomes the focus in the distribution phase, because the OziExplorer audience is historically Windows; Linux SHALL build and run without bespoke configuration. Mobile platforms are outside the scope.

#### Scenario: CI builds on all three desktop platforms

- **WHEN** `.github/workflows/ci.yml` is read
- **THEN** the Tauri smoke-build matrix lists `ubuntu-latest`, `macos-latest` and `windows-latest`, and a Windows NSIS installer job runs on every pull request

#### Scenario: Releases ship Windows and macOS bundles

- **WHEN** `.github/workflows/release.yml` is read
- **THEN** its matrix builds on `windows-latest` and `macos-latest`

### Requirement: Map printing is not provided

The product SHALL NOT provide map printing to PDF, image or any other rendered page output. Reintroducing printing requires a recorded decision that names the user workflow, the output formats and resolution constraints, the implementation surface, and why digital hand-off to navigators no longer suffices.

#### Scenario: No print pipeline in code

- **WHEN** the specta command registry is listed and `src/` is searched
- **THEN** no command name contains `print` or `pdf`, and no `window.print` call exists

#### Scenario: Roadmap and requirements list printing as not planned

- **WHEN** `docs/roadmap.md` and `docs/requirements.md` are read
- **THEN** printing is absent from remaining work and appears under "MVP Non-Goals" as not planned

### Requirement: KML import and export are outside the MVP

The product SHALL NOT include KML import or export in the MVP; KML stays recorded as post-MVP backlog.

#### Scenario: No KML parser or command

- **WHEN** the specta command registry, `src-tauri/src/infrastructure/import/` and `src-tauri/src/infrastructure/export/` are listed
- **THEN** no command name contains `kml` and neither module has a `kml` file

#### Scenario: KML is recorded as later work

- **WHEN** `docs/roadmap.md` and `docs/requirements.md` are read
- **THEN** KML appears as remaining or low-priority work and is absent from `docs/requirements.md` § "MVP Scope"

### Requirement: Navigator sync is post-MVP; layers are managed in the product

USB upload to and pull from navigators and FTP automation SHALL NOT be part of the MVP; they remain recorded future work.

Layer management is not among them any more. Creating, renaming and deleting track and waypoint layers is in the product, because a day's work does not fit in one layer: a search has a layer per group and a layer of finds, and an operator who cannot make one has nowhere to put the second group. Reordering layers is still not provided. The behaviour is specified in `layers`.

#### Scenario: No device-sync commands

- **WHEN** the specta command registry in `src-tauri/src/lib.rs` is listed
- **THEN** no command name contains `reorder`, `usb` or `ftp`

#### Scenario: Layers can be made, renamed and removed

- **WHEN** the same registry is listed
- **THEN** it contains `create_track_layer`, `create_waypoint_layer`,
  `rename_track_layer`, `rename_waypoint_layer`, `delete_track_layer` and
  `delete_waypoint_layer`

#### Scenario: Reordering is recorded as remaining work

- **WHEN** `docs/roadmap.md` is read
- **THEN** reordering layers is listed as remaining work, not as delivered

### Requirement: Never planned: datum, geodesy, telemetry, routes, polygons, privileged tracks

The product SHALL NOT include datum management, advanced geodesy beyond what the on-map tools require, live GPS device telemetry, routes and events, polygon or search-sector drawing, or privileged legacy objects such as a special `Track 1`.

#### Scenario: No such concepts in the backend

- **WHEN** the specta command registry and `src-tauri/src/domain/` are listed
- **THEN** no command or entity is named after routes, events, polygons, sectors, datums or telemetry

#### Scenario: Non-goals are documented

- **WHEN** `docs/requirements.md` § "MVP Non-Goals" is read
- **THEN** datum management, advanced geodesy, live telemetry, routes and events, polygon/sector drawing and privileged `Track 1` objects are all listed

#### Scenario: No track name is privileged

- **WHEN** production (non-test) code under `src-tauri/src` and `src` is searched for `"Track 1"`
- **THEN** no match exists
