# Documentation Audit Fixes and Feature Reconciliation

## TL;DR
> **Summary**: Fix all 10 audit findings by implementing the user-facing behavior that docs already claim where practical, downgrading only over-broad wording where full implementation would exceed MVP, and adding ADR-backed documentation so future docs distinguish backend capability from surfaced UI.
> **Deliverables**:
> - Startup session restore for last project + active map, with documented non-goals.
> - Timestamp rendering in the point list.
> - Track color and line-width UI controls.
> - Undoable waypoint symbol changes.
> - OK-standard warning for track names.
> - `10-Tracks/` export default behavior.
> - Minimal multi-layer UI MVP that removes hardcoded layer `1n` from user workflows.
> - Documentation for `src/lib/windows.ts`, persistence/session behavior, and feature-status split.
> - ADR documenting the reconciliation decisions and boundaries.
> **Effort**: Large
> **Parallel**: YES - 5 waves
> **Critical Path**: Task 1 → Task 2/3/4/5/6 → Task 7 → Task 8/9/10/11 → Final Verification

## Context

### Original Request
The user asked: “create a plan of that doc fixes, additional docs and all 10 found issues fixes. Dont forget to doc all you decision in ADR and other docs.”

### Interview Summary
No extra interview was needed because the prior audit produced concrete file-backed findings. Interpretation: “fixes” means each finding must either be implemented to match existing docs or the docs must be corrected if a full implementation would exceed the intended MVP. For this plan, choose implementation for the claimed user-facing features where bounded, and documentation/status clarification for broad capability claims.

### Audit Findings Covered
1. Startup restore is documented as complete but not implemented.
2. Track point timestamps are documented as shown but not rendered.
3. Track color/line-width controls are documented but not surfaced.
4. `set_waypoint_symbol` is documented as undoable but bypasses `ProjectCommand`.
5. OK-standard track-name validation/warning is documented but missing.
6. `10-Tracks/` export suggestion is documented but missing from export dialogs.
7. Multi-layer support is partial: backend supports it, UI hardcodes layer `1n`.
8. `src/lib/windows.ts` is used but under-documented.
9. No dedicated persistence/session behavior documentation exists.
10. No feature-status split exists between backend, UI, docs, and planned work.

### Metis Review (gaps addressed)
- Bound multi-layer UI to an MVP: expose/select active track and waypoint layers and use those selections in existing workflows; do not build full layer management.
- Bound session restore to last project path + active map reference only; do not restore viewport, selected entities, open windows, or unsaved UI state.
- Choose warning-only OK-standard validation, not hard blocking.
- Define `10-Tracks/` default for GPX layer export and PLT track export only.
- Make waypoint symbol changes undoable as one command per committed change; no coalescing for symbol picker.
- Add `docs/feature-status.md` to prevent future over-broad “supported” claims.

## Work Objectives

### Core Objective
Bring documentation and implementation back into alignment for all 10 audit findings, with executable tests and ADR-backed decisions.

### Deliverables
- New ADR: `docs/adr/adr-0019-doc-audit-reconciliation.md`.
- New doc: `docs/persistence-session.md`.
- New doc: `docs/feature-status.md`.
- Updates to `README.md`, `docs/requirements.md`, `docs/roadmap.md`, `docs/frontend-architecture.md`, `docs/architecture.md`, and `docs/commands-reference.md`.
- Backend/session persistence code and tests.
- Backend command-stack support for waypoint symbol changes and tests.
- Frontend UI fixes and Vitest/component coverage for timestamp, style controls, OK warning, export defaults, and minimal layer selection.

### Definition of Done
- [ ] Every audit finding has either an implementation fix or explicit doc/status correction in `docs/feature-status.md`.
- [ ] `docs/adr/adr-0019-doc-audit-reconciliation.md` records all decisions listed above.
- [ ] `README.md`, `docs/requirements.md`, and `docs/roadmap.md` no longer claim unavailable behavior without qualification.
- [ ] No direct `invoke()` calls are introduced outside `src/lib/api.ts`.
- [ ] Run `just test-rust` and it passes, or any pre-existing unrelated blocker is documented with exact error.
- [ ] Run `just test-ui` and it passes.
- [ ] Run `just clippy` and it passes, or any pre-existing unrelated blocker is documented with exact error.
- [ ] Run `just test` and it passes, or failures are traced to pre-existing unrelated blockers with exact evidence.

### Must Have
- Preserve the documented Rust layer boundaries: domain pure, application orchestrates state, infrastructure handles file/session persistence, commands remain thin IPC wrappers.
- Frontend components must call only wrappers in `src/lib/api.ts`.
- TypeScript DTOs in `src/lib/types.ts` must remain manually synchronized with Rust DTOs in `src-tauri/src/commands/mod.rs`.
- Every task must update docs if behavior or status changes.
- Every implementation task must include happy path and failure/edge-case tests or QA.

