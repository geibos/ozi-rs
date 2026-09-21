## ADDED Requirements

### Requirement: A day is handed over as one file

Exporting the day SHALL write every track and every waypoint in the project to
a single GPX file, and SHALL report how many of each it wrote. A project
holding waypoints and no tracks SHALL be exported, since a project of marks is
a day's work too; only a project holding neither SHALL be refused, and a
refusal SHALL leave no file behind.

#### Scenario: A day of routes and marks

- **WHEN** the operator exports the day from a project holding tracks and waypoints
- **THEN** one file carries both, and the operator is told how many tracks and how many marks were written

#### Scenario: A project of marks alone

- **WHEN** the project holds waypoints and no tracks
- **THEN** the export writes the marks rather than refusing

#### Scenario: Nothing to hand over

- **WHEN** the project holds neither tracks nor waypoints
- **THEN** the export is refused and no file is written
