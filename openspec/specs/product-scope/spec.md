# product-scope Specification

## Purpose
What the product includes and excludes, independent of implementation status:
the LizaAlert search-and-rescue workflow (open a bundle → import or draw tracks
→ edit → place waypoints → save `.ozp` → export GPX/PLT/WPT), the field tools a
crew reaches for on the map, the supported platforms and the explicit
non-goals. Behaviour lives in the other capabilities; this one answers "is X
something we build at all?".

Decision history:

- `how-far-is-that` (2026-09-22): three on-map tools — a distance measured by
  clicking, a geodesic radius ring and a waypoint placed by bearing and
  distance; rationale: a search crew takes these measurements constantly, and
  without them they measure by eye. Reachable from the command palette rather
  than the toolbar, because `ui-shell` still requires the mode chips that would
  otherwise be the place for them. Codified as the three requirements below;
  their rendered pixels were verified on the stand on 2026-09-22
  (`docs/progress/2026-09-22-verification/`).
- The scope statements drawn from ADR-0020, ADR-0022 and ADR-0023 — the MVP
  workflow, WPT export in and WPT import out, no map printing — arrive with
  `codify-architecture-decisions`, together with the owner's 2026-09-19
  decision that macOS is primary while they are the only user.

## Requirements

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

### Requirement: Length and area are measured by separate tools

The system SHALL provide measuring a length and measuring an area as two
tools, not as one readout carrying both numbers.

Measuring a length is what a crew does most often — how long is this ride,
how far from the road to the stream — and for an open path the enclosed area
is not a small number but a meaningless one. A number shown in a place where
people read numbers gets read.

The length tool SHALL report the length of the clicked path and nothing else.
The area tool SHALL report what the points enclose, with the perimeter beside
it, because a sector is described by both: "прочесать 2.4 км², обойти по
кромке 6 км". Its outline SHALL be drawn closed, so the shape agrees with the
number.

#### Scenario: Measuring how far it is

- **WHEN** the operator measures a path with the length tool
- **THEN** the readout carries the length alone

#### Scenario: Measuring a sector

- **WHEN** the operator clicks round a sector with the area tool
- **THEN** the readout carries the enclosed area and the perimeter, and the outline on the map is closed

#### Scenario: Too few points to enclose anything

- **WHEN** fewer than three points have been placed with the area tool
- **THEN** no area is claimed

### Requirement: A radius ring reports the ground it covers

The radius ring SHALL report the area it encloses beside its radius.

A ring is drawn to say "everything within five hundred metres of the last
known position", and the next question a coordinator asks is how much ground
that is.

#### Scenario: A ring around the last known position

- **WHEN** the operator sets a ring's radius
- **THEN** the readout carries both the radius and the enclosed area
