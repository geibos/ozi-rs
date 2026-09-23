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

The ADR-0020 must-have list is closed. Every item that stood here on
2026-09-21 was built over the two days that followed, each verified against
`src-tauri/src/lib.rs::generate_handler!`, `src/lib/api.ts` and its caller in
`src/components/` rather than against the previous version of this list:

- **Sort track points by timestamp** — `sort_track_points`, reached from the
  Track Inspector. It had been implemented for some time; this page said
  otherwise until 2026-09-21.
- **Crop track** — `crop_track_to_extent` and `crop_track_to_time`. Crop by
  selected points arrived on 2026-09-22 as `trim_track_at_point`, in two
  halves: trim before a point, trim after it. Two trims are a selection crop,
  and each half is the gesture a crew actually has.
- **On-map tools** — all three, from the command palette: distance, a geodesic
  radius ring, and placing a waypoint by bearing and distance.
- **Waypoint export to GPX** — `export_gpx_waypoints`, from the Waypoint
  Inspector and the row menu, beside the OziExplorer WPT export.
- **Open LizaAlert bundle by URL** — a catalogue link pasted into the loader's
  search box opens that search.
- **ZIP archive import via UI** — the import picker takes `gpx`, `plt` and
  `zip`.
- **Track point walkthrough** — previous and next in the points table, with the
  position shown and the map following.
- **Recent projects list** — `src/lib/recent-projects.ts`, offered in the
  command palette, beside the recent-maps list it is not the same thing as.

What is left is not a feature list. `docs/STATE.md` holds it: five decisions
that are the owner's to make, FTP when they are ready, the MapLibre 4 → 6
upgrade, and the internal rebuild — a view-model layer and a screenshot matrix
— which no operator can see and which is tracked as its own OpenSpec change.

The smoke gate is no longer blocked: two desktop journeys ran green on
2026-09-23 against a freshly built bundle. Six of the eight Customer Journeys
still have no desktop journey of their own, and `tools/ozi-rs-mcp/tests/smoke_core_workflow.rs`
carries a line each saying what stands in the way.

Regressions from the 2026-05-26 workspace redesign, re-checked 2026-09-22:

- ~~Split/join segment UI~~ — the Track Inspector's segments table calls both.
- ~~Theme picker~~ — the themes are in the command palette
  (`CommandPalette.svelte`), which is where a setting changed twice a year
  belongs. `ThemePicker.svelte` stayed unimported and was dropped.
- ~~Undo/redo keyboard shortcuts~~ — the layout binds Cmd/Ctrl+Z and
  Shift for redo (`handleGlobalKeydown`, `src/routes/+layout.svelte`).

Other remaining work before 1.0:

- KML import/export (low priority, no current user demand)
- Reordering layers. Creating, renaming and deleting them shipped on
  2026-09-21 (`layers`); reordering is the part still not built.

## Deferred (post-1.0)

- GPS device sync and live track recording
- Datum management and advanced projection
- Multi-map simultaneous display
- Polygon / search sector drawing
- Overlay layers (wiki, hybrid, archive)
- Style and naming templates
- Multi-device coordination
