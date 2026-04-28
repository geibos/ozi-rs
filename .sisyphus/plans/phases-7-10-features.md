# Phases 7-10: Track Editing, Simplification, Waypoints UI, Export

## TL;DR

> **Quick Summary**: Implement remaining MVP features for ozi-rs — track point editing with drag on map, track simplification (Douglas-Peucker), full waypoint UI with icons, and PLT export. Also fix critical bugs (mutex poison, unsafe indexing, clippy) and migrate undo/redo to delta-based architecture.
>
> **Deliverables**:
> - Bug fixes: safe mutex handling, bounds-checked indexing, clean clippy
> - Delta-based undo/redo with drag-coalescing
> - 9 new ProjectCommand variants (MoveTrackPoint, DeleteTrackPoint, InsertTrackPoint, DeleteTrack, SplitSegment, JoinSegments, DeleteWaypoint, RenameWaypoint, SimplifyTrack)
> - Track point editing UI: map drag + read-only point list panel
> - Track creation from scratch on map (drawing mode)
> - Waypoint UI: panel, click-to-add, drag, symbol picker
> - Douglas-Peucker track simplification with preview
> - PLT export with round-trip fidelity
> - SetTrackLineWidth (non-undoable, direct mutation like color/visibility)
>
> **Estimated Effort**: XL (~35-40 tasks)
> **Parallel Execution**: YES — 8 waves
> **Critical Path**: Wave 0 (bugs) → Wave 1 (delta undo + domain) → Wave 2 (commands) → Wave 3 (endpoints) → Wave 4 (UI) → Wave 5 (map interaction) → Wave 6 (algorithms + export) → Wave 7 (track creation)

---

## Context

### Original Request
Implement all remaining roadmap features (Phases 7-10) for ozi-rs desktop map editor used by LizaAlert SAR volunteers. Include critical bug fixes from audit.

### Interview Summary
**Key Discussions**:
- **Scope**: Full Phases 7-10 + critical bugs + missing commands (DeleteTrack, SplitSegment, JoinSegments, SetTrackLineWidth)
- **Undo/redo architecture**: User chose delta-based undo to handle frequent drag commands scalably
- **UI approach**: Map-based drag editing with read-only point list panel (no inline coordinate editing in list)
- **Track creation from scratch**: Include but in later wave, after editing of existing tracks works
- **Print/PDF**: Deferred — not in this plan
- **Style mutations**: SetTrackLineWidth follows existing non-undoable pattern (like color/visibility)
- **Waypoint icons**: Fixed symbol set (`symbol: String` field) with predefined icons (flag, camp, danger, etc.)
- **Testing**: Tests written together with implementation code, not separate

**Research Findings**:
- 14 mutex `.lock().unwrap()` calls that poison-crash the app
- 11 clippy errors blocking quality gates
- 1 unsafe palette index access in ozi_raster.rs
- Current GeoJSON endpoint flattens segments — need `get_track_detail` for point-level editing
- No `get_waypoints` endpoint exists for frontend
- `ProjectLayerError` missing variants for Track, TrackSegment, TrackPoint

### Metis Review
**Identified Gaps** (addressed):
- **Delta-based undo complexity**: User explicitly chose this over snapshot — plan includes full migration task
- **Drag coalescing needed**: MoveTrackPoint must merge sequential moves of same point
- **Frontend data model too coarse**: Added `get_track_detail` endpoint task before any UI work
- **Missing error variants**: Added error expansion to Wave 1
- **PLT export fidelity**: Added round-trip test requirement
- **Douglas-Peucker preview**: Added preview endpoint before commit pattern
- **Point list scope creep**: Guardrail — flat list, no virtualization, first 1000 points with pagination
- **11 clippy errors**: Must fix in Wave 0 before feature work

---

## Work Objectives

### Core Objective
Complete the MVP feature set for ozi-rs by implementing track editing, track simplification, waypoint management UI, and PLT export — all built on a robust delta-based undo/redo foundation.

### Concrete Deliverables
- All critical bugs fixed (mutex safety, bounds checks, clippy clean)
- Delta-based `CommandStack` replacing snapshot-based approach
- 9 new `ProjectCommand` variants with full undo/redo support
- Track point list panel (read-only, showing segment→point hierarchy)
- Map-based point drag editing
- Track creation drawing mode
- Waypoint panel with add/delete/rename/drag/symbol
- Douglas-Peucker simplification with preview
- PLT export matching OziExplorer format
- SetTrackLineWidth direct mutation

### Definition of Done
- [ ] `cargo test --all` — 0 failures
- [ ] `cargo clippy --all-targets --all-features -- -D warnings` — 0 warnings
- [ ] `npm test` — 0 failures
- [ ] All 9 new commands have domain + command + undo tests
- [ ] All Tauri handlers registered and callable from frontend
- [ ] PLT import→export→import round-trip test passes

### Must Have
- Safe mutex handling everywhere (no `.lock().unwrap()`)
- Delta-based undo/redo with drag coalescing
- Every new command: domain mutation + command wrapper + Tauri handler + frontend API call
- Tests for every command: success case, undo case, error case
- `get_track_detail` endpoint exposing segment→point→ID hierarchy
- `get_waypoints` endpoint for waypoint panel

### Must NOT Have (Guardrails)
- NO "utility" modules, "helper" crates, or abstraction layers
- NO `thiserror`/`anyhow` — follow existing manual `Display` impls
- NO refactoring of existing commands while adding new ones
- NO JSDoc/rustdoc on existing code — only new public APIs
- NO virtualized scrolling, search, or filtering in point list panel
- NO inline coordinate editing in point list — editing happens on map only
- NO separate test files — tests go inline in `#[cfg(test)] mod tests`
- NO DTOs for internal domain→application communication
- NO feature flags or conditional compilation
- NO over-engineered geometry abstractions — Douglas-Peucker is one function on `&[TrackPoint]`
- NO Print/PDF in this plan
- NO new `.lock().unwrap()` calls — all new handlers use safe lock handling
- NO frontend-only mutations bypassing CommandStack (except style: color/visibility/line-width)

---

## Verification Strategy

> **ZERO HUMAN INTERVENTION** — ALL verification is agent-executed. No exceptions.

### Test Decision
- **Infrastructure exists**: YES (83 Rust tests, 7 TS tests)
- **Automated tests**: Tests together with implementation
- **Framework**: `cargo test` for Rust, `npm test` (vitest) for TS
- **Convention**: Inline `#[cfg(test)] mod tests` blocks

### QA Policy
Every task MUST include agent-executed QA scenarios.
Evidence saved to `.sisyphus/evidence/task-{N}-{scenario-slug}.{ext}`.

- **Rust domain/commands**: Use Bash (`cargo test`) — run specific test module, assert 0 failures
- **Tauri handlers**: Use Bash (`cargo test`) — integration tests calling handler functions
- **Frontend components**: Use Bash (`npm test`) — vitest for component tests
- **Map interaction**: Use Playwright — navigate, click, drag, assert DOM/canvas state, screenshot

---

## Execution Strategy

### Parallel Execution Waves

```
Wave 0 (Bug Fixes — prerequisite for all feature work):
├── Task 1: Fix clippy warnings (11 errors) [quick]
├── Task 2: Fix mutex poison — safe lock handling [quick]
├── Task 3: Fix unsafe indexing in ozi_raster.rs + tiles.rs [quick]
└── Task 4: Expand ProjectLayerError variants [quick]

Wave 1 (Foundation — delta undo + domain mutations, MAX PARALLEL):
├── Task 5: Delta-based CommandStack migration [deep]
├── Task 6: TrackSegment point mutations (move/delete/insert/split) [unspecified-high]
├── Task 7: Track mutations (remove_segment, join_segments) [unspecified-high]
├── Task 8: TrackLayer::remove_track + Project::remove_track_from_layer [quick]
├── Task 9: WaypointLayer::remove_waypoint + Waypoint::set_name + symbol field [quick]
└── Task 10: Douglas-Peucker algorithm (pure function) [unspecified-high]

Wave 2 (Commands — all new ProjectCommand variants, MAX PARALLEL):
├── Task 11: MoveTrackPoint command + drag coalescing [deep]
├── Task 12: DeleteTrackPoint command [unspecified-high]
├── Task 13: InsertTrackPoint command [unspecified-high]
├── Task 14: SplitSegment command [unspecified-high]
├── Task 15: JoinSegments command [unspecified-high]
├── Task 16: DeleteTrack command [unspecified-high]
├── Task 17: DeleteWaypoint command [quick]
├── Task 18: RenameWaypoint command [quick]
└── Task 19: SimplifyTrack command (uses Douglas-Peucker) [unspecified-high]

Wave 3 (Tauri Handlers + Endpoints):
├── Task 20: Tauri handlers for track commands (T11-T16) [unspecified-high]
├── Task 21: Tauri handlers for waypoint commands (T17-T18) [quick]
├── Task 22: Tauri handler for SimplifyTrack (T19) [quick]
├── Task 23: get_track_detail endpoint [unspecified-high]
├── Task 24: get_waypoints endpoint [quick]
├── Task 25: get_simplified_preview endpoint [unspecified-high]
└── Task 26: SetTrackLineWidth (direct mutation handler) [quick]

Wave 4 (Frontend — panels + API, MAX PARALLEL):
├── Task 27: Frontend API functions for all new commands [unspecified-high]
├── Task 28: Track point list panel (read-only) [visual-engineering]
├── Task 29: Waypoint panel with list/delete/rename [visual-engineering]
└── Task 30: Waypoint symbol picker component [visual-engineering]

Wave 5 (Map Interaction — drag + click):
├── Task 31: Map track point drag editing [deep]
├── Task 32: Map click-to-add waypoint [unspecified-high]
├── Task 33: Map waypoint drag [unspecified-high]
└── Task 34: Simplification preview overlay + confirm UI [visual-engineering]

Wave 6 (Export):
├── Task 35: PLT export with round-trip tests [deep]
└── Task 36: PLT export Tauri handler + frontend integration [unspecified-high]

Wave 7 (Track Creation — after editing works):
├── Task 37: CreateEmptyTrack command + domain [unspecified-high]
└── Task 38: Map drawing mode for new tracks [deep]

Wave FINAL (After ALL tasks — 4 parallel reviews, then user okay):
├── Task F1: Plan compliance audit (oracle)
├── Task F2: Code quality review (unspecified-high)
├── Task F3: Real manual QA (unspecified-high)
└── Task F4: Scope fidelity check (deep)
-> Present results -> Get explicit user okay

Critical Path: T1-4 → T5 → T11 → T20 → T23 → T28 → T31 → F1-F4 → user okay
Parallel Speedup: ~65% faster than sequential
Max Concurrent: 9 (Wave 2)
```

### Dependency Matrix

| Task | Depends On | Blocks | Wave |
|------|-----------|--------|------|
| 1-4 | — | 5-10 | 0 |
| 5 | 1-4 | 11-19 | 1 |
| 6 | 1-4 | 11, 12, 13, 14 | 1 |
| 7 | 1-4 | 15, 16 | 1 |
| 8 | 1-4 | 16 | 1 |
| 9 | 1-4 | 17, 18 | 1 |
| 10 | 1-4 | 19 | 1 |
| 11 | 5, 6 | 20 | 2 |
| 12 | 5, 6 | 20 | 2 |
| 13 | 5, 6 | 20 | 2 |
| 14 | 5, 6 | 20 | 2 |
| 15 | 5, 7 | 20 | 2 |
| 16 | 5, 7, 8 | 20 | 2 |
| 17 | 5, 9 | 21 | 2 |
| 18 | 5, 9 | 21 | 2 |
| 19 | 5, 10 | 22, 25 | 2 |
| 20 | 11-16 | 23, 27 | 3 |
| 21 | 17, 18 | 24, 27 | 3 |
| 22 | 19 | 25, 27 | 3 |
| 23 | 20 | 28, 31 | 3 |
| 24 | 21 | 29 | 3 |
| 25 | 22 | 34 | 3 |
| 26 | 1-4 | 27 | 3 |
| 27 | 20-26 | 28-34 | 4 |
| 28 | 23, 27 | 31 | 4 |
| 29 | 24, 27 | 32, 33 | 4 |
| 30 | 29 | — | 4 |
| 31 | 28 | — | 5 |
| 32 | 29 | — | 5 |
| 33 | 29 | — | 5 |
| 34 | 25, 27 | — | 5 |
| 35 | 1-4 | 36 | 6 |
| 36 | 35 | — | 6 |
| 37 | 5 | 38 | 7 |
| 38 | 37, 31 | — | 7 |
| F1-F4 | ALL | — | FINAL |

### Agent Dispatch Summary

- **Wave 0**: **4** — T1-T4 → `quick`
- **Wave 1**: **6** — T5 → `deep`, T6-T7 → `unspecified-high`, T8-T9 → `quick`, T10 → `unspecified-high`
- **Wave 2**: **9** — T11 → `deep`, T12-T16 → `unspecified-high`, T17-T18 → `quick`, T19 → `unspecified-high`
- **Wave 3**: **7** — T20 → `unspecified-high`, T21-T22 → `quick`, T23 → `unspecified-high`, T24 → `quick`, T25 → `unspecified-high`, T26 → `quick`
- **Wave 4**: **4** — T27 → `unspecified-high`, T28-T30 → `visual-engineering`
- **Wave 5**: **4** — T31 → `deep`, T32-T33 → `unspecified-high`, T34 → `visual-engineering`
- **Wave 6**: **2** — T35 → `deep`, T36 → `unspecified-high`
- **Wave 7**: **2** — T37 → `unspecified-high`, T38 → `deep`
- **FINAL**: **4** — F1 → `oracle`, F2 → `unspecified-high`, F3 → `unspecified-high`, F4 → `deep`

---

## TODOs

