## ADDED Requirements

### Requirement: The point under the cursor is named

With a track selected and no map tool on, the system SHALL name the selected
track's point nearest the cursor within 10 pixels: its number among the
track's points, its time when it has one, and the distance and speed of the
leg into it. A click on that point SHALL select it, and the points table
SHALL bring its row into view. Nothing SHALL be shown while a tool — the box,
edit mode, drawing, measuring, the ring, projection, placing a mark — owns
the cursor.

#### Scenario: What is this point

- **WHEN** the cursor rests on the second of a track's five points
- **THEN** the map shows «Точка 2 из 5», its time, and the leg into it

#### Scenario: Find it in the list

- **WHEN** the operator clicks that point
- **THEN** it becomes the selected point and its row is in view in the table
