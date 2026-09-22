## ADDED Requirements

### Requirement: A waypoint exported to GPX and read back is the same waypoint

A waypoint written to GPX and imported again SHALL carry the same name, the
same coordinates, and the same symbol — including the absence of one, because a
waypoint that acquires a symbol on the way back puts a mark on the map that
nobody placed.

#### Scenario: Waypoints go out and come back

- **WHEN** waypoints, one with a symbol and one without, are exported to GPX and imported again
- **THEN** each carries its name, its coordinates and its symbol, and the one without a symbol still has none
