## ADDED Requirements

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
