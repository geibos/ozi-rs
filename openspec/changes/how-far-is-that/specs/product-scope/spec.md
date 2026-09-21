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
