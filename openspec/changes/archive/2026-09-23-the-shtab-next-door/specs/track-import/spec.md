## ADDED Requirements

### Requirement: OziExplorer waypoint files import

The system SHALL import OziExplorer Waypoint Files (`.wpt`), reading each
row's name, latitude, longitude and symbol, and SHALL place them in a waypoint
layer named after the source file.

A file whose first line is not the waypoint-file signature SHALL be refused
rather than parsed, and rows that carry no usable position — the placeholder
rows the original writes for empty GPS slots, and blank lines — SHALL be
skipped without failing the import.

#### Scenario: A waypoint file from another headquarters

- **WHEN** the operator imports a `.wpt` handed over by a headquarters running OziExplorer
- **THEN** its marks appear on the map and in the Waypoints tab, in a layer named after the file

#### Scenario: A file that is not a waypoint file

- **WHEN** the operator picks a `.plt` or any other file through the waypoint-file path
- **THEN** the import is refused with a message, and no marks are created

#### Scenario: Placeholder rows

- **WHEN** the file carries placeholder rows for empty GPS slots
- **THEN** those rows produce no marks and the rest of the file still imports

### Requirement: Cyrillic in an OziExplorer file reads as written

Text in an OziExplorer file SHALL be decoded through the same chain the track
files use — byte-order mark, strict UTF-8, statistical detection, then
Windows-1251 — because the files these crews exchange were written on Russian
Windows.

The `chr(209)` escape the format reserves for a comma inside a text field
SHALL NOT be interpreted. That escape is a byte, and in Windows-1251 the byte
is `С`: interpreting it would put a comma inside ordinary Russian words.

#### Scenario: A mark named СТАРТ

- **WHEN** a Windows-1251 waypoint file carries a mark named «СТАРТ»
- **THEN** the mark is named «СТАРТ», with no comma in it
