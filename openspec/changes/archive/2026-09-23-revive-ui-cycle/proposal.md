## Why

The Rust core of ozi-rs is healthy (clean layers, atomic I/O, typed IPC via tauri-specta, 245 passing tests) and the desktop app works end-to-end, yet the UI has been rebuilt three times and each version regressed within days. The September 2026 review found the common cause: the agents that build the UI never see it. There is no working screenshot channel (the MCP binary in `.mcp.json` predates the July fixes, Tier-1 capture returns a frame without windows), 21 of 30 frontend test files assert on source text via `readFileSync`, exactly one E2E scenario exists, and the display contract (track list derived from geometry, catalog of 13k projects inside every `AppStateDto`, six boolean mode flags, a 1246-line `MapView`) reproduces the same regressions after every redesign. The owner is close to abandoning the project; the July M2 slice is uncommitted and 68 commits are unpushed.

This change rebuilds the UI development cycle rather than the UI: it gives agents eyes (a browser stand on real fixtures plus a screenshot matrix), a design contract, a view-model layer that later screen migrations target, and a session ritual that makes progress visible. It also ships the three most visible defects fixed first so the cycle proves itself on day one.

## What Changes

- **Visible fixes first.** Track GeoJSON is emitted as one `MultiLineString` per track (one part per segment) instead of a flattened `LineString`, so segment gaps stop rendering as straight lines across the map and `split`/`join` become visible. Icon-only buttons in Library rows render visible icons with accessible labels and tooltips. The Maps tab lists the active map even when no LizaAlert project is current.
- **Correctness fixes from the ADR translation.** Where the September spec audit found code contradicting a requirement the owner wants kept, the code is fixed: bundles root persists across restarts; Esc discards an in-progress draw without a redo entry and restores the dirty flag; map layer identifiers are allocated as max+1; `qa_observe` captures logs and a screenshot again; both HTTP clients get timeouts; GPX/WPT export failures reach the caller as errors.
- **Browser stand.** `just stand` serves the frontend in a browser against a mocked Tauri transport (`invoke` + `listen`) fed by real fixtures that `just fixtures` dumps from the Rust core (AppStateDto, tracks GeoJSON, track details, waypoints, catalog chunk). The mock supports latency, error injection and event replay.
- **Screenshot matrix.** `just shots` renders every registered screen × state (empty, loading, loaded, error, overflow) × locale (ru, en) × theme (light, dark) with Playwright into `docs/progress/<date>-<slice>/`, and `just shots --compare` diffs against a committed baseline.
- **Verification policy.** Recorded as a `visual-verification` requirement and a Purpose decision-history entry (no new ADR file): Playwright on the stand is the required evidence for visual and UX changes; Appium Mac2 smoke remains the required evidence for desktop integration; a UI slice needs both. The ADR-0024 stub is pointed at the new policy, narrowing its meaning to "not evidence for desktop integration". `docs/agent-verification.md`, `AGENTS.md` and `CLAUDE.md` are updated accordingly. **BREAKING** for the current agent protocol: structural `readFileSync` tests are no longer accepted for new frontend code.
- **View-model layer.** `src/lib/vm/` exposes typed derived stores and actions per area (workspace, library, catalog, selection, map). Components consume vm only. A single `interactionMode` (`view | draw | edit | measure | addWaypoint`) replaces `drawingModeActive`, `editModeActive`, `addWaypointMode` and `simplifyState.active`. `MapView` becomes a thin host over attachable map modules (`attach(map, vm) → detach()`). This change lands the layer, its fixture-based tests, the mode enum and the first consumer (workspace status bar); screen migrations follow as separate changes.
- **Backend contract.** The project catalog leaves `AppStateDto` (the `projects-chunk` stream already exists); `get_tracks_geojson` returns a typed FeatureCollection DTO with per-segment geometry; a `dump_fixtures` command writes fixtures from a `.ozp` project without launching the GUI.
- **Russian by default.** UI locale defaults to Russian, the ru/en switch is persisted and reachable from the workspace, not only from Cmd-K. Every new or migrated screen ships both dictionaries.
- **Design contract.** `docs/design.md` fixes density, typography, screen states, icon/label rules, warning indicators and map styling. The 14 third-party "taste" skills under `.agents/skills` and `.claude/skills` are removed.
- **Agent workflow.** `docs/STATE.md` (where we are, next slice, known broken), `docs/progress/` gallery, a Definition of Done for slices, slice = branch = PR to `main`, one GUI batch per session, one smoke test per Customer Journey, `.mcp.json` runs the MCP server via `cargo run` so it never goes stale.
- **Legacy specs.** `docs/superpowers/specs/*` and `docs/superpowers/plans/*` are moved to `docs/archive/superpowers/` with a README mapping each document to the OpenSpec change or ADR that absorbed it; the only live content (the Meetily-inspired backlog) moves to `docs/backlog.md`.

