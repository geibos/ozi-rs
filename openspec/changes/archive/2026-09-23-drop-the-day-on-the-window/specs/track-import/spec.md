## ADDED Requirements

### Requirement: Files dropped on the window are imported

Dropping files on the application window SHALL import them exactly as the
import dialog does, and SHALL report how many of them landed.

A day's recordings arrive as a handful of files from several navigators.
Dropping them is how anybody expects to hand them over.

#### Scenario: A day's files dropped at once

- **WHEN** the operator drops a GPX and an OziExplorer waypoint file on the window
- **THEN** both are imported, their tracks and marks appear on the map, and the summary says two of two

#### Scenario: A folder dropped

- **WHEN** the operator drops a folder of per-date subfolders
- **THEN** it goes through the recursive import rather than being refused

### Requirement: One dispatch behind every import surface

Which reader a file reaches SHALL be decided in one place, shared by the
import dialog and the window's drop target, so the two cannot accept different
things.

An import that fails SHALL NOT stop the files after it; the failures SHALL be
reported together once the rest have landed, naming each file.

#### Scenario: One unreadable file among a day's

- **WHEN** one file of ten cannot be read
- **THEN** the other nine are imported and the report names the one that failed