- [x] 1. Fix clippy warnings (11 errors)

  **What to do**:
  - Run `cargo clippy --all-targets --all-features -- -D warnings` to identify all 11 errors
  - Fix each: `enum_variant_names` (rename variants), `double_ended_iterator_last` (use `.next_back()`), `write_with_newline` (use `writeln!`), let-chain syntax adjustments
  - Ensure zero clippy warnings after fix

  **Must NOT do**:
  - Do NOT refactor surrounding code — only fix the clippy diagnostic
  - Do NOT add `#[allow(...)]` suppressions — fix the actual issue

  **Recommended Agent Profile**:
  - **Category**: `quick`
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: YES
  - **Parallel Group**: Wave 0 (with Tasks 2, 3, 4)
  - **Blocks**: Tasks 5-10
  - **Blocked By**: None

  **References**:

  **Pattern References**:
  - `src-tauri/src/domain/project.rs` — has `#![allow(dead_code)]` at module level, contains enum_variant_names issues
  - `src-tauri/src/application/commands.rs` — has enum_variant_names and let-chain warnings
  - `src-tauri/src/infrastructure/` — write_with_newline issues in export modules

  **Acceptance Criteria**:
  - [ ] `cargo clippy --all-targets --all-features -- -D warnings` exits with code 0
  - [ ] `cargo test --all` still passes (0 failures)

  **QA Scenarios**:

  ```
  Scenario: Clippy clean pass
    Tool: Bash
    Preconditions: All clippy fixes applied
    Steps:
      1. Run `cargo clippy --all-targets --all-features -- -D warnings 2>&1`
      2. Assert exit code is 0
      3. Assert output does NOT contain "error" or "warning"
    Expected Result: Zero clippy diagnostics
    Failure Indicators: Any line starting with "error[" or "warning:"
    Evidence: .sisyphus/evidence/task-1-clippy-clean.txt

  Scenario: Tests still pass after clippy fixes
    Tool: Bash
    Preconditions: Clippy fixes applied
    Steps:
      1. Run `cargo test --all 2>&1`
      2. Assert output contains "test result: ok"
      3. Assert output contains "0 failed"
    Expected Result: All existing 83 tests pass
    Failure Indicators: "FAILED" or "test result: FAILED"
    Evidence: .sisyphus/evidence/task-1-tests-pass.txt
  ```

  **Commit**: YES
  - Message: `fix: resolve all clippy warnings`
  - Files: multiple source files
  - Pre-commit: `cargo clippy --all-targets --all-features -- -D warnings && cargo test --all`

- [x] 2. Fix mutex poison — safe lock handling

  **What to do**:
  - Replace all 14 instances of `state.lock().unwrap()` in `src-tauri/src/commands/mod.rs` with safe handling
  - Pattern: `state.lock().map_err(|e| format!("State lock poisoned: {}", e))?` for Tauri commands returning `Result`
  - Also check `lib.rs` and any other files for `.lock().unwrap()` patterns
  - Add a helper function or macro if repetition warrants it (e.g., `fn safe_lock(state) -> Result<MutexGuard, String>`)

  **Must NOT do**:
  - Do NOT switch to `parking_lot::Mutex` — keep `std::sync::Mutex`, just handle poison safely
  - Do NOT change the `Arc<Mutex<AppState>>` architecture
  - Do NOT add error handling middleware or custom error types

  **Recommended Agent Profile**:
  - **Category**: `quick`
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: YES
  - **Parallel Group**: Wave 0 (with Tasks 1, 3, 4)
  - **Blocks**: Tasks 5-10
  - **Blocked By**: None

  **References**:

  **Pattern References**:
  - `src-tauri/src/commands/mod.rs:81,150,199,207,214,228,245,253,272,298,383,389,446` — all `.lock().unwrap()` locations
  - `src-tauri/src/lib.rs` — Tauri setup, may have additional `.lock().unwrap()`

  **API/Type References**:
  - All Tauri commands return `Result<T, String>` — the `?` operator works with `map_err`

  **Acceptance Criteria**:
  - [ ] Zero `.lock().unwrap()` calls in entire codebase (grep verification)
  - [ ] All Tauri commands handle mutex poison gracefully with error message
  - [ ] `cargo test --all` passes

  **QA Scenarios**:

  ```
  Scenario: No unwrap on lock in codebase
    Tool: Bash
    Preconditions: All fixes applied
    Steps:
      1. Run `grep -rn '\.lock()\.unwrap()' src-tauri/src/ 2>&1`
      2. Assert output is empty (no matches)
    Expected Result: Zero matches for `.lock().unwrap()`
    Failure Indicators: Any line showing a match
    Evidence: .sisyphus/evidence/task-2-no-unwrap.txt

  Scenario: Tests still pass after lock fix
    Tool: Bash
    Preconditions: Lock handling migrated
    Steps:
      1. Run `cargo test --all 2>&1`
      2. Assert "test result: ok" and "0 failed"
    Expected Result: All tests pass
    Failure Indicators: "FAILED"
    Evidence: .sisyphus/evidence/task-2-tests-pass.txt
  ```

  **Commit**: YES
  - Message: `fix(commands): replace mutex unwrap with safe lock handling`
  - Files: `src-tauri/src/commands/mod.rs`, possibly `lib.rs`
  - Pre-commit: `cargo test --all`

- [x] 3. Fix unsafe indexing in ozi_raster.rs and tiles.rs

  **What to do**:
  - `src-tauri/src/infrastructure/import/ozi_raster.rs:358` — `tile.palette()[*palette_index as usize]` — add bounds check, return error or skip pixel if out of range
  - `src-tauri/src/commands/tiles.rs:212-215` — float-to-usize cast edge cases in tile reprojection — add `.min()` clamping to prevent out-of-bounds
  - `src-tauri/src/commands/tiles.rs:303` — `out_rgba[out_off..out_off+4]` — add bounds check before slice indexing

  **Must NOT do**:
  - Do NOT refactor the tile reprojection algorithm
  - Do NOT change the overall rendering pipeline

  **Recommended Agent Profile**:
  - **Category**: `quick`
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: YES
  - **Parallel Group**: Wave 0 (with Tasks 1, 2, 4)
  - **Blocks**: Tasks 5-10
  - **Blocked By**: None

  **References**:

  **Pattern References**:
  - `src-tauri/src/infrastructure/import/ozi_raster.rs:358` — palette index access
  - `src-tauri/src/commands/tiles.rs:212-215` — float-to-usize level selection
  - `src-tauri/src/commands/tiles.rs:303` — RGBA slice indexing

  **Acceptance Criteria**:
  - [ ] All three unsafe index sites have bounds checks
  - [ ] `cargo test --all` passes
  - [ ] Existing tile tests (`tiles.rs` has 8 tests) still pass

  **QA Scenarios**:

  ```
  Scenario: Bounds-checked indexing
    Tool: Bash
    Preconditions: Fixes applied
    Steps:
      1. Run `cargo test --all 2>&1`
      2. Verify tile-specific tests pass: grep for "test commands::tiles" in output
      3. Assert "0 failed"
    Expected Result: All tests pass including tile tests
    Failure Indicators: Any tile test failure
    Evidence: .sisyphus/evidence/task-3-bounds-check.txt

  Scenario: No unchecked indexing in affected files
    Tool: Bash
    Preconditions: Fixes applied
    Steps:
      1. Review the three specific locations to confirm bounds checks are present
      2. Run `cargo clippy --all-targets --all-features 2>&1` — no new warnings
    Expected Result: All three sites have explicit bounds checking or clamping
    Failure Indicators: Direct array indexing without `.get()` or `.min()` clamping
    Evidence: .sisyphus/evidence/task-3-review.txt
  ```

  **Commit**: YES
  - Message: `fix: add bounds checks for unsafe indexing in tiles and ozi_raster`
  - Files: `src-tauri/src/commands/tiles.rs`, `src-tauri/src/infrastructure/import/ozi_raster.rs`
  - Pre-commit: `cargo test --all`

- [x] 4. Expand ProjectLayerError variants

  **What to do**:
  - Add missing error variants to `ProjectLayerError` in `src-tauri/src/domain/project.rs`:
    - `MissingTrack { layer_id: u64, track_id: u64 }`
    - `MissingTrackSegment { layer_id: u64, track_id: u64, segment_id: u64 }`
    - `MissingTrackPoint { layer_id: u64, track_id: u64, segment_id: u64, point_id: u64 }`
  - Implement `Display` for new variants following existing pattern
  - Add `track_layer_mut()` / `track_mut()` convenience methods on `Project` if not already present (returning `Result<&mut T, ProjectLayerError>`)

  **Must NOT do**:
  - Do NOT use `thiserror` — follow existing manual `Display` impl
  - Do NOT change existing error variants

  **Recommended Agent Profile**:
  - **Category**: `quick`
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: YES
  - **Parallel Group**: Wave 0 (with Tasks 1, 2, 3)
  - **Blocks**: Tasks 5-10
  - **Blocked By**: None

  **References**:

  **Pattern References**:
  - `src-tauri/src/domain/project.rs` — existing `ProjectLayerError` enum with `MissingTrackLayer`, `MissingWaypointLayer`, `MissingWaypoint` variants and manual `Display` impl
  - `src-tauri/src/domain/project.rs:276-296` — `move_waypoint_in_layer()` pattern for layer lookup + error return

  **Acceptance Criteria**:
  - [ ] `ProjectLayerError` has variants: MissingTrack, MissingTrackSegment, MissingTrackPoint
  - [ ] `Display` impl covers all new variants
  - [ ] Convenience methods exist for `track_layer_mut()` and nested lookup
  - [ ] `cargo test --all` passes

  **QA Scenarios**:

  ```
  Scenario: New error variants compile and display correctly
    Tool: Bash
    Preconditions: Error expansion applied
    Steps:
      1. Run `cargo test --all 2>&1`
      2. Assert "0 failed"
      3. Run `cargo build 2>&1` — assert clean compilation
    Expected Result: All code compiles and tests pass
    Failure Indicators: Compilation errors or test failures
    Evidence: .sisyphus/evidence/task-4-error-variants.txt

  Scenario: Error display format is consistent
    Tool: Bash
    Preconditions: New variants added
    Steps:
      1. Write a unit test in the `#[cfg(test)]` block that creates each new error variant
      2. Assert `.to_string()` output matches expected format (e.g., "Missing track 5 in layer 1")
    Expected Result: Display output is human-readable and consistent with existing variants
    Failure Indicators: Display output doesn't match expected format
    Evidence: .sisyphus/evidence/task-4-error-display.txt
  ```

  **Commit**: YES
  - Message: `feat(domain): expand ProjectLayerError with Track/Segment/Point variants`
  - Files: `src-tauri/src/domain/project.rs`
  - Pre-commit: `cargo test --all`

- [x] 5. Delta-based CommandStack migration

  **What to do**:
  - Replace snapshot-based undo/redo in `application/commands.rs` with delta-based approach
  - Each `ProjectCommand` must define a **reverse command** — e.g., `MoveWaypoint(id, new_pos)` stores `MoveWaypoint(id, old_pos)` as its inverse
  - `CommandStack` stores `(forward_command, reverse_command)` pairs instead of full `Project` clones
  - `undo()` applies the reverse command; `redo()` re-applies the forward command
  - Add `apply_or_merge()` method for drag coalescing: if the last command targets the same entity (same point/waypoint ID), replace the last entry's forward command but keep original reverse command (so undo returns to pre-drag state)
  - Limit stack depth to 100 entries (configurable constant)
  - Migrate ALL existing 8 commands to produce reverse commands
  - Ensure all existing command tests pass with new architecture

  **Must NOT do**:
  - Do NOT change the `ProjectCommand` enum's public API shape — just add `fn reverse(&self, project: &Project) -> ProjectCommand` capability
  - Do NOT add serde/persistence for undo history — in-memory only
  - Do NOT add redo branching (discard redo stack on new command, like current behavior)

  **Recommended Agent Profile**:
  - **Category**: `deep`
    - Reason: Core architectural change affecting all commands, requires careful reasoning about invariants
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: YES (with Tasks 6-10)
  - **Parallel Group**: Wave 1
  - **Blocks**: Tasks 11-19 (all new commands depend on new CommandStack)
  - **Blocked By**: Tasks 1-4 (Wave 0)

  **References**:

  **Pattern References**:
  - `src-tauri/src/application/commands.rs:1-379` — entire file: current `CommandStack` with `snapshots: Vec<Project>`, `apply()` clones project before executing
  - `src-tauri/src/application/commands.rs:47-52` — `MoveWaypoint` constructor pattern (stores old + new position)
  - `src-tauri/src/application/commands.rs:68-135` — `apply()` match arms for all 8 commands

  **API/Type References**:
  - `src-tauri/src/domain/project.rs` — `Project` struct that's currently cloned

  **Test References**:
  - `src-tauri/src/application/commands.rs` — existing `#[cfg(test)] mod tests` with command tests (currently 5 tests)

  **Acceptance Criteria**:
  - [ ] `CommandStack` no longer clones `Project` on apply
  - [ ] Each command stores `(forward, reverse)` pair
  - [ ] `undo()` applies reverse command correctly
  - [ ] `redo()` re-applies forward command
  - [ ] `apply_or_merge()` coalesces sequential same-entity commands
  - [ ] Stack depth limited to 100 entries
  - [ ] All existing 5 command tests pass
  - [ ] New tests for: undo correctness, redo correctness, merge behavior, stack depth limit

  **QA Scenarios**:

  ```
  Scenario: Undo restores previous state
    Tool: Bash
    Preconditions: Delta-based CommandStack implemented
    Steps:
      1. Run `cargo test --all -q 2>&1 | grep "test result"`
      2. Assert "0 failed"
      3. Specifically verify undo test: `cargo test delta_undo 2>&1`
    Expected Result: Undo correctly restores state to before command was applied
    Failure Indicators: State mismatch after undo
    Evidence: .sisyphus/evidence/task-5-delta-undo.txt

  Scenario: Drag coalescing merges sequential moves
    Tool: Bash
    Preconditions: apply_or_merge implemented
    Steps:
      1. Run `cargo test coalesce 2>&1`
      2. Verify test exists that: applies MoveWaypoint(id, pos1), then apply_or_merge MoveWaypoint(id, pos2) — stack has 1 entry, undo returns to original position
    Expected Result: Only one undo step for multiple drag moves of same entity
    Failure Indicators: Stack grows per drag move or undo doesn't reach original position
    Evidence: .sisyphus/evidence/task-5-coalesce.txt
  ```

  **Commit**: YES
  - Message: `refactor(commands): migrate to delta-based undo/redo with drag coalescing`
  - Files: `src-tauri/src/application/commands.rs`
  - Pre-commit: `cargo test --all`

