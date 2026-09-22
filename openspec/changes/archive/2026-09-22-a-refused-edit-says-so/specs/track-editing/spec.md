## ADDED Requirements

### Requirement: A refused edit tells the operator

An edit that the system declines SHALL be reported to the operator, carrying
the reason the backend gave, rather than only being written to a developer
console which is not present in a release build. This covers moving, deleting
and inserting a track point, placing a point while drawing, cancelling a draw,
and bringing the tracks into view.

Where the refused edit has already been shown on the map, the map SHALL be
brought back into agreement with the stored data rather than continuing to
show the edit that did not happen.

#### Scenario: A track point dragged to somewhere the system refuses

- **WHEN** a track point is dragged and the move is declined
- **THEN** the operator is told, with the reason, and the point is shown at its stored position

#### Scenario: Drawing a point that cannot be placed

- **WHEN** placing a point while drawing is declined
- **THEN** the operator is told, with the reason
