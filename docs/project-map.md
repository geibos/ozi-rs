# Project Map

> Last verified against code: 2026-07-15

Single-page navigator for new contributors and AI agents. Use this to skip blind exploration: pick the task, jump to the files.

If you are starting fresh, also read `AGENTS.md` (entry rules), `docs/architecture.md` (layer design), and `docs/feature-status.md` (what is and isn't implemented).

## Repository Layout

```
ozi-rs/
├── AGENTS.md               # Read first. Project rules and session-startup checklist.
├── CLAUDE.md               # Claude Code overrides on top of AGENTS.md.
├── README.md               # User-facing overview.
├── justfile                # All dev commands. `just` lists recipes.
├── package.json            # Frontend deps and Vitest scripts.
├── opencode.json           # Local opencode/MCP config (registers ozi-rs-mcp).
│
├── docs/                   # All design docs (this is your map).
│   └── adr/                # 24 architecture decision records (ADR-0001..0024).
│
├── src/                    # Svelte 5 + SvelteKit (adapter-static) + MapLibre frontend.
│   ├── app.css             # Tailwind 4 entry + semantic-token layer.
│   ├── app.html            # SvelteKit document shell.
│   ├── routes/             # File-based routing:
│   │   ├── +layout.svelte  # Single host for MapView (mounted once), Console,
│   │   │                   # CommandPalette (Cmd-K), Toaster, global Tooltip.Provider.
│   │   ├── +layout.ts      # `ssr=false`, `prerender=true`.
│   │   ├── +page.svelte    # `/` — bundle loader page (hosts BundleLoader.svelte).
│   │   └── project/+page.svelte  # `/project` — workspace: WorkspaceShell + LibraryRail
│   │                       # + InspectorRail + Sheet-hosted BundleLoader.
│   ├── components/         # Feature components: WorkspaceShell, LibraryRail,
│   │   │                   # InspectorRail, CommandPalette, MapView, BundleLoader,
│   │   │                   # Console, SymbolPicker, ThemePicker (unmounted).
│   │   ├── library/        # MapsTab, TracksTab, WaypointsTab, LibraryRow.
│   │   └── inspector/      # MapInspector, TrackInspector, TrackSegmentsTable,
│   │                       # WaypointInspector.
│   ├── lib/
│   │   ├── api.ts          # Typed Tauri IPC wrappers — never bypass.
│   │   ├── stores.ts       # Svelte stores (synced + UI-only).
│   │   ├── types.ts        # TS DTOs mirroring Rust structs (manual sync).
│   │   ├── theme.ts        # Native + Catppuccin themes, semantic-token layer.
│   │   ├── utils.ts        # `cn()` helper (twMerge ∘ clsx) for shadcn primitives.
│   │   ├── track-names.ts  # OK-standard regex.
│   │   ├── recentFiles.ts  # localStorage recent map opens (Cmd-K palette).
│   │   ├── track-points.ts / track-stats.ts / waypoint-layers.ts  # UI helpers.
│   │   ├── components/
│   │   │   └── ui/         # shadcn-svelte primitives over bits-ui: button, card, command,
│   │   │                   # dialog, dropdown-menu, input, popover, scroll-area, select,
│   │   │                   # separator, sheet, slider, sonner, switch, table, tabs,
│   │   │                   # tooltip, label, textarea.
│   │   ├── forms/          # felte + zod helpers (`create-form.ts`, `schemas/`).
│   │   ├── themes/         # Theme definitions.
│   │   └── maplibre/       # Tile protocols and track GeoJSON layer.
│   └── test/               # Vitest specs.
│
├── src-tauri/              # Rust backend (Tauri 2 host).
│   ├── Cargo.toml          # Edition 2024, MSRV 1.85.
│   ├── tauri.conf.json     # Tauri runtime config (windows, bundle, CSP).
│   ├── capabilities/       # IPC permission allowlists.
│   └── src/
│       ├── lib.rs          # Tauri Builder, command registry, state init.
│       ├── main.rs         # Windows entry shim.
│       ├── domain/         # Pure entities. No IO. No GUI.
│       ├── application/    # AppState, ProjectCommand, CommandStack, import workflow.
│       ├── infrastructure/ # Format adapters, persistence, LizaAlert API, tiles.
│       └── commands/       # Tauri #[command] handlers (mod.rs + tiles.rs).
│
├── tools/
│   └── ozi-rs-mcp/         # Native QA MCP server (13 tools). See docs/native-qa-mcp.md.
│
├── scripts/                # Hygiene scripts (release-warnings gate).
└── example_data/           # Sample maps and tracks for manual testing.
```

Sibling repository `../ozf2-rs` (path dependency) supplies the OZF2 raster decoder. See ADR-0006.

## Where things live (by responsibility)

| Concern | Code | Tests | Docs |
|---------|------|-------|------|
| Project / Track / Waypoint entities | `src-tauri/src/domain/` | inline `#[cfg(test)]` | architecture.md, ADR-0014, ADR-0018 |
| ID newtypes (`LayerId`, `TrackId`, …) | `domain/project.rs`, `domain/track.rs`, `domain/waypoint.rs` | inline | ADR-0014 |
| Edit commands (apply/reverse/merge) | `application/commands.rs` (~2.6k lines) | inline | commands-reference.md, ADR-0017 |
| Application state + history | `application/mod.rs` | inline | architecture.md |
| Import (GPX/PLT/OZI/ZIP) | `infrastructure/import/` | inline | ADR-0007 (archive boundary), requirements.md |
| Export (GPX/PLT) | `infrastructure/export/` | inline | commands-reference.md |
| Project save/load (`.ozp`) | `infrastructure/persistence.rs` | inline | ADR-0004, persistence-session.md |
| App session restore | `application/mod.rs::restore_session`, `persistence.rs::*_app_session` | inline (`session_restore_*`) | persistence-session.md |
| LizaAlert API + bundles | `infrastructure/lizaalert.rs` (~1.1k lines) | inline | ADR-0008, requirements.md |
| Tile serving (sqlite/ozi) | `commands/tiles.rs` | inline | architecture.md, ADR-0010, ADR-0012 |
| Tauri IPC handlers | `commands/mod.rs` (~1.2k lines) | inline | commands-reference.md, ADR-0016 |
| MapLibre integration | `src/lib/maplibre/` | `src/test/tile-url.test.ts` | frontend-architecture.md, conventions.md |
| Svelte stores | `src/lib/stores.ts` | n/a | frontend-architecture.md |
| Typed IPC wrappers | `src/lib/api.ts` | n/a | frontend-architecture.md |
| OK-standard validation | `src/lib/track-names.ts` (UI only) | `src/test/track-name-validation.test.ts` | ADR-0019, feature-status.md |
| Bundle loader (single-window) | `src/components/BundleLoader.svelte` (`/` route + Sheet on `/project`) | `src/test/bundle-loader-*.test.ts` | frontend-architecture.md |
| shadcn-svelte primitives | `src/lib/components/ui/` (button, dialog, popover, select, scroll-area, slider, switch, separator, sonner, table, tabs, tooltip, …) | n/a | frontend-architecture.md, `add-design-tokens-and-shadcn`, `migrate-panels-to-shadcn` |
| Catppuccin theming + semantic tokens | `src/lib/theme.ts`, `src/app.css` | n/a | frontend-architecture.md |
| Native QA / desktop checks | `tools/ozi-rs-mcp/` | `tools/ozi-rs-mcp/tests/` | native-qa-mcp.md, testing-strategy.md |

## Common tasks → entry points

| If you want to… | Start in… | Then touch… |
|-----------------|-----------|-------------|
| Add a new edit operation | `application/commands.rs` (add `ProjectCommand` variant + apply + reverse) | inline test → handler in `commands/mod.rs` → registration in `lib.rs` → `api.ts` wrapper → caller component. See "Adding a ProjectCommand" in `commands-reference.md`. |
| Add a new Tauri-only IPC command (read-only/non-undoable) | `commands/mod.rs` | register in `lib.rs::generate_handler!` → `api.ts` wrapper → DTO in `types.ts` if needed. |
| Add a new import format | new file in `infrastructure/import/`, classifier in `archive.rs` | wire workflow in `application/import.rs`. ADR-0007. |
| Add a new export format | new file in `infrastructure/export/` | wire handler in `commands/mod.rs` → `api.ts` wrapper. |
| Add a Svelte store | `src/lib/stores.ts` | document in `frontend-architecture.md` (synced vs UI-only). |
| Add a new component | `src/components/` (or `library/` / `inspector/` subdir) | mount in `WorkspaceShell.svelte`, a rail/tab component, or `routes/+layout.svelte` for global surfaces. |
| Change track rendering | `src/lib/maplibre/tracks-layer.ts`, `commands/mod.rs::get_tracks_geojson` | — |
| Change tile delivery | `commands/tiles.rs`, matching protocol in `src/lib/maplibre/` | tile-url format must match in both. |
| Change project file schema | `domain/` structs | bump compatibility (ADR-0004 has no version field yet — design migration before breaking). |
| Touch session-restore behavior | `application/mod.rs::restore_session`, `infrastructure/persistence.rs::*_app_session` | docs in `persistence-session.md`. |
| Add a new ADR | `docs/adr/adr-NNNN-slug.md` | bump number, link supersessions if applicable. |

## State lifecycle (one-paragraph mental model)

1. Frontend mounts → `routes/+layout.svelte` `onMount` calls `appState.refresh()` and `loadProjects()`.
2. User action → component calls a typed wrapper from `src/lib/api.ts` (never `invoke()` directly).
3. Tauri handler in `commands/mod.rs` locks `SharedState`, mutates `AppState`, emits `state-changed`.
4. Frontend `listen("state-changed", …)` triggers `appState.refresh()` → derived stores update reactively.
5. Long-running ops (downloads, bundle scans) spawn `std::thread::spawn` and emit `download-progress` / `bundle-progress` / `projects-chunk` events; results land via `apply_*` methods on `AppState`. ADR-0011 explains why no async runtime.
6. Mutating commands flow through `CommandStack::apply` (or `apply_or_merge` for drag-coalescing). Each variant has a computed reverse for undo. ADR-0017.
7. On project save and active-map open, `AppState::persist_session_snapshot` writes the bounded session file.

## Dev commands cheat-sheet

```bash
just dev              # Full Tauri dev (Vite HMR + Rust). Default port 5173.
just dev-ui           # Frontend only (no Tauri).
just watch            # cargo watch -x check on src-tauri.
just test             # cargo test + npm test.
just clippy           # cargo clippy -- -D warnings (strict, must pass).
just check            # check-rust (cargo check) + check-ui (svelte-check).
just test-filter X    # cargo test X.
just build            # Debug Tauri bundle (needed for `just smoke`).
just smoke            # E2E gate: drives the real app via Appium Mac2
                      # (tools/ozi-rs-mcp/tests/smoke_core_workflow.rs).
just ci               # clippy + check + lint + test (pre-merge gates).
just run-release      # Build no-bundle release and run binary.
```

`RUST_LOG=debug just dev` for verbose logs. ADR-0009.

## Recommended onboarding read order

1. `AGENTS.md` — rules.
2. `README.md` — what the product does.
3. `docs/architecture.md` — layer model.
4. `docs/feature-status.md` — what is *actually* in UI vs backend-only vs planned.
5. `docs/frontend-architecture.md` — UI internals.
6. `docs/commands-reference.md` — IPC and `ProjectCommand` tables.
7. `docs/conventions.md` — coordinate systems, units, encodings (cross-cutting).
8. `docs/glossary.md` — domain and code terms.
9. `docs/persistence-session.md` — what is and isn't restored.
10. `docs/adr/adr-0019-doc-audit-reconciliation.md` … `adr-0024-playwright-not-for-desktop-qa.md` —
    the latest audit, MVP-scope (ADR-0020), no-printing (ADR-0023), and QA-policy decisions.
11. `docs/native-qa-mcp.md` — when you need to drive the desktop app for QA.

ADR-0001 + ADR-0017 are the two most load-bearing for understanding the editing model.
