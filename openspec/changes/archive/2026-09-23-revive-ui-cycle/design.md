## Context

ozi-rs is a Tauri 2 desktop app (Rust core, Svelte 5 + MapLibre frontend) for LizaAlert search-and-rescue field work. State as of 2026-09-19:

- The Rust core is the project's asset: domain/application/infrastructure layers, atomic saves and downloads, encoding handling, typed IPC through tauri-specta, 245 tests. Milestones M0 (data-loss fixes) and M1 (specta bindings, CI repair, `just smoke` gate) landed in July.
- The frontend is the third UI generation. Facts from the code: `AppStateDto` is a single snapshot that carries the full LizaAlert catalog (`projects`, ~13k entries) on every `state-changed`; `stores.ts` exports 50+ stores mixing derived snapshot slices, ~25 UI flags and download state; six booleans encode the interaction mode; `MapView.svelte` is 1246 lines / 28 functions (drawing, point editing, waypoint markers, context menu, focus, FPS, keyboard); components subscribe to the snapshot directly.
- Verification is blind. `.mcp.json` runs `target/release/ozi-rs-mcp` built 2026-05-26, so the July fixes (600 s command timeout, PNG decoding) are absent: sessions die after 60 s idle and screenshots are base64 text of the whole screen. Tier-1 `capture_screenshot` lacks Screen Recording permission. 21 of 30 frontend test files read component source with `readFileSync` and assert `toContain`; one file renders a component. One Appium scenario (`smoke_core_workflow_draw_track`) exists.
- Visible defects on the current build (evidence in `.sisyphus/evidence/review-2026-09-19/`): straight lines across the map caused by `get_tracks_geojson` flattening all segments into one `LineString`; icon-only row buttons rendering as empty squares; Maps tab showing "No maps in this project" while a map is active; durations such as "629h" without explanation; English UI by default with the ru/en switch hidden in Cmd-K.
- Owner constraints: unpredictable rhythm that becomes regular only if progress is visible; sole user on macOS for the first phase; Claude Code as the primary agent with other agents possibly joining through shared memory; GUI-hijacking test runs batched once per session.
- An independent review (GPT, 2026-09-19) agreed with keeping the core and rejected a rewrite, but argued that the display contract, not only the process, must change. This design adopts that: the cycle is rebuilt first, and screens are then migrated one by one onto a new state layer inside the running app (strangler pattern).

## Goals / Non-Goals

**Goals:**
- Any agent can see the UI it changes: a browser stand on real fixtures, a screenshot matrix, and a policy that makes screenshots mandatory evidence for visual work.
- A state layer (`src/lib/vm/`) that later screen migrations target, with one interaction mode and attachable map modules, so regressions stop re-emerging from the same places.
- The three most visible defects fixed in the first slice, so the new cycle produces a visible result immediately.
- A repository-level ritual (STATE, progress gallery, DoD, PR gating, per-CJ smoke) that survives month-long pauses and works for any agent.
- Russian as the default UI language with a reachable, persisted switch.

**Non-Goals:**
- Rewriting the frontend or replacing the stack (Svelte 5, shadcn-svelte, MapLibre, Tailwind 4 stay).
- Migrating screens beyond the status bar in this change; each screen is its own follow-up change.
- New field tools, offline glyphs, export round-trip tests, `.ozp` versioning, download retries (roadmap phase 4).
- Signing, updater, Windows verification, distribution (roadmap phase 5).
- Replacing Appium: it stays as the desktop-integration gate and is used once per session.

## Decisions

### Decision 1: Strangler migration inside the running app, not a parallel rewrite

Alternatives: (a) repair the current shell in place without changing the state contract; (b) greenfield frontend. (a) keeps the causes of regressions (direct snapshot subscriptions, mode flags, monolithic MapView). (b) burns the only verified asset's integration and repeats the "agent without eyes" failure. Chosen: introduce the vm layer beside the existing stores, migrate one screen per change, delete the old subscriptions in the same change. The app stays shippable at every commit.

### Decision 2: Browser stand with a mocked Tauri transport fed by real fixtures

Alternatives: Storybook-style component isolation; Playwright driving the Tauri webview; Appium screenshots only. Component isolation misses layout and cross-component state; Playwright cannot attach to WKWebView in Tauri 2 without a remote-debugging build; Appium is slow, screen-hijacking and cannot run per commit. Chosen: `just stand` starts Vite with `VITE_STAND=1`; `src/test/stand/transport.ts` replaces `@tauri-apps/api/core` `invoke`/`listen` via a Vite alias, answers from fixture JSON, supports latency, error injection and scripted event replay (`state-changed`, `projects-chunk`, `download-progress`, `bundle-progress`). Fixtures are produced by the Rust core (`dump_fixtures`) from a `.ozp` project plus a local bundle's metadata, never hand-written, and a test asserts they parse against the specta-generated types. Tiles on the stand come from a handful of PNG tiles copied from the Lavrovo bundle and served statically; the stand is for chrome, lists, states and interaction, not for cartographic fidelity.

