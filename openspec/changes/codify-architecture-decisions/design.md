## Context

Three document families overlap: `docs/adr/` (24 ADRs, 2026-03 to 2026-04-28; statuses: 21 accepted, ADR-0005 superseded by ADR-0017, ADR-0003 "accepted (under review)" but superseded in practice by ADR-0016, ADR-0011 contradicted by the tokio-based downloader in `src-tauri/src/infrastructure/lizaalert.rs`), `docs/superpowers/` (five design specs and three plans from April–May 2026, all implemented or non-committed), and `openspec/specs/` (14 capabilities imported by `bootstrap-current-state`, Purposes left as `TBD`). Zero cross-references exist between specs and ADRs. About 150 `ADR-00NN` references live in docs, `docs/qa/*`, four code comments and archived changes.

OpenSpec mechanics that constrain the design: requirement blocks in main specs change only through delta files that are merged at archive time; delta files carry requirements only, so a new capability's `## Purpose` can be written only after archive creates the file; the `## Purpose` of an existing spec is free text and may be edited directly.

## Goals / Non-Goals

**Goals:**
- One normative source: every valid decision is a requirement with scenarios; rationale lives in the owning capability's Purpose decision history.
- No stale decision survives as normative text: verification against code precedes codification.
- All existing `ADR-00NN` references keep resolving.
- Legacy superpowers documents stop competing with OpenSpec.
- The rule for future decisions is explicit: OpenSpec change `design.md` + Purpose decision history, no new ADR files.

**Non-Goals:**
- Changing behavior or code.
- Re-deriving requirements for behavior that is already specified (duplicates are the main failure mode of this work).
- Touching the QA process capabilities, which are owned by `revive-ui-cycle`.

## Decisions

### Decision 1: Decisions are split into requirement (what) and Purpose history (why)

An ADR mixes context, decision, rationale and consequences. OpenSpec has two homes: requirements (normative, testable) and Purpose (free text). The decision's observable consequence becomes one or more requirements with scenarios; the rationale and alternatives become a `### Decision history` bullet in the capability's Purpose: `- ADR-00NN (<date>, <status>): <decision>; rationale: <why>. Codified as: <requirement names> | superseded by … | not codified because …`. Alternative: keep ADRs as the rationale record and only add cross-links. Rejected by the owner: two normative sources is the problem being solved.

### Decision 2: Verify against code before codifying

Each agent translating a slice greps the code for the decision's footprint (Cargo.toml, tests, implementation) and codifies only what holds today. Known contradictions: ADR-0011 (tokio present, 14 async fns in the downloader), ADR-0003 (egui stack replaced by Tauri). Contradicted or superseded decisions appear only in decision history with file:line evidence. Alternative: codify ADRs verbatim and fix later. Rejected: it would freeze false statements as SHALL.

### Decision 3: Three new capabilities for homeless decisions

`architecture` (ADR-0001, 0006, 0009, 0011-as-implemented, 0013, 0014, 0015, backend parts of 0016), `product-scope` (ADR-0020, 0022 scope, 0023), `documentation` (ADR-0019 and the documentation-update spec). Alternatives: put scope and architecture invariants into `ui-shell`/`layers`. Rejected: they cut across capabilities and would be found by nobody.

### Decision 4: ADR files become pointer stubs, identifiers stay

Given ~150 references, deleting or moving ADR files would break docs, QA smoke documents and code comments. Each ADR file is reduced to: title, date, status, "Codified in: `openspec/specs/<cap>/spec.md` → requirement names; rationale: Purpose decision history", and for superseded ones "Superseded by … / see decision history". `docs/adr/README.md` indexes all 24 with their capability. Full texts stay in git history. Alternative: move to `docs/archive/adr/`. Rejected: every reference would need rewriting for no gain.

### Decision 5: Superpowers documents are archived, not translated one-to-one

All five specs are implemented (documentation-update → ADR-0019 → `documentation`; mvp-scope → ADR-0020 → `product-scope`; qa-debug-process → ADR-0024 → `visual-verification`/`agent-workflow` in `revive-ui-cycle`; shadcn-ui-kit → the three archived redesign changes and `ui-shell`) or non-committed (meetily → `docs/backlog.md`). The three plans are executed. They move to `docs/archive/superpowers/` with a README mapping and the four live references are repointed. Alternative: rewrite each as an OpenSpec change. Rejected: they would be retroactive changes describing already-archived work.

