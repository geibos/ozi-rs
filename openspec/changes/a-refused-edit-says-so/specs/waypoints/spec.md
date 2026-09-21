## ADDED Requirements

### Requirement: A refused waypoint edit tells the operator

Adding or moving a waypoint that the system declines SHALL be reported to the
operator with the reason, rather than only to a developer console. A failure to
load a waypoint for inspection SHALL likewise be reported instead of leaving an
empty pane.

#### Scenario: Placing a waypoint that is declined

- **WHEN** adding a waypoint by clicking the map is declined
- **THEN** the operator is told, with the reason

#### Scenario: A waypoint that cannot be loaded for inspection

- **WHEN** loading a waypoint's detail fails
- **THEN** the operator is told rather than shown an empty inspector
