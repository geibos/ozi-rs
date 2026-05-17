## MODIFIED Requirements

### Requirement: Multiple waypoints render simultaneously with per-waypoint visibility

The system SHALL render all waypoints in visible layers simultaneously and SHALL allow toggling individual waypoint visibility; hidden waypoints SHALL NOT render on the map.

#### Scenario: Toggle a waypoint hidden

- **WHEN** the user toggles a waypoint's visibility off
- **THEN** its marker disappears from the map but the panel row remains and indicates hidden state

#### Scenario: Toggle visibility off and back on

- **WHEN** the user toggles a waypoint's visibility off via the Waypoints panel and then toggles it on again
- **THEN** the marker disappears from the map after the first toggle and reappears unchanged after the second; the panel row remains visible during both states

#### Scenario: Visibility toggle is not undoable

- **WHEN** the user toggles a waypoint's visibility off and then invokes undo
- **THEN** undo reverts the most recent undoable edit (not the visibility toggle) and the waypoint stays hidden