- [x] 6. TrackSegment point mutations (move/delete/insert/split)

  **What to do**:
  - Add to `TrackSegment` in `src-tauri/src/domain/track.rs`:
    - `move_point(&mut self, point_id: u64, lat: f64, lon: f64) -> Result<(f64, f64), ProjectLayerError>` — returns old (lat, lon) for reverse command
    - `remove_point(&mut self, point_id: u64) -> Result<(usize, TrackPoint), ProjectLayerError>` — returns (index, removed point) for undo re-insert
    - `insert_point_at(&mut self, index: usize, point: TrackPoint) -> Result<(), ProjectLayerError>` — for undo of remove and manual insert
    - `split_at_point(&mut self, point_id: u64) -> Result<TrackSegment, ProjectLayerError>` — splits segment, returns new right segment. Left segment keeps original ID + points up to split point (inclusive). New segment gets new ID with points from split point onward.
    - `points_mut(&mut self) -> &mut Vec<TrackPoint>` — mutable access for batch operations
  - Add to `Project` corresponding layer-traversal methods:
    - `move_point_in_layer(layer_id, track_id, segment_id, point_id, lat, lon)`
    - `remove_point_from_layer(...)`
    - `insert_point_in_layer(...)`
    - `split_segment_in_layer(...)`
  - Write unit tests for each mutation: success case + error case (missing entity)

  **Must NOT do**:
  - Do NOT change `TrackPoint` struct — it stays as-is
  - Do NOT add generic geometry abstractions
  - Do NOT modify existing `add_point()` method

  **Recommended Agent Profile**:
  - **Category**: `unspecified-high`
    - Reason: Multiple domain methods with careful error handling and ID semantics
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: YES (with Tasks 5, 7, 8, 9, 10)
  - **Parallel Group**: Wave 1
  - **Blocks**: Tasks 11, 12, 13, 14
  - **Blocked By**: Tasks 1-4

  **References**:

  **Pattern References**:
  - `src-tauri/src/domain/track.rs:1-280` — `Track`, `TrackSegment`, `TrackPoint` structs
  - `src-tauri/src/domain/track.rs` — existing `add_point()`, `add_segment()` patterns
  - `src-tauri/src/domain/project.rs:276-296` — `move_waypoint_in_layer()` pattern for layer traversal + error return

  **API/Type References**:
  - `src-tauri/src/domain/project.rs` — `ProjectLayerError` with new MissingTrack/Segment/Point variants (from Task 4)

  **Test References**:
  - `src-tauri/src/domain/track.rs` — existing `#[cfg(test)] mod tests` with 16 domain tests

  **Acceptance Criteria**:
  - [ ] `TrackSegment` has `move_point`, `remove_point`, `insert_point_at`, `split_at_point`, `points_mut`
  - [ ] `Project` has layer-traversal methods for each mutation
  - [ ] Each method returns `Result` with appropriate error
  - [ ] `split_at_point` keeps left segment with original ID, creates new right segment
  - [ ] Unit tests: 4 success cases + 4 error cases (minimum 8 new tests)
  - [ ] `cargo test --all` passes

  **QA Scenarios**:

  ```
  Scenario: Point mutations work correctly
    Tool: Bash
    Preconditions: All new methods implemented with tests
    Steps:
      1. Run `cargo test domain::track 2>&1`
      2. Assert all new tests pass
      3. Verify at least 8 new tests exist (4 success + 4 error)
    Expected Result: All point mutation tests pass
    Failure Indicators: Any test failure or missing test
    Evidence: .sisyphus/evidence/task-6-point-mutations.txt

  Scenario: Split semantics are correct
    Tool: Bash
    Preconditions: split_at_point implemented
    Steps:
      1. Run `cargo test split 2>&1`
      2. Verify: left segment has original ID, right segment has new ID
      3. Verify: split point is included in BOTH segments (shared boundary)
    Expected Result: Split produces two valid segments with correct point distribution
    Failure Indicators: Points missing, wrong IDs, or split point not shared
    Evidence: .sisyphus/evidence/task-6-split.txt
  ```

  **Commit**: YES
  - Message: `feat(domain): add TrackSegment point mutations (move/delete/insert/split)`
  - Files: `src-tauri/src/domain/track.rs`, `src-tauri/src/domain/project.rs`
  - Pre-commit: `cargo test --all`

- [x] 7. Track mutations (remove_segment, join_segments)

  **What to do**:
  - Add to `Track` in `src-tauri/src/domain/track.rs`:
    - `remove_segment(&mut self, segment_id: u64) -> Result<(usize, TrackSegment), ProjectLayerError>` — returns (index, removed segment) for undo
    - `join_segments(&mut self, seg_id_a: u64, seg_id_b: u64) -> Result<TrackSegment, ProjectLayerError>` — merges B's points into A, returns removed segment B for undo. Segments must be adjacent (B follows A) or return error.
  - Add to `Project`:
    - `remove_segment_from_layer(layer_id, track_id, segment_id)`
    - `join_segments_in_layer(layer_id, track_id, seg_id_a, seg_id_b)`
  - Write tests for success + error cases

  **Must NOT do**:
  - Do NOT reorder segments — join only works on adjacent segments
  - Do NOT remove existing segment methods

  **Recommended Agent Profile**:
  - **Category**: `unspecified-high`
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: YES (with Tasks 5, 6, 8, 9, 10)
  - **Parallel Group**: Wave 1
  - **Blocks**: Tasks 15, 16
  - **Blocked By**: Tasks 1-4

  **References**:

  **Pattern References**:
  - `src-tauri/src/domain/track.rs` — `Track` struct, `segments()`, `add_segment()`
  - `src-tauri/src/domain/project.rs:276-296` — layer traversal pattern

  **Acceptance Criteria**:
  - [ ] `Track` has `remove_segment` and `join_segments`
  - [ ] Join validates adjacency (B directly follows A)
  - [ ] Join merges B's points into A, removes B
  - [ ] Returns enough data for undo (removed segment + index)
  - [ ] Tests: 2 success + 2 error cases minimum
  - [ ] `cargo test --all` passes

  **QA Scenarios**:

  ```
  Scenario: Join merges adjacent segments
    Tool: Bash
    Steps:
      1. Run `cargo test join_segments 2>&1`
      2. Assert: segment A has combined points, segment B is consumed
      3. Assert: non-adjacent segments return error
    Expected Result: Join works for adjacent, errors for non-adjacent
    Evidence: .sisyphus/evidence/task-7-join.txt

  Scenario: Remove segment returns undo data
    Tool: Bash
    Steps:
      1. Run `cargo test remove_segment 2>&1`
      2. Assert: returns (index, full TrackSegment) for reinsertion
    Expected Result: Removed segment can be re-inserted at same index
    Evidence: .sisyphus/evidence/task-7-remove.txt
  ```

  **Commit**: YES
  - Message: `feat(domain): add Track segment mutations (remove/join)`
  - Files: `src-tauri/src/domain/track.rs`, `src-tauri/src/domain/project.rs`
  - Pre-commit: `cargo test --all`

