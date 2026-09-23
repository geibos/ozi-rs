## ADDED Requirements

### Requirement: A mark carries a note

A waypoint SHALL carry an optional note, and the operator SHALL be able to
write and clear it from the Waypoint Inspector. Setting it SHALL be undoable.

The name of a mark is the place; the note is what a crew is actually sent to.

A mark with no note SHALL hold none, which is not the same as holding an empty
one: clearing the field means the mark has nothing to say.

#### Scenario: Writing a note

- **WHEN** the operator writes a note on a mark and leaves the field
- **THEN** the note is saved with the mark and undo takes it back

#### Scenario: Clearing a note

- **WHEN** the operator empties the note field
- **THEN** the mark holds no note rather than an empty one

#### Scenario: A project saved before notes existed

- **WHEN** such a project is opened
- **THEN** its marks load with no notes and nothing else about them changes
