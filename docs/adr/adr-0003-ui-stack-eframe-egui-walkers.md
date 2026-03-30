# ADR-0003: UI Stack — eframe + egui + walkers

- Status: accepted (under review)
- Date: 2026-03-23

## Context

The project needed a Rust-native desktop UI framework that could:
- render custom raster tile maps with pan and zoom
- support immediate, interactive track/waypoint overlays
- compile to a single native binary without a web runtime
- allow fast prototyping of a novel editing UX

Three alternatives were evaluated (see `.tmp/external-context/`):

| Option | Summary |
|--------|---------|
| **eframe + egui** | Immediate-mode, pure Rust, minimal startup, active community |
| **iced** | Elm-architecture, retain mode, better suited for static UIs |
| **tauri** | Web frontend over Rust backend, Electron-style overhead |

`walkers` was evaluated separately as a tile map widget built on top of egui that
provides OSM-compatible tile rendering and a plugin API for custom overlays.

## Decision

Use **eframe 0.34 + egui 0.34 + walkers 0.53** as the UI stack.

Rationale:
- eframe/egui has the lowest boilerplate for a map editor prototype; editing widgets
  are expressed as plain Rust functions without callback wiring
- `egui` does not store application state — state lives in `AppState`, respecting the
  architectural boundary
- `walkers` provides OSM tile rendering and a `Plugin` trait for overlays out of the box
- the combination was already proven to support custom SQLite tile backends and
  raster rendering via `PaintCallback`
- single binary, no web runtime, pure Rust

## Known Limitations (recorded at decision time)

- Immediate mode redraws every frame; wasteful for a mostly-static map editor
- First-frame drag jitter is documented by egui itself
- egui explicitly does not target native-looking UI
- Breaking changes between egui minor versions are common
- Native multi-window support (viewports) is relatively new and may have rough edges
- `catppuccin-egui` did not yet support egui 0.34 at decision time (theme implemented
  manually as a result)

## Status

This decision is **under review** as of 2026-03-30. The user has raised that egui's
immediate mode architecture creates real friction for drag-based track point editing
and native multi-window layout — both features that are in the Phase 7 backlog.
Alternatives (iced, slint) should be re-evaluated before Phase 7 begins.

## Consequences

- All rendering and interaction code lives in `src/ui/`
- Track/waypoint editing gestures must be implemented against egui's input model
- Each new egui version may require migration work