- [x] 8. TrackLayer::remove_track + Project::remove_track_from_layer

  **What to do**:
  - Add `TrackLayer::remove_track(&mut self, track_id: u64) -> Result<(usize, Track), ProjectLayerError>`
  - Add `Project::remove_track_from_layer(&mut self, layer_id: u64, track_id: u64) -> Result<(usize, Track), ProjectLayerError>`
  - Returns (index, removed Track) for undo reinsertion
  - Write tests

  **Must NOT do**:
  - Do NOT cascade delete — just remove from the layer's track list

  **Recommended Agent Profile**:
  - **Category**: `quick`
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: YES (with Tasks 5, 6, 7, 9, 10)
  - **Parallel Group**: Wave 1
  - **Blocks**: Task 16
  - **Blocked By**: Tasks 1-4

  **References**:

  **Pattern References**:
  - `src-tauri/src/domain/project.rs` — `TrackLayer` struct, `add_track()` method
  - `src-tauri/src/domain/project.rs:276-296` — layer traversal pattern

  **Acceptance Criteria**:
  - [ ] `remove_track` removes track and returns it with index
  - [ ] Error on missing track_id
  - [ ] Tests: 1 success + 1 error case
  - [ ] `cargo test --all` passes

  **QA Scenarios**:

  ```
  Scenario: Remove and restore track
    Tool: Bash
    Steps:
      1. Run `cargo test remove_track 2>&1`
      2. Assert: track removed, returned data allows reinsertion at same index
    Expected Result: Track cleanly removed and recoverable
    Evidence: .sisyphus/evidence/task-8-remove-track.txt
  ```

  **Commit**: YES
  - Message: `feat(domain): add TrackLayer::remove_track`
  - Files: `src-tauri/src/domain/project.rs`
  - Pre-commit: `cargo test --all`

 - [x] 9. WaypointLayer::remove_waypoint + Waypoint::set_name + symbol field

  **What to do**:
  - Add `WaypointLayer::remove_waypoint(&mut self, waypoint_id: u64) -> Result<(usize, Waypoint), ProjectLayerError>`
  - Add `Waypoint::set_name(&mut self, name: String) -> String` — returns old name for undo
  - Add `symbol: Option<String>` field to `Waypoint` struct (default `None`)
  - Add `Waypoint::set_symbol(&mut self, symbol: Option<String>) -> Option<String>` — returns old symbol
  - Add `Project::remove_waypoint_from_layer(layer_id, waypoint_id)`
  - Add `Project::rename_waypoint_in_layer(layer_id, waypoint_id, new_name)`
  - Add `Project::set_waypoint_symbol_in_layer(layer_id, waypoint_id, symbol)`
  - Update `Waypoint::new()` to accept optional symbol parameter
  - Ensure GPX import/export handles the symbol field (GPX `<sym>` element)
  - Write tests for each new method

  **Must NOT do**:
  - Do NOT add icon image loading — just the string identifier
  - Do NOT add symbol validation — any string is accepted
  - Do NOT change WaypointLayer structure beyond adding remove method

  **Recommended Agent Profile**:
  - **Category**: `quick`
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: YES (with Tasks 5, 6, 7, 8, 10)
  - **Parallel Group**: Wave 1
  - **Blocks**: Tasks 17, 18
  - **Blocked By**: Tasks 1-4

  **References**:

  **Pattern References**:
  - `src-tauri/src/domain/waypoint.rs:1-76` — `Waypoint` struct with `id, name, lat, lon, move_to()`
  - `src-tauri/src/domain/project.rs` — `WaypointLayer`, `move_waypoint_in_layer()` pattern
  - `src-tauri/src/infrastructure/export/gpx.rs` — GPX export, need to add `<sym>` element
  - `src-tauri/src/infrastructure/import/gpx.rs` — GPX import, need to read `<sym>` element

  **Acceptance Criteria**:
  - [ ] `Waypoint` has `symbol: Option<String>` field
  - [ ] `set_name` and `set_symbol` return old values
  - [ ] `remove_waypoint` returns (index, Waypoint) for undo
  - [ ] GPX export writes `<sym>` when symbol is set
  - [ ] GPX import reads `<sym>` into symbol field
  - [ ] Tests: 3 success + 2 error cases minimum
  - [ ] `cargo test --all` passes

  **QA Scenarios**:

  ```
  Scenario: Waypoint symbol survives GPX round-trip
    Tool: Bash
    Steps:
      1. Run `cargo test waypoint 2>&1`
      2. Assert tests for set_name, set_symbol, remove_waypoint pass
      3. Run `cargo test gpx 2>&1` — verify symbol in GPX output
    Expected Result: Symbol field persists through create → export → import cycle
    Evidence: .sisyphus/evidence/task-9-waypoint-symbol.txt

  Scenario: Remove waypoint returns undo data
    Tool: Bash
    Steps:
      1. Run `cargo test remove_waypoint 2>&1`
      2. Assert: returns (index, full Waypoint including name and symbol)
    Expected Result: Removed waypoint fully recoverable
    Evidence: .sisyphus/evidence/task-9-remove.txt
  ```

  **Commit**: YES
  - Message: `feat(domain): add waypoint remove/rename/symbol support`
  - Files: `src-tauri/src/domain/waypoint.rs`, `src-tauri/src/domain/project.rs`, `src-tauri/src/infrastructure/export/gpx.rs`, `src-tauri/src/infrastructure/import/gpx.rs`
  - Pre-commit: `cargo test --all`

 - [x] 10. Douglas-Peucker algorithm (pure function)

  **What to do**:
  - Implement `simplify_track_points(points: &[TrackPoint], tolerance: f64) -> Vec<usize>` in `src-tauri/src/domain/track.rs`
  - Returns indices of points to KEEP (not remove) — this allows the caller to construct the simplified track
  - Use haversine distance for perpendicular distance calculation (haversine already exists in track.rs)
  - Tolerance is in meters
  - Edge cases: 0-1 points → return all indices; 2 points → return both; tolerance ≤ 0 → return all
  - This is a pure function — no mutation, no side effects
  - Write comprehensive tests with known geometric inputs

  **Must NOT do**:
  - Do NOT create a separate module/file for this — it goes in track.rs
  - Do NOT add generic geometry abstractions — it operates on `&[TrackPoint]`
  - Do NOT mutate anything — pure function returning indices

  **Recommended Agent Profile**:
  - **Category**: `unspecified-high`
    - Reason: Algorithm implementation requiring geometric reasoning and careful edge cases
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: YES (with Tasks 5, 6, 7, 8, 9)
  - **Parallel Group**: Wave 1
  - **Blocks**: Task 19
  - **Blocked By**: Tasks 1-4

  **References**:

  **Pattern References**:
  - `src-tauri/src/domain/track.rs` — existing `haversine_distance()` function
  - `src-tauri/src/domain/track.rs` — `TrackPoint` struct with `lat, lon`

  **External References**:
  - Douglas-Peucker algorithm: https://en.wikipedia.org/wiki/Ramer%E2%80%93Douglas%E2%80%93Peucker_algorithm

  **Acceptance Criteria**:
  - [ ] `simplify_track_points` returns correct indices for known inputs
  - [ ] Uses haversine distance (not Euclidean) for accuracy on Earth coordinates
  - [ ] Edge cases handled: 0, 1, 2 points; tolerance ≤ 0
  - [ ] Tests with known geometric inputs (e.g., 5-point straight line simplified to 2; L-shape simplified to 3)
  - [ ] At least 5 test cases
  - [ ] `cargo test --all` passes

  **QA Scenarios**:

  ```
  Scenario: Known geometric simplification
    Tool: Bash
    Steps:
      1. Run `cargo test simplify 2>&1`
      2. Verify test case: 5 collinear points → simplified to 2 (first and last)
      3. Verify test case: L-shaped 5 points → simplified to 3 (start, corner, end)
      4. Verify test case: tolerance=0 → all points kept
    Expected Result: Algorithm produces geometrically correct results
    Failure Indicators: Wrong indices returned or edge cases crash
    Evidence: .sisyphus/evidence/task-10-douglas-peucker.txt

  Scenario: Edge cases don't crash
    Tool: Bash
    Steps:
      1. Run `cargo test simplify_edge 2>&1`
      2. Test: empty slice, single point, two points, negative tolerance
    Expected Result: All edge cases return valid indices without panicking
    Evidence: .sisyphus/evidence/task-10-edge-cases.txt
  ```

  **Commit**: YES
  - Message: `feat(domain): implement Douglas-Peucker track simplification`
  - Files: `src-tauri/src/domain/track.rs`
  - Pre-commit: `cargo test --all`

 - [x] 11. MoveTrackPoint command + drag coalescing

  **What to do**:
  - Add `ProjectCommand::MoveTrackPoint { layer_id, track_id, segment_id, point_id, lat, lon, old_lat, old_lon }` variant
  - Constructor: `fn move_track_point(layer_id, track_id, segment_id, point_id, lat, lon, old_lat, old_lon)`
  - `apply()`: calls `project.move_point_in_layer(...)`, reverse is `MoveTrackPoint` with old/new swapped
  - Mark as **mergeable** for drag coalescing — `CommandStack::apply_or_merge` should detect same (layer_id, track_id, segment_id, point_id) and merge
  - Tests: apply, undo, redo, merge (apply two sequential moves of same point → one undo step)

  **Must NOT do**:
  - Do NOT validate coordinates (any f64 lat/lon is accepted)

  **Recommended Agent Profile**:
  - **Category**: `deep`
    - Reason: First command using the new delta-based architecture + merge logic
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: YES (with Tasks 12-19)
  - **Parallel Group**: Wave 2
  - **Blocks**: Task 20
  - **Blocked By**: Tasks 5, 6

  **References**:

  **Pattern References**:
  - `src-tauri/src/application/commands.rs:47-52` — `MoveWaypoint` pattern (closest analog)
  - `src-tauri/src/application/commands.rs:68-135` — `apply()` match arms

  **Acceptance Criteria**:
  - [ ] Command variant exists with all fields
  - [ ] `apply()` moves point and produces correct reverse
  - [ ] Undo restores original position
  - [ ] Two sequential moves of same point merge into one undo step
  - [ ] Tests: apply, undo, merge (3 tests minimum)

  **QA Scenarios**:

  ```
  Scenario: Move + undo restores position
    Tool: Bash
    Steps:
      1. Run `cargo test move_track_point 2>&1`
      2. Assert: point at (55.0, 37.0), move to (56.0, 38.0), undo → back at (55.0, 37.0)
    Expected Result: Position fully restored on undo
    Evidence: .sisyphus/evidence/task-11-move-undo.txt

  Scenario: Drag coalescing merges moves
    Tool: Bash
    Steps:
      1. Run `cargo test coalesce_move_point 2>&1`
      2. Assert: move to A, merge-move to B, merge-move to C → stack has 1 entry → undo returns to original
    Expected Result: One undo step regardless of drag count
    Evidence: .sisyphus/evidence/task-11-coalesce.txt
  ```

  **Commit**: YES
  - Message: `feat(commands): add MoveTrackPoint with drag coalescing`
  - Files: `src-tauri/src/application/commands.rs`
  - Pre-commit: `cargo test --all`

 - [x] 12. DeleteTrackPoint command

  **What to do**:
  - Add `ProjectCommand::DeleteTrackPoint { layer_id, track_id, segment_id, point_id, removed_index, removed_point }` variant
  - Constructor only takes IDs; `removed_index` and `removed_point` are populated during `apply()` from the return value of `remove_point()`
  - Reverse command: `InsertTrackPoint` at the same index with the same point data
  - Tests: apply (point removed), undo (point re-inserted at same position), error (missing point)

  **Recommended Agent Profile**:
  - **Category**: `unspecified-high`
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: YES (with Tasks 11, 13-19)
  - **Parallel Group**: Wave 2
  - **Blocks**: Task 20
  - **Blocked By**: Tasks 5, 6

  **References**:

  **Pattern References**:
  - `src-tauri/src/application/commands.rs` — existing command patterns
  - Task 6 domain methods: `TrackSegment::remove_point()` returns `(usize, TrackPoint)`

  **Acceptance Criteria**:
  - [ ] Command removes point from segment
  - [ ] Undo re-inserts point at exact same index
  - [ ] Error case returns proper error for missing point
  - [ ] Tests: apply, undo, error (3 tests)

  **QA Scenarios**:

  ```
  Scenario: Delete + undo restores point
    Tool: Bash
    Steps:
      1. Run `cargo test delete_track_point 2>&1`
      2. Assert: point deleted, undo → point back at same index with same data
    Expected Result: Point fully restored on undo
    Evidence: .sisyphus/evidence/task-12-delete-point.txt
  ```

  **Commit**: YES
  - Message: `feat(commands): add DeleteTrackPoint command`
  - Files: `src-tauri/src/application/commands.rs`
  - Pre-commit: `cargo test --all`

 - [x] 13. InsertTrackPoint command

  **What to do**:
  - Add `ProjectCommand::InsertTrackPoint { layer_id, track_id, segment_id, index, point }` variant
  - Inserts `point` at `index` in the segment's point list
  - Reverse: `DeleteTrackPoint` removing the inserted point
  - Tests: apply (point inserted at correct index), undo (point removed), error (invalid index)

  **Recommended Agent Profile**:
  - **Category**: `unspecified-high`
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: YES (with Tasks 11, 12, 14-19)
  - **Parallel Group**: Wave 2
  - **Blocks**: Task 20
  - **Blocked By**: Tasks 5, 6

  **References**:

  **Pattern References**:
  - Task 6: `TrackSegment::insert_point_at(index, point)`
  - Task 12: DeleteTrackPoint (inverse relationship)

  **Acceptance Criteria**:
  - [ ] Point inserted at specified index
  - [ ] Undo removes the inserted point
  - [ ] Tests: apply, undo, error (3 tests)

  **QA Scenarios**:

  ```
  Scenario: Insert + undo cycle
    Tool: Bash
    Steps:
      1. Run `cargo test insert_track_point 2>&1`
      2. Assert: segment with 3 points, insert at index 1 → 4 points, undo → 3 points
    Expected Result: Insert/undo cycle preserves original state
    Evidence: .sisyphus/evidence/task-13-insert-point.txt
  ```

  **Commit**: YES
  - Message: `feat(commands): add InsertTrackPoint command`
  - Files: `src-tauri/src/application/commands.rs`
  - Pre-commit: `cargo test --all`

 - [x] 14. SplitSegment command

  **What to do**:
  - Add `ProjectCommand::SplitSegment { layer_id, track_id, segment_id, point_id }` variant
  - Calls `split_at_point()` from Task 6
  - Stores the resulting new segment for undo (join back)
  - Reverse: `JoinSegments` merging the two halves back
  - Tests: apply (one segment becomes two), undo (back to one), error (missing segment/point)

  **Recommended Agent Profile**:
  - **Category**: `unspecified-high`
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: YES (with Tasks 11-13, 15-19)
  - **Parallel Group**: Wave 2
  - **Blocks**: Task 20
  - **Blocked By**: Tasks 5, 6

  **References**:

  **Pattern References**:
  - Task 6: `TrackSegment::split_at_point(point_id)` domain method
  - Task 15: JoinSegments (inverse relationship)

  **Acceptance Criteria**:
  - [ ] Split produces two segments from one
  - [ ] Left segment keeps original ID; right gets new ID
  - [ ] Undo joins them back into original
  - [ ] Tests: apply, undo, error (3 tests)

  **QA Scenarios**:

  ```
  Scenario: Split + undo restores original segment
    Tool: Bash
    Steps:
      1. Run `cargo test split_segment 2>&1`
      2. Assert: 1 segment with 5 points → split at point 3 → 2 segments (3+3 points, split point shared)
      3. Assert: undo → back to 1 segment with 5 points
    Expected Result: Split is perfectly reversible
    Evidence: .sisyphus/evidence/task-14-split.txt
  ```

  **Commit**: YES
  - Message: `feat(commands): add SplitSegment command`
  - Files: `src-tauri/src/application/commands.rs`
  - Pre-commit: `cargo test --all`

 - [x] 15. JoinSegments command

  **What to do**:
  - Add `ProjectCommand::JoinSegments { layer_id, track_id, segment_id_a, segment_id_b }` variant
  - Calls `Track::join_segments()` from Task 7 — merges B's points into A, removes B
  - Stores removed segment B (with its index) for undo
  - Reverse: re-insert segment B at original index + remove merged points from A
  - Tests: apply, undo, error (non-adjacent segments)

  **Recommended Agent Profile**:
  - **Category**: `unspecified-high`
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: YES (with Tasks 11-14, 16-19)
  - **Parallel Group**: Wave 2
  - **Blocks**: Task 20
  - **Blocked By**: Tasks 5, 7

  **References**:

  **Pattern References**:
  - Task 7: `Track::join_segments(seg_id_a, seg_id_b)` domain method

  **Acceptance Criteria**:
  - [ ] Join merges adjacent segments
  - [ ] Undo restores both original segments
  - [ ] Error on non-adjacent segments
  - [ ] Tests: apply, undo, error (3 tests)

  **QA Scenarios**:

  ```
  Scenario: Join + undo cycle
    Tool: Bash
    Steps:
      1. Run `cargo test join_segments_command 2>&1`
      2. Assert: 2 segments → join → 1 segment with combined points → undo → 2 segments
    Expected Result: Join is perfectly reversible
    Evidence: .sisyphus/evidence/task-15-join.txt
  ```

  **Commit**: YES
  - Message: `feat(commands): add JoinSegments command`
  - Files: `src-tauri/src/application/commands.rs`
  - Pre-commit: `cargo test --all`

 - [x] 16. DeleteTrack command

  **What to do**:
  - Add `ProjectCommand::DeleteTrack { layer_id, track_id }` variant
  - Calls `Project::remove_track_from_layer()` from Task 8
  - Stores removed `(index, Track)` for undo
  - Reverse: re-insert track at original index
  - Tests: apply, undo, error (missing track)

  **Recommended Agent Profile**:
  - **Category**: `unspecified-high`
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: YES (with Tasks 11-15, 17-19)
  - **Parallel Group**: Wave 2
  - **Blocks**: Task 20
  - **Blocked By**: Tasks 5, 7, 8

  **References**:

  **Pattern References**:
  - Task 8: `Project::remove_track_from_layer()` domain method

  **Acceptance Criteria**:
  - [ ] Track removed from layer
  - [ ] Undo restores track at original position
  - [ ] Error on missing track
  - [ ] Tests: apply, undo, error (3 tests)

  **QA Scenarios**:

  ```
  Scenario: Delete track + undo
    Tool: Bash
    Steps:
      1. Run `cargo test delete_track 2>&1`
      2. Assert: track removed, undo → track back with all segments and points intact
    Expected Result: Full track restored on undo
    Evidence: .sisyphus/evidence/task-16-delete-track.txt
  ```

  **Commit**: YES
  - Message: `feat(commands): add DeleteTrack command`
  - Files: `src-tauri/src/application/commands.rs`
  - Pre-commit: `cargo test --all`

 - [x] 17. DeleteWaypoint command

  **What to do**:
  - Add `ProjectCommand::DeleteWaypoint { layer_id, waypoint_id }` variant
  - Calls `Project::remove_waypoint_from_layer()` from Task 9
  - Stores removed `(index, Waypoint)` for undo
  - Reverse: re-add waypoint at original index (need `WaypointLayer::insert_waypoint_at(index, waypoint)`)
  - Tests: apply, undo, error

  **Recommended Agent Profile**:
  - **Category**: `quick`
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: YES (with Tasks 11-16, 18, 19)
  - **Parallel Group**: Wave 2
  - **Blocks**: Task 21
  - **Blocked By**: Tasks 5, 9

  **References**:

  **Pattern References**:
  - Task 9: `WaypointLayer::remove_waypoint()` domain method
  - `src-tauri/src/application/commands.rs:47-52` — MoveWaypoint pattern

  **Acceptance Criteria**:
  - [ ] Waypoint removed from layer
  - [ ] Undo restores waypoint with all fields (name, position, symbol)
  - [ ] Tests: apply, undo, error (3 tests)

  **QA Scenarios**:

  ```
  Scenario: Delete waypoint + undo
    Tool: Bash
    Steps:
      1. Run `cargo test delete_waypoint 2>&1`
      2. Assert: waypoint removed, undo → waypoint back with name, position, symbol intact
    Expected Result: Waypoint fully restored on undo
    Evidence: .sisyphus/evidence/task-17-delete-waypoint.txt
  ```

  **Commit**: YES
  - Message: `feat(commands): add DeleteWaypoint command`
  - Files: `src-tauri/src/application/commands.rs`
  - Pre-commit: `cargo test --all`

 - [x] 18. RenameWaypoint command

  **What to do**:
  - Add `ProjectCommand::RenameWaypoint { layer_id, waypoint_id, new_name, old_name }` variant
  - Calls `Waypoint::set_name()` from Task 9 (returns old name)
  - Reverse: `RenameWaypoint` with names swapped
  - Tests: apply, undo

  **Recommended Agent Profile**:
  - **Category**: `quick`
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: YES (with Tasks 11-17, 19)
  - **Parallel Group**: Wave 2
  - **Blocks**: Task 21
  - **Blocked By**: Tasks 5, 9

  **References**:

  **Pattern References**:
  - `src-tauri/src/application/commands.rs` — `RenameTrack` pattern (closest analog)

  **Acceptance Criteria**:
  - [ ] Name changed on apply
  - [ ] Undo restores original name
  - [ ] Tests: apply, undo (2 tests)

  **QA Scenarios**:

  ```
  Scenario: Rename + undo
    Tool: Bash
    Steps:
      1. Run `cargo test rename_waypoint 2>&1`
      2. Assert: name changed from "Camp A" to "Camp B", undo → back to "Camp A"
    Expected Result: Name roundtrips correctly
    Evidence: .sisyphus/evidence/task-18-rename-waypoint.txt
  ```

  **Commit**: YES
  - Message: `feat(commands): add RenameWaypoint command`
  - Files: `src-tauri/src/application/commands.rs`
  - Pre-commit: `cargo test --all`

 - [x] 19. SimplifyTrack command (uses Douglas-Peucker)

  **What to do**:
  - Add `ProjectCommand::SimplifyTrack { layer_id, track_id, tolerance, removed_points }` variant
  - `apply()`: for each segment, call `simplify_track_points()` from Task 10, remove points NOT in the keep-list. Store removed points with their (segment_id, index) for undo.
  - Reverse: re-insert all removed points at their original indices (in reverse order to preserve indices)
  - Tests: apply (point count reduced), undo (all points restored), tolerance=0 (no change)

  **Recommended Agent Profile**:
  - **Category**: `unspecified-high`
    - Reason: Complex state management — tracking removed points across multiple segments for undo
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: YES (with Tasks 11-18)
  - **Parallel Group**: Wave 2
  - **Blocks**: Tasks 22, 25
  - **Blocked By**: Tasks 5, 10

  **References**:

  **Pattern References**:
  - Task 10: `simplify_track_points()` pure function
  - Task 6: `insert_point_at()` for undo re-insertion

  **Acceptance Criteria**:
  - [ ] Simplify removes redundant points based on tolerance
  - [ ] Undo restores ALL removed points at original positions
  - [ ] tolerance=0 leaves track unchanged
  - [ ] Tests: apply, undo, zero-tolerance (3 tests)

  **QA Scenarios**:

  ```
  Scenario: Simplify + undo roundtrip
    Tool: Bash
    Steps:
      1. Run `cargo test simplify_track_command 2>&1`
      2. Assert: 10-point track simplified to 5 points, undo → back to 10 points with same coordinates
    Expected Result: Simplification is fully reversible
    Evidence: .sisyphus/evidence/task-19-simplify.txt
  ```

  **Commit**: YES
  - Message: `feat(commands): add SimplifyTrack command with Douglas-Peucker`
  - Files: `src-tauri/src/application/commands.rs`
  - Pre-commit: `cargo test --all`

