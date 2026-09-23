## ADDED Requirements

### Requirement: A mark's note is read from the file it came in

Importing marks SHALL read the note each one carries: `<desc>` from GPX and
field 11 from an OziExplorer waypoint file.

A headquarters that sends a mark sends the reason for it in the same file.
Reading the place and dropping the reason makes the exchange half useful.

#### Scenario: A GPX from another headquarters

- **WHEN** a GPX whose waypoints carry `<desc>` is imported
- **THEN** each mark carries its note

#### Scenario: An OziExplorer waypoint file

- **WHEN** a `.wpt` whose rows carry a description is imported
- **THEN** each mark carries its note
