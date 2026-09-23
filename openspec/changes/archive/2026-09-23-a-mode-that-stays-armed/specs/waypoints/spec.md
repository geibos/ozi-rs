## ADDED Requirements

### Requirement: Placing marks is a mode the operator leaves, not one that leaves them

While the add-waypoint mode is active, every click on the map SHALL place a
mark, and the mode SHALL remain active until the operator leaves it. A mark
that fails to be placed SHALL NOT leave the mode either — the operator is still
placing marks, and one that did not land is a reason to try again rather than a
reason to be put back where they started.

The mode SHALL be left by pressing Escape, by pressing the mode control again,
or by starting to draw a track. Panning or zooming the map SHALL NOT leave it.

#### Scenario: Ten marks for the next outing

- **WHEN** the operator arms the mode and clicks three places on the map
- **THEN** three marks are placed and the mode is still armed

#### Scenario: A mark that could not be placed

- **WHEN** placing a mark fails
- **THEN** the failure is reported and the mode is still armed

#### Scenario: Leaving the mode

- **WHEN** the operator presses Escape, or presses the mode control again, or
  starts drawing a track
- **THEN** the mode is no longer armed and a click on the map places nothing
