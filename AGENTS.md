# AGENTS.md

Project instructions for AI coding agents. Read this file first when working on this repository.

## Session startup context

For maximum context in every new session:

1. Read this `AGENTS.md` first.
2. Read `docs/project-map.md` for the file/responsibility navigator and onboarding read order.
3. Read `docs/feature-status.md` to understand what is implemented in backend, surfaced in UI, documented, or still planned.
4. Read `docs/adr/adr-0019-doc-audit-reconciliation.md` for the latest audit decisions and non-goals.
5. For architecture-sensitive work, also read `docs/architecture.md`, `docs/frontend-architecture.md`, `docs/commands-reference.md`, and `docs/conventions.md` (coordinate order, IDs, encodings) before changing code.
6. For session/project/map behavior, read `docs/persistence-session.md`.
7. For native desktop QA, read `docs/native-qa-mcp.md` (the project-local MCP server replaces Playwright as the default) and `docs/agent-verification.md` (binding verification protocol — read before claiming any desktop fix or feature works).
8. Check the current git state before editing because this repository may contain unrelated dirty files from prior work.
9. Prefer evidence over assumptions: search existing code patterns, keep frontend IPC calls in `src/lib/api.ts`, and run targeted tests plus `just test` / `just clippy` before completion.

## What this project is

