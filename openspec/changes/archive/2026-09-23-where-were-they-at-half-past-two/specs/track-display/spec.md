## ADDED Requirements

### Requirement: A recording can be played back against its own clock

For a track whose points carry timestamps, the system SHALL offer a replay: a
control spanning the track's first and last recorded moment, the moment shown
as a time of day, and a marker on the map at the position the track gives for
that moment. The system SHALL also be able to advance the moment on its own,
fast enough that a day's walk plays in about a minute.

Between two recorded points the position SHALL be interpolated by time, not
snapped to the nearer point.

A moment falling in a recorded silence longer than five minutes SHALL be shown
as such, because the straight line across a gap the navigator did not record is
a guess and a position read off it must not look like a fix.

Outside the recording the system SHALL answer the nearest end rather than
nothing.

A track whose points carry no times SHALL say there is nothing to play back.

#### Scenario: Where the crew was at a given moment

- **WHEN** the operator moves the replay control to half past two
- **THEN** the moment reads `14:30:00` and a marker stands where the track puts
  the crew then

#### Scenario: Between two recorded points

- **WHEN** the moment falls half way between a point at 14:00 and one at 14:01
- **THEN** the position is half way between them, not at either

#### Scenario: A silence

- **WHEN** the moment falls inside a gap of half an hour between two points
- **THEN** the interface says the position there is across a gap, and the
  marker is drawn differently

#### Scenario: A recording with no times

- **WHEN** the selected track's points carry no timestamps
- **THEN** the interface says there is nothing to play back and offers no
  control