### Must NOT Have
- Do not implement KML, PDF/image export, GPS sync, live telemetry, polygon drawing, multi-device sync, or full GIS-style layer manager.
- Do not add direct Tauri `invoke()` imports in Svelte components.
- Do not make OK-standard validation block rename, save, or export; warning-only is the selected behavior.
- Do not persist viewport, selected entities, panel open/closed state, bundle-loader window state, or unsaved project edits as part of session restore.
- Do not broaden multi-layer UI beyond active layer selection and removal of hardcoded layer `1n` from existing track/waypoint workflows.

## Verification Strategy
> ZERO HUMAN INTERVENTION - all verification is agent-executed.

- Test decision: tests-after for existing codebase patterns; add targeted Rust inline tests and Vitest tests where behavior is changed.
- QA policy: Every task has agent-executed scenarios.
- Evidence path: `.sisyphus/evidence/task-{N}-{slug}.{ext}`.
- Full-suite commands: `just test-rust`, `just test-ui`, `just clippy`, `just test`.

## Execution Strategy

### Parallel Execution Waves
- Wave 1: Task 1 foundation decisions/docs skeleton.
- Wave 2: Tasks 2, 3, 4, 5, 6, 7 independent bounded fixes.
- Wave 3: Task 8 minimal multi-layer UI after layer/status decisions are documented.
- Wave 4: Tasks 9 and 10 documentation reconciliation and status docs.
- Wave 5: Task 11 final docs alignment + final verification.

### Dependency Matrix
| Task | Depends On | Blocks |
|------|------------|--------|
| 1 | none | 2,3,4,5,6,7,8,9,10,11 |
| 2 | 1 | 11 |
| 3 | 1 | 11 |
| 4 | 1 | 11 |
| 5 | 1 | 11 |
| 6 | 1 | 11 |
| 7 | 1 | 10,11 |
| 8 | 1 | 11 |
| 9 | 1 | 11 |
| 10 | 1,2,3,4,5,6,7,8,9 | 11 |
| 11 | 1-10 | Final Verification |

### Agent Dispatch Summary
| Wave | Task Count | Categories |
|------|------------|------------|
| 1 | 1 | writing |
| 2 | 6 | quick, visual-engineering, unspecified-high |
| 3 | 1 | deep |
| 4 | 2 | writing |
| 5 | 1 | unspecified-high |

## TODOs

- [x] 1. Record reconciliation decisions in ADR and docs skeleton

  **What to do**: Create `docs/adr/adr-0019-doc-audit-reconciliation.md` documenting the audit-triggered decisions: implement bounded missing features; warning-only OK validation; session restore scope; minimal multi-layer UI scope; feature-status table policy; waypoint symbol command-stack behavior. Create empty/skeleton sections in `docs/persistence-session.md` and `docs/feature-status.md` so later tasks fill them consistently.
  **Must NOT do**: Do not claim task-specific fixes are complete before their tasks land. Use “Planned in this reconciliation” wording until later tasks update status.

  **Recommended Agent Profile**:
  - Category: `writing` - ADR and documentation structure.
  - Skills: [] - No special skill required.
  - Omitted: [`frontend-ui-ux`] - No UI implementation in this task.

  **Parallelization**: Can Parallel: NO | Wave 1 | Blocks: 2-11 | Blocked By: none

  **References**:
  - Pattern: `docs/adr/adr-0016-tauri-maplibre-svelte.md` - ADR style and consequences sections.
  - Pattern: `docs/adr/adr-0017-delta-command-stack.md` - command-stack decision format.
  - Existing claims: `README.md:52-80`, `docs/requirements.md:100-164`, `docs/roadmap.md:20-123`.
  - Architecture: `AGENTS.md` - layer boundaries and frontend conventions.

  **Acceptance Criteria**:
  - [ ] `docs/adr/adr-0019-doc-audit-reconciliation.md` exists and lists all 10 audit findings with selected implement-vs-document decisions.
  - [ ] `docs/persistence-session.md` exists with sections: Persisted, Not Persisted, Restore Flow, Missing File Behavior, Tests.
  - [ ] `docs/feature-status.md` exists with columns: Feature, Backend, UI, Docs, Status, Evidence.

  **QA Scenarios**:
  ```
  Scenario: ADR covers all decisions
    Tool: Bash
    Steps: Run `python3 - <<'PY'\nfrom pathlib import Path\np=Path('docs/adr/adr-0019-doc-audit-reconciliation.md').read_text()\nfor term in ['startup restore','timestamp','color','line width','waypoint symbol','OK-standard','10-Tracks','multi-layer','windows.ts','feature-status']:\n    assert term.lower() in p.lower(), term\nPY`
    Expected: Command exits 0.
    Evidence: .sisyphus/evidence/task-1-adr-coverage.txt

  Scenario: Skeleton docs are not empty
    Tool: Bash
    Steps: Run `test -s docs/persistence-session.md && test -s docs/feature-status.md`
    Expected: Command exits 0.
    Evidence: .sisyphus/evidence/task-1-doc-skeletons.txt
  ```

  **Commit**: YES | Message: `docs(adr): record audit reconciliation decisions` | Files: [`docs/adr/adr-0019-doc-audit-reconciliation.md`, `docs/persistence-session.md`, `docs/feature-status.md`]