### Decision 6: QA process belongs to `revive-ui-cycle`

ADR-0024 and the qa-debug-process spec describe the verification process that `revive-ui-cycle` is redefining (`visual-verification`, `agent-workflow`). Their still-valid parts are folded into that change's delta specs by a dedicated agent. `revive-ui-cycle` is amended so its "ADR-0025" becomes a requirement plus decision-history entry, consistent with the `documentation` rule introduced here. Alternative: codify ADR-0024 here and let revive modify it. Rejected: two active changes creating the same capability would collide at archive.

### Decision 7: Parallel translation by capability ownership

Six agents each own disjoint delta files and Purpose sections (architecture+tiles+layers; persistence+bundles+lizaalert; undo+editing; import+export+waypoints; scope+documentation+ui-shell+track-display; QA→revive). Ownership by file, not by ADR, prevents write conflicts; each agent reports coverage verdicts (covered / ADDED / MODIFIED / superseded) that this design's follow-up tasks consolidate.

## Risks / Trade-offs

- [Duplicate requirements added because an agent missed an existing block] → agents must grep the target spec first and report "covered by"; consolidation task greps for near-duplicate requirement names before archive.
- [Requirement codifies an ADR claim that is false today] → verification rule with file:line evidence; consolidation spot-checks every ADDED requirement against the cited evidence.
- [Purpose edits collide with the archive merge] → archive touches only requirement blocks; Purposes are outside the merge. New capabilities' Purposes are applied right after archive.
- [Stubbed ADRs lose nuance some reader needs] → git history keeps full text; the stub names the commit range and the Purpose entry carries the one-line rationale.
- [Two active changes both add `ui-shell` requirements] → names are disjoint by instruction; a pre-archive check greps both deltas for identical requirement headers.

## Migration Plan

1. Agents write deltas and Purpose edits (parallel). 2. Consolidate reports into this design (coverage table), fix duplicates, run `openspec validate codify-architecture-decisions --strict`. 3. Owner reviews. 4. Archive the change (specs merge). 5. Apply Purposes for the three new capabilities. 6. Reduce ADR files to stubs, write `docs/adr/README.md`. 7. Move superpowers docs to `docs/archive/superpowers/`, write mapping README, create `docs/backlog.md`, repoint the four live references. 8. Update `AGENTS.md`, `CLAUDE.md`, `README.md`, `docs/project-map.md`, `docs/roadmap.md`, `docs/conventions.md` to describe OpenSpec as the single source and the no-new-ADR rule. 9. Link check: every `ADR-00NN` reference resolves to a stub; no reference to `docs/superpowers/` remains outside archives.

Rollback: the change is documentation-only; reverting the commit restores the previous state.

## Open Questions

