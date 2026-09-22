## ADDED Requirements

### Requirement: A track exported to PLT and read back is the same track

A track written to OziExplorer's PLT format and imported again SHALL carry the
same name, the same division into sittings, the same coordinates, the same
timestamps including their absence, the same colour and the same line width.
Elevation SHALL survive the format's round to whole feet and no worse.

The round trip SHALL be measured through the same colour packing the export
command performs, since a comparison that bypasses the caller does not describe
what a receiver gets.

#### Scenario: A day's track goes to OziExplorer and back

- **WHEN** a two-sitting track with a Cyrillic name, an elevation, one timestamped point and one without, a colour and a line width is exported to PLT and imported again
- **THEN** every one of those is as it was, and the two sittings are still two
