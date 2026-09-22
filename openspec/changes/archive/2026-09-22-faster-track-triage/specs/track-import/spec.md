## ADDED Requirements

### Requirement: Import-created layers are named after the source file

A layer created by importing a file SHALL be named after that file rather than
its full path, so the layer selector shows the part that distinguishes one
import from another.

#### Scenario: Importing a folder of GPX files

- **WHEN** `/Users/owner/Downloads/10-Tracks/20260709/20260708_Veter2.gpx` is imported
- **THEN** the created track layer is named `20260708_Veter2.gpx`

#### Scenario: Two files of the same name from different folders

- **WHEN** two imported files share a file name
- **THEN** both layers carry that name and remain distinguishable by their identifiers, which stay unique