- [x] 2. Implement bounded startup session restore and document persistence behavior

  **What to do**: Add infrastructure-level app session persistence storing only last project path and active map reference/path sufficient to restore active map if still present. Application startup should load this session after `AppState::new()` and before first frontend refresh. Save/update session after successful project load/save and active map open. Missing project/map paths must not panic; emit diagnostic and continue with fresh state. Fill `docs/persistence-session.md` with final behavior and update README/roadmap claims to match.
  **Must NOT do**: Do not persist viewport, selected tracks/waypoints, panel state, theme, hidden bundle window state, or unsaved edits. Do not store session data inside `.ozp` project files.

  **Recommended Agent Profile**:
  - Category: `unspecified-high` - Rust application/infrastructure change with docs.
  - Skills: [] - No extra skill required.
  - Omitted: [`frontend-ui-ux`] - Minimal frontend involvement only if a startup command wrapper is required.

  **Parallelization**: Can Parallel: YES | Wave 2 | Blocks: 11 | Blocked By: 1

  **References**:
  - Current fresh state: `src-tauri/src/application/mod.rs:154-178`.
  - Current project load clears map: `src-tauri/src/application/mod.rs:501-509`.
  - Existing unwired helper: `src-tauri/src/application/mod.rs:917-920`.
  - Current frontend mount: `src/App.svelte:14-18`.
  - Project persistence pattern: `src-tauri/src/infrastructure/persistence.rs`.
  - Claims to align: `README.md:77-80`, `docs/roadmap.md:22-60`.

  **Acceptance Criteria**:
  - [ ] Rust tests prove valid session restores project path and active map.
  - [ ] Rust tests prove missing project or map results in diagnostic/status warning and fresh startup, not panic.
  - [ ] `.ozp` project serialization remains unchanged for existing project files.
  - [ ] `docs/persistence-session.md`, `README.md`, and `docs/roadmap.md` accurately state exactly what is restored.

  **QA Scenarios**:
  ```
  Scenario: Valid last session restores
    Tool: Bash
    Steps: Run `cargo test --manifest-path src-tauri/Cargo.toml session_restore_valid -- --nocapture`
    Expected: Test exits 0 and asserts project_path and active_map are restored.
    Evidence: .sisyphus/evidence/task-2-session-restore-valid.txt

  Scenario: Missing files degrade gracefully
    Tool: Bash
    Steps: Run `cargo test --manifest-path src-tauri/Cargo.toml session_restore_missing -- --nocapture`
    Expected: Test exits 0 and asserts no panic plus warning diagnostic/status.
    Evidence: .sisyphus/evidence/task-2-session-restore-missing.txt
  ```

  **Commit**: YES | Message: `feat(session): restore last project and active map` | Files: [`src-tauri/src/application/mod.rs`, `src-tauri/src/infrastructure/persistence.rs` or new infrastructure session file, `src-tauri/src/lib.rs`, `docs/persistence-session.md`, `README.md`, `docs/roadmap.md`]

- [x] 3. Render track point timestamps in TrackPointsPanel

  **What to do**: Update `TrackPointsPanel.svelte` to render `point.timestamp` when present using a deterministic readable format. Decision: display ISO string as received initially to avoid timezone ambiguity; show no extra placeholder when absent. Add or update frontend tests if component testing utilities exist; otherwise add a targeted Vitest helper test for formatting and document manual-free QA evidence.
  **Must NOT do**: Do not sort points by timestamp; that remains deferred in roadmap. Do not mutate backend DTO shape unless necessary.

  **Recommended Agent Profile**:
  - Category: `quick` - small UI rendering fix.
  - Skills: [] - No extra skill required.
  - Omitted: [`frontend-ui-ux`] - No design exploration needed.

  **Parallelization**: Can Parallel: YES | Wave 2 | Blocks: 11 | Blocked By: 1

  **References**:
  - Current UI: `src/components/TrackPointsPanel.svelte:79-85`.
  - DTO source: `src-tauri/src/commands/mod.rs:849-919`.
  - TS type: `src/lib/types.ts:92`.
  - Overstated doc claim: `docs/requirements.md:108`.

  **Acceptance Criteria**:
  - [ ] Point rows show timestamp text when `point.timestamp` is defined.
  - [ ] Point rows without timestamp still render without `undefined`, `null`, or broken punctuation.
  - [ ] `docs/requirements.md` still marks timestamp display done only after the UI test passes.

  **QA Scenarios**:
  ```
  Scenario: Timestamp renders
    Tool: Bash
    Steps: Run `npm run test -- --run src/test/track-points-panel.test.ts`
    Expected: Test exits 0 and finds `2024-06-01T12:34:56Z` or equivalent ISO text in rendered point row.
    Evidence: .sisyphus/evidence/task-3-timestamp-render.txt

  Scenario: Missing timestamp is clean
    Tool: Bash
    Steps: Run the same test file and assert no row contains `undefined` or `null`.
    Expected: Test exits 0.
    Evidence: .sisyphus/evidence/task-3-timestamp-missing.txt
  ```

  **Commit**: YES | Message: `fix(ui): show track point timestamps` | Files: [`src/components/TrackPointsPanel.svelte`, `src/test/track-points-panel.test.ts`, `docs/requirements.md`]

