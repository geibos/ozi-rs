# ozi-rs

> Last verified against code: 2026-07-15

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
- Import GPX and PLT files (ZIP archives supported by the backend; the UI file picker currently filters to `.gpx`/`.plt` only)
- Display tracks on all map types
- Per-track visibility, color picker, line width
- Track name editing with warning-only LizaAlert OK-standard validation (`YYYYMMDD_Callsign`)
- Track statistics: distance (km), duration, point count
- Export track layer to GPX (with Garmin color extension)
- Export individual tracks to PLT (OziExplorer format)
- `10-Tracks/` subfolder suggestion on GPX/PLT export when an active bundle is known

### Track Editing
- Per-segment track point table in the Inspector rail (select a track to open it)
- Move track points by drag on map (edit mode)
- Delete and insert track points (right-click context menu)
- Split/join segments — backend commands exist; no UI entry point at the moment (lost in the 2026-05-26 workspace redesign)
- Create new tracks by drawing on map (click to add points, double-click or Enter to finish, Esc to cancel)
- Douglas-Peucker track simplification with live preview and tolerance control

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
- Workspace shell: Library rail with Maps / Tracks / Waypoints tabs (left), map canvas (center), Inspector rail for the selected map/track/waypoint (right)
- Cmd/Ctrl+K command palette: open/save project, switch project/map, undo/redo, find track/waypoint, exports, recent files
- Bundle loader on the `/` route and as an in-workspace overlay ("Maps…")
- Theme engine (Native Auto default + Catppuccin palette, persisted); the theme picker is currently not mounted in the redesigned shell
- Developer console toggled with `` ` ``
- FPS counter (F3)
- Keyboard shortcuts: Cmd/Ctrl+K (palette), Enter/Esc (drawing mode); undo/redo is reachable via the palette — no Ctrl+Z/Y bindings yet

## Map Bundle vs Project

| Concept | What it is |
|---------|-----------|
| **Map bundle** | A directory with one or more maps for a geographic area. Downloaded from LizaAlert or opened locally. |
| **Project** | One SAR search operation. Contains tracks and waypoints. Saved as `.ozp`. GPX/PLT export dialogs suggest the active bundle's `10-Tracks/` subfolder when available; users may choose another path. |

One bundle can be referenced by multiple projects.

## LizaAlert OK Standard

Track names must follow `YYYYMMDD_Callsign` (e.g. `20240601_Иванов`). The UI shows a
warning on tracks whose names do not match this pattern.

## Remaining Work

ADR-0020 must-have items not yet implemented (see `docs/roadmap.md` for the full list):

- Sort track points by timestamp
- Crop track (by map extent, time range, selected points)
- On-map tools: distance measurement, circle with explicit radius, waypoint by projection (azimuth + distance)
- Waypoint export to GPX and PLT (WPT export is implemented)
- ZIP import via the UI file picker; open LizaAlert bundle by URL
- Re-expose split/join segments and the theme picker in the redesigned UI

Other:

- KML import/export
- Full layer management UI for create/rename/delete/reorder; current UI surfaces active-layer selection only

## Explicit Non-Goals

- Map printing to PDF or image (ADR-0023 — deliberate decision, not a deferral)
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

- `docs/project-map.md` — file/responsibility navigator (start here)
- `docs/requirements.md` — product requirements and user workflows
- `docs/architecture.md` — layer responsibilities and editing model
- `docs/frontend-architecture.md` — UI stack, components, state management
- `docs/commands-reference.md` — ProjectCommand and Tauri IPC reference
- `docs/conventions.md` — coordinate order, tile URLs, color encodings, naming
- `docs/glossary.md` — domain and code terminology
- `docs/feature-status.md` — backend/UI/docs status matrix
- `docs/persistence-session.md` — startup restore scope
- `docs/native-qa-mcp.md` — native desktop QA via the project-local MCP
- `docs/testing-strategy.md` — test layers and quality gates
- `docs/roadmap.md` — phases and status
- `docs/adr/` — architecture decision records (24 ADRs)

## Credits

The UI kit stands on the shoulders of the following projects (see
[`THIRD_PARTY_LICENSES.md`](THIRD_PARTY_LICENSES.md) for the full list and
their licences):

- [`shadcn-svelte`](https://shadcn-svelte.com) — primitive scaffolding
- [`bits-ui`](https://bits-ui.com) — headless behaviour
- [`@catppuccin/palette`](https://github.com/catppuccin/palette) — canonical colour source