- Whether `ci-pipeline` and `build-tooling` Purposes should also carry decision history for the CI ADR-less decisions (the GitHub CI change's design). Default: yes, written by the QA-slice agent.
- Whether code comments citing ADRs (4 occurrences) should be repointed to requirement names. Default: leave; stubs keep them resolvable.

## Coverage and findings (consolidated from the six translation slices, 2026-09-19)

Delta inventory: 13 capability deltas, 63 requirements (54 ADDED, 9 MODIFIED, 2 REMOVED); Purposes rewritten for all 14 existing capabilities; `revive-ui-cycle` gained 12 QA-process requirements. No duplicate requirement headers within or across the two active changes; no ADDED header duplicates an existing main-spec requirement.

### ADRs

| ADR | Verdict | Evidence / notes |
|---|---|---|
| 0001 initial architecture | ADDED `architecture`: layering, thin command handlers; edit rule covered by `undo-redo` | application↔infrastructure share data types both ways (lizaalert.rs:1-3, application/mod.rs:12-16); codified as-is |
| 0002 bundle/project separation | covered (map-bundles, project-persistence); ADDED default bundles root | eframe storage for active-map ref superseded by `PersistedAppSession` |
| 0003 egui stack | superseded by 0016 | history only |
| 0004 JSON `.ozp` | covered; ADDED pretty-printed stable shape; ADDED atomic writes (July 2026) | `version` field implemented 2026-09-23 (`a-file-says-who-wrote-it`): a saved file carries `format_version`, a file without one is the oldest format, a newer one is refused |
| 0005 snapshot undo | superseded by 0017 | `CommandStack` holds `Vec<CommandDelta>` (commands.rs:1289-1296) |
| 0006 ozf2 sibling crate | ADDED decode-through-adapter | crate is crates.io `ozf2 = "0.1"`, not a path sibling; docs/architecture.md stale |
| 0007 zip import boundary | ADDED zip→GPX entries, per-file layers; architecture history only | only `.gpx` consumed from archives (import/gpx.rs:104-110); zip-slip guard only on bundle staging (archive.rs:174-178 via lizaalert.rs:1297) |
| 0008 reqwest+rustls | ADDED no-native-TLS | "blocking only" and "webpki roots" stale (tokio async client; rustls-platform-verifier) |
| 0009 tracing | ADDED | init in lib.rs:118-123; MAX_DIAGNOSTICS=200; zero println! |
| 0010 tiled rendering + LRU | partially stale; ADDED coarsest-level selection, per-map context reuse | no tile cache, no `lru` dep, no 4096 guard anywhere |
| 0011 no async runtime | stale; reality ADDED "Long-running work runs off the IPC thread and reports via events" | 52 sync commands; std::thread + tauri::async_runtime + tokio (commands/mod.rs:394-481; lizaalert.rs:15-16, 501-556); docs/conventions.md stale |
| 0012 sqlite sync + LRU 512 | MODIFIED sqlite protocol (zoom inversion, not row inversion); ADDED empty response for missing tiles | fresh `rusqlite::Connection` per request, no cache (tiles.rs:65) |
| 0013 style in domain | ADDED | domain/track.rs:46-63 |
| 0014 u64 newtypes | ADDED | six newtypes, caller-assigned max+1. The `len()+1` collision this note reported is gone: `import::next_layer_id` takes max+1 across map, track and waypoint layers. Re-checked 2026-09-23, when layer deletion first made a collision reachable |
| 0015 edition 2024 | ADDED; pin covered by ci-pipeline | rust-toolchain.toml 1.95.0 |
| 0016 Tauri/MapLibre/Svelte | covered (ui-shell stack); ADDED raw-byte tile IPC, OZF2 metadata command, generated bindings, MapLibre sole engine | manual `types.ts` mirroring and floating windows superseded |
| 0017 delta undo | covered; MODIFIED inverse/forward semantics and coalescing (code coalesces MoveTrackPoint, MoveWaypoint, RenameTrack) | commands.rs:1204-1382 |
| 0018 symbols as strings | ADDED open optional string | ADR emoji table differs from SymbolPicker.svelte:17-26 |
| 0019 doc reconciliation | ADDED `documentation`: verified-only claims, matrix contract | product decisions covered by track-display |
| 0020 MVP scope | ADDED `product-scope` inclusions/exclusions | "single layer, selector hidden" stale (selector exists); many declared items absent in code (see Findings) |
| 0021 depth 100 / no hybrid | covered | MAX_STACK_DEPTH commands.rs:9 |
| 0022 WPT export | MODIFIED WPT export (exact layout, cp1251); ADDED scope statement | default path `<bundle>/<layer>.wpt`; elevation/time always -777/0; "GPX for waypoints" has no command |
| 0023 no printing | ADDED `Map printing is not provided` | 0 print/pdf matches in lib.rs, no window.print |
| 0024 Playwright | folded into `revive-ui-cycle` evidence policy | ADR becomes a stub pointing to `visual-verification` |

### Legacy superpowers documents

| Document | Verdict |
|---|---|
| specs/2026-04-08-documentation-update-design | tasks executed; standing rule ADDED "Navigation docs are dated and match the source tree" |
| specs/2026-04-28-mvp-scope-design | duplicates ADR-0020; acceptance criterion partly ADDED ("shipped tool is a first-class command") |
| specs/2026-04-28-qa-debug-process-design | folded into `revive-ui-cycle` (protocol, template, classification, promotion, anti-loop); fully redundant afterwards |
| specs/2026-05-17-shadcn-ui-kit-svelte-design | covered by ui-shell and the three redesign changes; ADDED Lucide icons, single toast/tooltip host |
| specs/2026-05-17-meetily-inspired-future-work | not codified; items → `docs/backlog.md` |
| plans/2026-04-12-production-bugs-fix | executed; behaviors covered except empty-LineString filter (covered by revive per-segment geometry) and two MapView details (unspecified frontend) |
| plans/2026-04-28-mvp-audit, plans/2026-04-29-mcp-tooling-fixes | executed; tool behaviors ADDED in revive (13-tool contract, structured errors, W3C selectors, session timeouts) |

### Findings that need a decision or a fix (not resolved by this change)

Spec contradicts code — owner decides fix-code vs modify-spec. Re-checked
against the code on 2026-09-21; two of the three have since been closed by
fixing the code, which is owner decision 1:
- ~~`map-bundles` "Bundles root directory is user-configurable" promises persistence across restarts; `set_bundles_root` is in-memory only.~~ Closed in slice 0.3: `PersistedAppSession` carries `bundles_root` (infrastructure/persistence.rs:19).
- ~~`waypoints` "System exports waypoints to GPX" has no command or UI.~~ Closed on 2026-09-21: `export_gpx_waypoints` is registered (lib.rs:122) and reached from the Waypoint Inspector and the Waypoints tab row menu.
- `ui-shell` theme requirements presume a reachable picker; `ThemePicker.svelte` is imported by nothing. **Still true.** Both stores behind it work; only the placement is missing, and that is a design call — see `docs/backlog.md`.

Process decisions for the owner (raised by the QA slice):
- Keep `docs/qa/smoke-*.md` as a manual layer beside per-CJ tests (codified) or make CJ tests the only artifact.
- "Tier 1 sufficient for backend-only changes" was dropped because AGENTS.md makes `just smoke` mandatory; confirm.
- Drawing cancel via Esc leaves the cancelled draw on the redo stack and dirties the project; confirm or change.

Documentation bugs (make `documentation` scenarios pass): feature-status and roadmap mark sort/crop/split-join and ZIP as absent though present; command preamble says 45 commands and commands-reference omits `sort_track_points`, `crop_track_to_*`, `toggle_waypoint_visible`, `ReorderTrackPoints`, `CropTrackPoints`; seven feature-status rows use `TBD`; conventions.md "no async runtime"; architecture.md "local ozf2 crate"; glossary "DTO mirrored manually"; frontend-architecture.md "no Ctrl+Z"; persistence-session.md legacy paths and "bundles webview"; native-qa-mcp.md `qa_observe` description.

Code findings for `docs/backlog.md`, re-checked against the code on 2026-09-21:

Closed since, by slice 0.3 and the slices after it: map `LayerId` allocation by `len()+1`; `qa_observe` captures nothing; no HTTP timeouts on either reqwest client; `.kml` classified Supported without a parser; unused `lucide-svelte` dependency; GPX/WPT export errors reach only the status string.

Was never true as written: **zip-slip guard absent on the track-import archive path** — `extract_zip_entries_to_directory` has always used `enclosed_name()`, which refuses an entry that escapes, and both extraction paths (track import and cached bundle archives) go through it. What was missing was a test, since a one-call guard is exactly what a refactor drops silently; there are two now (`import/archive.rs`).

Closed on 2026-09-21: waypoint map markers draw no symbol glyph (`a-waypoint-looks-like-what-it-is`); cp1251 unrepresentable characters — pinned by a test, and the `write_line` comment corrected: it claimed `?`, which was never what `encoding_rs` does. Whether `&#NNNN;` or `?` suits a legacy OziExplorer better is a question for whoever has one in front of them, and is in `docs/backlog.md`.

Still open, and in `docs/backlog.md`: `get_ozi_tile` registered but unused — the map renders OZF2 through `get_ozi_tile_projected`, so this is dead IPC surface rather than a missing feature; declared-but-absent ADR-0020 items (open-by-URL, on-map tools, crop by selection, walkthrough, waypoint colour, recent `.ozp`).

### Owner decisions (2026-09-19)

1. Code is primary: where a spec promises behavior the code lacks and the owner wants the behavior, the code is fixed rather than the spec weakened. Bundles-root persistence → `revive-ui-cycle` slice 0.3.
2. "System exports waypoints to GPX" stays as a target requirement; implementation with CJ-6.
3. The manual smoke-document layer (`docs/qa/smoke-*.md`, template, classification vocabulary) is dropped; per-CJ Appium smoke tests must assert journey outcomes (files re-import equal, projects reopen equal, tiles serve offline), never only element presence. The owner's words: repeatedly drawing a three-point track proves nothing.
4. "Tier 1 alone suffices for backend-only changes" stays dropped.
5. Esc during a draw discards the draw commands without a redo entry and restores the dirty flag (requirement rewritten in the `track-editing` delta; implemented in slice 0.3).
6. Platforms: all effort on macOS while the owner is the only user; Windows returns in the distribution phase.

### Purpose texts for the new capabilities (apply right after archive)

**architecture.** Covers the structural invariants of the Rust backend that every other capability relies on: the four-module layering and its dependency direction, the thin Tauri command layer over `AppState`, opaque `u64` newtype identifiers, `TrackStyle` as domain data, the Rust edition, `tracing`-based logging, the background-work and async model as implemented, the `ozf2` decoding boundary and the generated IPC bindings. Decision history: ADR-0001 (2026-03-22) four layers, domain pure, edits via commands — codified as the layering and thin-handler requirements, edit rule in `undo-redo`; reality: application and infrastructure share data types both ways. ADR-0006 (2026-03-29) OZF2 decoder behind one adapter — codified; the crate is crates.io `ozf2 = "0.1"`, not a path sibling. ADR-0009 (2026-03-30) `tracing` + env-filter — codified. ADR-0011 (2026-03-23) no async runtime — not codified, stale: tokio drives concurrent downloads; reality codified as "Long-running work runs off the IPC thread and reports via events". ADR-0013 (2026-03-29) `TrackStyle` on `Track` — codified. ADR-0014 (2026-03-23) u64 newtypes, caller-assigned — codified. ADR-0015 (2026-03-23) edition 2024 — codified, pin in `ci-pipeline`. ADR-0016 (2026-03-30) Tauri 2 IPC — codified as generated bindings; manual `types.ts` mirroring superseded by tauri-specta. ADR-0003 superseded by ADR-0016; ADR-0005 superseded by ADR-0017.

**product-scope.** What the product includes and excludes, independent of implementation status: the LizaAlert SAR workflow (open bundle → import or draw tracks → edit → place waypoints → save `.ozp` → export GPX/PLT/WPT), the supported platforms and the explicit non-goals. Feature behavior lives in the other capabilities; this one answers "is X something we build at all?". Decision history: ADR-0020 (2026-04-28) MVP as one SAR workflow — codified as the "in scope" requirements; "single layer in UI" superseded by the active-layer selector present in code; waypoint "export to PLT" contradicted by ADR-0022 and not codified. ADR-0022 (2026-04-28) WPT export in, WPT import out — codified. ADR-0023 (2026-04-28) no map printing — codified. Legacy mvp-scope design (2026-04-28) restates ADR-0020; its acceptance criterion "a shipped tool is a first-class command" is codified. Owner decision (2026-09-19): macOS primary while the owner is the only user; Windows in the distribution phase.

**documentation.** Rules for the repository's prose: what docs may claim, how the feature-status matrix is shaped, which navigation docs must track the tree, and where decisions are recorded now that OpenSpec is the normative source. Decision history: ADR-0019 (2026-04-25) implemented / backend-only / planned states and `docs/feature-status.md` — codified. Legacy documentation-update design (2026-04-08) one-off resync — tasks done; standing rule codified as "Navigation docs are dated and match the source tree". Change `codify-architecture-decisions` (2026-09-19): decisions move from `docs/adr/` (frozen at 24 pointer stubs) into Purpose decision history and change `design.md` — codified as "Decisions are recorded in OpenSpec, not in new ADR files" and "OpenSpec specs are normative; feature-status is evidence".
