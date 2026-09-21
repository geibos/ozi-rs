## ADDED Requirements

### Requirement: The project catalogue can be walked from the keyboard

The project list SHALL be operable from the keyboard without a pointer: it
SHALL expose itself as a listbox that takes focus and names the row the
keyboard is on, SHALL move that position with the arrow keys, by a screenful
with Page Up and Page Down and to either end with Home and End, and SHALL open
the row it is on when Enter is pressed.

The position SHALL be held against the full filtered list rather than against
the rows currently rendered, because the list is virtualized and most rows do
not exist in the document. It SHALL NOT move past either end, the list SHALL
scroll to keep it visible, and it SHALL be marked distinctly from the selected
row.

Narrowing the list SHALL clear the position.

#### Scenario: Finding a search without the pointer

- **WHEN** the operator types part of a name, presses Down and presses Enter
- **THEN** the first matching project is opened

#### Scenario: Reaching the end of the catalogue

- **WHEN** the operator presses End and then Down
- **THEN** the position is on the last project and stays there

#### Scenario: Narrowing the list after moving

- **WHEN** the operator has moved the position and then changes the filter
- **THEN** no row is pointed at until the keyboard is used again
