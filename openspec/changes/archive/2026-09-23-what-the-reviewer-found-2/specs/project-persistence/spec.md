## ADDED Requirements

### Requirement: The format version is read before the rest of the file

The system SHALL read a project's format version on its own, before parsing
the rest of the file, so that a file from a newer build is reported as such
even when this build cannot parse its contents at all.

Reading the whole file first works only while a future format still parses as
this one. The day it moves a field, the operator is told "format error" about
a file whose real problem is that it is newer — the diagnosis failing in
exactly the case it exists for.

#### Scenario: A newer format this build cannot parse

- **WHEN** a project file declares a newer version and holds a structure this build does not understand
- **THEN** the operator is told the file comes from a newer version, not that it is malformed