## Capabilities

### New Capabilities
- `visual-verification`: browser stand on mocked transport with real fixtures, screenshot matrix and baseline comparison, two-channel evidence policy (Playwright for visual, Appium for desktop integration), per-CJ smoke tests.
- `frontend-view-model`: the view-model layer contract between backend DTOs and Svelte components, the single interaction mode, attachable map modules, fixture conformance.
- `agent-workflow`: repository-level state page, progress gallery, slice Definition of Done, branch/PR gating, GUI-batching rule, MCP server freshness.

### Modified Capabilities
- `track-display`: track geometry is delivered and rendered per segment; per-track duration statistics gain explicit elapsed-span semantics and long-span formatting.
- `ui-shell`: default locale is Russian with a persisted, reachable switch; icon-only controls carry visible icons, labels and tooltips; every screen declares its five states; the Maps tab lists the active map independently of the current LizaAlert project.
- `build-tooling`: new recipes `just stand`, `just fixtures`, `just shots`; `just smoke` runs every CJ smoke.
- `ci-pipeline`: CI runs fixture conformance and the screenshot baseline comparison, publishing the matrix as an artifact.

## Impact

- **Frontend**: new `src/lib/vm/` modules and tests; `src/lib/maplibre/` gains attachable modules (this change: `tracks-layer` per-segment rendering and the module contract; further extraction is follow-up); `MapView.svelte`, `TracksTab.svelte`, `LibraryRow.svelte`, `MapsTab.svelte`, `WorkspaceShell.svelte` touched for the visible fixes and the status-bar consumer; `src/lib/i18n.ts` default locale; new `src/test/fixtures/`, `src/test/stand/` (mock transport), Playwright config and screen registry.
- **Backend**: `get_tracks_geojson` geometry and typed DTO; `AppStateDto.projects` removed (catalog via `projects-chunk` + `load_projects`); new `dump_fixtures` binary or `#[test]` entry point; no domain changes.
- **Tooling & docs**: `justfile`, `.mcp.json`, `tools/ozi-rs-mcp` screenshot cropping and per-CJ smoke tests, `.github/workflows/ci.yml`, the `visual-verification` Purpose entry, `docs/design.md`, `docs/STATE.md`, `docs/progress/`, `docs/backlog.md`, `docs/archive/superpowers/`, `docs/agent-verification.md`, `docs/roadmap.md`, `AGENTS.md`, `CLAUDE.md`.
- **Breaking**: `AppStateDto` loses `projects`; frontend code reading `$appState.projects` moves to the catalog vm. Structural source-grep tests are no longer accepted for new code (existing ones are retired as their screens migrate).
- **Risk**: medium. Mitigated by landing the visible fixes as the first slice, keeping every slice within one session, and gating every merge on `just ci`, `just shots --compare` and the relevant smoke.

## Out of scope

- Migrating individual screens to the view-model layer beyond the status bar. Follow-up changes, in order: `migrate-library-tracks`, `migrate-track-inspector`, `split-mapview-modules`, `migrate-maps-and-bundle-loader`, `migrate-waypoints`, `migrate-toolbar-and-palette`.
- New field tools (measure: ruler, circle, point projection), offline glyphs, export round-trip tests, `.ozp` `format_version`, download retries — roadmap phase 4.
- Signing, notarization, updater, Windows verification, distribution — roadmap phase 5.
- Any change to tile rendering, import/export formats or the domain model.
