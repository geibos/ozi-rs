## ADDED Requirements

### Requirement: Every project mutation marks the project dirty until saved

The system SHALL derive the project's dirty state structurally from two counters rather than from per-call-site flags: `CommandStack::mutation_count`, incremented by every successful apply, merge, undo and redo, and `AppState::style_revision`, incremented by every non-undoable style or visibility setter (`set_track_color`, `set_track_line_width`, `toggle_track_visible`, `toggle_waypoint_visible`). `project_dirty()` SHALL be true whenever the counter pair differs from the pair recorded at the last successful save, load or session restore. A failed save SHALL NOT clear the dirty state.

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

## MODIFIED Requirements

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
