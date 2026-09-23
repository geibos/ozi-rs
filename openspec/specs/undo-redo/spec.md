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

The system SHALL provide `undo` and `redo` operations. The reverse command of a delta SHALL be computed by `ProjectCommand::reverse()` against the project state immediately before the forward command mutates it. `undo` SHALL pop the delta at the top of the undo stack, apply its reverse command and push the delta onto the redo stack; `redo` SHALL pop the delta at the top of the redo stack, apply its forward command and push the delta back onto the undo stack. If applying the reverse or forward command fails, the delta SHALL be returned to the stack it was popped from, the project SHALL stay unchanged and the operation SHALL report failure. `undo` and `redo` on an empty stack SHALL be no-ops that report failure.

#### Scenario: Undo then redo is identity

- **WHEN** the user edits a track, then invokes undo, then redo
- **THEN** the final project state equals the state immediately after the edit

#### Scenario: Reverse captures the pre-edit state

- **WHEN** a waypoint at coordinates A is moved to B
- **THEN** the stored reverse is a move back to A, taken from the project before the move was applied

#### Scenario: Undo on an empty stack is a no-op

- **WHEN** the user invokes undo with no undoable edits recorded
- **THEN** the project is unchanged and `can_undo()` stays false

#### Scenario: Failed inverse leaves history intact

- **WHEN** the reverse command at the top of the undo stack cannot be applied
- **THEN** the delta remains on the undo stack, the redo stack is unchanged and the project is unchanged

### Requirement: Issuing a new command clears the redo stack

The system SHALL clear the redo stack whenever a new (non-undo, non-redo) command is applied.

#### Scenario: Redo lost after a new edit

- **WHEN** the user undoes an edit, then makes a different edit
- **THEN** redo is no longer available for the originally undone edit

### Requirement: Drag operations coalesce into a single undo step

The system SHALL route every command through `apply_or_merge()` (`CommandStack::apply` delegates to it). When the new command targets the same entity as the most recent undo entry — `MoveTrackPoint` on the same layer, track, segment and point; `MoveWaypoint` on the same layer and waypoint; `RenameTrack` on the same layer and track — the system SHALL apply the command and replace the entry's forward command while keeping the entry's original reverse, so the whole sequence is one undo step. Commands that target different entities SHALL create separate undo entries. Merging SHALL still clear the redo stack.

#### Scenario: Track point drag

- **WHEN** the user drags a track point through ten intermediate positions and releases
- **THEN** a single undo step reverts the entire drag to the pre-drag coordinates

#### Scenario: Waypoint drag

- **WHEN** the user drags a waypoint and releases
- **THEN** a single undo step reverts the move

#### Scenario: Moves of two different points are two steps

- **WHEN** the user drags point A and then drags point B of the same segment
- **THEN** the first undo reverts only B and the second undo reverts A

#### Scenario: Consecutive renames of one track collapse

- **WHEN** the user renames a track twice in a row without any other edit in between
- **THEN** a single undo restores the name the track had before the first rename

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

### Requirement: One gesture is one undo step

Commands SHALL be merged into a single undo step only when the caller states
that they belong to the same gesture — one continuous action as the operator
would describe it — and they touch the same entity. A command with no gesture
SHALL be its own undo step.

#### Scenario: The same point dragged twice

- **WHEN** a point is dragged, released, and dragged again
- **THEN** undo takes back the second drag and leaves the first

#### Scenario: A continuous drag

- **WHEN** one drag produces several commands under the same gesture
- **THEN** they form a single undo step

### Requirement: A refused command leaves the redo stack alone

The redo stack SHALL be cleared only after a command has been applied. A
command the system refuses SHALL leave the history exactly as it was.

#### Scenario: An edit the system declines

- **WHEN** a command fails to apply after something was undone
- **THEN** redo is still available and still holds what it held

### Requirement: Removing a layer is undoable with its contents

Undoing the removal of a layer SHALL restore the layer together with
everything it held. Restoring the layer's name alone is a silent loss of the
work inside it.

#### Scenario: A layer of forty tracks removed and taken back

- **WHEN** a track layer holding forty tracks is removed and the operator undoes it
- **THEN** the layer is back with all forty tracks, their names, geometry and styles

#### Scenario: A waypoint layer removed and taken back

- **WHEN** a waypoint layer holding marks is removed and the operator undoes it
- **THEN** the layer is back with every mark, its symbol and its colour

### Requirement: Renaming a layer is undoable

A layer's name SHALL change through the command stack, so that renaming can be
undone and redone like every other edit.

#### Scenario: A rename taken back