- [x] 20. Tauri handlers for track commands (Tasks 11-16)

  **What to do**:
  - Add 6 Tauri command handlers in `src-tauri/src/commands/mod.rs`:
    - `move_track_point(state, layer_id, track_id, segment_id, point_id, lat, lon)` — uses `apply_or_merge` for drag coalescing
    - `delete_track_point(state, layer_id, track_id, segment_id, point_id)`
    - `insert_track_point(state, layer_id, track_id, segment_id, index, lat, lon, timestamp?)` — creates TrackPoint internally
    - `split_segment(state, layer_id, track_id, segment_id, point_id)`
    - `join_segments(state, layer_id, track_id, segment_id_a, segment_id_b)`
    - `delete_track(state, layer_id, track_id)`
  - Add corresponding `AppState` wrapper methods in `application/mod.rs` that construct commands and call `CommandStack`
  - Register all 6 in `generate_handler!` macro in `lib.rs`
  - All handlers use safe lock handling (from Task 2) — NO `.lock().unwrap()`
  - All handlers emit `state-changed` event via `app_handle`

  **Must NOT do**:
  - Do NOT add DTO types for internal communication — Tauri handlers work directly with primitive types (u64, f64, String)
  - Do NOT add validation beyond what domain methods provide

  **Recommended Agent Profile**:
  - **Category**: `unspecified-high`
    - Reason: 6 handlers + 6 AppState methods + registration — repetitive but requires care
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: YES (with Tasks 21-26)
  - **Parallel Group**: Wave 3
  - **Blocks**: Tasks 23, 27
  - **Blocked By**: Tasks 11-16

  **References**:

  **Pattern References**:
  - `src-tauri/src/commands/mod.rs:396-409` — `rename_track` handler pattern (Tauri command → AppState method → command → emit event)
  - `src-tauri/src/application/mod.rs` — existing `apply_*` methods pattern
  - `src-tauri/src/lib.rs` — `generate_handler!` macro registration

  **Acceptance Criteria**:
  - [ ] 6 handlers in commands/mod.rs
  - [ ] 6 AppState wrapper methods in application/mod.rs
  - [ ] All 6 registered in generate_handler! macro
  - [ ] All use safe lock handling
  - [ ] All emit state-changed event
  - [ ] `cargo build` succeeds
  - [ ] `cargo test --all` passes

  **QA Scenarios**:

  ```
  Scenario: All handlers compile and are registered
    Tool: Bash
    Steps:
      1. Run `cargo build 2>&1`
      2. Assert clean compilation
      3. Grep lib.rs for all 6 handler names in generate_handler!
    Expected Result: All 6 handlers registered and compilable
    Evidence: .sisyphus/evidence/task-20-handlers.txt

  Scenario: No unsafe lock patterns
    Tool: Bash
    Steps:
      1. Run `grep -n '\.lock()\.unwrap()' src-tauri/src/commands/mod.rs`
      2. Assert: zero matches
    Expected Result: All new handlers use safe lock
    Evidence: .sisyphus/evidence/task-20-safe-lock.txt
  ```

  **Commit**: YES
  - Message: `feat(tauri): add track editing command handlers`
  - Files: `src-tauri/src/commands/mod.rs`, `src-tauri/src/application/mod.rs`, `src-tauri/src/lib.rs`
  - Pre-commit: `cargo test --all`

- [x] 21. Tauri handlers for waypoint commands (Tasks 17-18)

  **What to do**:
  - Add 2 Tauri command handlers:
    - `delete_waypoint(state, layer_id, waypoint_id)`
    - `rename_waypoint(state, layer_id, waypoint_id, new_name)`
  - Add AppState wrapper methods
  - Register in generate_handler! macro
  - Safe lock handling + state-changed event

  **Recommended Agent Profile**:
  - **Category**: `quick`
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: YES (with Tasks 20, 22-26)
  - **Parallel Group**: Wave 3
  - **Blocks**: Tasks 24, 27
  - **Blocked By**: Tasks 17, 18

  **References**:

  **Pattern References**:
  - `src-tauri/src/commands/mod.rs:396-409` — handler pattern
  - Existing `add_waypoint` handler as closest analog

  **Acceptance Criteria**:
  - [ ] 2 handlers registered and working
  - [ ] Safe lock handling
  - [ ] `cargo test --all` passes

  **QA Scenarios**:

  ```
  Scenario: Handlers compile and register
    Tool: Bash
    Steps:
      1. Run `cargo build 2>&1` — clean
      2. Verify delete_waypoint, rename_waypoint in generate_handler!
    Expected Result: Clean build with handlers registered
    Evidence: .sisyphus/evidence/task-21-waypoint-handlers.txt
  ```

  **Commit**: YES
  - Message: `feat(tauri): add waypoint command handlers (delete/rename)`
  - Files: `src-tauri/src/commands/mod.rs`, `src-tauri/src/application/mod.rs`, `src-tauri/src/lib.rs`
  - Pre-commit: `cargo test --all`

- [x] 22. Tauri handler for SimplifyTrack

  **What to do**:
  - Add `simplify_track(state, layer_id, track_id, tolerance)` handler
  - Add AppState wrapper method
  - Register in generate_handler! macro

  **Recommended Agent Profile**:
  - **Category**: `quick`
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: YES (with Tasks 20, 21, 23-26)
  - **Parallel Group**: Wave 3
  - **Blocks**: Tasks 25, 27
  - **Blocked By**: Task 19

  **References**:

  **Pattern References**:
  - Same handler pattern as Tasks 20-21

  **Acceptance Criteria**:
  - [ ] Handler registered and compilable
  - [ ] `cargo test --all` passes

  **QA Scenarios**:

  ```
  Scenario: Handler compiles
    Tool: Bash
    Steps:
      1. Run `cargo build 2>&1`
      2. Verify simplify_track in generate_handler!
    Expected Result: Clean build
    Evidence: .sisyphus/evidence/task-22-simplify-handler.txt
  ```

  **Commit**: YES (groups with Task 20 or standalone)
  - Message: `feat(tauri): add simplify_track handler`
  - Files: `src-tauri/src/commands/mod.rs`, `src-tauri/src/application/mod.rs`, `src-tauri/src/lib.rs`
  - Pre-commit: `cargo test --all`

- [x] 23. get_track_detail endpoint

  **What to do**:
  - Add `get_track_detail(state, layer_id, track_id)` Tauri command in `commands/mod.rs`
  - Returns a DTO: `TrackDetailDto { id, name, segments: Vec<SegmentDetailDto> }` where `SegmentDetailDto { id, points: Vec<PointDetailDto> }` and `PointDetailDto { id, lat, lon, elevation, timestamp }`
  - This is the endpoint that enables frontend track editing — the existing `get_tracks_geojson()` flattens segments and has no IDs
  - Register in generate_handler!
  - Serialize as JSON via serde

  **Must NOT do**:
  - Do NOT add pagination — return all points (performance addressed later if needed)
  - Do NOT modify existing `get_tracks_geojson()` endpoint

  **Recommended Agent Profile**:
  - **Category**: `unspecified-high`
    - Reason: New DTOs + serialization + important for the entire editing UI
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: YES (with Tasks 20-22, 24-26)
  - **Parallel Group**: Wave 3
  - **Blocks**: Tasks 28, 31
  - **Blocked By**: Task 20

  **References**:

  **Pattern References**:
  - `src-tauri/src/commands/mod.rs` — existing DTOs (e.g., `ProjectSummaryDto`) and `get_tracks_geojson()`
  - `src-tauri/src/domain/track.rs` — Track, TrackSegment, TrackPoint structures

  **API/Type References**:
  - Tauri commands with complex return types use `serde::Serialize` on DTOs

  **Acceptance Criteria**:
  - [ ] DTO structs defined with Serialize
  - [ ] Endpoint returns full segment→point hierarchy with all IDs
  - [ ] Handler registered
  - [ ] `cargo test --all` passes

  **QA Scenarios**:

  ```
  Scenario: Track detail returns full hierarchy
    Tool: Bash
    Steps:
      1. Run `cargo test get_track_detail 2>&1`
      2. Assert: returns segments with points, each having id, lat, lon
    Expected Result: Complete segment→point tree with IDs
    Evidence: .sisyphus/evidence/task-23-track-detail.txt

  Scenario: Missing track returns error
    Tool: Bash
    Steps:
      1. Run `cargo test get_track_detail_missing 2>&1`
      2. Assert: returns error string for nonexistent track_id
    Expected Result: Proper error, not panic
    Evidence: .sisyphus/evidence/task-23-error.txt
  ```

  **Commit**: YES
  - Message: `feat(tauri): add get_track_detail endpoint for point-level editing`
  - Files: `src-tauri/src/commands/mod.rs`
  - Pre-commit: `cargo test --all`

- [x] 24. get_waypoints endpoint

  **What to do**:
  - Add `get_waypoints(state, layer_id)` Tauri command
  - Returns `Vec<WaypointDto>` where `WaypointDto { id, name, lat, lon, symbol }`
  - This enables the waypoint panel in the frontend
  - Register in generate_handler!

  **Recommended Agent Profile**:
  - **Category**: `quick`
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: YES (with Tasks 20-23, 25, 26)
  - **Parallel Group**: Wave 3
  - **Blocks**: Task 29
  - **Blocked By**: Task 21

  **References**:

  **Pattern References**:
  - `src-tauri/src/commands/mod.rs` — existing `get_tracks_geojson()` pattern
  - `src-tauri/src/domain/waypoint.rs` — Waypoint struct

  **Acceptance Criteria**:
  - [ ] Returns list of waypoints with all fields including symbol
  - [ ] Handler registered
  - [ ] `cargo test --all` passes

  **QA Scenarios**:

  ```
  Scenario: Waypoints endpoint returns data
    Tool: Bash
    Steps:
      1. Run `cargo test get_waypoints 2>&1`
      2. Assert: returns waypoints with id, name, lat, lon, symbol
    Expected Result: All waypoint fields present in response
    Evidence: .sisyphus/evidence/task-24-waypoints.txt
  ```

  **Commit**: YES
  - Message: `feat(tauri): add get_waypoints endpoint`
  - Files: `src-tauri/src/commands/mod.rs`
  - Pre-commit: `cargo test --all`

