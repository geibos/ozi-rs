# Architecture

## Overview

The system is organized around a small, explicit core that keeps domain rules independent from UI concerns. The UI is a Tauri 2 desktop app with a Svelte 5 frontend and MapLibre GL 4 for map rendering.

```
UI (Svelte 5 + MapLibre GL 4)
  ↕ Tauri IPC (invoke / events)
Commands layer  ── Tauri #[command] handlers, thin wrappers
Application     ── AppState, ProjectCommand enum, delta-based undo/redo
Infrastructure  ── File I/O, LizaAlert API, tile serving, persistence
Domain          ── Pure entities: Project, Track, Waypoint (no IO, no GUI)
```

## Architectural Principles

- Prioritize correctness, clarity, and testability
- Keep business logic outside the GUI
- Use explicit commands for all edits
- Delta-based undo/redo with command merging for drag operations
- Lower layers must not depend on upper layers
- Domain is always pure Rust with no external I/O

## Layer Responsibilities

### Domain

Owns:
- Core entities: `Project`, `MapLayer`, `TrackLayer`, `WaypointLayer`, `Track`, `TrackSegment`, `TrackPoint`, `Waypoint`
- u64 newtype identifiers: `ProjectId`, `LayerId`, `TrackId`, `TrackSegmentId`, `TrackPointId`, `WaypointId`
- `TrackStyle` (color, line width, visibility, opacity)
- Entity mutation methods (move point, split segment, join, etc.)
- Domain-level validation

Constraints:
- No GUI dependencies
- No persistence-format leakage
- Deterministic behavior suitable for unit tests

### Application

Owns:
- `ProjectCommand` enum — all edit operations as data (20+ variants)
- `CommandStack` — delta-based undo/redo (forward + reverse command pairs)
- `AppState` — root state container with project, history, LizaAlert state
- `apply_or_merge()` — command coalescing for drag sequences
- Import workflow orchestration

Constraints:
- May depend on domain
- Exposes stable operations to the UI
- Testable with lightweight fakes

### Infrastructure

Owns:
- Import adapters: GPX, PLT, OZI map metadata, OZI raster (OZF2), ZIP archives
- Export adapters: GPX (with Garmin color extension), PLT (OLE dates, COLORREF BGR)
- Project persistence: JSON save/load (`.ozp` files)
- App session persistence: bounded restore of last project path and active map reference/path
- LizaAlert API: browse/download projects from maps.lizaalert.ru
- Tile serving: SQLite (MBTiles) and OZF2 raster tile decoding

Constraints:
- May depend on domain and application contracts
- Format-specific behavior stays out of the core model

### UI (Svelte + MapLibre)

Owns:
- Map rendering via MapLibre GL 4 with custom tile protocols
- Svelte 5 components: Sidebar, panels, pickers
- Interaction modes: drawing, editing, waypoint placement
- Svelte stores for reactive state management
- `api.ts` — typed wrappers for Tauri IPC (never call `invoke` directly in components)

Constraints:
- Acts as an adapter, not the home for business rules
- UI-only state (selections, panel visibility) stays transient

## Module Layout

```
src-tauri/src/
  domain/
    project.rs      # Project, MapLayer, TrackLayer, WaypointLayer
    track.rs         # Track, TrackSegment, TrackPoint, TrackStyle
    waypoint.rs      # Waypoint (name, position, optional symbol)
    mod.rs           # Public exports

  application/
    commands.rs      # ProjectCommand enum, CommandStack, CommandDelta
    mod.rs           # AppState, mutation methods, LizaAlert state
    import.rs        # Import workflow orchestration

  infrastructure/
    export/
      gpx.rs         # GPX 1.1 with Garmin color extensions
      plt.rs         # OziExplorer PLT v2.1
    import/
      gpx.rs         # GPX import (file and ZIP archive)
      plt.rs         # PLT import (Windows-1251 encoding)
      ozi_map.rs     # .map file parsing (calibration points)
      ozi_georeference.rs  # Affine transformation (lat/lon ↔ pixel)
      ozi_raster.rs  # OZF2 tile decoding via ozf2-rs
      archive.rs     # ZIP entry classification and extraction
    lizaalert.rs     # maps.lizaalert.ru API, bundle download
    persistence.rs   # JSON project save/load

  commands/
    mod.rs           # All Tauri #[command] handlers
    tiles.rs         # sqlite:// and ozi:// tile serving

  lib.rs             # Tauri app init, command registration
  main.rs            # Windows entry point

src/
  components/        # Svelte 5 components (see frontend-architecture.md)
  views/             # Page-level views
  lib/
    api.ts           # Typed Tauri IPC wrappers
    stores.ts        # Svelte stores (app state, UI state)
    types.ts         # TypeScript interfaces matching Rust structs
    theme.ts         # Catppuccin CSS custom properties
    maplibre/        # MapLibre integration
      sqlite-protocol.ts   # sqlite:// tile protocol handler
      ozi-protocol.ts      # ozi:// tile protocol handler
      tracks-layer.ts      # GeoJSON track rendering
      tile-url.ts          # Tile URL construction
```