- [x] 4. Add track color and line-width controls to TracksPanel

  **What to do**: Add compact controls in `TracksPanel.svelte`: color input bound to `setTrackColor()` and numeric/range width input bound to `setTrackLineWidth()`. Use existing color-dot style as preview. Convert CSS hex to `[r,g,b,255]`. Width bounds: 1-12 px, step 1, default from feature property if available; if `getTracksGeojson()` does not currently expose `line_width`, add it to backend GeoJSON properties and TS local interface. Update frontend architecture docs.
  **Must NOT do**: Do not make track style undoable in this task; `docs/architecture.md` explicitly says track style mutations bypass CommandStack.

  **Recommended Agent Profile**:
  - Category: `visual-engineering` - UI controls and styling.
  - Skills: [`frontend-ui-ux`] - Small UI ergonomics.
  - Omitted: [`superpowers:test-driven-development`] - Existing codebase uses tests-after here.

  **Parallelization**: Can Parallel: YES | Wave 2 | Blocks: 11 | Blocked By: 1

  **References**:
  - Current panel: `src/components/TracksPanel.svelte:1-156`.
  - API wrappers: `src/lib/api.ts:82-88`, `src/lib/api.ts:215-220`.
  - Track rendering uses properties: `src/lib/maplibre/tracks-layer.ts`.
  - Backend GeoJSON likely in `src-tauri/src/commands/mod.rs` around `get_tracks_geojson`.
  - Docs claim: `README.md:55`, `docs/frontend-architecture.md:22`.

  **Acceptance Criteria**:
  - [ ] User can change track color from `TracksPanel.svelte`; UI calls `setTrackColor()` through `src/lib/api.ts`.
  - [ ] User can change line width 1-12 px; UI calls `setTrackLineWidth()` through `src/lib/api.ts`.
  - [ ] Map render updates after `state-changed` refresh.
  - [ ] No direct `invoke()` appears in Svelte components.

  **QA Scenarios**:
  ```
  Scenario: Style controls call API wrappers
    Tool: Bash
    Steps: Run `npm run test -- --run src/test/tracks-panel-style.test.ts`
    Expected: Test exits 0 and verifies color and width handlers call wrapper mocks with selected layerId/trackId.
    Evidence: .sisyphus/evidence/task-4-track-style-controls.txt

  Scenario: No direct invoke regression
    Tool: Bash
    Steps: Run `python3 - <<'PY'\nfrom pathlib import Path\nfor p in Path('src').glob('**/*.svelte'):\n    assert 'invoke(' not in p.read_text(), p\nPY`
    Expected: Command exits 0.
    Evidence: .sisyphus/evidence/task-4-no-direct-invoke.txt
  ```

  **Commit**: YES | Message: `feat(ui): expose track style controls` | Files: [`src/components/TracksPanel.svelte`, `src-tauri/src/commands/mod.rs`, `src/test/tracks-panel-style.test.ts`, `docs/frontend-architecture.md`, `README.md`]