- **WHEN** a layer is renamed and the operator undoes it
- **THEN** the layer carries the name it had before

### Requirement: Renaming a layer that is not there is an error

A rename aimed at a layer the project does not hold SHALL fail rather than
report success.

A command that answers `Ok` without doing anything still clears the redo
history and counts a mutation — the operator loses a redo they had, for an
edit that never happened.

#### Scenario: A rename of a layer that has been removed

- **WHEN** a rename names a layer that is not in the project
- **THEN** the command fails and the redo history is untouched

### Requirement: Every project mutation marks the project dirty until saved

The system SHALL derive the project's dirty state structurally from two counters rather than from per-call-site flags: `CommandStack::mutation_count`, incremented by every successful apply, merge, undo and redo, and `AppState::style_revision`, incremented by every non-undoable style or visibility setter — `set_track_color`, `set_track_line_width`, `toggle_track_visible`, `set_all_tracks_visible`, `show_only_track`, `toggle_waypoint_visible`, `set_all_waypoints_visible` and `show_only_waypoint`.

A waypoint's colour and symbol are not in that list and must not be: they are undoable `ProjectCommand`s, so they reach the dirty state through `mutation_count` like any other edit. A setter that appears in both would count twice, which is harmless for a flag and misleading for anyone reading it. `project_dirty()` SHALL be true whenever the counter pair differs from the pair recorded at the last successful save, load or session restore. A failed save SHALL NOT clear the dirty state.

#### Scenario: Undoable edit dirties a clean project

- **WHEN** the project has just been saved and the user moves a track point
- **THEN** `project_dirty()` is true

#### Scenario: Undo after a save dirties the project again

- **WHEN** the user saves the project and then invokes undo
- **THEN** `project_dirty()` is true even though no new command was issued

#### Scenario: Non-undoable style change dirties the project

- **WHEN** the user changes a track colour or toggles a waypoint's visibility on a clean project
- **THEN** `project_dirty()` is true and the undo depth is unchanged

#### Scenario: Successful save clears, failed save keeps

- **WHEN** the user saves a dirty project successfully
- **THEN** `project_dirty()` is false
- **WHEN** a later save fails (for example the target directory is not writable)
- **THEN** `project_dirty()` stays true

### Requirement: Undo and redo never duplicate or reassign identifiers

The system SHALL keep every `TrackPointId`, `TrackSegmentId`, `TrackId` and `WaypointId` unique within a project across any sequence of apply, undo and redo. A reverse command SHALL restore a removed entity with its original identifier at its original position, and SHALL remove an inserted entity by the identifier allocated at apply time. Undo and redo SHALL NOT allocate new identifiers; fresh identifiers are allocated only when the forward command is first built (`max + 1` within the owning track).

#### Scenario: Split, undo, redo, undo leaves every point exactly once

- **WHEN** a segment holds points `10, 11, 12`, the user splits it at point `11`, then invokes undo, redo, undo
- **THEN** the track has one segment whose point ids are `10, 11, 12` in order and no id appears twice

#### Scenario: Deleted point returns with the same id and index

- **WHEN** the user deletes the second point of a segment and invokes undo
- **THEN** a point with the original id is present again at index 1

#### Scenario: Inserted point is removed by its allocated id

- **WHEN** the user inserts a point (allocated id `max + 1`), invokes undo, then redo
- **THEN** after undo no point with that id exists, and after redo the point reappears with the same id

#### Scenario: Crop and simplify undo restore original ids and positions

- **WHEN** the user crops or simplifies a track and invokes undo
- **THEN** every removed point is back with its original id at its original index

### Requirement: Undo and redo are reachable through global keyboard chords

The system SHALL bind Cmd/Ctrl+Z to undo and Cmd/Ctrl+Shift+Z to redo on the window in the root layout, so the chords work on every route, and SHALL call `preventDefault` so the WebView does not treat them as text-editing shortcuts. The chords SHALL be ignored while keyboard focus is inside an editable element and while Alt is held. Undo and redo SHALL also remain available from the command palette and the shell's Undo/Redo buttons.

#### Scenario: Cmd+Z reverts the last edit from the map

- **WHEN** the user moves a track point and presses Cmd/Ctrl+Z with focus on the map canvas
- **THEN** the point returns to its previous coordinates

#### Scenario: Cmd+Shift+Z re-applies the undone edit

- **WHEN** the user presses Cmd/Ctrl+Shift+Z right after the undo above
- **THEN** the point is back at the moved coordinates

#### Scenario: Chords do not fire inside a text field

- **WHEN** the user presses Cmd/Ctrl+Z while typing in a rename input
- **THEN** no project undo happens and the input handles the keystroke
