# track-export Specification

## Purpose
TBD - created by archiving change bootstrap-current-state. Update Purpose after archive.
## Requirements
### Requirement: System exports the active track layer to GPX

The system SHALL provide an "Export GPX" action that writes the contents of the active track layer to a user-chosen `.gpx` file containing all tracks, segments, and points.

#### Scenario: Export active track layer

- **WHEN** the user invokes "Export GPX" on a project whose active track layer has three tracks
- **THEN** the resulting `.gpx` file contains exactly those three tracks with their segments and points

### Requirement: GPX export includes the Garmin color extension for per-track color

The system SHALL write each track's color as a Garmin `<gpxx:DisplayColor>` extension element so that color round-trips with compatible GPS software.

#### Scenario: Colored track export

- **WHEN** a track has a non-default color and the user exports GPX
- **THEN** the GPX file includes the Garmin DisplayColor extension reflecting that color

### Requirement: System exports an individual track to PLT

The system SHALL provide an "Export PLT" action that writes a single chosen track to a user-chosen `.plt` file in the OziExplorer PLT format.

#### Scenario: Single track to PLT

- **WHEN** the user invokes "Export PLT" for a selected track
- **THEN** the resulting `.plt` file contains that single track in valid PLT format

### Requirement: PLT export encodes color as COLORREF BGR and timestamps as OLE dates

The system SHALL encode track color in the PLT header as a Windows COLORREF value (BGR byte order) and SHALL encode point timestamps using the OLE automation date format expected by OziExplorer.

#### Scenario: Color and timestamp encoding

- **WHEN** the user exports a track with timestamps and a non-default color to PLT
- **THEN** the PLT header color field is BGR-ordered and point timestamps are OLE-formatted floating-point dates

### Requirement: Export dialog suggests `10-Tracks/` under the active bundle

The system SHALL, when an active bundle is known, pre-fill the GPX/PLT export file picker with a suggested path under `<active-bundle>/10-Tracks/<track>.<ext>`. When no active bundle is known, the system SHALL fall back to a filename-only suggestion. The user MAY override the suggestion and choose any path; the system SHALL NOT block exports outside the suggested location.

#### Scenario: Active bundle present

- **WHEN** the user invokes PLT export with a bundle active and a selected track named `20240601_Иванов`
- **THEN** the file picker pre-fills with `<bundle>/10-Tracks/20240601_Иванов.plt`

#### Scenario: No active bundle

- **WHEN** the user invokes GPX export with no active bundle
- **THEN** the file picker suggests the filename only, without a directory, and lets the user pick a location

#### Scenario: User overrides suggestion

- **WHEN** the user changes the export path to a directory outside the active bundle
- **THEN** the system writes the export to the chosen location without warning