**ozi-rs** is a Tauri 2 desktop application for [LizaAlert](https://lizaalert.org) search-and-rescue volunteers — an offline-first map editor replacing OziExplorer. We don't need all OziExplorer features, only those actually used by SAR volunteers.

**Stack:** Rust backend (Tauri 2) + Svelte 5 + MapLibre GL 4 frontend.

## Commands

All common tasks are in `justfile` (requires `just`). Run `just` to list recipes.

| Task | Command |
|------|---------|
| Dev server (full) | `just dev` |
| Frontend only | `just dev-ui` |
| Rust watch/check | `just watch` |
| All tests | `just test` |
| Rust tests only | `just test-rust` |
| Specific Rust test | `just test-filter <name>` |
| Frontend tests | `just test-ui` |
| Clippy | `just clippy` |
| Cargo check | `just check` |

Clippy is strict: `cargo clippy -- -D warnings`. All warnings must be fixed, not suppressed.

To run a single Rust test: `cargo test --manifest-path src-tauri/Cargo.toml <test_name>`

## Architecture

Four strict layers (details in `docs/architecture.md`):

```
UI (Svelte 5 + MapLibre GL 4)
  ↕ Tauri IPC (invoke / events)
Commands layer  ── Tauri #[command] handlers, thin wrappers only
Application     ── AppState, ProjectCommand enum, delta-based undo/redo
Infrastructure  ── File I/O: GPX/PLT import-export, LizaAlert API, tile serving
Domain          ── Pure entities: Project, Track, Waypoint, LayerId (no IO, no GUI)
```

**Key rule:** lower layers must not depend on upper layers. Domain is always pure Rust with no external I/O.

### Module layout

```
src-tauri/src/
  domain/          # Project, Track, Waypoint, TrackStyle, ID newtypes
  application/     # ProjectCommand, CommandStack (delta-based undo), AppState
  infrastructure/  # Import/export (GPX, PLT, OZI), persistence (JSON), LizaAlert API
  commands/        # Tauri IPC handlers (mod.rs) + tile serving (tiles.rs)
  lib.rs           # Tauri init + command registration

src/
  components/      # Svelte 5 components (MapView, Sidebar, panels, pickers)
  views/           # Page-level views (BundleLoaderView)
  lib/
    api.ts         # Typed Tauri IPC wrappers — ONLY way to call backend
    stores.ts      # Svelte stores (app state + UI-only state)
    types.ts       # TypeScript interfaces matching Rust structs (manual sync)
    theme.ts       # Catppuccin CSS custom properties
    maplibre/      # Tile protocols (sqlite://, ozi://), track rendering
```

### Command-driven editing

All edits go through `ProjectCommand` variants in `application/commands.rs`. Each command:
- Validates before applying
- Has a computed inverse via `reverse()` for undo
- Is stored as a `CommandDelta` (forward + reverse pair) in the undo stack
- Drag operations are coalesced via `apply_or_merge()` into single undo steps

New edit operations need: a new `ProjectCommand` variant, `apply()` + `reverse()` implementation, a Tauri handler in `commands/mod.rs`, and a typed wrapper in `api.ts`.

Full command list: `docs/commands-reference.md`.

### Tauri IPC

Frontend calls: `await api.someFunction(params)` (wrappers in `src/lib/api.ts`).
Never call `invoke()` directly in components.

Backend registers handlers in `lib.rs` via `tauri::generate_handler![]`.
Handlers emit `state-changed` event after mutations; frontend re-fetches state reactively.

TypeScript types in `src/lib/types.ts` must stay in sync with Rust structs manually — there is no auto-generation.

### Tile delivery

MapLibre uses custom protocol handlers (not an HTTP server):
- `sqlite://` → `get_sqlite_tile` command (MBTiles)
- `ozi://` → `get_ozi_tile_projected` command (OZF2 raster, reprojected to Web Mercator in Rust)

Both protocols registered via `addProtocol()` in `src/lib/maplibre/`.

### Map bundles vs. projects

- **Bundle**: a directory of georeferenced maps (SQLite or OZF2) downloaded from LizaAlert or local. Shared across projects.
- **Project**: a single SAR operation — references one bundle, contains track/waypoint layers, saved as `.ozp` (JSON). GPX/PLT export dialogs suggest the active bundle's `10-Tracks/` subfolder when available; users may choose another path.

## LizaAlert OK standard

Track names must follow `YYYYMMDD_Callsign` (e.g. `20240601_Иванов`). The UI shows warning-only validation for this pattern. GPX/PLT export dialogs default to the active bundle's `10-Tracks/` directory when available; users may choose another path.

## Frontend conventions

- **State**: Svelte stores in `src/lib/stores.ts`
- **API calls**: typed wrappers in `src/lib/api.ts` (never call `invoke` directly)
- **Theming**: Catppuccin palette via CSS custom properties (`--ctp-*`); applied by `src/lib/theme.ts`
- **Theme options**: Auto (OS), Latte, Frappé, Macchiato, Mocha
- **Interaction modes**: drawing (track creation), editing (point drag), waypoint placement, simplification preview

Details: `docs/frontend-architecture.md`.

## Testing conventions

- Rust tests live alongside source files (inline `#[cfg(test)]` modules)
- Domain layer: pure unit tests, no IO
- Application layer: command tests (apply, undo, error cases)
- Infrastructure: format adapter tests (GPX, PLT round-trips)
- Frontend: Vitest in `src/test/`
- Desktop QA for native Tauri behavior should use the project-local `ozi-rs-mcp` native MCP tools by default. Use Tier 1 native tools for build/launch/log/screenshot/stop checks without requiring Appium. Appium Mac2 checks are optional and dependency-gated; unavailable Appium is a valid degraded path when Tier 1 passes.
- Playwright/browser testing is not the default for native desktop QA. It may still be added later for intentional isolated web/frontend experiments.

Verification: `just test` (all), `just clippy` (strict linting).

## Documentation

- `docs/project-map.md` — single-page navigator: where things live, common-task entry points, onboarding read order
- `docs/architecture.md` — layer design, module layout, dependencies
- `docs/frontend-architecture.md` — components, stores, tile protocols, theme
- `docs/commands-reference.md` — `ProjectCommand` and Tauri IPC command tables, plus the "Adding a new `ProjectCommand`" checklist
- `docs/conventions.md` — coordinate order, tile URL formats, color encodings, naming, concurrency
- `docs/glossary.md` — domain (LizaAlert, OK-standard, bundles) and code terms
- `docs/feature-status.md` — backend / UI / docs / status / evidence matrix
- `docs/persistence-session.md` — what is and isn't restored at startup
- `docs/native-qa-mcp.md` — `tools/ozi-rs-mcp` reference for native desktop QA
- `docs/requirements.md` — product goals, user workflows, MVP scope
- `docs/roadmap.md` — phase status and remaining work
- `docs/testing-strategy.md` — test layers and quality gates
- `docs/adr/` — 19 architecture decision records
