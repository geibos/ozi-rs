## ADDED Requirements

### Requirement: A track with nothing to measure states only its point count

A track SHALL NOT be shown with a distance or a duration when it holds fewer
than two points, because neither is a measurement of anything: it SHALL state
how many points it has and nothing else. A point count of one SHALL use the
singular unit where the language has one.

#### Scenario: A track of a single point

- **WHEN** the track list shows a track whose only segment holds one point
- **THEN** its statistics read as a point count alone, without a distance or a duration