- [x] 5. Make waypoint symbol changes undoable through ProjectCommand

  **What to do**: Add `ProjectCommand::SetWaypointSymbol { layer_id, waypoint_id, old_symbol, new_symbol }`, apply/reverse handling, constructor, and AppState method that uses `history.apply()`. Update `set_waypoint_symbol` Tauri handler to route through AppState command flow and return errors for missing waypoint/layer. Update docs to move `set_waypoint_symbol` into undoable waypoint mutations truthfully.
  **Must NOT do**: Do not coalesce repeated symbol changes; each committed picker choice is one undo step. Do not alter add/move/rename waypoint behavior.

  **Recommended Agent Profile**:
  - Category: `unspecified-high` - command-stack Rust change with tests.
  - Skills: [] - No extra skill required.
  - Omitted: [`frontend-ui-ux`] - No UI work required except maybe error propagation.

  **Parallelization**: Can Parallel: YES | Wave 2 | Blocks: 11 | Blocked By: 1

  **References**:
  - Existing enum: `src-tauri/src/application/commands.rs:23-146`.
  - Existing handler bypass: `src-tauri/src/commands/mod.rs:564-576`.
  - Current doc contradiction: `docs/commands-reference.md:99-107`, `docs/architecture.md:145`.
  - Waypoint domain: `src-tauri/src/domain/waypoint.rs`.

  **Acceptance Criteria**:
  - [ ] `set_waypoint_symbol` updates symbol through command stack.
  - [ ] `undo` restores previous symbol; `redo` reapplies new symbol.
  - [ ] Missing waypoint/layer returns `Err` and does not mutate state.
  - [ ] `docs/commands-reference.md` accurately lists `set_waypoint_symbol` as undoable after implementation.

  **QA Scenarios**:
  ```
  Scenario: Symbol undo/redo
    Tool: Bash
    Steps: Run `cargo test --manifest-path src-tauri/Cargo.toml waypoint_symbol_undo -- --nocapture`
    Expected: Test exits 0 and asserts set → undo → redo symbol sequence.
    Evidence: .sisyphus/evidence/task-5-symbol-undo.txt

  Scenario: Missing waypoint safe error
    Tool: Bash
    Steps: Run `cargo test --manifest-path src-tauri/Cargo.toml waypoint_symbol_missing -- --nocapture`
    Expected: Test exits 0 and asserts command returns error without mutation.
    Evidence: .sisyphus/evidence/task-5-symbol-missing.txt
  ```

  **Commit**: YES | Message: `feat(commands): make waypoint symbols undoable` | Files: [`src-tauri/src/application/commands.rs`, `src-tauri/src/application/mod.rs`, `src-tauri/src/commands/mod.rs`, `docs/commands-reference.md`, `docs/architecture.md`]

- [x] 6. Add warning-only OK-standard track-name validation

  **What to do**: Add shared frontend validation helper for `YYYYMMDD_Callsign` where callsign is non-empty after underscore. Show warning in `TracksPanel.svelte` during rename and for existing non-conforming names. Do not block rename/export/save. Add docs clarifying warning-only behavior. If backend already has no validation, leave backend permissive but document that UI surfaces warning.
  **Must NOT do**: Do not reject Cyrillic callsigns. Do not block user workflows. Do not enforce date validity beyond 8 digits unless ADR explicitly chooses stricter validation; selected decision is pattern warning only.

  **Recommended Agent Profile**:
  - Category: `visual-engineering` - UI warning state.
  - Skills: [`frontend-ui-ux`] - Clear inline warning.
  - Omitted: []

  **Parallelization**: Can Parallel: YES | Wave 2 | Blocks: 11 | Blocked By: 1

  **References**:
  - Rename UI: `src/components/TracksPanel.svelte:38-48`, `src/components/TracksPanel.svelte:101-115`.
  - Rename command: `src-tauri/src/commands/mod.rs:534-546`.
  - Claims: `README.md:56,99-102`, `docs/requirements.md:161-162`, `docs/roadmap.md:59`.

  **Acceptance Criteria**:
  - [ ] `20240601_Иванов` shows no warning.
  - [ ] `Track 1` shows a warning but can still be saved as the track name.
  - [ ] Warning behavior is documented as non-blocking.

  **QA Scenarios**:
  ```
  Scenario: Valid OK name has no warning
    Tool: Bash
    Steps: Run `npm run test -- --run src/test/track-name-validation.test.ts`
    Expected: Test exits 0 and valid Cyrillic callsign has no warning.
    Evidence: .sisyphus/evidence/task-6-ok-valid.txt

  Scenario: Invalid name warns but does not block
    Tool: Bash
    Steps: Run the same test file and verify invalid name warning plus commit callback still fires.
    Expected: Test exits 0.
    Evidence: .sisyphus/evidence/task-6-ok-invalid.txt
  ```

  **Commit**: YES | Message: `feat(ui): warn on non-standard track names` | Files: [`src/components/TracksPanel.svelte`, `src/lib/track-names.ts`, `src/test/track-name-validation.test.ts`, `README.md`, `docs/requirements.md`]

