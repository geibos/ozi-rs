# undo-redo Specification

## Purpose
Undo and redo for every project edit in the Rust core: the `ProjectCommand` vocabulary, the delta-based `CommandStack` (forward + reverse pairs, depth 100, drag coalescing, redo clearing), the small set of style and visibility setters that deliberately bypass the stack, the structural dirty flag derived from the stack's mutation counter, identifier stability across undo/redo, and the global keyboard chords that reach undo/redo from any route. Persistence of history across restarts is explicitly out of scope (see `project-persistence`).

### Decision history

- ADR-0005 (2026-03-23, superseded by ADR-0017): undo/redo by cloning the whole `Project` before every command (`Vec<Project>` snapshots); rationale: trivially correct, no hand-written inverses, memory cost accepted at that stage. Superseded by ADR-0017 — not codified.
- ADR-0017 (2026-04-04, accepted): delta-based stack of `CommandDelta { forward, reverse }`, reverse computed by `ProjectCommand::reverse()` against the pre-mutation project, `apply_or_merge()` coalescing of same-entity commands, `MAX_STACK_DEPTH = 100`, style setters (`SetTrackColor`, `SetTrackLineWidth`, `ToggleTrackVisible`) bypass the stack; rationale: drags produced dozens of full-project clones per second and memory grew with project size instead of command count. Codified as: All non-trivial edits flow through `ProjectCommand`; Command stack is delta-based with maximum depth 100; Undo and redo apply the stored inverse and forward commands; Issuing a new command clears the redo stack; Drag operations coalesce into a single undo step; Track style mutations bypass the command stack. Code is broader than the ADR text: `CommandStack::apply` delegates to `apply_or_merge` (`src-tauri/src/application/commands.rs:1300-1306`), so `MoveWaypoint` and `RenameTrack` coalesce too, and `toggle_waypoint_visible` joined the non-undoable set (`src-tauri/src/application/mod.rs:715`).
- ADR-0021 (2026-04-28, accepted): keep ADR-0017 as-is for MVP and beyond — depth stays 100, no hybrid/thinned history; rationale: SAR sessions produce far fewer than 100 distinct actions, drags already collapse into one delta, deltas are cheap next to the tile cache, and thinning would break the invariant that any stack prefix composes to a valid state. Codified as: Command stack is delta-based with maximum depth 100 (the rejection of thinning is a non-decision and adds no requirement).
- Commit f5f44bb (2026-07-14, landed): `SplitSegment`/`JoinSegments` made exact inverses — split no longer copied the split point into both halves, so each undo/redo cycle had been adding a phantom `TrackPoint` with a duplicate id that leaked into `.ozp` and GPX/PLT exports; join rejects empty segments so its reverse is always well-defined. Codified as: Undo and redo never duplicate or reassign identifiers.
- CJ-7 slice 1.1, commit c60dc6e (2026-07-15, landed; `docs/customer-journeys.md`): dirty flag derived structurally from `CommandStack::mutation_count` (apply, merge, undo, redo) plus `AppState::style_revision` (non-undoable setters), cleared only by successful save/load/restore; Cmd/Ctrl+Z and Cmd/Ctrl+Shift+Z bound globally in the root layout; rationale: field operators must never lose an hour of work to a missed dirty flag or an unreachable undo. Codified as: Every project mutation marks the project dirty until saved; Undo and redo are reachable through global keyboard chords. Note: `docs/frontend-architecture.md` still claims there are no Ctrl+Z bindings; the code (`src/routes/+layout.svelte:165-185`) is authoritative.
## Requirements
### Requirement: All non-trivial edits flow through `ProjectCommand`

The system SHALL express all non-trivial project edits — track CRUD and geometry, waypoint CRUD and symbol changes, drawing, simplification, layer CRUD — as variants of `ProjectCommand`. Each command SHALL validate inputs before applying and SHALL produce a computed inverse for undo.

#### Scenario: New point added via command

- **WHEN** the user inserts a track point in edit mode
- **THEN** an `InsertTrackPoint` command is recorded with the data needed to reverse the insertion

### Requirement: Command stack is delta-based with maximum depth 100

The system SHALL store applied commands as a `CommandDelta` (forward + reverse pair) on a bounded stack of depth 100. When the stack exceeds 100 entries the oldest entry SHALL be discarded.

#### Scenario: Long undo chain truncates at the oldest entry

- **WHEN** the user performs 101 undoable edits in one session
- **THEN** undo can step back through the most recent 100 edits and the 101st (oldest) is no longer reachable

### Requirement: Undo and redo apply the stored inverse and forward commands

The system SHALL provide `undo` and `redo` operations. `undo` SHALL apply the reverse command at the top of the undo stack and push it onto the redo stack; `redo` SHALL apply the forward command at the top of the redo stack and push it back onto the undo stack.

#### Scenario: Undo then redo is identity

- **WHEN** the user edits a track, then invokes undo, then redo
- **THEN** the final project state equals the state immediately after the edit

### Requirement: Issuing a new command clears the redo stack

The system SHALL clear the redo stack whenever a new (non-undo, non-redo) command is applied.

#### Scenario: Redo lost after a new edit

- **WHEN** the user undoes an edit, then makes a different edit
- **THEN** redo is no longer available for the originally undone edit

### Requirement: Drag operations coalesce into a single undo step

The system SHALL detect consecutive commands that target the same entity (e.g. successive `MoveTrackPoint` for the same point during a drag) and SHALL coalesce them into a single undo step via `apply_or_merge()`.

#### Scenario: Track point drag

- **WHEN** the user drags a track point through ten intermediate positions and releases
- **THEN** a single undo step reverts the entire drag to the pre-drag coordinates

#### Scenario: Waypoint drag

- **WHEN** the user drags a waypoint and releases
- **THEN** a single undo step reverts the move

### Requirement: Track style mutations bypass the command stack

The system SHALL NOT route `set_track_color`, `set_track_line_width`, `toggle_track_visible`, or `toggle_waypoint_visible` through the undo stack. These mutations SHALL apply immediately and SHALL NOT add undo steps.

#### Scenario: Color change is not undoable

- **WHEN** the user changes a track's color and then invokes undo
- **THEN** undo reverts the most recent undoable edit (not the color change) and the color stays as set

#### Scenario: Visibility toggle is not undoable

- **WHEN** the user toggles a track's visibility off and invokes undo
- **THEN** undo reverts the most recent undoable edit (not the visibility toggle)

#### Scenario: Waypoint visibility toggle is not undoable

- **WHEN** the user toggles a waypoint's visibility off and invokes undo
- **THEN** undo reverts the most recent undoable edit (not the waypoint visibility toggle) and the waypoint stays hidden

### Requirement: Undo history is not persisted across app restarts

The system SHALL NOT persist the undo or redo stack between application sessions. On startup the stacks SHALL be empty regardless of restored project content.

#### Scenario: Restart clears undo

- **WHEN** the user makes edits, closes the app, and reopens
- **THEN** the project content reflects the edits but the undo stack is empty

