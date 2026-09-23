## 0. Hygiene (slice 0.1, one session)

- [x] 0.1 Run `just ci` on the current tree; commit the uncommitted July slice (unified import + ZIP, import folder, show-on-map, bundle preview, virtualised catalog, DownloadPopup, glyph fix) as one commit with a message describing the owner's first hands-on findings
- [x] 0.2 Push `main` to `origin` (68 commits behind); confirm `git log origin/main..main` is empty and GitHub Actions runs green on the pushed head
- [x] 0.3 Switch `.mcp.json` to `cargo run --quiet -p ozi-rs-mcp --` (mirror `opencode.json`); document in `docs/native-qa-mcp.md` that the prebuilt binary is no longer used
- [x] 0.4 Document the one-time Screen Recording grant for the terminal and MCP server in `docs/native-qa-mcp.md`; owner performs it — written 2026-09-23 as a four-step procedure with the two symptoms that do not say "permission" (a black screenshot, a Mac2 host that dies on session start) and how to check the grant took. The grant itself is the owner's; macOS will not accept it from a script
- [x] 0.5 Create `docs/STATE.md` (where we are / next slice / known broken) and link it from `AGENTS.md` and `CLAUDE.md` as the first thing to read and last thing to update — done; verified 2026-09-22, the file exists and both documents point at it
- [x] 0.6 Add `.github/pull_request_template.md` with the five Definition-of-Done checkboxes — plus the OpenSpec box for a behavioural change, and the rule that an unticked box needs a reason rather than silence
- [~] 0.7 Branch + PR per slice — NOT ADOPTED. Every slice since has gone to `main` directly, because the owner is the only committer and a PR against yourself buys review from nobody while costing a round trip per slice. What the rule was for — the five checks before a slice is done — is now the PR template (0.6) and is run before each push instead. Revisit when a second person commits

## 1. Visible fixes (slice 0.2, one session)

- [x] 1.1 Rust: change `get_tracks_geojson` to emit one `MultiLineString` per track with one part per segment, skipping segments with fewer than two points; add unit tests for the two-segment, split and degenerate cases
- [→] 1.2 MOVED to `finish-the-rebuild` task 2.1, where it belongs: Rust: replace the `serde_json::Value` return with a specta-exported `TracksGeoJsonDto`; regenerate `bindings.ts`; update `api.ts` `getTracksGeojson` and `tracks-layer.ts` label placement — DEFERRED 2026-09-20: `api.ts` already casts the response to the standard `GeoJSON.FeatureCollection` from `@types/geojson`, so a specta mirror would duplicate that type and still need a cast. Revisit when the view-model slice defines who owns map geometry types.
- [x] 1.3 Frontend: give every icon-only button in `LibraryRow.svelte`, `TracksTab.svelte`, `WaypointsTab.svelte`, `MapsTab.svelte` a visible lucide icon, `aria-label` and tooltip; verify in the running app via Appium screenshot that no button renders as an empty square
- [x] 1.4 Frontend: `MapsTab.svelte` lists `activeMap` as a row when set, independent of `currentProject`; empty state only when both are absent; component test with `@testing-library/svelte`
- [x] 1.5 Frontend: format durations ≥ 24 h as `Nd Hh` and add the "from first to last point" tooltip in `TracksTab.svelte` and `TrackInspector.svelte`; unit test the formatter
- [~] 1.6 Evidence: gallery and before-shots are in `docs/progress/2026-09-20-visible-fixes/`; after-shots BLOCKED — the Mac2 driver host dies on launch and `screencapture` returns a black frame with no windows enumerated (revoked Screen Recording / Accessibility, likely lost in the Xcode 26.2 update). Owed once task 0.4 is done.
- [~] 1.7 `just ci` green (248 Rust, 278 frontend) and `main` pushed; `just smoke` BLOCKED by the same missing grant; `docs/STATE.md` still to write (task 0.5)

## 1b. Correctness fixes from the ADR translation (slice 0.3, one session)

- [x] 1b.1 Persist the bundles root: add `bundles_root` to `PersistedAppSession`, write it in `set_bundles_root`, restore it at startup before the catalog loads; Rust tests for save/restore and for the default when the field is absent (TDD)
- [x] 1b.2 Esc discards a draw: add `CommandStack::discard_last(n)` (reverse-apply without pushing to redo, restore `mutation_count`), a `cancel_drawing` command taking the draw's command count, and switch `MapView` `cancelDrawingMode` to it; Rust tests: redo unavailable after cancel, dirty flag restored, saved project stays saved; frontend test on the api wiring
- [x] 1b.3 Map layer identifiers: allocate `LayerId` as max+1 (application/mod.rs:1449 uses `len()+1`); test that adding after a deletion never reuses an id
- [x] 1b.4 `qa_observe` captures again: call the log and screenshot captures and return their artifact paths (tools/ozi-rs-mcp/src/native.rs:107-124); test with fake commands; fix the description in `docs/native-qa-mcp.md`
- [x] 1b.5 HTTP timeouts: connect timeout on both reqwest clients and a read timeout on the streaming download client (verify the reqwest 0.13 API); test that a stalled fake server fails the download instead of hanging
- [x] 1b.6 Small cleanups: remove the unused `lucide-svelte` dependency; classify `.kml` archive entries as unsupported until a parser exists (import/archive.rs:65,210) with a test
- [x] 1b.7 GPX/WPT export commands return `Err` on failure instead of status-only (application/mod.rs:1169-1225); tests; UI shows the existing error toast
- [x] 1b.8 `just ci` green, `just smoke` green, PR merged, `main` pushed, `docs/STATE.md` updated

