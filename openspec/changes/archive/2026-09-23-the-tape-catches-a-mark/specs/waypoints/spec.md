## ADDED Requirements

### Requirement: Measuring catches hold of a mark

While the measuring tool is active, a click that lands within a short screen
distance of a waypoint SHALL take that waypoint's own position rather than the
position clicked. When a measurement has exactly two ends and both were taken
from waypoints, the readout SHALL name them.

The reach SHALL be measured in screen pixels, not on the ground: a reach in
metres would catch nothing at a wide view and everything at a close one, while
what the operator is aiming at is the marker under the cursor.

#### Scenario: From the headquarters to the drop-off

- **WHEN** the operator measures by clicking near the mark «ШТАБ» and then near
  «ЗАБРОС»
- **THEN** the distance is between those two marks exactly, and the readout
  reads `ШТАБ → ЗАБРОС`

#### Scenario: A measurement that is not between marks

- **WHEN** one end of the measurement is a bare click on the map, or the
  measurement has more than two ends
- **THEN** the distance is reported without naming anything

#### Scenario: Two marks close together

- **WHEN** a click falls within reach of two marks
- **THEN** the nearer one is taken
