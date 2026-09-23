## ADDED Requirements

### Requirement: A name sits on a line the crew actually walked

A track's name SHALL be placed half way along the longest segment of that
track, not half way along its segments taken as one run.

A track is split where the recording stopped. Half the combined length falls
in the gap whenever the two stretches are far apart, and a name floating over
empty forest belongs to nothing.

#### Scenario: A track recorded in two stretches a kilometre apart

- **WHEN** a track has two segments with a kilometre of nothing between them
- **THEN** its name sits on one of the segments, not between them

### Requirement: Which name survives is decided by distance walked

When names compete for room, the system SHALL keep the selected track's name
first and then the name of the track whose crew walked furthest, measured in
distance rather than in the number of recorded points.

A navigator logging once a second at a rest stop records more points than a
long route logged once a minute. Whose callsign stays on the map must not be
decided by how often somebody's GPS wrote a line.

#### Scenario: A rest stop against a long route

- **WHEN** a short track with a thousand crowded points competes with a long track with two
- **THEN** the long track keeps its name

### Requirement: Names are kept apart by the room they take

Overlap SHALL be judged by the space each name occupies on screen, not by the
distance between their anchor points.

#### Scenario: Two long names sixty pixels apart

- **WHEN** two long names' anchors are further apart than the minimum spacing but their text overlaps
- **THEN** only one of them is drawn
