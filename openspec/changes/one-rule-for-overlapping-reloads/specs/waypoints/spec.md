## ADDED Requirements

### Requirement: Only the newest waypoint-marker refresh may draw

The map SHALL draw the marker set of the most recently started refresh only,
and SHALL discard an earlier refresh's result, so that a waypoint the operator
has just added or moved is not undone by a refresh that started before it.
Refreshes overlap because the map refreshes both on state changes and on every
waypoint action.

The map SHALL request every visible layer's waypoints concurrently rather than
one layer after another.

#### Scenario: A waypoint added while a refresh is in flight

- **WHEN** a waypoint is added while an earlier marker refresh is still running
- **AND** that earlier refresh completes afterwards
- **THEN** the markers on the map include the new waypoint