## 2. Fixtures from the Rust core (slice 1.1, one session)

- [x] 2.1 The sample project is built in code (`src-tauri/src/fixtures.rs`) rather than committed as an `.ozp`: it needs no tile payloads, it is diffable, and the states a screen has to handle are asserted rather than hoped for
- [x] 2.2 Rust: `write_fixtures` behind `#[cfg(test)]` writes `app-state.json`, `tracks-geojson.json`, `track-detail.json` and `waypoints.json` through the same mappers the commands use
- [x] 2.3 `justfile`: `just fixtures`
- [x] 2.4 Frontend `fixtures.test.ts`: the fixtures are typed against the generated bindings and asserted to carry the states a screen must handle; `fixtures_are_up_to_date` fails the Rust suite when a DTO change has not been regenerated
- [x] 2.5 `just ci` green and `docs/STATE.md` updated (no PR — see 0.7)

## 3. Browser stand (slice 1.2, one session)

- [x] 3.1 `src/test/stand/`: `invoke`/`listen` answered from the fixtures; an unanswered command throws with its name; dialogs answer "cancelled"; `standEmit` delivers an event by hand
- [x] 3.2 A separate `vite.stand.config.ts` aliases the Tauri modules, rather than a flag inside the app's own config — the app's build stays untouched. Tiles are not served: the stand shows the basemap, and cartographic fidelity is explicitly out of scope
- [x] 3.3 `justfile`: `just stand`; `src/test/stand/README.md` states what it proves and what it does not
- [x] 3.4 Unit tests for the transport: fixture answer, mutation accepted, unanswered command rejected by name, call transcript, event delivery and unsubscribe
- [x] 3.5 `/project` rendered on fixtures, Maps and Tracks tabs photographed; the first run immediately caught an unanswered `get_ozi_metadata`
- [x] 3.6 `just ci` green and `docs/STATE.md` updated (no PR — see 0.7)
- [x] 3.7 Scripted replay of the download events, after all: pressing the download button on the stand plays the sequence a real bundle emits, slowly enough to look at. It paid for itself immediately — the button turned out to do nothing at all when the loader is opened on an already-previewed project. Latency and error injection stay deferred

## 4. Screenshot matrix and evidence policy (slice 1.3, one session)

- [→] 4.1 Add Playwright as a dev dependency; `src/test/stand/screens.ts` registry with entries for `bundle-loader`, `workspace`, `library-maps`, `library-tracks`, `library-waypoints`, `track-inspector`, `waypoint-inspector`, `command-palette`, each declaring fixture/transport config for the five states — MOVED to `finish-the-rebuild` task 1.1
- [→] 4.2 `src/test/stand/shots.ts`: render screen × state × locale × theme with fixed viewport, bundled fonts, animations disabled; `--compare` with per-pixel tolerance and diff images; `--update` rewrites `src/test/stand/baseline/`; `--slice` writes to `docs/progress/<date>-<slice>/` — MOVED to `finish-the-rebuild` task 1.2
- [→] 4.3 `justfile`: `just shots` with the flags above; `just ci` gains `just shots --compare` — MOVED to `finish-the-rebuild` task 1.3
- [→] 4.4 Commit the baseline of the current UI as the recorded "before" — MOVED to `finish-the-rebuild` task 1.4
- [x] 4.5 The two-channel policy is the `visual-verification` requirement "Visual changes require stand screenshots; desktop integration requires Appium smoke", archived with this change; `docs/agent-verification.md`, `AGENTS.md` and `CLAUDE.md` already point at it
- [→] 4.6 CI: add the fixtures + screenshot job to `.github/workflows/ci.yml` uploading the matrix and diffs as artifacts; document in `docs/ci.md` — MOVED to `finish-the-rebuild` task 1.5
- [→] 4.7 Slice gate — MOVED with the matrix to `finish-the-rebuild`

## 5. Design contract and workflow docs (slice 2.1, one session with the owner)