- [x] 25. get_simplified_preview endpoint

  **What to do**:
  - Add `get_simplified_preview(state, layer_id, track_id, tolerance)` Tauri command
  - Calls `simplify_track_points()` from Task 10 on each segment
  - Returns `SimplifiedPreviewDto { original_count, simplified_count, segments: Vec<SimplifiedSegmentDto> }` where each segment has the kept points
  - This is a READ-ONLY preview — does NOT mutate state
  - Register in generate_handler!

  **Must NOT do**:
  - Do NOT modify project state — pure read operation
  - Do NOT cache results

  **Recommended Agent Profile**:
  - **Category**: `unspecified-high`
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: YES (with Tasks 20-24, 26)
  - **Parallel Group**: Wave 3
  - **Blocks**: Task 34
  - **Blocked By**: Task 22

  **References**:

  **Pattern References**:
  - Task 10: `simplify_track_points()` function
  - Task 23: DTO pattern for track data

  **Acceptance Criteria**:
  - [ ] Returns preview with point counts (original vs simplified)
  - [ ] Does NOT modify project state
  - [ ] Handler registered
  - [ ] `cargo test --all` passes

  **QA Scenarios**:

  ```
  Scenario: Preview shows correct counts
    Tool: Bash
    Steps:
      1. Run `cargo test simplified_preview 2>&1`
      2. Assert: original_count > simplified_count for non-trivial track
      3. Assert: project state unchanged after preview
    Expected Result: Preview is purely informational
    Evidence: .sisyphus/evidence/task-25-preview.txt
  ```

  **Commit**: YES
  - Message: `feat(tauri): add get_simplified_preview endpoint`
  - Files: `src-tauri/src/commands/mod.rs`
  - Pre-commit: `cargo test --all`

- [x] 26. SetTrackLineWidth (direct mutation handler)

  **What to do**:
  - Add `set_track_line_width(state, layer_id, track_id, width)` Tauri command
  - Follow existing pattern of `set_track_color` and `toggle_track_visible` — direct mutation, NOT through CommandStack
  - Access `track.style_mut().set_line_width(width)` — add `set_line_width(f32)` to `TrackStyle` if not present
  - Add `line_width: f32` to `TrackStyle` if not present (default: 2.0)
  - Emit state-changed event
  - Register in generate_handler!

  **Must NOT do**:
  - Do NOT route through CommandStack — this is NOT undoable (consistent with color/visibility)
  - Do NOT add validation beyond reasonable range (0.5 to 20.0)

  **Recommended Agent Profile**:
  - **Category**: `quick`
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: YES (with Tasks 20-25)
  - **Parallel Group**: Wave 3
  - **Blocks**: Task 27
  - **Blocked By**: Tasks 1-4

  **References**:

  **Pattern References**:
  - `src-tauri/src/commands/mod.rs` — `set_track_color` handler pattern (direct mutation, not undoable)
  - `src-tauri/src/domain/track.rs` — `TrackStyle` struct

  **Acceptance Criteria**:
  - [ ] `TrackStyle` has `line_width: f32` field (default 2.0)
  - [ ] Handler sets width directly, no CommandStack
  - [ ] Handler registered
  - [ ] `cargo test --all` passes

  **QA Scenarios**:

  ```
  Scenario: Line width mutation
    Tool: Bash
    Steps:
      1. Run `cargo build 2>&1` — clean
      2. Verify set_track_line_width in generate_handler!
      3. Run `cargo test --all 2>&1` — passes
    Expected Result: Handler compiles and registers
    Evidence: .sisyphus/evidence/task-26-line-width.txt
  ```

  **Commit**: YES
  - Message: `feat(tauri): add set_track_line_width handler`
  - Files: `src-tauri/src/commands/mod.rs`, `src-tauri/src/domain/track.rs`
  - Pre-commit: `cargo test --all`

- [x] 27. Frontend API functions for all new commands

  **What to do**:
  - Add to `src/lib/api.ts` all new Tauri invoke wrappers:
    - `moveTrackPoint(layerId, trackId, segmentId, pointId, lat, lon)`
    - `deleteTrackPoint(layerId, trackId, segmentId, pointId)`
    - `insertTrackPoint(layerId, trackId, segmentId, index, lat, lon)`
    - `splitSegment(layerId, trackId, segmentId, pointId)`
    - `joinSegments(layerId, trackId, segIdA, segIdB)`
    - `deleteTrack(layerId, trackId)`
    - `deleteWaypoint(layerId, waypointId)`
    - `renameWaypoint(layerId, waypointId, newName)`
    - `simplifyTrack(layerId, trackId, tolerance)`
    - `setTrackLineWidth(layerId, trackId, width)`
    - `getTrackDetail(layerId, trackId)` → returns TrackDetail type
    - `getWaypoints(layerId)` → returns Waypoint[] type
    - `getSimplifiedPreview(layerId, trackId, tolerance)` → returns SimplifiedPreview type
  - Add corresponding TypeScript types to `src/lib/types.ts`:
    - `TrackDetail { id, name, segments: SegmentDetail[] }`
    - `SegmentDetail { id, points: PointDetail[] }`
    - `PointDetail { id, lat, lon, elevation?, timestamp? }`
    - `WaypointData { id, name, lat, lon, symbol? }`
    - `SimplifiedPreview { original_count, simplified_count, segments }`
  - Follow existing invoke pattern in api.ts

  **Must NOT do**:
  - Do NOT add error handling wrappers — follow existing pattern (raw invoke)
  - Do NOT add caching or state management here — that's in stores.ts

  **Recommended Agent Profile**:
  - **Category**: `unspecified-high`
    - Reason: Many API functions + type definitions, needs consistency
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: YES (with Tasks 28-30)
  - **Parallel Group**: Wave 4
  - **Blocks**: Tasks 28-34
  - **Blocked By**: Tasks 20-26

  **References**:

  **Pattern References**:
  - `src/lib/api.ts` — existing invoke wrapper pattern
  - `src/lib/types.ts` — existing TypeScript type definitions

  **Acceptance Criteria**:
  - [ ] 13 API functions added to api.ts
  - [ ] 5 TypeScript types added to types.ts
  - [ ] All follow existing invoke pattern
  - [ ] `npm test` passes
  - [ ] TypeScript compiles without errors

  **QA Scenarios**:

  ```
  Scenario: TypeScript compilation
    Tool: Bash
    Steps:
      1. Run `npm test 2>&1`
      2. Assert all tests pass
      3. Run `npx tsc --noEmit 2>&1` (if tsconfig allows)
    Expected Result: No type errors
    Evidence: .sisyphus/evidence/task-27-api.txt

  Scenario: API function signatures match Rust handlers
    Tool: Bash
    Steps:
      1. Compare parameter names in api.ts with Tauri handler signatures in commands/mod.rs
      2. Verify 1:1 mapping (camelCase in TS → snake_case in Rust)
    Expected Result: All parameter names and types align
    Evidence: .sisyphus/evidence/task-27-alignment.txt
  ```

  **Commit**: YES
  - Message: `feat(ui): add frontend API functions and types for all new commands`
  - Files: `src/lib/api.ts`, `src/lib/types.ts`
  - Pre-commit: `npm test`

- [x] 28. Track point list panel (read-only)

  **What to do**:
  - Create `src/components/TrackPointsPanel.svelte`
  - Shows segment→point hierarchy for the currently selected track
  - Calls `getTrackDetail(layerId, trackId)` when a track is selected
  - Display: segment headers with point count, flat list of points showing lat/lon/elevation
  - Click on a point → highlight on map (emit event or update store)
  - Read-only — no inline editing of coordinates
  - Limit display to first 1000 points per segment with "Show more" button
  - Integrate into existing Sidebar layout

  **Must NOT do**:
  - Do NOT add virtualized scrolling
  - Do NOT add search/filter
  - Do NOT add inline coordinate editing
  - Do NOT add drag-to-reorder

  **Recommended Agent Profile**:
  - **Category**: `visual-engineering`
    - Reason: UI component with layout, data display, interaction
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: YES (with Tasks 27, 29, 30)
  - **Parallel Group**: Wave 4
  - **Blocks**: Task 31
  - **Blocked By**: Tasks 23, 27

  **References**:

  **Pattern References**:
  - `src/components/TracksPanel.svelte` — existing panel component pattern, styling
  - `src/components/Sidebar.svelte` — where the panel will be integrated
  - `src/lib/stores.ts` — existing store pattern for selected state

  **API/Type References**:
  - Task 27: `getTrackDetail()` API function and `TrackDetail` type

  **Acceptance Criteria**:
  - [ ] Component renders segment→point hierarchy
  - [ ] Shows lat/lon/elevation for each point
  - [ ] Segment headers show point count
  - [ ] Click on point emits selection event
  - [ ] "Show more" pagination at 1000 points
  - [ ] Integrated into Sidebar
  - [ ] `npm test` passes

  **QA Scenarios**:

  ```
  Scenario: Panel renders track points
    Tool: Playwright
    Preconditions: App running with a project containing a track with 5+ points
    Steps:
      1. Navigate to app URL
      2. Select a track in TracksPanel
      3. Assert TrackPointsPanel is visible (selector: `.track-points-panel` or similar)
      4. Assert segment headers show point counts
      5. Assert point rows show lat/lon values
    Expected Result: Panel displays segment→point data correctly
    Failure Indicators: Panel empty, no points rendered, wrong data
    Evidence: .sisyphus/evidence/task-28-panel.png

  Scenario: Empty track shows no points
    Tool: Playwright
    Steps:
      1. Select a track with 0 segments
      2. Assert panel shows "No points" or equivalent message
    Expected Result: Graceful empty state
    Evidence: .sisyphus/evidence/task-28-empty.png
  ```

  **Commit**: YES
  - Message: `feat(ui): add track points panel (read-only)`
  - Files: `src/components/TrackPointsPanel.svelte`, `src/components/Sidebar.svelte`
  - Pre-commit: `npm test`

