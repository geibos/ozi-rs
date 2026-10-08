## ADDED Requirements

### Requirement: Points can be chosen with a box on the map and removed together

The system SHALL offer a box tool. While it is on, a drag on the map SHALL
draw a rectangle instead of moving the map, and the points of the selected
track whose screen position lies inside it SHALL become the chosen points; a
drag with Shift held SHALL add to the chosen points; a click without a drag
SHALL clear them. The chosen points SHALL be drawn on the map and counted in
a bar that offers to delete them, to keep only them, or to clear the choice.
Deleting, and keeping only, SHALL each be one undoable edit. An edit that
would leave the track without points SHALL be refused. Turning the tool on
SHALL turn the other map tools off; turning it off SHALL give the drag back
to the map and drop the choice.

#### Scenario: Delete a cluster of outliers

- **WHEN** the operator turns the box tool on and drags a box over three
  points of the selected track
- **THEN** the bar says three points are chosen, and «Удалить выбранные»
  removes them in one undoable step

#### Scenario: Keep only the search

- **WHEN** the operator boxes the part of a track that belongs to this
  search and chooses «Оставить только их»
- **THEN** every other point of the track is removed in one undoable step

#### Scenario: Two boxes

- **WHEN** the operator draws a second box with Shift held
- **THEN** its points are added to those already chosen

#### Scenario: Nothing left

- **WHEN** the chosen points are every point of the track and the operator
  deletes them
- **THEN** the edit is refused and the track is unchanged