- [x] 7. Default track exports to active bundle `10-Tracks/` when available

  **What to do**: Add backend IPC helper or extend app state DTO to expose an export default directory/path when an active bundle exists. Frontend `handleExport()` and `handleExportPlt()` should use `10-Tracks/<track-name>.gpx|.plt` as dialog default when available; otherwise keep current filename-only fallback. Use platform-safe path construction in Rust; frontend receives a string path only.
  **Must NOT do**: Do not change actual export formats. Do not force users to export there; this is only a default suggestion. Do not apply to project `.ozp` save dialogs.

  **Recommended Agent Profile**:
  - Category: `unspecified-high` - cross-layer Rust/TS path behavior.
  - Skills: [] - No extra skill required.
  - Omitted: [`frontend-ui-ux`] - Dialog default only.

  **Parallelization**: Can Parallel: YES | Wave 2 | Blocks: 11 | Blocked By: 1

  **References**:
  - Current dialogs: `src/components/TracksPanel.svelte:54-73`.
  - Existing backend helper behavior: `src-tauri/src/infrastructure/lizaalert.rs:224,263`.
  - Current app state active map: `src/lib/stores.ts:31`, `src/lib/types.ts:44`.
  - Claims: `README.md:60`, `docs/requirements.md:163-164`, `docs/roadmap.md:58`.

  **Acceptance Criteria**:
  - [ ] With active bundle, GPX export dialog default path includes `10-Tracks` and `<track-name>.gpx`.
  - [ ] With active bundle, PLT export dialog default path includes `10-Tracks` and `<track-name>.plt`.
  - [ ] Without active bundle, fallback default remains `<track-name>.gpx|.plt`.
  - [ ] Backend path tests are platform-safe.

  **QA Scenarios**:
  ```
  Scenario: Active bundle default path
    Tool: Bash
    Steps: Run `cargo test --manifest-path src-tauri/Cargo.toml export_default_tracks_dir -- --nocapture`
    Expected: Test exits 0 and asserts path ends with `10-Tracks/<name>.gpx` using PathBuf components.
    Evidence: .sisyphus/evidence/task-7-export-default-rust.txt

  Scenario: Frontend dialog uses helper path
    Tool: Bash
    Steps: Run `npm run test -- --run src/test/export-default-path.test.ts`
    Expected: Test exits 0 and verifies open/save dialog defaultPath uses backend-provided default when present and fallback when absent.
    Evidence: .sisyphus/evidence/task-7-export-default-ui.txt
  ```

  **Commit**: YES | Message: `feat(export): suggest active bundle tracks folder` | Files: [`src-tauri/src/application/mod.rs`, `src-tauri/src/commands/mod.rs`, `src/lib/api.ts`, `src/components/TracksPanel.svelte`, `src/test/export-default-path.test.ts`, `README.md`, `docs/requirements.md`]

- [x] 8. Implement minimal active-layer UI and remove hardcoded layer 1 from workflows

  **What to do**: Add active track layer and active waypoint layer state to frontend stores, initialized from first available layer. Expose minimal selectors in Sidebar or panels listing existing project track/waypoint layers. Replace hardcoded `1n` usages in `Sidebar.svelte`, `MapView.svelte`, and `WaypointsPanel.svelte` with active layer store values. Ensure imported additional layers can be selected and used for drawing, waypoint add/move, waypoint list, and track detail fetches. Update docs to say backend supports multiple layers and UI supports active-layer selection, not full layer management.
  **Must NOT do**: Do not add create/rename/delete/reorder layer UI unless existing backend commands make it trivial and docs require it. Do not change persisted project format except if TS DTO exposure requires adding already-existing layers to app state.

  **Recommended Agent Profile**:
  - Category: `deep` - multi-file frontend/backend state coordination.
  - Skills: [] - No extra skill required.
  - Omitted: [`frontend-ui-ux`] - Simple selectors only; no major design work.

  **Parallelization**: Can Parallel: NO | Wave 3 | Blocks: 10,11 | Blocked By: 1

  **References**:
  - Hardcoded drawing: `src/components/Sidebar.svelte:65-68`.
  - Hardcoded waypoint workflows: `src/components/MapView.svelte`, `src/components/WaypointsPanel.svelte` audit findings.
  - Default layer creation: `src-tauri/src/application/mod.rs:154-158`.
  - Existing roadmap note: `docs/roadmap.md:123`.
  - App state DTO: `src-tauri/src/commands/mod.rs` app state serialization and `src/lib/types.ts`.

  **Acceptance Criteria**:
  - [ ] No user-workflow code path in `Sidebar.svelte`, `MapView.svelte`, or `WaypointsPanel.svelte` hardcodes layer `1n` except default initialization fallback with comment.
  - [ ] User can select a non-default track layer and create/draw a track in it.
  - [ ] User can select a non-default waypoint layer and add/move/list waypoints in it.
  - [ ] Existing layer 1 behavior remains unchanged when only default layers exist.
  - [ ] Docs explicitly mark full layer management as not implemented if no create/rename/delete layer UI is added.

  **QA Scenarios**:
  ```
  Scenario: No hardcoded layer 1 in workflows
    Tool: Bash
    Steps: Run `python3 - <<'PY'\nfrom pathlib import Path\nfor f in ['src/components/Sidebar.svelte','src/components/MapView.svelte','src/components/WaypointsPanel.svelte']:\n    txt=Path(f).read_text()\n    assert 'createEmptyTrack(1n' not in txt, f\n    assert 'getWaypoints(1n' not in txt, f\n    assert 'addWaypoint(1n' not in txt, f\n    assert 'moveWaypoint(1n' not in txt, f\nPY`
    Expected: Command exits 0.
    Evidence: .sisyphus/evidence/task-8-no-hardcoded-layer.txt

  Scenario: Active layer selection routes operations
    Tool: Bash
    Steps: Run `npm run test -- --run src/test/active-layer-ui.test.ts`
    Expected: Test exits 0 and verifies selected layer id is passed to create track and waypoint APIs.
    Evidence: .sisyphus/evidence/task-8-active-layer-ui.txt
  ```

  **Commit**: YES | Message: `feat(ui): use active track and waypoint layers` | Files: [`src/lib/stores.ts`, `src/lib/types.ts`, `src-tauri/src/commands/mod.rs`, `src/components/Sidebar.svelte`, `src/components/MapView.svelte`, `src/components/WaypointsPanel.svelte`, `src/test/active-layer-ui.test.ts`, `docs/roadmap.md`, `docs/requirements.md`, `docs/feature-status.md`]

