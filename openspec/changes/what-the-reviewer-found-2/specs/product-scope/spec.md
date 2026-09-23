## ADDED Requirements

### Requirement: The measuring tool reports area as well as distance

Once three points have been clicked, the measuring tool SHALL also report the
area they enclose, in the unit a coordinator would say it in.

OziExplorer's tool is called "Distance & Area". The area is the half a
coordinator writes down: a sector is handed to a crew as "прочесать 2.4 км²",
and the number decides how many people it takes and how long it runs.

#### Scenario: Measuring a search sector

- **WHEN** the operator clicks round a sector with three or more points
- **THEN** the readout carries the enclosed area beside the distance

#### Scenario: Fewer than three points

- **WHEN** only two points have been clicked
- **THEN** only the distance is reported
