## 0. Hygiene (slice 0.1, one session)

- [ ] 0.1 Run `just ci` on the current tree; commit the uncommitted July slice (unified import + ZIP, import folder, show-on-map, bundle preview, virtualised catalog, DownloadPopup, glyph fix) as one commit with a message describing the owner's first hands-on findings
- [ ] 0.2 Push `main` to `origin` (68 commits behind); confirm `git log origin/main..main` is empty and GitHub Actions runs green on the pushed head
- [ ] 0.3 Switch `.mcp.json` to `cargo run --quiet -p ozi-rs-mcp --` (mirror `opencode.json`); document in `docs/native-qa-mcp.md` that the prebuilt binary is no longer used
- [ ] 0.4 Document the one-time Screen Recording grant for the terminal and MCP server in `docs/native-qa-mcp.md`; owner performs it
- [ ] 0.5 Create `docs/STATE.md` (where we are / next slice / known broken) and link it from `AGENTS.md` and `CLAUDE.md` as the first thing to read and last thing to update
- [ ] 0.6 Add `.github/pull_request_template.md` with the five Definition-of-Done checkboxes
- [ ] 0.7 Create branch `slice/0.2-visible-fixes` for the next group; from here on every slice is a branch + PR

## 1. Visible fixes (slice 0.2, one session)

- [ ] 1.1 Rust: change `get_tracks_geojson` to emit one `MultiLineString` per track with one part per segment, skipping segments with fewer than two points; add unit tests for the two-segment, split and degenerate cases
- [ ] 1.2 Rust: replace the `serde_json::Value` return with a specta-exported `TracksGeoJsonDto`; regenerate `bindings.ts`; update `api.ts` `getTracksGeojson` and `tracks-layer.ts` label placement
- [ ] 1.3 Frontend: give every icon-only button in `LibraryRow.svelte`, `TracksTab.svelte`, `WaypointsTab.svelte`, `MapsTab.svelte` a visible lucide icon, `aria-label` and tooltip; verify in the running app via Appium screenshot that no button renders as an empty square
- [ ] 1.4 Frontend: `MapsTab.svelte` lists `activeMap` as a row when set, independent of `currentProject`; empty state only when both are absent; component test with `@testing-library/svelte`
- [ ] 1.5 Frontend: format durations ≥ 24 h as `Nd Hh` and add the "from first to last point" tooltip in `TracksTab.svelte` and `TrackInspector.svelte`; unit test the formatter
- [ ] 1.6 Evidence: Appium screenshots before/after (cropped to the window) into `docs/progress/<date>-visible-fixes/`; create `docs/progress/README.md` with the first gallery entry
- [ ] 1.7 `just ci` green, `just smoke` green, PR merged, `main` pushed, `docs/STATE.md` updated

## 1b. Correctness fixes from the ADR translation (slice 0.3, one session)

- [ ] 1b.1 Persist the bundles root: add `bundles_root` to `PersistedAppSession`, write it in `set_bundles_root`, restore it at startup before the catalog loads; Rust tests for save/restore and for the default when the field is absent (TDD)
- [ ] 1b.2 Esc discards a draw: add `CommandStack::discard_last(n)` (reverse-apply without pushing to redo, restore `mutation_count`), a `cancel_drawing` command taking the draw's command count, and switch `MapView` `cancelDrawingMode` to it; Rust tests: redo unavailable after cancel, dirty flag restored, saved project stays saved; frontend test on the api wiring
- [ ] 1b.3 Map layer identifiers: allocate `LayerId` as max+1 (application/mod.rs:1449 uses `len()+1`); test that adding after a deletion never reuses an id
- [ ] 1b.4 `qa_observe` captures again: call the log and screenshot captures and return their artifact paths (tools/ozi-rs-mcp/src/native.rs:107-124); test with fake commands; fix the description in `docs/native-qa-mcp.md`
- [ ] 1b.5 HTTP timeouts: connect timeout on both reqwest clients and a read timeout on the streaming download client (verify the reqwest 0.13 API); test that a stalled fake server fails the download instead of hanging
- [ ] 1b.6 Small cleanups: remove the unused `lucide-svelte` dependency; classify `.kml` archive entries as unsupported until a parser exists (import/archive.rs:65,210) with a test
- [ ] 1b.7 GPX/WPT export commands return `Err` on failure instead of status-only (application/mod.rs:1169-1225); tests; UI shows the existing error toast
- [ ] 1b.8 `just ci` green, `just smoke` green, PR merged, `main` pushed, `docs/STATE.md` updated

