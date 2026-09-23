## ADDED Requirements

### Requirement: The mode control sets the mode and shows which one is current

The workspace SHALL offer a control naming the modes the map can be in — look,
draw, edit, measure — and that control SHALL both enter a mode and show which
one is active. Pressing the active mode SHALL leave it. No control in the
workspace SHALL carry the name of a working feature while doing nothing.

Entering a mode SHALL leave whichever mode was active, so the operator is never
in two at once.

Entering a mode SHALL be defined in one place, shared by every surface that
offers it — the mode control, the Library tabs and the command palette — so two
surfaces cannot come to mean different things by the same mode.

#### Scenario: Drawing from the mode control

- **WHEN** the operator presses the drawing mode and clicks three places on the
  map
- **THEN** the mode shows as active and three points are added to a new track

#### Scenario: Leaving by pressing the same mode

- **WHEN** the operator presses the mode that is already active
- **THEN** the workspace returns to looking at the map

#### Scenario: One mode at a time

- **WHEN** the operator is measuring and presses the editing mode
- **THEN** measuring stops and editing starts

#### Scenario: Drawing with nowhere to draw

- **WHEN** the operator asks for drawing mode with no active track layer
- **THEN** the interface says a layer is needed and no mode is entered