### Decision 3: Two-channel evidence policy

Alternatives: keep ADR-0024 as is (Playwright never evidence) or drop Appium. Chosen: Playwright on the stand is required evidence for anything visual or UX (layout, states, text, icons, locale, theme); Appium Mac2 smoke is required evidence for anything crossing IPC, protocols, dialogs or window lifecycle; a UI slice needs both. The policy is recorded as a requirement in `visual-verification` and as a decision-history entry in that capability's Purpose, not as a new ADR file: the `codify-architecture-decisions` change ends the ADR series and turns existing ADRs into pointer stubs. The ADR-0024 stub points to this policy, which preserves the reason ADR-0024 exists while removing its side effect.

### Decision 4: View-model layer shape

`src/lib/vm/` has one module per area, each exporting typed `Readable` stores derived from the latest `AppStateDto` plus explicit async actions that call `api.ts`:

- `workspace.ts`: project name, path, dirty, saved, busy, status, diagnostics; actions save/undo/redo.
- `library.ts`: track rows grouped by layer (from `tracks` + `track_layers`), waypoint rows, maps of the current project, active map; actions toggle visibility, rename, set color/width, delete, import.
- `catalog.ts`: project summaries accumulated from `projects-chunk`, local cache, download progress per file, active download; actions preview, load, cancel, set bundles root.
- `selection.ts`: selected track / waypoint / point / map, `interactionMode`, drawing session; actions select, clear, setMode.
- `map.ts`: viewport bounds, focus requests, geometry version, visible layers; actions focus track / all.

Rules: components import from `$lib/vm/*` only; `appState` and the legacy flags become private to vm during migration; each module has fixture-driven unit tests. Alternative considered: a single global vm object. Rejected: it would recreate the 50-export `stores.ts`.

### Decision 5: One interaction mode

`interactionMode: 'view' | 'draw' | 'edit' | 'measure' | 'addWaypoint'` with a single `setMode()` that performs exit logic for the previous mode (clear preview, drop markers, restore map interactions). Legacy booleans are derived from it during migration and deleted when the toolbar migrates. This removes mode "sticking" by construction (CJ-5 finding) and gives the toolbar chips real state.

### Decision 6: Attachable map modules

`MapView.svelte` becomes a host that creates the map, registers protocols and calls `attach(map, vm)` on modules in `src/lib/maplibre/`: `tracks-layer` (this change), then `waypoint-markers`, `point-editing`, `drawing`, `measure`, `focus`, `viewport` (follow-up `split-mapview-modules`). Each module returns `detach()`, holds no Svelte state, and is unit-tested against a fake map object (`addSource`, `addLayer`, `getSource().setData`, `on/off`). Target host size after the follow-up: under 300 lines.

### Decision 7: Per-segment geometry

`get_tracks_geojson` emits one Feature per track with `MultiLineString` whose parts are the track's segments (segments with fewer than two points are skipped; tracks with zero parts are skipped). `tracks-layer.ts` needs no change for rendering (MapLibre draws MultiLineString parts separately) but the label layer switches to `symbol-placement: line` on the first part. The response becomes a typed `TracksGeoJsonDto` through specta instead of `JsonValue`. Alternative: keep LineString and insert NaN breaks. Rejected: not valid GeoJSON.

### Decision 8: Catalog leaves the snapshot

`AppStateDto.projects` is removed. The frontend already receives `projects-chunk` events and caches the catalog locally (spec `lizaalert-integration`); `catalog.ts` becomes the single owner. `state-changed` payloads shrink from ~13k entries to the project's own state, which also fixes the stale-while-revalidate clobbering guarded in `syncProjectsFromAppState`. The `get_app_state` command keeps its name; the specta binding change is caught by `typescript_bindings_are_up_to_date`.

### Decision 9: Duration semantics

`duration_seconds` stays the elapsed span between the first and last timestamp (the current computation is correct for what it measures). The UI labels it as span: spans of 24 hours or more render as `Nd Hh` and every duration carries a tooltip "from first to last point". A separate moving-time statistic is deferred to the backlog. Alternative: compute moving time now. Rejected: needs a stop-detection threshold that the owner has not chosen.

### Decision 10: Progress made visible inside the repository

`docs/progress/README.md` is a reverse-chronological gallery; each slice appends a dated folder with before/after screenshots and a five-line entry. `docs/STATE.md` holds the three answers every agent needs first (where we are, next slice, known broken). Alternative: external dashboard or artifact page. Rejected: must survive tool changes and offline work; the repository is the one place every agent already reads.

### Decision 11: Russian default

`i18n.ts` defaults to `ru` when no stored preference exists; the switch is exposed as a status-bar control and in Cmd-K; every new dictionary key must exist in both languages (existing test pattern kept). Alternative: follow the OS language. Rejected: the audience is Russian-speaking regardless of OS locale.

### Decision 12: Legacy spec documents

