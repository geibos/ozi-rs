# Roadmap

## Phase 0 — Kickoff

Status: **complete**

- Defined scope and non-goals
- Recorded initial architecture ADR
- Defined testing strategy
- Created prioritized backlog

## Phase 1 — Core Model and Commands

Status: **complete**

- Domain entities: Project, MapLayer, TrackLayer, WaypointLayer, Track, TrackSegment,
  TrackPoint, Waypoint
- Explicit command model with CommandStack
- Undo/redo via full project snapshots (later replaced by delta-based undo in Phase 7)
- Unit and workflow tests for core edits

## Phase 2 — Project Persistence

Status: **complete**

- JSON project save/load (`.ozp` format)
- Last saved/loaded project path restored on startup when the `.ozp` file still exists
- Persistence boundaries separated from UI state; viewport, selections, panels, undo history, theme,
  and unsaved edits are not part of Rust session restore

## Phase 3 — Data Import/Export

Status: **complete**

- GPX import (single file and ZIP archive)
- PLT import (handles Windows-1251 encoding)
- GPX export with Garmin color extension
- Clear user-facing error reporting

## Phase 4 — Map Display

Status: **complete**

- SQLite tile maps (LizaAlert format) via custom `sqlite://` protocol
- OziExplorer OZF2 raster maps via custom `ozi://` protocol
- OZI georeference (`.map` file parsing, affine transformation)
- Tiled rendering with zoom level selection
- OpenStreetMap online fallback
- Track overlay on all map types

## Phase 5 — LizaAlert Integration

Status: **complete**

- Browse and download projects from maps.lizaalert.ru
- Configurable bundle storage directory
- Open local bundle from folder
- Reveal active bundle in Finder/Explorer
- Automatic `10-Tracks/` subfolder on export
- LizaAlert OK-standard track name validation (`YYYYMMDD_Callsign`)
- Last active map restored on startup when its local map path still exists

## Phase 6 — UI Polish and Stack Migration

Status: **complete**

- Migrated from egui/eframe to Tauri 2 + Svelte 5 + MapLibre GL 4 (ADR-0016)
- Catppuccin theme with Auto/Latte/Frappé/Macchiato/Mocha picker, persisted
- Sidebar with project management, import/export, mode toggles
- Developer console toggled with backtick
- Structured logging via `tracing` / `RUST_LOG`

## Phase 7 — Track Editing

Status: **complete**

- Delta-based CommandStack replacing snapshot undo (ADR-0017)
- Domain error variants for all mutations
- Track point list panel with segment hierarchy
- Move track point by drag on map (edit mode with coalesced undo)
- Delete and insert track points via right-click context menu
- Split segment at point, join adjacent segments
- Create track from scratch (drawing mode on map)
- All edits flow through CommandStack with full undo/redo

## Phase 8 — Track Simplification

Status: **complete**

- Douglas-Peucker simplification with configurable tolerance (1–1000m)
- Live preview before committing (orange overlay on map)
- Point reduction statistics shown in panel
- Implemented as reversible SimplifyTrack / RestoreTrackPoints command pair

## Phase 9 — Waypoint Editing UI

Status: **complete**

- Waypoint list panel with delete, rename, symbol picker
- Add waypoint by clicking on map (toggle mode)
- Move waypoint by drag on map
- Rename waypoint inline
- Symbol picker with 10 predefined symbols (flag, camp, danger, water, etc.)
- All edits via commands with undo/redo

## Phase 10 — Export

Status: **partially complete**

Done:
- PLT export with OLE date format, COLORREF BGR encoding
- Round-trip tested (import → export → re-import)

Deferred:
- Print map view with tracks and waypoints to PDF or image

## What's Next

Remaining work before 1.0:

- Print/export map view to PDF or image (configurable scale, paper size)
- Sort track points by timestamp (fix out-of-order GPS recordings)
- KML import/export (low priority, no current user demand)
- Multi-layer UI (backend supports it, UI currently hardcodes layer 1)

## Deferred (post-1.0)

- GPS device sync and live track recording
- Datum management and advanced projection
- Multi-map simultaneous display
- Polygon / search sector drawing
- Overlay layers (wiki, hybrid, archive)
- Style and naming templates
- Multi-device coordination
