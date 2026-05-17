# track-import Specification

## Purpose
TBD - created by archiving change bootstrap-current-state. Update Purpose after archive.
## Requirements
### Requirement: System imports GPX files into the active track layer

The system SHALL accept GPX files via a file picker and SHALL import all tracks contained in the file into the active track layer as `Track` entities with their original geometry preserved.

#### Scenario: Single-track GPX file

- **WHEN** the user picks a GPX file containing one track
- **THEN** the track appears in the Tracks panel and renders on the map in the active track layer

#### Scenario: Multi-track GPX file

- **WHEN** the user picks a GPX file containing multiple tracks
- **THEN** each track is imported as a separate `Track` entity in the active track layer

### Requirement: System imports ZIP archives containing GPX or PLT files

The system SHALL accept `.zip` archives via the same file pickers used for GPX and PLT, and SHALL classify their entries to import each contained recognized track file.

#### Scenario: ZIP archive of GPX files

- **WHEN** the user picks a ZIP archive containing several GPX files
- **THEN** every recognized GPX entry is imported as a separate track in the active track layer

### Requirement: System imports OziExplorer PLT files

The system SHALL accept PLT files (OziExplorer format) and SHALL import their points and segment boundaries into the active track layer.

#### Scenario: Single PLT file

- **WHEN** the user picks a PLT file
- **THEN** the track is imported into the active track layer with its points and segment boundaries preserved

### Requirement: PLT import accepts Windows-1251 encoded text

The system SHALL decode PLT file content using Windows-1251 encoding so that Cyrillic track names and metadata round-trip correctly.

#### Scenario: Cyrillic track name in PLT

- **WHEN** the user imports a PLT file whose header contains a Cyrillic track name encoded as Windows-1251
- **THEN** the track is imported with the name decoded as readable Cyrillic, not as mojibake

### Requirement: Import failures surface as user-facing errors

The system SHALL convert import errors (unreadable file, malformed XML, unknown PLT version, etc.) into user-facing error messages and SHALL NOT crash the application or partially mutate the project on failure.

#### Scenario: Malformed GPX file

- **WHEN** the user attempts to import a file that fails GPX parsing
- **THEN** the application reports an import error and the project state is unchanged

