## ADDED Requirements

### Requirement: A mark's note goes out with it

Exporting marks SHALL write the note each one carries: `<desc>` in GPX and
field 11 in an OziExplorer waypoint file, within that format's length limit.

A mark handed to the headquarters next door used to arrive as a place with no
reason attached.

#### Scenario: A mark with a note, out and back

- **WHEN** a mark carrying a note is exported and read back in either format
- **THEN** it comes back with the same note

#### Scenario: A mark with no note

- **WHEN** a mark with no note is exported and read back
- **THEN** it does not acquire one
