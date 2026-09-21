## ADDED Requirements

### Requirement: Distance can be measured on the map

The operator SHALL be able to measure a distance on the map by clicking points,
and SHALL see the running total while they do. This is the measurement a search
crew takes constantly, and without it they measure by eye.

The tool SHALL be reachable from the command palette, SHALL be dismissable with
Escape, and SHALL take a click whole while it is active so that measuring does
not also select or place something.

Measured points SHALL NOT be part of the project: they are scratch, discarded
when the tool is switched off. A measurement worth keeping is a track.

A measured distance SHALL be shown in metres below a kilometre.

The points clicked SHALL be drawn on the map, joined in the order they were
clicked and distinguishable from anything belonging to the project, so that the
operator can see where each click landed and whether it landed at all. The last
point SHALL be removable without ending the measurement.

#### Scenario: Measuring a leg

- **WHEN** the operator turns the tool on and clicks two points on the map
- **THEN** the distance between them is shown, in metres if it is under a kilometre

#### Scenario: A misclick

- **WHEN** the operator removes the last measured point
- **THEN** it is gone from the map, the total is recomputed, and the measurement continues

#### Scenario: Finishing

- **WHEN** the operator presses Escape while measuring
- **THEN** the tool is off and nothing it measured remains

#### Scenario: A click while measuring

- **WHEN** the operator clicks the map while measuring
- **THEN** a point is added and no track is selected and no waypoint is placed

### Requirement: A radius ring can be drawn on the map

The operator SHALL be able to draw a ring of a chosen radius around a chosen
point, by placing a centre and then setting the radius, and SHALL see the
radius while they do. A search works in rings around a last known position.

The ring SHALL be geodesic: every point on it SHALL be the stated distance from
the centre on the ground, not in screen pixels. A ring drawn flat is wrong
everywhere but the equator and worse the further north the search is.

The ring SHALL be scratch, like a measurement, and SHALL NOT be part of the
project.

Only one on-map tool SHALL be listening for clicks at a time, and switching
tools SHALL discard what the previous one held.

#### Scenario: A ring around the last known position

- **WHEN** the operator places a centre and then sets a radius
- **THEN** a ring of that radius is drawn, and every point on it is that distance from the centre

#### Scenario: Switching tools

- **WHEN** the operator turns on one on-map tool while another is active
- **THEN** the other is off and what it held is discarded

### Requirement: A waypoint can be placed by bearing and distance

The operator SHALL be able to place a waypoint by choosing a point on the map
and entering a bearing and a distance from it. A position given over a radio
arrives as an azimuth and a range, not as somewhere to point at.

Where the waypoint will land SHALL be shown on the map before it is placed, so
that a misheard bearing is caught before it becomes a mark, and the placement
SHALL NOT be offered until there is a distance to place at.

#### Scenario: A position given over the radio

- **WHEN** the operator chooses a point, enters a bearing and a distance, and confirms
- **THEN** a waypoint is placed at that bearing and distance from the chosen point

#### Scenario: Before confirming

- **WHEN** a bearing and a distance have been entered
- **THEN** the resulting position is shown on the map before the waypoint exists
