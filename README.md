# ozi-rs

`ozi-rs` is a Rust desktop map editor for raster maps, tracks, and waypoints.

The goal is to build a modern, predictable, and testable replacement for the useful core of OziExplorer without copying legacy UX or legacy data-model constraints.

## Status

This repository is in early bootstrap with the first domain, application, and map-loading slices underway.

Current focus:
- define product scope and non-goals;
- lock architecture boundaries;
- establish testing strategy;
- create a prioritized implementation roadmap;
- stand up a minimal desktop shell for fast iteration;
- grow the Phase 1 core project model and command flow around independent map, track, and waypoint layer collections.

## Quick Start

```bash
cargo run
```

The current application opens a minimal native window and shows the initial project shell.

## Planned MVP

- open raster maps;
- load multiple tracks and waypoint collections;
- create and edit tracks and waypoints;
- split and join track segments;
- save projects and export data.

## Explicit Non-Goals For MVP

- datum management;
- advanced geodesy and projection features beyond immediate needs;
- GPS device sync;
- routes, events, and live telemetry;
- privileged legacy concepts such as a special `Track 1`.

## Architecture

The codebase is intended to keep these boundaries explicit:
- `domain` - business entities and invariants;
- `application` - commands, use-cases, undo/redo orchestration;
- `infrastructure` - persistence, import/export, file formats;
- `ui` - rendering, interaction, and view state.

The UI is an adapter over application services. Domain logic stays serializable and testable without the UI runtime.

## Documentation

Project planning and architecture docs live in `docs/`:
- `docs/requirements.md`
- `docs/architecture.md`
- `docs/testing-strategy.md`
- `docs/roadmap.md`
- `docs/adr/`

## Current Gaps

- project editing workflows are still in the first command-driven slices;
- external OziExplorer add-on material from Yonote still needs a fuller structured extraction, especially where workflows depend on screenshots and image-heavy explanations;
- the first implementation backlog still needs to be refined from the Yonote-derived workflow material.

## Next Step

Keep building the domain and application core needed to register opened maps, attach track/waypoint layers, and support explicit edit commands with undo/redo.
