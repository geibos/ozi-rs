# Roadmap

> Last verified against code: 2026-07-15

Phase statuses below are historical records of what was completed at the time.
The binding scope declaration is ADR-0020 (`docs/adr/adr-0020-mvp-scope.md`);
"What's Next" lists the ADR-0020 must-have items that are still **not implemented**.

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
- Last project path and active map restored on startup when referenced files still exist
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
- Active-bundle `10-Tracks/` subfolder suggestion for GPX/PLT export dialogs
- Warning-only LizaAlert OK-standard track name validation (`YYYYMMDD_Callsign`)
- Active-layer selection for existing track and waypoint workflows; full layer management is not implemented

## Phase 6 — UI Polish and Stack Migration

Status: **complete**

- Migrated from egui/eframe to Tauri 2 + Svelte 5 + MapLibre GL 4 (ADR-0016)
- Catppuccin theme with Auto/Latte/Frappé/Macchiato/Mocha picker, persisted
  (the picker lost its mount point in the 2026-05-26 workspace redesign — see "What's Next")
- Sidebar with project management, import/export, mode toggles
  (replaced on 2026-05-26 by `WorkspaceShell` + `LibraryRail` tabs + `InspectorRail` + Cmd-K palette)
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
  (`split_segment` / `join_segments` backend + `api.ts` wrappers remain, but the UI entry
  point was lost in the 2026-05-26 workspace redesign — see "What's Next")
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
- WPT waypoint export (ADR-0022)

Not done:
- Waypoint export to GPX and PLT (ADR-0020 must — see "What's Next")

Map printing is **not planned** (ADR-0023) and is intentionally absent from this roadmap.

## What's Next

ADR-0020 must-have items that are **not implemented** (no backend command and/or no UI),
verified against `src-tauri/src/lib.rs::generate_handler!` and `src/components/`:

- **Sort track points by timestamp** — no backend command, no UI (required per ADR-0020;
  `docs/requirements.md` tracks it as required, not deferred)
- **Crop track** by current map extent / time range / selected points — no backend, no UI
- **On-map tools** (ADR-0020 section "On-map tools") — none exist:
  - Distance measurement
  - Circle with center at point/cursor and explicit radius
  - Place waypoint by projection (azimuth + distance) from a selected point
- **Waypoint export to GPX and PLT** — no commands (`export_gpx` exports track layers only;
  an unwired `build_waypoint_gpx_xml` helper sits in `infrastructure/export/gpx.rs`);
  only WPT export is implemented
- **Open LizaAlert bundle by URL** — no URL input; only catalog browsing and local folders
- **ZIP archive import via UI** — backend classifies ZIPs, but the import pickers filter
  to `.gpx`/`.plt` only
- **Track point walkthrough** — points are click-selectable, but there are no
  next/previous controls
- **Recent projects list** — only recent *map opens* exist (frontend localStorage in the
  Cmd-K palette), not recent `.ozp` projects

Regressions from the 2026-05-26 workspace redesign:

- **Split/join segment UI** — `split_segment`/`join_segments` backend and `api.ts`
  wrappers exist, but no component calls them
- **Theme picker** — `ThemePicker.svelte` is not mounted anywhere; theme cannot be
  changed from the UI (ADR-0020 lists Catppuccin themes as in-scope)
- **Undo/redo keyboard shortcuts** — no Ctrl+Z/Ctrl+Y bindings; palette entries only

Other remaining work before 1.0:

- KML import/export (low priority, no current user demand)
- Full layer management UI (create, rename, delete, reorder); backend supports multiple layers and the UI currently surfaces active-layer selection only

## Deferred (post-1.0)

- GPS device sync and live track recording
- Datum management and advanced projection
- Multi-map simultaneous display
- Polygon / search sector drawing
- Overlay layers (wiki, hybrid, archive)
- Style and naming templates
- Multi-device coordination
