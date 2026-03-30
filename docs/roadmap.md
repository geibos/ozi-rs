# Roadmap

## Phase 0 — Kickoff

Status: **complete**

- defined scope and non-goals
- recorded initial architecture ADR
- defined testing strategy
- created prioritized backlog

## Phase 1 — Core Model and Commands

Status: **complete**

- domain entities: Project, MapLayer, TrackLayer, WaypointLayer, Track, TrackSegment,
  TrackPoint, Waypoint
- explicit command model with CommandStack
- undo/redo via full project snapshots
- unit and workflow tests for core edits

## Phase 2 — Project Persistence

Status: **complete**

- JSON project save/load
- last open project restored on startup
- persistence boundaries separated from UI state

## Phase 3 — Data Import/Export

Status: **complete**

- GPX import (single file and ZIP archive)
- PLT import (handles Windows-1251 encoding)
- GPX export with Garmin color extension
- clear user-facing error reporting

Deferred to later:
- PLT export
- KML import/export
- WPT import/export

## Phase 4 — Map Display

Status: **complete**

- SQLite tile maps (LizaAlert format)
- OziExplorer OZF2 raster maps
- OZI georeference (`.map` file parsing)
- tiled rendering with zoom level selection and LRU texture cache
- OpenStreetMap online fallback
- track overlay on all map types

## Phase 5 — LizaAlert Integration

Status: **complete**

- browse and download projects from maps.lizaalert.ru
- configurable bundle storage directory
- open local bundle from folder
- reveal active bundle in Finder/Explorer
- automatic `10-Tracks/` subfolder on export
- LizaAlert OK-standard track name validation (`YYYYMMDD_Callsign`)
- last active map restored on startup

## Phase 6 — UI Polish

Status: **in progress**

Done:
- Catppuccin theme with Auto/Latte/Frappé/Macchiato/Mocha picker, persisted
- floating resizable Tracks window
- developer console toggled with backtick, full scroll history
- structured logging via `tracing` / `RUST_LOG`

Pending:
- review UI framework choice (egui limitations for drag editing and native windows)
- separate Tracks window as a real OS-level window

## Phase 7 — Track Editing (next)

Priority backlog:

1. track point list panel: all properties (lat/lon, elevation, timestamp)
2. sort track points by timestamp (fix out-of-order GPS recordings)
3. select and delete track points
4. move track point in edit mode (drag on map)
5. split track segment at selected point
6. join adjacent segments
7. insert track point
8. create track from scratch on map

All edits must flow through CommandStack for undo/redo.

## Phase 8 — Track Simplification

- Douglas-Peucker simplification with configurable tolerance
- preview before committing
- preserve timestamp and elevation through simplification

## Phase 9 — Waypoint Editing UI

The domain model already supports waypoints; the UI is missing:

1. waypoint list panel
2. add waypoint by clicking on map
3. move waypoint by drag
4. rename waypoint inline
5. delete waypoint
6. waypoint icon or symbol support

## Phase 10 — Export and Print

- PLT export
- print map view with tracks and waypoints to PDF or image
- configurable scale and paper size

## Deferred

- GPS device sync and live track recording
- KML import/export
- datum management and advanced projection
- multi-map simultaneous display
- polygon / search sector drawing
- overlay layers (wiki, hybrid, archive)
- style and naming templates

## Open Questions

- **UI framework**: egui works but has real limitations for drag-based point editing
  and native multi-window UIs. Alternatives (iced, slint) should be evaluated before
  Phase 7 begins. Cost of migration grows with each UI-heavy feature added.
- **PLT export**: needed for round-trip compatibility with OziExplorer in the field;
  assess priority with users.
- **Sector drawing**: LizaAlert workflows involve drawing search sectors on maps;
  this is post-MVP but should inform the geometry model now.
