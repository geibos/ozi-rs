## ADDED Requirements

### Requirement: A restore that cannot find the project still opens the map

Restoring a session SHALL restore the project and the active map
independently. A project that is missing or cannot be read SHALL be reported
and SHALL NOT prevent the active map from being restored.

CJ-2 puts a laptop in a field camp with no link and asks for a working map
inside a minute. The map is on the disk whether or not the project beside it
still is.

#### Scenario: The project file has been moved

- **WHEN** the session names a project file that no longer exists, and an active map that does
- **THEN** the map is restored, and the missing project is reported

#### Scenario: The project file cannot be read

- **WHEN** the session names a project file that fails to parse, and an active map that is present
- **THEN** the map is restored, and the unreadable project is reported
