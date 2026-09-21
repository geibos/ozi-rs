## ADDED Requirements

### Requirement: Only the newest track-geometry fetch may be drawn

The map SHALL draw the geometry of the most recently started fetch only, and
SHALL discard an earlier fetch's result, so that the map does not fall behind
the track list it is meant to match. Two changes in quick succession leave two
fetches in flight.

#### Scenario: Two track edits in quick succession

- **WHEN** a second track-geometry fetch is started before the first answers
- **AND** the first answers last
- **THEN** the map draws the second fetch's geometry