- [x] 9. Document bundle-loader window architecture and `src/lib/windows.ts`

  **What to do**: Update `docs/frontend-architecture.md` to document `src/lib/windows.ts`, `precreateBundleLoader()`, `openBundleLoader()`, `/?view=bundles`, and why the hidden webview is pre-created. Update `docs/feature-status.md` with the secondary-window status. Mention that session restore does not persist window open/closed state.
  **Must NOT do**: Do not change window behavior unless tests reveal it is broken. This is a documentation-only fix.

  **Recommended Agent Profile**:
  - Category: `writing` - documentation-only.
  - Skills: [] - No extra skill required.
  - Omitted: [`frontend-ui-ux`] - No UI changes.

  **Parallelization**: Can Parallel: YES | Wave 4 | Blocks: 11 | Blocked By: 1

  **References**:
  - Window utility: `src/lib/windows.ts:1-38`.
  - Startup precreate: `src/main.ts:16-19`.
  - Sidebar opens window: `src/components/Sidebar.svelte:31,95`.
  - Current doc only mentions view: `docs/frontend-architecture.md:30-34`.

  **Acceptance Criteria**:
  - [ ] `docs/frontend-architecture.md` names and explains `src/lib/windows.ts`.
  - [ ] `docs/frontend-architecture.md` documents `/?view=bundles` routing in `src/main.ts`.
  - [ ] `docs/persistence-session.md` states bundle-loader window visibility is not session-persisted.

  **QA Scenarios**:
  ```
  Scenario: windows.ts documented
    Tool: Bash
    Steps: Run `python3 - <<'PY'\nfrom pathlib import Path\nt=Path('docs/frontend-architecture.md').read_text()\nfor term in ['src/lib/windows.ts','precreateBundleLoader','openBundleLoader','view=bundles']:\n    assert term in t, term\nPY`
    Expected: Command exits 0.
    Evidence: .sisyphus/evidence/task-9-windows-doc.txt

  Scenario: Session docs exclude window state
    Tool: Bash
    Steps: Run `grep -n "window" docs/persistence-session.md`
    Expected: Output includes statement that bundle-loader window state is not persisted.
    Evidence: .sisyphus/evidence/task-9-window-session-nongoal.txt
  ```

  **Commit**: YES | Message: `docs(frontend): document bundle loader window lifecycle` | Files: [`docs/frontend-architecture.md`, `docs/persistence-session.md`, `docs/feature-status.md`]

- [x] 10. Complete feature-status matrix and reconcile all public docs

  **What to do**: Fill `docs/feature-status.md` with all audited features and evidence links. Update `README.md`, `docs/requirements.md`, `docs/roadmap.md`, `docs/frontend-architecture.md`, `docs/architecture.md`, and `docs/commands-reference.md` so every claim matches implementation after Tasks 2-9. Use precise language: “backend supports,” “UI surfaces,” “planned,” and “not implemented.”
  **Must NOT do**: Do not use broad “done” labels without evidence. Do not mark full layer management done if Task 8 implements only active-layer selection.

  **Recommended Agent Profile**:
  - Category: `writing` - docs reconciliation.
  - Skills: [] - No extra skill required.
  - Omitted: []

  **Parallelization**: Can Parallel: YES | Wave 4 | Blocks: 11 | Blocked By: 1-9

  **References**:
  - Public docs: `README.md`, `docs/requirements.md`, `docs/roadmap.md`, `docs/frontend-architecture.md`, `docs/architecture.md`, `docs/commands-reference.md`.
  - Audit contradictions: all 10 findings in this plan context.
  - ADR from Task 1: `docs/adr/adr-0019-doc-audit-reconciliation.md`.

  **Acceptance Criteria**:
  - [ ] `docs/feature-status.md` has rows for all 10 audit findings.
  - [ ] No doc claims startup restore includes viewport/window/panel state.
  - [ ] No doc claims timestamp display unless Task 3 tests pass.
  - [ ] No doc claims full multi-layer management if only active-layer selection exists.
  - [ ] `set_waypoint_symbol` docs match Task 5 implementation.

  **QA Scenarios**:
  ```
  Scenario: Feature-status covers all audit findings
    Tool: Bash
    Steps: Run `python3 - <<'PY'\nfrom pathlib import Path\nt=Path('docs/feature-status.md').read_text().lower()\nfor term in ['startup restore','timestamp','color','line width','waypoint symbol','ok-standard','10-tracks','multi-layer','windows.ts','backend capability']:\n    assert term in t, term\nPY`
    Expected: Command exits 0.
    Evidence: .sisyphus/evidence/task-10-feature-status-coverage.txt

  Scenario: Docs avoid broad unsupported claims
    Tool: Bash
    Steps: Run `python3 - <<'PY'\nfrom pathlib import Path\ncombined='\n'.join(Path(p).read_text() for p in ['README.md','docs/requirements.md','docs/roadmap.md'])\nassert 'full layer management' not in combined.lower() or 'not implemented' in combined.lower()\nassert 'Last open project and map restored on startup' not in combined or 'last project path and active map' in combined\nPY`
    Expected: Command exits 0.
    Evidence: .sisyphus/evidence/task-10-doc-claim-check.txt
  ```

  **Commit**: YES | Message: `docs: reconcile feature status with implementation` | Files: [`README.md`, `docs/requirements.md`, `docs/roadmap.md`, `docs/frontend-architecture.md`, `docs/architecture.md`, `docs/commands-reference.md`, `docs/feature-status.md`, `docs/adr/adr-0019-doc-audit-reconciliation.md`]

