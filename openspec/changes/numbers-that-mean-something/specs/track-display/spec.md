## MODIFIED Requirements

### Requirement: System computes and surfaces per-track statistics

The system SHALL compute, for each track, total distance in kilometres, total duration when point timestamps are present, and total point count, and SHALL surface these in the track row UI.

A track holding fewer than two points SHALL surface its point count alone. It has no length and no elapsed span, so a distance and a duration for it are not measurements of anything: the row read "0.0 км · 0мин · 1 тчк", two such numbers standing in front of the one that means something. A point count of one SHALL use the singular unit where the language has one.

This last part is new because such tracks had no row at all until the track list stopped being derived from the map's geometry, which omits what it cannot draw.

#### Scenario: Track with timestamps

- **WHEN** a track has GPS timestamps on its points
- **THEN** the track row shows distance in km, duration, and point count

#### Scenario: Track without timestamps

- **WHEN** a track has no point timestamps
- **THEN** the track row shows distance and point count; duration is omitted

#### Scenario: Track row shows distance, duration, and point count

- **WHEN** the user opens a project containing a track with three timestamped segments totalling 12.3 km, 1 hour 24 minutes, and 156 points
- **THEN** the Tracks panel row for that track displays distance "12.3 km", duration "1h 24m", and point count "156 pts" alongside the track name

#### Scenario: Track without timestamps hides duration

- **WHEN** the user displays a track whose points have no timestamps
- **THEN** the Tracks panel row shows distance and point count but omits the duration field

#### Scenario: A track of a single point

- **WHEN** the track list shows a track whose only segment holds one point
- **THEN** its statistics read as a point count alone, without a distance or a duration
