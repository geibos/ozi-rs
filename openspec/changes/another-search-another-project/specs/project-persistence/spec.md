## ADDED Requirements

### Requirement: Opening another search's map starts its own project

When the operator opens a map of a catalogue search other than the one the
displayed map belongs to, and the project holds work — at least one track,
unsaved changes, or a file on disk — the system SHALL start a new project
before the map opens, and SHALL say which search the new project is for.

A project is one search. The catalogue is where a crew moves to the next one,
and without this the previous search's routes are drawn over a district they
have nothing to do with and saved into the same file as the next operation.

Opening another map of the same search, or a map while the project is empty,
SHALL leave the project as it is.

#### Scenario: The next search, from the catalogue

- **WHEN** the operator has a saved project with tracks over a map of one search and opens a map of another
- **THEN** the new map opens over an empty project, and the operator is told a new project was started for that search

#### Scenario: Another sheet of the same search

- **WHEN** the operator opens a second map of the search the current map belongs to
- **THEN** the project and its tracks are unchanged

#### Scenario: The first map of the day

- **WHEN** the project is empty and has never been saved, and the operator opens a map
- **THEN** no question is asked and nothing is reset

### Requirement: Unsaved work is asked about before another search opens

When opening another search's map would start a new project over unsaved
changes, the system SHALL first ask, offering to save and continue, to continue
without saving, or to stay. Staying, dismissing the question, or a save that
fails or is cancelled SHALL leave the project and the displayed map as they
were and open nothing.

#### Scenario: Save and continue

- **WHEN** the operator answers "save" and the save succeeds
- **THEN** the work is on disk, a new project is started and the map opens

#### Scenario: Stay

- **WHEN** the operator answers "stay" or dismisses the question
- **THEN** the project, its unsaved changes and the displayed map are untouched, and no map is opened or downloaded

#### Scenario: A save that does not happen

- **WHEN** the operator answers "save" and then cancels the file dialog
- **THEN** nothing is reset and no map is opened