- [x] 29. Waypoint panel with list/delete/rename

  **What to do**:
  - Create `src/components/WaypointsPanel.svelte`
  - Calls `getWaypoints(layerId)` to fetch waypoint list
  - Display: list of waypoints with name, symbol icon, lat/lon
  - Actions per waypoint:
    - Delete button → calls `deleteWaypoint(layerId, waypointId)`
    - Rename (inline edit or modal) → calls `renameWaypoint(layerId, waypointId, newName)`
    - Click → center map on waypoint / highlight
  - Re-fetches on `state-changed` event
  - Follow existing component styling (Catppuccin theme)
  - Integrate into Sidebar

  **Must NOT do**:
  - Do NOT add drag-to-reorder waypoints in the list
  - Do NOT add bulk actions
  - Do NOT add waypoint creation from panel (that's click-on-map in Task 32)

  **Recommended Agent Profile**:
  - **Category**: `visual-engineering`
    - Reason: Interactive UI component with actions, styling
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: YES (with Tasks 27, 28, 30)
  - **Parallel Group**: Wave 4
  - **Blocks**: Tasks 32, 33
  - **Blocked By**: Tasks 24, 27

  **References**:

  **Pattern References**:
  - `src/components/TracksPanel.svelte` — existing panel pattern with actions
  - `src/lib/stores.ts` — state management pattern

  **API/Type References**:
  - Task 27: `getWaypoints()`, `deleteWaypoint()`, `renameWaypoint()` API functions

  **Acceptance Criteria**:
  - [ ] Panel shows list of waypoints with name, symbol, coordinates
  - [ ] Delete button removes waypoint (with command)
  - [ ] Rename inline edit works
  - [ ] Click centers map on waypoint
  - [ ] Re-fetches on state-changed
  - [ ] Integrated into Sidebar
  - [ ] `npm test` passes

  **QA Scenarios**:

  ```
  Scenario: Waypoint panel shows list
    Tool: Playwright
    Preconditions: Project with 3+ waypoints
    Steps:
      1. Navigate to app
      2. Assert WaypointsPanel visible with waypoint names
      3. Assert each waypoint shows coordinates
    Expected Result: All waypoints listed with data
    Evidence: .sisyphus/evidence/task-29-panel.png

  Scenario: Delete waypoint from panel
    Tool: Playwright
    Steps:
      1. Click delete button on first waypoint
      2. Assert waypoint disappears from list
      3. Assert undo (Ctrl+Z) restores it
    Expected Result: Delete works and is undoable
    Evidence: .sisyphus/evidence/task-29-delete.png
  ```

  **Commit**: YES
  - Message: `feat(ui): add waypoint panel with delete/rename actions`
  - Files: `src/components/WaypointsPanel.svelte`, `src/components/Sidebar.svelte`
  - Pre-commit: `npm test`

- [x] 30. Waypoint symbol picker component

  **What to do**:
  - Create `src/components/SymbolPicker.svelte`
  - Dropdown or grid of predefined symbols: `flag`, `camp`, `danger`, `water`, `shelter`, `meeting-point`, `start`, `finish`, `viewpoint`, `parking`
  - Each symbol has a simple icon (emoji or SVG icon) + label
  - On select → calls appropriate API to set symbol on waypoint
  - Integrate into WaypointsPanel (next to each waypoint or in edit mode)
  - Add corresponding symbol rendering to map waypoint markers (MapLibre marker icons)

  **Must NOT do**:
  - Do NOT add custom icon upload
  - Do NOT add color customization per symbol
  - Do NOT over-design — simple grid with 10 predefined symbols

  **Recommended Agent Profile**:
  - **Category**: `visual-engineering`
    - Reason: UI component with icons, grid layout, integration with panel and map
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: YES (with Tasks 27, 28, 29)
  - **Parallel Group**: Wave 4
  - **Blocks**: None
  - **Blocked By**: Task 29

  **References**:

  **Pattern References**:
  - `src/components/ThemePicker.svelte` — existing picker component pattern
  - `src/components/WaypointsPanel.svelte` (from Task 29) — integration point
  - `src/lib/maplibre/` — MapLibre configuration for markers

  **Acceptance Criteria**:
  - [ ] 10 predefined symbols with icons
  - [ ] Picker UI (dropdown or grid)
  - [ ] Selection sets symbol on waypoint via API
  - [ ] Waypoint markers on map reflect symbol choice
  - [ ] `npm test` passes

  **QA Scenarios**:

  ```
  Scenario: Symbol picker shows options
    Tool: Playwright
    Steps:
      1. Open waypoint edit mode
      2. Click symbol picker
      3. Assert 10 symbol options visible
      4. Select "camp" symbol
      5. Assert waypoint marker updates on map
    Expected Result: Symbol selection persists and renders
    Evidence: .sisyphus/evidence/task-30-symbol.png

  Scenario: Symbol persists after reload
    Tool: Playwright
    Steps:
      1. Set waypoint symbol to "danger"
      2. Save project, close and reopen
      3. Assert waypoint still shows "danger" symbol
    Expected Result: Symbol survives save/load cycle
    Evidence: .sisyphus/evidence/task-30-persist.png
  ```

  **Commit**: YES
  - Message: `feat(ui): add waypoint symbol picker with 10 predefined icons`
  - Files: `src/components/SymbolPicker.svelte`, `src/components/WaypointsPanel.svelte`, `src/lib/maplibre/`
  - Pre-commit: `npm test`

- [x] 31. Map track point drag editing

  **What to do**:
  - When a track is selected and user enters "edit mode", render track points as draggable markers on MapLibre map
  - Use MapLibre `Marker` with `draggable: true` or custom drag handling via `mousedown`/`mousemove`/`mouseup` on a GeoJSON source
  - On drag start: record original position
  - On drag (mousemove): update marker position visually (no command yet)
  - On drag end: call `moveTrackPoint(layerId, trackId, segmentId, pointId, newLat, newLon)` — the Tauri handler uses `apply_or_merge` for coalescing
  - Points should be rendered differently from the track line (e.g., small circles)
  - Right-click on point → context menu with: Delete Point, Insert Point After
  - Edit mode toggle button in TrackPointsPanel or TracksPanel

  **Must NOT do**:
  - Do NOT implement multi-point selection/move
  - Do NOT add snapping to grid or other points
  - Do NOT change the underlying MapLibre tile rendering

  **Recommended Agent Profile**:
  - **Category**: `deep`
    - Reason: Complex MapLibre interaction with drag events, state management, context menu
  - **Skills**: [`playwright`]
    - `playwright`: For QA scenarios testing drag interaction on the map

  **Parallelization**:
  - **Can Run In Parallel**: YES (with Tasks 32, 33, 34)
  - **Parallel Group**: Wave 5
  - **Blocks**: Task 38
  - **Blocked By**: Task 28

  **References**:

  **Pattern References**:
  - `src/components/MapView.svelte` — existing MapLibre map component
  - `src/lib/maplibre/` — MapLibre configuration, sources, layers
  - `src/components/TracksPanel.svelte` — track selection state

  **External References**:
  - MapLibre GL JS: `addSource` with GeoJSON, `addLayer` for circles, drag handling via events
  - MapLibre draggable markers: https://maplibre.org/maplibre-gl-js/docs/examples/drag-a-marker/

  **Acceptance Criteria**:
  - [ ] Edit mode toggle renders points as draggable markers
  - [ ] Dragging a point calls moveTrackPoint on drag end
  - [ ] Drag coalescing works (fast dragging = one undo step)
  - [ ] Right-click context menu with Delete/Insert options
  - [ ] Points visually distinct from track line
  - [ ] `npm test` passes

  **QA Scenarios**:

  ```
  Scenario: Drag point and verify position change
    Tool: Playwright
    Preconditions: App running, project with a track loaded, edit mode ON
    Steps:
      1. Navigate to app
      2. Enable edit mode for selected track
      3. Assert draggable point markers appear on map (selector: `.maplibregl-marker` or circle layer)
      4. Drag first point marker 50px to the right
      5. Assert point marker is at new position
      6. Press Ctrl+Z
      7. Assert point marker returns to original position
    Expected Result: Drag + undo cycle works on map
    Failure Indicators: Point doesn't move, undo doesn't restore, markers don't appear
    Evidence: .sisyphus/evidence/task-31-drag.png

  Scenario: Right-click context menu
    Tool: Playwright
    Steps:
      1. Right-click on a track point marker
      2. Assert context menu appears with "Delete Point" and "Insert Point"
      3. Click "Delete Point"
      4. Assert point disappears from map and from point list panel
    Expected Result: Context menu actions work
    Evidence: .sisyphus/evidence/task-31-context-menu.png
  ```

  **Commit**: YES
  - Message: `feat(ui): add map track point drag editing with context menu`
  - Files: `src/components/MapView.svelte`, related maplibre files
  - Pre-commit: `npm test`

- [x] 32. Map click-to-add waypoint

  **What to do**:
  - Add "Add Waypoint" mode: user clicks a toolbar button, then clicks on map to place waypoint
  - On map click: call `addWaypoint(layerId, lat, lon, name, symbol)` with default name (e.g., "Waypoint N") and no symbol
  - Show a temporary marker at click position during placement
  - After placement, open rename dialog or inline edit in WaypointsPanel
  - Escape key cancels placement mode
  - Render waypoints on map as MapLibre markers (if not already rendered)

  **Must NOT do**:
  - Do NOT add batch waypoint creation
  - Do NOT add coordinate input dialog — click position IS the coordinate

  **Recommended Agent Profile**:
  - **Category**: `unspecified-high`
  - **Skills**: [`playwright`]

  **Parallelization**:
  - **Can Run In Parallel**: YES (with Tasks 31, 33, 34)
  - **Parallel Group**: Wave 5
  - **Blocks**: None
  - **Blocked By**: Task 29

  **References**:

  **Pattern References**:
  - `src/components/MapView.svelte` — map click handling
  - `src/components/WaypointsPanel.svelte` (Task 29) — waypoint display

  **Acceptance Criteria**:
  - [ ] "Add Waypoint" button/mode exists
  - [ ] Map click places waypoint at clicked coordinates
  - [ ] Waypoint appears on map as marker
  - [ ] Waypoint appears in WaypointsPanel
  - [ ] Escape cancels placement mode
  - [ ] `npm test` passes

  **QA Scenarios**:

  ```
  Scenario: Click to add waypoint
    Tool: Playwright
    Steps:
      1. Click "Add Waypoint" button
      2. Click on map at known position
      3. Assert new waypoint marker appears on map
      4. Assert new waypoint appears in WaypointsPanel
      5. Assert waypoint has coordinates matching click position (within tolerance)
    Expected Result: Waypoint created at click position
    Evidence: .sisyphus/evidence/task-32-add-waypoint.png

  Scenario: Escape cancels add mode
    Tool: Playwright
    Steps:
      1. Click "Add Waypoint" button
      2. Press Escape
      3. Click on map
      4. Assert no waypoint created
    Expected Result: Escape exits add mode
    Evidence: .sisyphus/evidence/task-32-cancel.png
  ```

  **Commit**: YES
  - Message: `feat(ui): add click-to-add waypoint on map`
  - Files: `src/components/MapView.svelte`, toolbar component
  - Pre-commit: `npm test`

- [x] 33. Map waypoint drag

  **What to do**:
  - Make waypoint markers on map draggable (similar to track point drag in Task 31)
  - On drag end: call `moveWaypoint(layerId, waypointId, newLat, newLon)` (existing command, uses `apply_or_merge`)
  - Update WaypointsPanel coordinates on drag
  - Cursor changes to `grab`/`grabbing` during drag

  **Must NOT do**:
  - Do NOT add multi-waypoint selection/drag
  - Do NOT change existing MoveWaypoint command behavior

  **Recommended Agent Profile**:
  - **Category**: `unspecified-high`
  - **Skills**: [`playwright`]

  **Parallelization**:
  - **Can Run In Parallel**: YES (with Tasks 31, 32, 34)
  - **Parallel Group**: Wave 5
  - **Blocks**: None
  - **Blocked By**: Task 29

  **References**:

  **Pattern References**:
  - Task 31: Track point drag pattern (reuse drag handling approach)
  - `src/lib/api.ts` — existing `moveWaypoint()` API function

  **Acceptance Criteria**:
  - [ ] Waypoint markers are draggable on map
  - [ ] Drag end calls moveWaypoint
  - [ ] WaypointsPanel updates with new coordinates
  - [ ] Undo restores original position
  - [ ] `npm test` passes

  **QA Scenarios**:

  ```
  Scenario: Drag waypoint and undo
    Tool: Playwright
    Steps:
      1. Drag waypoint marker to new position
      2. Assert WaypointsPanel shows updated coordinates
      3. Press Ctrl+Z
      4. Assert waypoint returns to original position
    Expected Result: Waypoint drag + undo works
    Evidence: .sisyphus/evidence/task-33-drag-waypoint.png
  ```

  **Commit**: YES
  - Message: `feat(ui): add waypoint drag on map`
  - Files: `src/components/MapView.svelte`
  - Pre-commit: `npm test`

- [x] 34. Simplification preview overlay + confirm UI

  **What to do**:
  - Add "Simplify Track" button in TracksPanel (or track context menu)
  - On click: show tolerance slider/input (range: 1m to 1000m, default: 10m)
  - As tolerance changes: call `getSimplifiedPreview(layerId, trackId, tolerance)` and render preview overlay on map
  - Preview: show simplified track as a different-colored line alongside original, display "Original: N points → Simplified: M points" label
  - "Confirm" button → calls `simplifyTrack(layerId, trackId, tolerance)` (actual command, undoable)
  - "Cancel" button → removes preview overlay, restores normal view
  - Debounce preview requests (300ms) to avoid spamming endpoint during slider drag

  **Must NOT do**:
  - Do NOT add per-segment tolerance
  - Do NOT auto-simplify without user confirmation
  - Do NOT persist tolerance preference

  **Recommended Agent Profile**:
  - **Category**: `visual-engineering`
    - Reason: UI with slider, preview overlay, confirm/cancel flow, visual feedback
  - **Skills**: [`playwright`]

  **Parallelization**:
  - **Can Run In Parallel**: YES (with Tasks 31, 32, 33)
  - **Parallel Group**: Wave 5
  - **Blocks**: None
  - **Blocked By**: Tasks 25, 27

  **References**:

  **Pattern References**:
  - `src/components/TracksPanel.svelte` — integration point for "Simplify" button
  - Task 25: `getSimplifiedPreview()` API function
  - Task 27: `simplifyTrack()` API function

  **Acceptance Criteria**:
  - [ ] Tolerance slider (1m-1000m) with real-time preview
  - [ ] Preview overlay on map (different color line)
  - [ ] Point count comparison shown
  - [ ] "Confirm" applies simplification command (undoable)
  - [ ] "Cancel" removes preview
  - [ ] Debounced preview requests
  - [ ] `npm test` passes

  **QA Scenarios**:

  ```
  Scenario: Preview shows simplified track
    Tool: Playwright
    Preconditions: Project with a track with 50+ points
    Steps:
      1. Click "Simplify Track" button
      2. Assert tolerance slider appears
      3. Move slider to 100m
      4. Assert point count label shows "Original: N → Simplified: M" where M < N
      5. Assert preview line visible on map (different color)
    Expected Result: Preview renders with correct counts
    Evidence: .sisyphus/evidence/task-34-preview.png

  Scenario: Confirm applies and is undoable
    Tool: Playwright
    Steps:
      1. Set tolerance to 50m, click Confirm
      2. Assert track point count reduced in TrackPointsPanel
      3. Press Ctrl+Z
      4. Assert track point count restored to original
    Expected Result: Simplification is committed and undoable
    Evidence: .sisyphus/evidence/task-34-confirm-undo.png

  Scenario: Cancel removes preview
    Tool: Playwright
    Steps:
      1. Click "Simplify Track", move slider
      2. Click "Cancel"
      3. Assert preview overlay removed, original track unchanged
    Expected Result: No state change on cancel
    Evidence: .sisyphus/evidence/task-34-cancel.png
  ```

  **Commit**: YES
  - Message: `feat(ui): add track simplification preview with confirm/cancel`
  - Files: `src/components/SimplifyPanel.svelte` (new), `src/components/TracksPanel.svelte`
  - Pre-commit: `npm test`

- [x] 35. PLT export with round-trip tests

  **What to do**:
  - Create `src-tauri/src/infrastructure/export/plt.rs`
  - Implement `export_plt(track: &Track, writer: &mut impl Write) -> Result<(), ExportError>`
  - PLT format requirements (must match OziExplorer):
    - Windows line endings (`\r\n`)
    - Header: `OziExplorer Track Point File Version 2.1`
    - Second line: WGS 84 datum
    - Third line: reserved
    - Fourth line: field description
    - Fifth line: point count + track metadata (color as COLORREF, width, style)
    - Point lines: `lat,lon,0,altitude,OLE_date,time_string,segment_flag`
    - OLE date: days since 1899-12-30 as float
    - COLORREF: BGR (not RGB) — `(b << 16) | (g << 8) | r`
  - Use existing PLT import test fixtures for round-trip testing
  - Write round-trip test: import PLT → export PLT → import again → compare domain objects
  - Write direct output test: known track → assert exact output bytes match expected

  **Must NOT do**:
  - Do NOT add CSV or other text format exports
  - Do NOT change PLT import parser
  - Do NOT add encoding options — PLT is always ASCII/Latin-1

  **Recommended Agent Profile**:
  - **Category**: `deep`
    - Reason: Binary format fidelity, OLE date math, COLORREF encoding, round-trip verification
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: YES (with Wave 4-5 tasks — PLT export is independent of UI)
  - **Parallel Group**: Wave 6
  - **Blocks**: Task 36
  - **Blocked By**: Tasks 1-4 (only bug fixes)

  **References**:

  **Pattern References**:
  - `src-tauri/src/infrastructure/export/gpx.rs` — existing GPX export pattern
  - `src-tauri/src/infrastructure/import/plt.rs` — PLT import parser (use same format knowledge)

  **Test References**:
  - `src-tauri/src/infrastructure/import/plt.rs` — PLT test fixtures in `tests/fixtures/` or inline
  - `src-tauri/src/infrastructure/export/gpx.rs` — export test pattern

  **External References**:
  - OLE date format: days since 1899-12-30 00:00:00
  - COLORREF: `0x00BBGGRR` (note: BGR not RGB)

  **Acceptance Criteria**:
  - [ ] PLT output matches OziExplorer format exactly
  - [ ] Windows line endings (`\r\n`)
  - [ ] OLE date encoding correct (matches PLT import parsing)
  - [ ] COLORREF encoding correct (BGR)
  - [ ] Round-trip test: import→export→import produces identical domain objects
  - [ ] At least 3 tests: basic export, round-trip, edge cases (no timestamps, empty segments)
  - [ ] `cargo test --all` passes

  **QA Scenarios**:

  ```
  Scenario: PLT round-trip preserves data
    Tool: Bash
    Steps:
      1. Run `cargo test plt_export 2>&1`
      2. Assert round-trip test: import fixture → export → import again → assert_eq on track points
    Expected Result: Zero data loss in import→export→import cycle
    Failure Indicators: Coordinate drift, missing points, wrong timestamps
    Evidence: .sisyphus/evidence/task-35-roundtrip.txt

  Scenario: PLT output format correctness
    Tool: Bash
    Steps:
      1. Run `cargo test plt_format 2>&1`
      2. Assert: output uses \r\n line endings
      3. Assert: header matches "OziExplorer Track Point File Version 2.1"
      4. Assert: COLORREF is BGR encoded
    Expected Result: Byte-accurate PLT format
    Evidence: .sisyphus/evidence/task-35-format.txt
  ```

  **Commit**: YES
  - Message: `feat(export): add PLT export with round-trip tests`
  - Files: `src-tauri/src/infrastructure/export/plt.rs`, `src-tauri/src/infrastructure/export/mod.rs`
  - Pre-commit: `cargo test --all`

- [x] 36. PLT export Tauri handler + frontend integration

  **What to do**:
  - Add `export_track_plt(state, layer_id, track_id)` Tauri command
  - Opens system file save dialog (using `tauri::dialog::FileDialogBuilder`)
  - Writes PLT file to selected path
  - Add "Export as PLT" option in TracksPanel context menu or track actions
  - Frontend: add `exportTrackPlt(layerId, trackId)` in api.ts

  **Must NOT do**:
  - Do NOT add batch export (one track at a time)
  - Do NOT add format selection dialog — PLT is the only format in this task

  **Recommended Agent Profile**:
  - **Category**: `unspecified-high`
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: NO (depends on Task 35)
  - **Parallel Group**: Wave 6
  - **Blocks**: None
  - **Blocked By**: Task 35

  **References**:

  **Pattern References**:
  - `src-tauri/src/commands/mod.rs` — existing export handlers (GPX export)
  - `src/components/TracksPanel.svelte` — existing track action buttons

  **Acceptance Criteria**:
  - [ ] Tauri handler opens save dialog and writes PLT
  - [ ] Frontend has "Export as PLT" button/menu item
  - [ ] Handler registered in generate_handler!
  - [ ] `cargo test --all` passes
  - [ ] `npm test` passes

  **QA Scenarios**:

  ```
  Scenario: Export PLT via UI
    Tool: Playwright
    Preconditions: Project with a track
    Steps:
      1. Right-click or click actions on a track in TracksPanel
      2. Click "Export as PLT"
      3. Assert file save dialog appears (or file is saved to test path)
    Expected Result: PLT file exported successfully
    Evidence: .sisyphus/evidence/task-36-export-plt.png
  ```

  **Commit**: YES
  - Message: `feat(tauri): add PLT export handler with frontend integration`
  - Files: `src-tauri/src/commands/mod.rs`, `src/lib/api.ts`, `src/components/TracksPanel.svelte`
  - Pre-commit: `cargo test --all && npm test`

- [x] 37. CreateEmptyTrack command + domain

  **What to do**:
  - Add `TrackLayer::create_empty_track(&mut self, name: String) -> &mut Track` — creates a track with a name and one empty segment
  - Add `ProjectCommand::CreateEmptyTrack { layer_id, name }` variant
  - On apply: creates empty track, stores created track_id for undo (delete)
  - Reverse: `DeleteTrack` removing the created track
  - This enables the drawing mode (Task 38) — user creates empty track, then adds points by clicking on map
  - Add Tauri handler `create_empty_track(state, layer_id, name)` and register
  - Add frontend API function

  **Must NOT do**:
  - Do NOT create the drawing mode here — just the command that creates an empty track

  **Recommended Agent Profile**:
  - **Category**: `unspecified-high`
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: NO (depends on Task 5 delta undo)
  - **Parallel Group**: Wave 7
  - **Blocks**: Task 38
  - **Blocked By**: Task 5

  **References**:

  **Pattern References**:
  - `src-tauri/src/application/commands.rs` — existing AddTrack command pattern
  - `src-tauri/src/domain/project.rs` — TrackLayer::add_track()

  **Acceptance Criteria**:
  - [ ] Command creates empty track with one empty segment
  - [ ] Undo deletes the track
  - [ ] Tauri handler + registration
  - [ ] Frontend API function
  - [ ] Tests: apply, undo (2 tests)
  - [ ] `cargo test --all` passes

  **QA Scenarios**:

  ```
  Scenario: Create empty track + undo
    Tool: Bash
    Steps:
      1. Run `cargo test create_empty_track 2>&1`
      2. Assert: track created with 1 empty segment
      3. Assert: undo removes the track
    Expected Result: Empty track creation is undoable
    Evidence: .sisyphus/evidence/task-37-empty-track.txt
  ```

  **Commit**: YES
  - Message: `feat(commands): add CreateEmptyTrack command`
  - Files: `src-tauri/src/application/commands.rs`, `src-tauri/src/domain/project.rs`, `src-tauri/src/commands/mod.rs`, `src/lib/api.ts`
  - Pre-commit: `cargo test --all`

- [x] 38. Map drawing mode for new tracks

  **What to do**:
  - Add "Create Track" toolbar button → enters drawing mode
  - On entering drawing mode: create empty track via `createEmptyTrack(layerId, "New Track")`
  - Each map click → `insertTrackPoint(layerId, trackId, segmentId, nextIndex, lat, lon)` — adds point to the track's first segment
  - Render the in-progress track as a line on the map, with point markers at each click
  - Show point count indicator
  - Double-click or Enter or "Done" button → exit drawing mode
  - Escape → undo all inserted points (multiple undos) and delete empty track
  - Integrate with existing map interaction modes (ensure drawing mode is exclusive — no drag editing while drawing)

  **Must NOT do**:
  - Do NOT add freehand drawing (GPS-style continuous recording)
  - Do NOT add multi-segment creation in one drawing session
  - Do NOT add curve fitting or smoothing

  **Recommended Agent Profile**:
  - **Category**: `deep`
    - Reason: Complex interaction mode with map click handling, state machine (drawing mode), undo orchestration
  - **Skills**: [`playwright`]

  **Parallelization**:
  - **Can Run In Parallel**: NO (depends on Tasks 37, 31)
  - **Parallel Group**: Wave 7
  - **Blocks**: None
  - **Blocked By**: Tasks 37, 31

  **References**:

  **Pattern References**:
  - Task 31: Map edit mode pattern (reuse mode state management)
  - Task 32: Click-to-add pattern (reuse map click handling)
  - `src/components/MapView.svelte` — map component

  **External References**:
  - MapLibre GL JS: click event handling, dynamic GeoJSON source updates

  **Acceptance Criteria**:
  - [ ] "Create Track" button enters drawing mode
  - [ ] Each click adds a point to the new track
  - [ ] In-progress track rendered as line + point markers
  - [ ] Double-click/Enter/Done exits drawing mode
  - [ ] Escape cancels and removes the track
  - [ ] Drawing mode is exclusive (other interactions disabled)
  - [ ] `npm test` passes

  **QA Scenarios**:

  ```
  Scenario: Draw new track
    Tool: Playwright
    Preconditions: App running with empty project
    Steps:
      1. Click "Create Track" button
      2. Assert drawing mode indicator visible
      3. Click 5 different positions on map
      4. Assert 5 point markers visible and connected by line
      5. Click "Done" button
      6. Assert track appears in TracksPanel with 5 points
    Expected Result: New track created with 5 points from map clicks
    Evidence: .sisyphus/evidence/task-38-draw-track.png

  Scenario: Cancel drawing mode
    Tool: Playwright
    Steps:
      1. Click "Create Track" button
      2. Click 3 positions on map
      3. Press Escape
      4. Assert no new track in TracksPanel
      5. Assert drawing mode indicator gone
    Expected Result: Cancelled drawing creates nothing
    Evidence: .sisyphus/evidence/task-38-cancel.png
  ```

  **Commit**: YES
  - Message: `feat(ui): add map drawing mode for new tracks`
  - Files: `src/components/MapView.svelte`, toolbar component
  - Pre-commit: `npm test`

---

## Final Verification Wave

> 4 review agents run in PARALLEL. ALL must APPROVE. Present consolidated results to user and get explicit "okay" before completing.

- [x] F1. **Plan Compliance Audit** — `oracle` — APPROVE (0 suppressions, 160/160 tests, 4 error-case tests)
  Read the plan end-to-end. For each "Must Have": verify implementation exists (read file, run command). For each "Must NOT Have": search codebase for forbidden patterns — reject with file:line if found. Check evidence files exist in `.sisyphus/evidence/`. Compare deliverables against plan.
  Output: `Must Have [N/N] | Must NOT Have [N/N] | Tasks [N/N] | VERDICT: APPROVE/REJECT`

- [x] F2. **Code Quality Review** — `unspecified-high` — APPROVE (clean clippy, ? operator throughout, console.error accepted)
  Run `cargo clippy --all-targets --all-features -- -D warnings` + `cargo test --all` + `npm test`. Review all changed files for: `as any`/`@ts-ignore`, empty catches, `console.log` in prod, commented-out code, unused imports, `.lock().unwrap()`. Check AI slop: excessive comments, over-abstraction, generic names.
  Output: `Build [PASS/FAIL] | Clippy [PASS/FAIL] | Tests [N pass/N fail] | Files [N clean/N issues] | VERDICT`

- [x] F3. **Real Manual QA** — `unspecified-high` — APPROVE (8/8 scenarios pass, all edge cases clean)
  Start from clean state. Execute EVERY QA scenario from EVERY task — follow exact steps, capture evidence. Test cross-task integration (features working together). Test edge cases: empty project, track with 0 points, waypoint at map edge. Save to `.sisyphus/evidence/final-qa/`.
  Output: `Scenarios [N/N pass] | Integration [N/N] | Edge Cases [N tested] | VERDICT`

- [x] F4. **Scope Fidelity Check** — `deep` — APPROVE (8/8 tasks compliant, glue files accepted, no creep)
  For each task: read "What to do", read actual diff (`git log`/`git diff`). Verify 1:1 — everything in spec was built (no missing), nothing beyond spec was built (no creep). Check "Must NOT do" compliance. Detect cross-task contamination. Flag unaccounted changes.
  Output: `Tasks [N/N compliant] | Contamination [CLEAN/N issues] | Unaccounted [CLEAN/N files] | VERDICT`

---

## Commit Strategy

| Wave | Commits | Pattern |
|------|---------|---------|
| 0 | 1 per bug fix | `fix(scope): description` |
| 1 | 1 per domain module | `feat(domain): add TrackSegment point mutations` |
| 1 | 1 for delta undo | `refactor(commands): migrate to delta-based undo/redo` |
| 2 | 1 per command | `feat(commands): add MoveTrackPoint with drag coalescing` |
| 3 | 1 per handler batch | `feat(tauri): add track editing handlers` |
| 4 | 1 per component | `feat(ui): add track point list panel` |
| 5 | 1 per interaction | `feat(ui): add map track point drag editing` |
| 6 | 1 for PLT export | `feat(export): add PLT export with round-trip tests` |
| 7 | 1 per task | `feat(commands): add CreateEmptyTrack` |

Pre-commit gate: `cargo test --all && cargo clippy --all-targets --all-features -- -D warnings`

---

## Success Criteria

### Verification Commands
```bash
cargo test --all                    # Expected: 0 failures, ~120+ tests
cargo clippy --all-targets --all-features -- -D warnings  # Expected: 0 warnings
npm test                            # Expected: 0 failures
```

### Final Checklist
- [ ] All 9 new ProjectCommand variants implemented and tested
- [ ] Delta-based undo/redo working with drag coalescing
- [ ] Track point list panel displays segment→point hierarchy
- [ ] Point drag editing on map works with undo
- [ ] Track creation drawing mode functional
- [ ] Waypoint panel with add/delete/rename/drag/symbol
- [ ] Douglas-Peucker simplification with preview + confirm
- [ ] PLT export produces OziExplorer-compatible files
- [ ] SetTrackLineWidth working (non-undoable)
- [ ] Zero `.lock().unwrap()` calls
- [ ] Zero clippy warnings
- [ ] All tests pass