`docs/superpowers/specs/` and `docs/superpowers/plans/` hold five design specs and three plans from April–May 2026. All were either implemented (documentation-update → ADR-0019; mvp-scope → ADR-0020; qa-debug-process → ADR-0024 + `docs/agent-verification.md`; shadcn-ui-kit → archived changes `redesign-shell-layout`, `redesign-library-sidebar`, `redesign-inspector-pane`) or are a non-committed backlog (meetily-inspired). They are moved to `docs/archive/superpowers/` with a README mapping, and the backlog content moves to `docs/backlog.md`. New specs are OpenSpec changes only.

## Risks / Trade-offs

- [The fourth UI generation dies like the previous three] → migration in place, one screen per change, DoD with screenshots and smoke, PR gating on CI; no screen is "done" without its gallery entry.
- [Infrastructure slices show no product progress and the owner disengages] → slice 0 fixes the three most visible defects first; the harness phase is capped at three sessions, everything beyond goes to `docs/backlog.md`.
- [Mocked transport drifts from the real backend] → fixtures are generated by Rust on every CI run and type-checked against specta bindings; Appium smoke per CJ covers the real path.
- [Rhythm disappears for a month] → every slice fits one session; `docs/STATE.md` lets any agent resume; no cross-slice in-flight state.
- [Appium remains fragile] → it runs once per session at the end; the stand is the inner loop; the MCP server is built from source via `cargo run`.
- [Screenshot baselines flake on fonts or timing] → deterministic viewport, bundled fonts, animations disabled on the stand, a small per-pixel tolerance, and `--update` only in a dedicated commit.
- [Removing `projects` from `AppStateDto` breaks a consumer] → `typescript_bindings_are_up_to_date` and `svelte-check` fail the build; the catalog vm is landed before the field is removed.

## Migration Plan

This change is slices 0–2 plus the vm skeleton and backend contract (phases 3.1 and 3.1b). The full roadmap it enables:

| Phase | Slice | Visible result |
|---|---|---|
| 0 Visible start | 0.1 hygiene: commit the July slice, push, branch/PR flow, `.mcp.json` via `cargo run`, Screen Recording | GitHub catches up with the local repository |
| | 0.2 three fixes: per-segment geometry, row icons, Maps tab shows active map | No diagonals across the map, readable buttons |
| | 0.3 correctness fixes from the ADR translation: bundles-root persistence, Esc discards a draw, map LayerId allocation, `qa_observe` capture, HTTP timeouts, export errors surfaced | Spec and code agree; no silent failures |
| 1 Eyes | 1.1 fixtures from `.ozp` via Rust, conformance test | Real data for the stand without the GUI |
| | 1.2 `just stand`: mock transport, static tiles, workspace on fixtures | UI opens in a browser in seconds |
| | 1.3 `just shots`, gallery, evidence policy in `visual-verification`, doc updates, baseline of the current UI | The "before" is recorded |
| 2 Contract | 2.1 `docs/design.md` approved, taste skills removed, `docs/STATE.md`, review checklist | Something to check every screen against |
| 3 Rebuild | 3.1 vm layer + `interactionMode`, fixture tests, status bar consumer (this change) | Architectural skeleton |
| | 3.1b backend contract: catalog out of `AppStateDto`, per-CJ smoke skeletons, cropped Appium screenshots (this change) | Snapshot shrinks, smoke gaps visible |
| | 3.2 `migrate-library-tracks` (smoke CJ-3) | First screen at the new quality |
| | 3.3 `migrate-track-inspector` (smoke CJ-4) | Track cleanup visible on the map |
| | 3.4 `split-mapview-modules` | 1246-line debt closed |
| | 3.5 `migrate-maps-and-bundle-loader` (smoke CJ-1 on a fixture bundle) | Map loading legible |
| | 3.6 `migrate-waypoints` | Waypoints at the new quality |
| | 3.7 `migrate-toolbar-and-palette` (smoke CJ-7 save/kill/reopen) | Whole shell rebuilt, Russian everywhere |
| 4 Close CJs for the owner | offline glyphs + restore (CJ-2), measure tools (CJ-5), export round-trip (CJ-6), `format_version` (CJ-8), July backlog (partial bundles, map-type filter, retries) | Field-ready for one user |
| Checkpoint | A real outing with ozi-rs instead of OziExplorer; notes become new slices | |
| 5 Distribution | macOS signing + updater + v0.1 tag; Windows paths + CI verification + tester; 2–5 colleagues | First external users |

Rollback: every slice is a PR; the vm layer is additive until a screen migrates; `AppStateDto.projects` removal is the only breaking step and lands after `catalog.ts` is the sole consumer.

## Open Questions

- Moving-time statistic: which stop threshold (speed or gap) does the owner want? Deferred to backlog until answered.
- Stand tiles: a few static PNG tiles versus a tiny local MBTiles fixture served by the mock. Start with PNG; revisit when a map-centric screen migrates.
- Screen Recording permission for the terminal and MCP server is a one-time manual step on the owner's Mac; document it in `docs/native-qa-mcp.md`.