## 2. Fixtures from the Rust core (slice 1.1, one session)

- [ ] 2.1 Commit a sample project under `example_data/fixtures/` (`.ozp` derived from the Lavrovo project with tracks and waypoints, plus the local bundle's metadata files; no tile payloads beyond a handful of PNG tiles)
- [ ] 2.2 Rust: add `dump_fixtures` (binary or `#[test]` behind a feature) that loads the sample through the application layer and writes `app_state.json`, `tracks_geojson.json`, `track_detail_<id>.json`, `waypoints.json`, `projects_chunk.json` to `src/test/fixtures/`, deterministically ordered
- [ ] 2.3 `justfile`: add `just fixtures`; document in `docs/testing-strategy.md`
- [ ] 2.4 Frontend test `fixtures-conformance.test.ts`: parse every fixture against the specta types (runtime schema derived from `bindings.ts` or hand-written zod mirrors generated once) and fail on drift
- [ ] 2.5 `just ci` green, PR merged, `docs/STATE.md` updated

## 3. Browser stand (slice 1.2, one session)

- [ ] 3.1 `src/test/stand/transport.ts`: mock `invoke`/`listen` answering from fixtures; per-command latency and error injection via a `window.__stand` control object; scripted event replay for `state-changed`, `projects-chunk`, `download-progress`, `bundle-progress`, `bundle-file-ready`; unmocked commands reject and toast
- [ ] 3.2 Vite: alias `@tauri-apps/api/core` to the mock when `VITE_STAND=1`; serve `example_data/fixtures/tiles/` statically; register a stand tile protocol handler that maps `sqlite://` / `ozi://` requests to the static tiles
- [ ] 3.3 `justfile`: `just stand` (prints URL); `docs/testing-strategy.md` describes the stand and its limits (chrome, lists, states, interaction — not cartographic fidelity)
- [ ] 3.4 Unit tests for the transport: fixture answer, injected error, event replay order, unmocked command rejection
- [ ] 3.5 Manual check on the stand: `/` and `/project` render on fixtures; screenshot committed as the first stand evidence
- [ ] 3.6 `just ci` green, PR merged, `docs/STATE.md` updated

## 4. Screenshot matrix and evidence policy (slice 1.3, one session)

- [ ] 4.1 Add Playwright as a dev dependency; `src/test/stand/screens.ts` registry with entries for `bundle-loader`, `workspace`, `library-maps`, `library-tracks`, `library-waypoints`, `track-inspector`, `waypoint-inspector`, `command-palette`, each declaring fixture/transport config for the five states
- [ ] 4.2 `src/test/stand/shots.ts`: render screen × state × locale × theme with fixed viewport, bundled fonts, animations disabled; `--compare` with per-pixel tolerance and diff images; `--update` rewrites `src/test/stand/baseline/`; `--slice` writes to `docs/progress/<date>-<slice>/`
- [ ] 4.3 `justfile`: `just shots` with the flags above; `just ci` gains `just shots --compare`
- [ ] 4.4 Commit the baseline of the current UI as the recorded "before"
- [ ] 4.5 Record the two-channel evidence policy as the `visual-verification` requirement plus a decision-history entry in that spec's Purpose (no new ADR file, per the `documentation` capability introduced by `codify-architecture-decisions`); point the ADR-0024 stub to it; update `docs/agent-verification.md`, `AGENTS.md`, `CLAUDE.md` to reference the requirement and the two-channel rule
- [ ] 4.6 CI: add the fixtures + screenshot job to `.github/workflows/ci.yml` uploading the matrix and diffs as artifacts; document in `docs/ci.md`
- [ ] 4.7 `just ci` green, PR merged, `docs/STATE.md` updated

## 5. Design contract and workflow docs (slice 2.1, one session with the owner)

- [ ] 5.1 Draft `docs/design.md`: usage context, tokens (Zinc + Teal, Catppuccin opt-in), typography rules, the five screen states, icon/label/tooltip rules, warning indicator rule, map styling rules, Russian-first copy rules; owner approves in-session
- [ ] 5.2 Remove the 14 third-party taste skills from `.agents/skills/` and `.claude/skills/` (keep `openspec-*`); drop `skills-lock.json` entries accordingly
- [ ] 5.3 Create `docs/backlog.md` seeded with the Meetily-inspired backlog, the July manual-test backlog (partial bundle availability, map-type filter, download retries) and the moving-time statistic question
- [ ] 5.4 Move `docs/superpowers/specs/*` and `docs/superpowers/plans/*` to `docs/archive/superpowers/` with a README mapping each file to the OpenSpec change or ADR that absorbed it; update `docs/project-map.md` and `AGENTS.md` doc lists
- [ ] 5.5 Update `docs/roadmap.md` "What's Next" with the phase table from `design.md` (Migration Plan) and the follow-up change names
- [ ] 5.6 Add the slice review checklist to `AGENTS.md` (screenshots present, no `readFileSync` tests, smoke result, STATE updated)
- [ ] 5.7 PR merged, `docs/STATE.md` updated

## 6. View-model layer and first consumer (slice 3.1, one session)

- [ ] 6.1 Create `src/lib/vm/workspace.ts`, `library.ts`, `catalog.ts`, `selection.ts`, `map.ts` with typed derived stores and actions as specified in `design.md` Decision 4; `interactionMode` + `setMode()` with per-mode exit logic in `selection.ts`
- [ ] 6.2 Derive the legacy flags (`drawingModeActive`, `editModeActive`, `addWaypointMode`) from `interactionMode` so unmigrated components keep working; mark them `@deprecated`
- [ ] 6.3 Convert `src/lib/maplibre/tracks-layer.ts` to the `attach(map, vm) → detach()` contract; add a fake-map unit test for lifecycle and per-segment rendering
- [ ] 6.4 Fixture-driven unit tests for every vm module (library rows by layer, catalog chunk merge, workspace dirty/saved, selection mode transitions, map focus requests)
- [ ] 6.5 Migrate the workspace status bar in `WorkspaceShell.svelte` to `vm/workspace` and `vm/catalog`; add the language switch control there (Russian default per ui-shell spec); stand screenshots before/after
- [ ] 6.6 ESLint rule (`no-restricted-imports` scoped to migrated components) forbidding `appState` and legacy flags; apply to `WorkspaceShell.svelte`
- [ ] 6.7 `just ci`, `just shots --compare`, `just smoke` green; PR merged; gallery entry; `docs/STATE.md` updated with the follow-up change list

## 7. Backend contract (slice 3.1b, one session)

- [ ] 7.1 Rust: remove `projects` from `AppStateDto` once `catalog.ts` is the only consumer; regenerate bindings; fix `typescript_bindings_are_up_to_date`; update `layers`/`lizaalert-integration` docs where `AppStateDto.projects` is mentioned
- [ ] 7.2 Frontend: delete `syncProjectsFromAppState` and the stale-while-revalidate guard in `stores.ts`; catalog hydration stays cache + `projects-chunk`
- [ ] 7.3 Add `smoke_cj1_bundle_download` … `smoke_cj8_project_exchange` skeletons in `tools/ozi-rs-mcp/tests/` (pending ones `#[ignore = "pending: …"]`); `just smoke [<N>]` runs them; move `smoke_core_workflow_draw_track` under `smoke_cj5_draw_track`
- [ ] 7.4 `tools/ozi-rs-mcp`: crop `appium_screenshot` to the app window: decode the base64 PNG, read the `ozi-rs` window bounds through `CGWindowListCopyWindowInfo` (small Swift helper built by the crate, no Accessibility grant needed), scale by the display factor and crop; unit-test the crop math and the decoder with a fake WebDriver response
- [ ] 7.5 `just ci`, `just smoke` green; PR merged; `docs/STATE.md` updated

## 8. Verification and hand-off

- [ ] 8.1 `openspec validate revive-ui-cycle --strict` passes
- [ ] 8.2 Owner reviews the gallery entries for slices 0.2 and 3.1 and confirms the visible fixes on a real outing or a manual session
- [ ] 8.3 Create the first follow-up change `migrate-library-tracks` with `/opsx:propose`, referencing `design.md` Decisions 4–6 and the ui-shell state requirements; record it as the next slice in `docs/STATE.md`
- [ ] 8.4 Archive this change with `/opsx:archive` once tasks 0–7 are merged
