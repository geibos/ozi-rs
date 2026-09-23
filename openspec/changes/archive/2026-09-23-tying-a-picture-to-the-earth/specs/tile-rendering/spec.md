## ADDED Requirements

### Requirement: A picture with no calibration can be tied to the Earth

The system SHALL let the operator calibrate an image that has no `.map` beside
it, by naming at least two places in the picture whose latitude and longitude
are known, and SHALL write an OziExplorer `.map` file beside the image, named
after it, before opening the pair.

The file written SHALL be a valid OziExplorer `.map` — Windows-1251, CRLF, the
calibration points in degrees and decimal minutes with hemisphere letters, and
the four corner entries — so that the same folder opens in OziExplorer itself.

A calibration the system cannot solve SHALL be refused with the reason and no
file written: fewer than two points, or points that all share a latitude or all
share a longitude, or a coordinate that is not on the Earth.

#### Scenario: A screenshot becomes a map

- **WHEN** the operator chooses a picture, gives the coordinates of its
  top-left and bottom-right corners, and confirms
- **THEN** a `.map` is written beside the picture, the interface names the file
  it wrote, and the map opens

#### Scenario: What was written reads back as the same places

- **WHEN** a `.map` written by calibration is read by this application
- **THEN** each calibration point resolves to the coordinate the operator gave
  for it

#### Scenario: One point is not a calibration

- **WHEN** the operator gives one point only
- **THEN** the calibration is refused, saying that one point fixes a position
  but not the scale or the rotation, and no file is written

#### Scenario: Points on one parallel are not a calibration

- **WHEN** every point the operator gave shares a latitude
- **THEN** the calibration is refused, saying the points must not lie on one
  line, and no file is written

### Requirement: Coordinate fields read the notations a coordinator writes

Any field that takes a place SHALL accept decimal degrees, degrees with decimal
minutes, and degrees-minutes-seconds; with or without hemisphere letters; with
those letters in the Latin or the Cyrillic alphabet; before or after the
numbers; and in either order, so that `30.3609E 59.9311N` is the same place as
`59.9311, 30.3609`.

Text that does not hold exactly one place SHALL be refused rather than guessed
at, and the field SHALL show that it was not understood before anything is
submitted.

#### Scenario: The same place in four notations

- **WHEN** the operator enters `59.9311, 30.3609`, or `59°55'52"N 30°21'39"E`,
  or `N 59 55.87 E 30 21.65`, or `59.9311С 30.3609В`
- **THEN** each is read as the same place

#### Scenario: Half a place is refused

- **WHEN** the operator enters one number, or three
- **THEN** the field shows it was not understood and the action stays
  unavailable
