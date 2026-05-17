# track-display Specification

## Purpose
TBD - created by archiving change bootstrap-current-state. Update Purpose after archive.
## Requirements
### Requirement: Each track has independently controllable color, line width, opacity, and visibility

The system SHALL store `color`, `line_width`, `opacity`, and `visible` per `Track`, and SHALL render each track using its own style. These mutations are immediate visual updates and are NOT recorded in undo history (see `undo-redo`).

#### Scenario: Color change applied immediately

- **WHEN** the user picks a new color in the track row of the Tracks panel
- **THEN** the track re-renders with the new color and the new color persists in the project

#### Scenario: Visibility toggle hides the track

- **WHEN** the user toggles a track's visibility off in the Tracks panel
- **THEN** the track polyline is removed from the map and the track row indicates hidden state

### Requirement: System computes and surfaces per-track statistics

The system SHALL compute, for each track, total distance in kilometres, total duration when point timestamps are present, and total point count, and SHALL surface these in the track row UI.

#### Scenario: Track with timestamps

- **WHEN** a track has GPS timestamps on its points
- **THEN** the track row shows distance in km, duration, and point count

#### Scenario: Track without timestamps

- **WHEN** a track has no point timestamps
- **THEN** the track row shows distance and point count; duration is omitted

#### Scenario: Track row shows distance, duration, and point count

- **WHEN** the user opens a project containing a track with three timestamped segments totalling 12.3 km, 1 hour 24 minutes, and 156 points
- **THEN** the Tracks panel row for that track displays distance "12.3 km", duration "1h 24m", and point count "156 pts" alongside the track name

#### Scenario: Track without timestamps hides duration

- **WHEN** the user displays a track whose points have no timestamps
- **THEN** the Tracks panel row shows distance and point count but omits the duration field

### Requirement: Track points panel exposes per-point attributes

The system SHALL provide a Track Points panel that lists each point's latitude, longitude, elevation (when present), segment index, and timestamp (when present).

#### Scenario: Point with timestamp

- **WHEN** the user opens the Track Points panel for a track whose points carry timestamps
- **THEN** each row displays the point's timestamp in a human-readable form

#### Scenario: Point without timestamp

- **WHEN** the user opens the Track Points panel for a track without timestamps
- **THEN** the timestamp column is omitted or empty without rendering placeholder text

### Requirement: Track name validation surfaces a non-blocking warning for non-conforming names

The system SHALL display a non-blocking warning indicator on tracks whose names do not match the LizaAlert OK-standard pattern `^\d{8}_.*\S.*$` (i.e. `YYYYMMDD_Callsign` with a non-empty callsign). The backend SHALL remain permissive: rename, save, and export operations SHALL NOT be blocked by a non-conforming name.

#### Scenario: Non-conforming name shows warning but does not block

- **WHEN** a track is renamed to `temp` (does not match the pattern)
- **THEN** the Tracks panel shows a warning indicator on that track and the rename, save, and export operations still succeed

#### Scenario: Conforming name has no warning

- **WHEN** a track is renamed to `20240601_Иванов`
- **THEN** the warning indicator is not shown

### Requirement: Validation pattern is alphabet-agnostic and does not validate calendar dates

The system SHALL accept any 8 leading digits followed by an underscore and a non-empty callsign in any alphabet (including Cyrillic). The system SHALL NOT validate that the leading 8 digits form a real calendar date.

#### Scenario: Cyrillic callsign

- **WHEN** a track is named `20240601_Иванов`
- **THEN** no warning is shown

#### Scenario: Impossible date passes pattern

- **WHEN** a track is named `20249999_Иванов`
- **THEN** no warning is shown — the warning checks pattern only, not calendar validity