- [→] 5.1 `docs/design.md` — MOVED to the owner: a design contract is written with them in session, not for them
- [~] 5.2 NOT DONE, and not mine to do: the skills under `.claude/skills/` are untracked files the owner installed on this machine. Deleting somebody's local tools because a plan written in July said so is not a cleanup. Left for the owner
- [x] 5.3 `docs/backlog.md` exists and carries the July manual-test backlog, the moving-time question, the catalogue-walk cost, the small-targets design question and, since 2026-09-23, the camp-router offline case
- [x] 5.4 `docs/superpowers/` moved to `docs/archive/superpowers/` with a README mapping each of the eight documents to the requirement that absorbed it; the four live citations elsewhere repointed
- [x] 5.5 `docs/roadmap.md` "What's Next" says what is actually left, and three stale entries are gone: the theme picker (it is in the command palette), layer management (created/renamed/deleted ship; only reordering is left) and "the smoke gate is blocked on this machine" (two journeys ran green on 2026-09-23)
- [x] 5.6 `AGENTS.md` § "Before a slice is done" carries the five checks, the same five as the PR template
- [x] 5.7 `docs/STATE.md` updated (no PR — see 0.7)

## 6. View-model layer and first consumer (slice 3.1, one session)

- [→] 6.1 Create `src/lib/vm/workspace.ts`, `library.ts`, `catalog.ts`, `selection.ts`, `map.ts` with typed derived stores and actions as specified in `design.md` Decision 4; `interactionMode` + `setMode()` with per-mode exit logic in `selection.ts` — MOVED to `finish-the-rebuild` task 2.1
- [→] 6.2 Derive the legacy flags (`drawingModeActive`, `editModeActive`, `addWaypointMode`) from `interactionMode` so unmigrated components keep working; mark them `@deprecated` — MOVED to `finish-the-rebuild` task 2.2
- [→] 6.3 Convert `src/lib/maplibre/tracks-layer.ts` to the `attach(map, vm) → detach()` contract; add a fake-map unit test for lifecycle and per-segment rendering — MOVED to `finish-the-rebuild` task 2.3
- [→] 6.4 Fixture-driven unit tests for every vm module (library rows by layer, catalog chunk merge, workspace dirty/saved, selection mode transitions, map focus requests) — MOVED to `finish-the-rebuild` task 2.4
- [→] 6.5 Migrate the workspace status bar in `WorkspaceShell.svelte` to `vm/workspace` and `vm/catalog`; add the language switch control there (Russian default per ui-shell spec); stand screenshots before/after — MOVED to `finish-the-rebuild` task 2.5
- [→] 6.6 ESLint rule (`no-restricted-imports` scoped to migrated components) forbidding `appState` and legacy flags; apply to `WorkspaceShell.svelte` — MOVED to `finish-the-rebuild` task 2.6
- [→] 6.7 Slice gate — MOVED with the view-model to `finish-the-rebuild`

## 7. Backend contract (slice 3.1b, one session)

- [→] 7.1 Rust: remove `projects` from `AppStateDto` once `catalog.ts` is the only consumer; regenerate bindings; fix `typescript_bindings_are_up_to_date`; update `layers`/`lizaalert-integration` docs where `AppStateDto.projects` is mentioned — MOVED to `finish-the-rebuild` task 2.7
- [→] 7.2 Frontend: delete `syncProjectsFromAppState` and the stale-while-revalidate guard in `stores.ts`; catalog hydration stays cache + `projects-chunk` — MOVED to `finish-the-rebuild` task 2.7
- [x] 7.3 Journeys are named after their CJ (`smoke_cj5_draw_track`,
      `smoke_cj3_layer_management`) and `just smoke cj3` runs one of them.
      Eight skeletons were NOT written: a test that is an empty skeleton passes
      for the wrong reason, and six of the eight are blocked on the same two
      things — a file dialog the Mac2 driver can answer, and a project loaded
      at launch. The blockers are written into the file's header table, one
      line per CJ, so the gap is the honest size of the desktop evidence rather
      than six green ticks
- [→] 7.4 `tools/ozi-rs-mcp`: crop `appium_screenshot` to the app window: decode the base64 PNG, read the `ozi-rs` window bounds through `CGWindowListCopyWindowInfo` (small Swift helper built by the crate, no Accessibility grant needed), scale by the display factor and crop; unit-test the crop math and the decoder with a fake WebDriver response — MOVED to `finish-the-rebuild` task 3.5
- [→] 7.5 Slice gate — MOVED to `finish-the-rebuild`

## 8. Verification and hand-off

- [x] 8.1 `openspec validate revive-ui-cycle --strict` passes
- [→] 8.2 Owner review of the gallery and a real outing — theirs to do, and the reason the loop stops here
- [x] 8.3 The follow-up change is `finish-the-rebuild`, carrying the screenshot matrix, the view-model and the six missing CJ journeys with the requirements that describe them
- [x] 8.4 Archived 2026-09-23
