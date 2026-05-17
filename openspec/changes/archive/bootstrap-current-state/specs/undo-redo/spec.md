## ADDED Requirements

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

The system SHALL NOT route `set_track_color`, `set_track_line_width`, or `toggle_track_visible` through the undo stack. These mutations SHALL apply immediately and SHALL NOT add undo steps.

#### Scenario: Color change is not undoable

- **WHEN** the user changes a track's color and then invokes undo
- **THEN** undo reverts the most recent undoable edit (not the color change) and the color stays as set

#### Scenario: Visibility toggle is not undoable

- **WHEN** the user toggles a track's visibility off and invokes undo
- **THEN** undo reverts the most recent undoable edit (not the visibility toggle)

### Requirement: Undo history is not persisted across app restarts

The system SHALL NOT persist the undo or redo stack between application sessions. On startup the stacks SHALL be empty regardless of restored project content.

#### Scenario: Restart clears undo

- **WHEN** the user makes edits, closes the app, and reopens
- **THEN** the project content reflects the edits but the undo stack is empty

## References

- ADR-0005: Snapshot-Based Undo/Redo (superseded)
- ADR-0017: Delta-Based Undo/Redo
- ADR-0021: Undo Stack — Reaffirm Delta-Based, Depth 100, No Hybrid
