## ADDED Requirements

### Requirement: A project file says which format it is

A saved project SHALL carry a format version. A project file without one SHALL
be read as the oldest format and open unchanged, because that is every project
saved before the version existed.

#### Scenario: A project saved by this build

- **WHEN** a project is saved
- **THEN** the file carries the format version this build writes, and the rest of its shape is unchanged

#### Scenario: A project saved before versions existed

- **WHEN** a project file carrying no format version is opened
- **THEN** it opens with its contents intact

### Requirement: A project from a newer build is refused, not degraded

A project file whose format version is newer than the running build
understands SHALL NOT be opened. The system SHALL say that the file was
written by a newer version.

Opening it would read the parts this build knows, drop the rest, and write
that loss back over the other headquarters' file the moment the operator
saved. A project exchanged between штабы has to be safe in both directions.

#### Scenario: A file from the штаб running a newer build

- **WHEN** a project file declares a format version above the one this build supports
- **THEN** the open is refused and the operator is told the file comes from a newer version
