## ADDED Requirements

### Requirement: Every click while drawing becomes a point

While drawing a track, each click on the map SHALL add exactly one point,
however quickly the clicks follow one another. A click SHALL NOT be discarded
because another click arrived, and points SHALL be added in the order they were
clicked.

A double-click SHALL finish the track and SHALL NOT add either of its own
clicks as a point.

A point the backend refuses SHALL be reported and SHALL NOT stop the points
clicked after it.

#### Scenario: A route plotted at speed

- **WHEN** the operator clicks six places on the map 90 ms apart
- **THEN** the track has six points, in the order they were clicked

#### Scenario: Finishing with a double-click

- **WHEN** the operator double-clicks to finish
- **THEN** the track is finished and neither half of the double-click became a
  point

#### Scenario: One point that could not be added

- **WHEN** one click in a burst fails to reach the project
- **THEN** the failure is reported and the clicks after it still become points
