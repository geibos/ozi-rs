# ozi-rs

`ozi-rs` is a Rust desktop map editor for raster maps, tracks, and waypoints, built for
[LizaAlert](https://lizaalert.org) search-and-rescue volunteers.

The goal is a modern, predictable, offline-first replacement for the useful core of
OziExplorer — without copying legacy UX or legacy data-model constraints.

## Quick Start

```bash
cargo run
```

Set the log level via the standard `RUST_LOG` environment variable (default: `info`):

```bash
RUST_LOG=debug cargo run
RUST_LOG=ozi_rs=trace cargo run
```

Press `` ` `` (backtick / tilde) to open the in-app developer console.

## Current State

The following is working:

**Maps**
- SQLite tile maps downloaded from maps.lizaalert.ru
- OziExplorer OZF2 raster maps (`.map` + `.ozf2`)
- OpenStreetMap as an online fallback

**LizaAlert integration**
- browse and download projects from maps.lizaalert.ru
- configurable local map bundle storage directory
- open local map bundles (offline, from a picked folder)
- reveal active bundle in Finder / Explorer

**Tracks**
- import GPX and PLT files
- display tracks on all map types
- per-track visibility, color picker, line width
- track name editing with LizaAlert OK-standard validation hint (`YYYYMMDD_Callsign`)
- track statistics: distance (km), duration, point count
- export track layer to GPX (with Garmin color extension)
- automatic `10-Tracks/` subfolder suggestion on export

**Project**
- project save / load (JSON)
- undo / redo via command stack
- last open project and map restored on startup

**UI**
- Catppuccin theme (Auto / Latte / Frappé / Macchiato / Mocha), persisted across restarts
- floating, resizable Tracks window
- developer console toggled with `` ` ``

## Map Bundle vs Project

These are two distinct concepts:

| Concept | What it is |
|---------|-----------|
| **Map bundle** | A directory with one or more maps for a geographic area. Downloaded from LizaAlert or opened locally. Stored wherever you configure. |
| **Project** | One SAR search operation. References tracks and waypoints. Saved as a `.json` file. Tracks are stored in a `10-Tracks/` subfolder per LizaAlert cartographer standard. |

One bundle can be referenced by multiple projects.

## LizaAlert OK Standard

Track names must follow `YYYYMMDD_Callsign` (e.g. `20240601_Иванов`). The UI shows a
warning on tracks whose names do not match this pattern.

## Planned MVP

- track point editing (move, delete, insert)
- split and join track segments
- waypoint editing UI
- Douglas-Peucker track simplification
- PLT export
- print map with tracks

## Explicit Non-Goals For MVP

- datum management
- advanced geodesy and projection features beyond immediate needs
- GPS device sync and live telemetry
- routes and events
- privileged legacy concepts such as a special `Track 1`

## Architecture

Four explicit layers:

- `domain` — business entities and invariants, no GUI dependencies
- `application` — commands, use-cases, undo/redo orchestration
- `infrastructure` — persistence, import/export, file format adapters
- `ui` — rendering, interaction, transient view state

All non-trivial edits flow through explicit commands. Domain logic stays serializable
and testable without the UI runtime.

## Documentation

- `docs/requirements.md` — product requirements and user workflows
- `docs/architecture.md` — layer responsibilities and editing model
- `docs/testing-strategy.md` — test layers and quality gates
- `docs/roadmap.md` — phases, backlog, and triage
- `docs/adr/` — architectural decision records
