# ozi-rs

`ozi-rs` is a Tauri 2 desktop map editor for raster maps, tracks, and waypoints, built for
[LizaAlert](https://lizaalert.org) search-and-rescue volunteers.

The goal is a modern, predictable, offline-first replacement for the useful core of
OziExplorer — without copying legacy UX or legacy data-model constraints.

**Stack:** Rust (Tauri 2) + Svelte 5 + MapLibre GL 4

## Quick Start

Requires: Rust (2024 edition), Node.js, [`just`](https://github.com/casey/just) task runner.

```bash
npm install
just dev
```

Set the log level via `RUST_LOG` (default: `info`):

```bash
RUST_LOG=debug just dev
RUST_LOG=ozi_rs=trace just dev
```

Run `just` to see all available recipes. Key commands:

| Task | Command |
|------|---------|
| Dev server (full) | `just dev` |
| Frontend only | `just dev-ui` |
| All tests | `just test` |
| Clippy (strict) | `just clippy` |

Press `` ` `` (backtick) to open the in-app developer console.

## Current State

### Maps
- SQLite tile maps downloaded from maps.lizaalert.ru
- OziExplorer OZF2 raster maps (`.map` + `.ozf2`)
- OpenStreetMap as an online fallback
- Custom tile protocols: `sqlite://` for MBTiles, `ozi://` for OZF2 raster

### LizaAlert Integration
- Browse and download projects from maps.lizaalert.ru
- Configurable local map bundle storage directory
- Open local map bundles (offline, from a picked folder)
- Reveal active bundle in Finder / Explorer

### Tracks
- Import GPX and PLT files (including ZIP archives)
- Display tracks on all map types
- Per-track visibility, color picker, line width
- Track name editing with warning-only LizaAlert OK-standard validation (`YYYYMMDD_Callsign`)
- Track statistics: distance (km), duration, point count
- Export track layer to GPX (with Garmin color extension)
- Export individual tracks to PLT (OziExplorer format)
- `10-Tracks/` subfolder suggestion on GPX/PLT export when an active bundle is known

### Track Editing
- Track point list panel with segment hierarchy
- Move track points by drag on map (edit mode)
- Delete and insert track points (right-click context menu)
- Split segment at point, join adjacent segments
- Create new tracks by drawing on map (click to add points, double-click to finish)
- Douglas-Peucker track simplification with live preview and tolerance slider

### Waypoints
- Add waypoints by clicking on map
- Move waypoints by drag on map
- Rename and delete waypoints
- Symbol picker (flag, camp, danger, water, shelter, etc.)
- Waypoint markers with emoji icons

### Project
- Project save / load (JSON `.ozp` format)
- Undo / redo via delta-based command stack with drag coalescing
- Last project path and active map restored on startup when referenced files still exist
- Viewport, selected entities, panels, undo history, theme, and unsaved edits are not restored by the Rust session file

### UI
- Catppuccin theme (Auto / Latte / Frappé / Macchiato / Mocha), persisted
- Sidebar with project controls, active-layer selection, import/export, drawing/waypoint mode toggles
- Panels: Tracks, Track Points, Waypoints, Simplify
- Developer console toggled with `` ` ``
- FPS counter (F3)
- Keyboard shortcuts: Ctrl+Z/Y (undo/redo), Enter/Esc (drawing mode)

## Map Bundle vs Project

| Concept | What it is |
|---------|-----------|
| **Map bundle** | A directory with one or more maps for a geographic area. Downloaded from LizaAlert or opened locally. |
| **Project** | One SAR search operation. Contains tracks and waypoints. Saved as `.ozp`. Exports go to `10-Tracks/` per LizaAlert standard. |

One bundle can be referenced by multiple projects.

## LizaAlert OK Standard

Track names must follow `YYYYMMDD_Callsign` (e.g. `20240601_Иванов`). The UI shows a
warning on tracks whose names do not match this pattern.

## Remaining Work

- Print map with tracks and waypoints to PDF or image
- Sort track points by timestamp
- KML import/export
- Full layer management UI for create/rename/delete/reorder; current UI surfaces active-layer selection only

## Explicit Non-Goals

- Datum management
- Advanced geodesy and projection features beyond immediate needs
- GPS device sync and live telemetry
- Routes and events
- Privileged legacy concepts such as a special `Track 1`
- Polygon / search sector drawing (post-MVP)

## Architecture

Four explicit layers:

```
UI (Svelte 5 + MapLibre GL 4)
  ↕ Tauri IPC
Commands layer  ── Tauri #[command] handlers, thin wrappers
Application     ── AppState, ProjectCommand enum, delta-based undo/redo
Infrastructure  ── File I/O: GPX/PLT import-export, LizaAlert API, tile serving
Domain          ── Pure entities: Project, Track, Waypoint, LayerId (no IO)
```

All non-trivial edits flow through explicit commands. Domain logic stays serializable
and testable without the UI runtime.

## Documentation

- `docs/requirements.md` — product requirements and user workflows
- `docs/architecture.md` — layer responsibilities and editing model
- `docs/frontend-architecture.md` — UI stack, components, state management
- `docs/commands-reference.md` — ProjectCommand and Tauri IPC reference
- `docs/testing-strategy.md` — test layers and quality gates
- `docs/roadmap.md` — phases and status
- `docs/adr/` — architecture decision records (19 ADRs)
