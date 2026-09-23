## MODIFIED Requirements

### Requirement: System supports Douglas–Peucker simplification with live preview

The system SHALL provide a simplification action with a configurable tolerance slider. While the slider moves, the system SHALL show a live preview of the simplified track on the map. Confirming the action SHALL commit the simplification as a single undoable edit; cancelling SHALL leave the track unchanged.

The tolerance SHALL be the deviation in **metres** that the slider states, at
every layer it passes through. The simplification algorithm works in
kilometres; the conversion SHALL happen once, at the boundary, in a parameter
whose name carries the unit.

#### Scenario: Preview and commit

- **WHEN** the user opens the simplify panel, adjusts tolerance, and clicks Apply
- **THEN** the simplified geometry is committed to the project and undo restores the original points

#### Scenario: Preview and cancel

- **WHEN** the user opens the simplify panel, adjusts tolerance, and cancels
- **THEN** the original track geometry is preserved and no undo step is added

#### Scenario: The slider's number is metres

- **WHEN** the operator simplifies a track at a tolerance of 10 m, and the
  track has a 50 m deviation in it
- **THEN** that deviation is kept, because it is five times the tolerance the
  operator set

#### Scenario: A tolerance that is noise

- **WHEN** the same track is simplified at 100 m
- **THEN** the 50 m deviation is removed