- [x] 11. Run full verification and final documentation consistency audit

  **What to do**: Run targeted tests from Tasks 2-8, then full `just test-rust`, `just test-ui`, `just clippy`, and `just test`. If a known pre-existing blocker appears, capture exact command output and classify it separately; do not hide new failures. Run a final grep/static audit for direct `invoke()`, stale claims, hardcoded layer workflow calls, and missing ADR/doc references. Update `docs/feature-status.md` only if verification reveals status changes.
  **Must NOT do**: Do not mark plan complete if any new failure remains unexplained. Do not skip clippy warnings.

  **Recommended Agent Profile**:
  - Category: `unspecified-high` - integrated QA and docs audit.
  - Skills: [] - No extra skill required.
  - Omitted: []

  **Parallelization**: Can Parallel: NO | Wave 5 | Blocks: Final Verification | Blocked By: 1-10

  **References**:
  - Commands: `AGENTS.md` command table.
  - Known historical blocker: project memory notes mention a prior `walkers 0.52.0` attribution API mismatch; verify whether still relevant before classifying as pre-existing.
  - All files changed by Tasks 1-10.

  **Acceptance Criteria**:
  - [ ] All task-specific tests pass.
  - [ ] `just test-rust` passes or unrelated pre-existing failure is documented exactly.
  - [ ] `just test-ui` passes.
  - [ ] `just clippy` passes or unrelated pre-existing failure is documented exactly.
  - [ ] `just test` passes or unrelated pre-existing failure is documented exactly.
  - [ ] Final static audit finds no direct component `invoke()` calls and no stale doc claims from the 10 audit findings.

  **QA Scenarios**:
  ```
  Scenario: Full verification pipeline
    Tool: Bash
    Steps: Run `just test-rust`, `just test-ui`, `just clippy`, and `just test`.
    Expected: All commands exit 0, or exact unrelated pre-existing blocker output is saved and classified.
    Evidence: .sisyphus/evidence/task-11-full-verification.txt

  Scenario: Final stale-claim audit
    Tool: Bash
    Steps: Run static grep/python checks for direct invoke, hardcoded layer workflow calls, and feature-status coverage.
    Expected: All checks exit 0.
    Evidence: .sisyphus/evidence/task-11-static-audit.txt
  ```

  **Commit**: YES | Message: `test: verify documentation audit fixes` | Files: [`docs/feature-status.md`, `.sisyphus/evidence/*` only if project convention permits evidence commits; otherwise no commit if only verification output changes]

## Final Verification Wave (MANDATORY — after ALL implementation tasks)
> 4 review agents run in PARALLEL. ALL must APPROVE. Present consolidated results to user and get explicit "okay" before completing.
> **Do NOT auto-proceed after verification. Wait for user's explicit approval before marking work complete.**
> **Never mark F1-F4 as checked before getting user's okay.** Rejection or user feedback -> fix -> re-run -> present again -> wait for okay.
- [x] F1. Plan Compliance Audit — oracle
- [x] F2. Code Quality Review — unspecified-high
- [x] F3. Real Manual QA — unspecified-high (+ playwright only if the executor can run the Tauri UI in browser/webview mode with stable selectors)
- [x] F4. Scope Fidelity Check — deep

## Commit Strategy
- Create separate commits per logical slice, as listed in each task.
- Do not squash by default; user preference is separate thoughtful commits per feature/change.
- Do not commit secrets or local session files. Session persistence tests must use temporary directories.
- If verification-only evidence files are not meant to be committed in this repo, skip the Task 11 commit and report verification evidence paths only.

## Success Criteria
- All 10 audit findings are closed with code, docs, or explicitly documented status correction.
- ADR and docs explain every decision and non-goal.
- User-facing docs no longer overstate backend-only or planned features.
- Tests and QA cover the new behavior and key failure cases.
- Final review wave approves and the user explicitly okays completion.
