## ADDED Requirements

### Requirement: A search can be started fresh

The system SHALL provide a way to empty the project: no tracks, no waypoints,
the default track and waypoint layers restored, no file path, and an empty
undo history.

A project is one search. Without this, the crew that finishes one operation
and starts the next keeps adding to the same document, and yesterday's routes
stay under today's.

The undo history SHALL be cleared rather than carried over, so that undo
cannot walk back into a search that is over and put its tracks on the map
again.

#### Scenario: Starting the next search

- **WHEN** the operator starts a new project
- **THEN** the map and both library tabs are empty, the default layers are present, and undo cannot bring the old work back

#### Scenario: The next save asks where

- **WHEN** the operator saves after starting a new project
- **THEN** the system asks for a path rather than overwriting the file the previous search was saved to

### Requirement: Starting a search keeps the maps

Starting a new project SHALL leave the loaded bundle and the active raster map
in place. The map is the ground and the project is the work on it; a second
search in the same district should not blank the screen.

#### Scenario: A second search in the same district

- **WHEN** the operator starts a new project while a raster map is open
- **THEN** the same map is still displayed, with nothing drawn on it

### Requirement: Unsaved work is not discarded silently

Starting a new project while the current one has unsaved changes SHALL ask the
operator first, and SHALL do nothing if they decline.

#### Scenario: Unsaved changes

- **WHEN** the operator starts a new project with unsaved changes and declines the question
- **THEN** the project is untouched
