## ADDED Requirements

### Requirement: A track carries its name on the map

Every visible track that has a name SHALL show that name on the map, drawn in
the track's own colour, positioned at the point half way along the route
measured by length.

A day of recordings is a dozen coloured lines. Without names the only way to
find one is to read its colour off the list and hunt.

The position SHALL be measured by distance rather than by point index: a crew
that stopped for twenty minutes logs a crowd of points at the rest stop, and
an index midpoint puts the name there rather than on the route.

#### Scenario: A day's routes on the map

- **WHEN** the operator looks at a day of imported tracks
- **THEN** each visible track has its name on it, in its own colour

#### Scenario: A track that stopped for a break

- **WHEN** a track's points are crowded at one end and sparse across the rest
- **THEN** its name sits half way along the distance walked, not among the crowded points

### Requirement: Names that do not fit are dropped, not stacked

Names that would overlap on screen SHALL be reduced to those that fit. A
selected track SHALL keep its name; otherwise the longer route SHALL keep it.

Below the zoom at which a whole district fits on screen, no names SHALL be
drawn: at that scale they are a smear rather than information.

#### Scenario: Two tracks crossing at the same place

- **WHEN** two tracks' midpoints fall within a name's width of each other
- **THEN** one name is drawn, not two overlapping

#### Scenario: The operator selects a track in a crowd

- **WHEN** a track is selected and its name would otherwise be dropped
- **THEN** its name is the one drawn
