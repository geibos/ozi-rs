## ADDED Requirements

### Requirement: Product scope is the LizaAlert SAR track-handling workflow

The product SHALL cover one end-to-end workflow for a LizaAlert search-and-rescue volunteer: open a map bundle, import or draw tracks, edit and visualise them, place waypoints, save the operation as a project, and export the results in formats consumed by handheld navigators. Work that does not trace back to a stage of this workflow SHALL NOT be scheduled without a recorded product decision (see the `documentation` capability).

#### Scenario: Every workflow stage has a registered IPC command

- **WHEN** the `tauri_specta::collect_commands!` block in `src-tauri/src/lib.rs` is listed
- **THEN** it contains at least one command for each stage: bundle open (`open_local_bundle`, `load_project`), track import or drawing (`import_gpx`, `import_plt`, `create_empty_track`), track editing (`move_track_point`, `delete_track_point`), waypoint placement (`add_waypoint`), project persistence (`save_project`, `load_project_file`) and export (`export_gpx`, `export_track_plt`, `export_wpt_waypoints`)

### Requirement: Maps in scope: LizaAlert bundles, local folders, MBTiles, OZF2, OSM fallback

The product SHALL include opening a LizaAlert map bundle from the online catalog and from a previously downloaded local folder, switching between the maps of one bundle (typically Topo and Satellite) without restarting the application, rendering SQLite/MBTiles tile maps and OZF2 raster maps, and falling back to OpenStreetMap online tiles when no local map is active. Opening a bundle directly by URL is part of the declared scope.

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

### Requirement: WPT waypoint export is in the MVP; WPT import is not

The product SHALL export all waypoints of the active waypoint layer to the OziExplorer WPT format with a WGS-84 header, offered alongside GPX (tracks and waypoints) and PLT (tracks), and SHALL default the destination to the active bundle's track folder when one is known. WPT import SHALL NOT be part of the MVP; waypoint import stays GPX-based.

#### Scenario: WPT exporter and default-path command exist

- **WHEN** the specta command registry and `src-tauri/src/infrastructure/export/` are listed
- **THEN** `export_wpt_waypoints` and `get_wpt_export_default_path` are registered and `wpt.rs` exists in the export module

#### Scenario: No WPT importer exists

- **WHEN** the specta command registry and `src-tauri/src/infrastructure/import/` are listed
- **THEN** no command name contains `import_wpt` and the import module has no `wpt` file

### Requirement: On-map field tools in scope: distance, radius circle, projection

The product scope SHALL include three on-map tools: measuring a distance, drawing a circle of an explicit radius centred on a point or on the cursor, and placing a waypoint by projection (azimuth plus distance) from a selected point. Once a tool ships it SHALL be reachable from the main workspace without developer-console workarounds.

#### Scenario: Tools are tracked as MVP scope

- **WHEN** `docs/requirements.md` § "MVP Non-Goals", `docs/roadmap.md` § "Deferred (post-1.0)" and `docs/feature-status.md` are read
- **THEN** none of the three tools appears under the non-goal or deferred headings, and the feature-status matrix has one row per tool

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

### Requirement: Navigator sync and full layer management are post-MVP

USB upload to and pull from navigators, FTP automation, and a full layer manager (create, rename, delete, reorder layers) SHALL NOT be part of the MVP; they remain recorded future work. The MVP interface exposes active-layer selection only.

#### Scenario: No layer-management or device-sync commands

- **WHEN** the specta command registry in `src-tauri/src/lib.rs` is listed
- **THEN** no command name contains `create_layer`, `rename_layer`, `delete_layer`, `reorder`, `usb` or `ftp`

#### Scenario: Full layer management is recorded as remaining work

- **WHEN** `docs/roadmap.md` is read
- **THEN** full layer management UI is listed as remaining work, not as delivered

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
