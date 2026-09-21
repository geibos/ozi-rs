## ADDED Requirements

### Requirement: The inspector shows the track's elevation against distance

The track inspector SHALL draw the recorded elevation of the selected track
against distance travelled along it, and SHALL state the range in metres. A
point carrying no elevation SHALL still count towards the distance, so a gap in
the data does not move the samples around it. A track with fewer than two
elevation readings SHALL be reported as carrying no elevation rather than
drawn.

#### Scenario: A recording with elevation

- **WHEN** the operator selects a track whose points carry elevation
- **THEN** the inspector draws the profile across the card and states the lowest and highest readings

#### Scenario: A recording without elevation

- **WHEN** the selected track carries no elevation, or only one reading
- **THEN** the card says the recording carries no elevation, and no chart is drawn

#### Scenario: A gap in the elevation data

- **WHEN** a point between two readings carries no elevation
- **THEN** the later reading keeps its distance from the track's start, rather than moving towards the earlier one
