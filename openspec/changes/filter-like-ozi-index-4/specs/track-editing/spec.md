## MODIFIED Requirements

### Requirement: System supports Douglas–Peucker simplification with live preview

The system SHALL provide a simplification action with a configurable tolerance slider. While the slider moves, the system SHALL show a live preview of the simplified track on the map. Confirming the action SHALL commit the simplification as a single undoable edit; cancelling SHALL leave the track unchanged.

The tolerance SHALL be the deviation in **metres** that the slider states, at
every layer it passes through. The simplification algorithm works in
kilometres; the conversion SHALL happen once, at the boundary, in a parameter
whose name carries the unit.

The deviation SHALL be measured in metres on the ground at any latitude: the
foot of the perpendicular SHALL be found on a plane where a degree of
longitude is scaled by the cosine of the latitude, not in raw degrees.

The panel SHALL open at 2 m and SHALL say that 2 m reproduces OziExplorer's
track filter at index 4, the index the detachment's standard prescribes, and
SHALL offer to put the slider back at 2 m after it has been moved.

#### Scenario: Preview and commit

- **WHEN** the user opens the simplify panel, adjusts tolerance, and clicks Apply
- **THEN** the simplified geometry is committed to the project and undo restores the original points

#### Scenario: Preview and cancel

- **WHEN** the user opens the simplify panel, adjusts tolerance, and cancels
- **THEN** the original track geometry is preserved and no undo step is added

#### Scenario: The slider's number is metres

- **WHEN** the operator simplifies a track at a tolerance of 10 m, and the
  track has a 50 m deviation in it
- **THEN** that deviation is kept, because it is five times the tolerance the
  operator set

#### Scenario: A tolerance that is noise

- **WHEN** the same track is simplified at 100 m
- **THEN** the 50 m deviation is removed

#### Scenario: A diagonal at latitude 60

- **WHEN** a point lies 100 m off a diagonal segment at latitude 60
- **THEN** its deviation is measured as 100 m, not as the 117 m that raw degrees give

#### Scenario: The panel speaks Ozi's language

- **WHEN** the operator opens the simplify panel
- **THEN** the tolerance is 2 m and the panel says this is what OziExplorer's filter does at index 4

#### Scenario: Back to the standard

- **WHEN** the operator has moved the slider and presses the index-4 button
- **THEN** the tolerance is 2 m again and the preview follows it

## ADDED Requirements

### Requirement: Every visible track can be filtered at index 4 at once

The system SHALL offer to simplify every visible track at 2 m — OziExplorer's
filter at index 4, which the standard prescribes for every foot track
(п. 10) — after saying how many tracks and points that is and asking. Each
track SHALL be its own undoable step. The owner filtered track after track by
hand (recordings of 2026-10-06 and 2026-10-08).

#### Scenario: A search day filtered

- **WHEN** two tracks are visible and the operator agrees to filter them
- **THEN** both are simplified at 2 m, and each can be undone on its own
