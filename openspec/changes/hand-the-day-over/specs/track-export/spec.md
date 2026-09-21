## ADDED Requirements

### Requirement: Every track in the project exports in one step

The system SHALL provide an action that writes every track in every track layer
to a single user-chosen `.gpx` file, and SHALL report how many tracks it wrote.
A project with no tracks SHALL be refused rather than producing an empty file.

#### Scenario: Handing over a day of searching

- **WHEN** a project holds tracks across several layers, one per imported navigator file
- **THEN** one action writes all of them into one GPX, and the operator is told how many

#### Scenario: Nothing to hand over

- **WHEN** the project has no tracks
- **THEN** the export is refused with a reason and no file is written
