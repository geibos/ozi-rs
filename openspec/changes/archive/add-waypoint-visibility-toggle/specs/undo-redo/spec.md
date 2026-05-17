## MODIFIED Requirements

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
