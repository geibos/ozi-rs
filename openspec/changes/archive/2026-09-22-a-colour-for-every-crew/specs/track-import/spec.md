## ADDED Requirements

### Requirement: Imported tracks are told apart by colour

An imported track that does not declare a colour SHALL be given one from a
fixed palette, chosen by the track's position among the project's tracks, so
that the tracks of one day's import are drawn in different colours. An imported
track that does declare a colour SHALL keep it.

Whether a colour was declared SHALL be reported by the code that reads the
file, not inferred from the value: a file may declare the same colour the
application uses by default.

The palette SHALL hold at least eight distinct colours and SHALL avoid those a
topographic basemap is made of.

#### Scenario: A day's folder of recordings

- **WHEN** a folder of GPX files that carry no colour is imported
- **THEN** each track is drawn in a different colour

#### Scenario: A file that names its colour

- **WHEN** an imported track declares a colour, through the GPX extension or a PLT colour field
- **THEN** it keeps that colour, including when it is the same as the application's default

#### Scenario: The same day twice

- **WHEN** the same folder is imported into a fresh project on two occasions
- **THEN** the tracks come out in the same colours both times