## Editing Model

All non-trivial edits flow through `ProjectCommand` variants. Each command:

1. Validates before applying
2. Has a computed inverse via `reverse()` for undo
3. Is stored as a `CommandDelta` (forward + reverse pair) in the undo stack

The `CommandStack` supports:
- **Undo/redo** with a max depth of 100
- **Command merging** via `apply_or_merge()` — consecutive commands targeting the same entity (e.g., drag moves) coalesce into one undo step
- **Redo clearing** — new commands clear the redo history

Waypoint symbol changes are project edits and use `ProjectCommand::SetWaypointSymbol`, so
undo restores the previous symbol and redo reapplies the committed picker choice. Some
track style mutations (`SetTrackColor`, `SetTrackLineWidth`, `ToggleTrackVisible`) bypass
the command stack and remain immediate, non-undoable changes because they are
non-destructive visual styling updates.

## Tauri IPC

Frontend communicates with the backend exclusively through `invoke()` calls wrapped in `src/lib/api.ts`. The backend emits events (`state-changed`, `download-progress`, `projects-chunk`, `bundle-progress`) for async updates.

State flow:
1. Frontend calls `api.someCommand(params)`
2. Tauri handler locks `AppState`, applies mutation
3. Handler emits `state-changed` event
4. Frontend `$effect` catches event, calls `appState.refresh()`
5. Derived stores update reactively

## Tile Delivery

MapLibre uses custom protocol handlers instead of an HTTP tile server:

- **`sqlite://`** — MBTiles format. URL: `sqlite://<path>/<base_zoom>/{z}/{x}/{y}`. Backend queries `SELECT image FROM tiles WHERE x=?, y=?, z=?` with zoom inversion.

- **`ozi://`** — OZF2 raster. URL: `ozi://<map_path>/{z}/{x}/{y}`. Backend reprojects OZF2 tiles to Web Mercator and returns 256×256 PNG. All coordinate math happens in Rust.

Both protocols are registered in MapLibre via `addProtocol()`.

## Persistence Boundaries

- **Project files** (`.ozp`): JSON serialization of the `Project` struct. Contains tracks, waypoints, layer metadata. Does not contain map data.
- **Map bundles**: directories with `.sqlitedb` or `.map`+`.ozf2` files. Shared across projects, stored separately.
- **App session file**: stores only the last project path and active map reference/path for startup restore when referenced files still exist. It does not restore viewport, selected entities, panel state, bundle-loader windows, undo history, theme, or unsaved edits.
- **UI state**: transient. Panel visibility, selections, edit mode — not persisted to project files. Theme choice persisted to localStorage.

## Key External Dependencies

| Crate | Purpose |
|-------|---------|
| `tauri` 2 | Desktop app framework, IPC |
| `serde` / `serde_json` | Serialization |
| `gpx` 0.10 | GPX XML parsing |
| `rusqlite` 0.39 | SQLite tile queries |
| `reqwest` 0.13 | HTTP (LizaAlert downloads) |
| `ozf2-rs` | OZF2 raster format decoder (local crate) |
| `chrono` 0.4 | Date/time handling |
| `zip` 8.4 | ZIP archive handling |
| `tracing` 0.1 | Structured logging |
| `image` 0.25 | PNG encoding for tiles |
