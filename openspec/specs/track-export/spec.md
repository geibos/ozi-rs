# track-export Specification

## Purpose
Covers writing tracks out of a project: whole-layer GPX export with the Garmin colour extension, single-track OziExplorer PLT export (layout, text encoding, colour and date encoding), and the default export location under the active bundle. Waypoint export (WPT) is specified in `waypoints`.

### Decision history

- Change `bootstrap-current-state`: GPX layer export with `<gpxx:DisplayColor>`, single-track PLT export with COLORREF BGR colour and OLE dates, and the `<bundle>/10-Tracks/<track>.<ext>` default path; codified as the existing requirements of this spec.
- Commits 0d17104 (2026-04-28) and f5f44bb (2026-07-14, M0 data-loss fixes): the PLT properties line was rewritten to the ten-field order OziExplorer and `import/plt.rs::parse_track_style` expect (visible, width, COLORREF, name, fixed tail) and the output switched from UTF-8 to Windows-1251 with CRLF, guarded by an export → import round-trip test; rationale: OziExplorer on Russian Windows showed mojibake and misread columns (CJ-6). Codified as: PLT export writes the OziExplorer 2.1 track layout; PLT export text is Windows-1251 with CRLF line endings.
- ADR-0022 (2026-04-28, accepted): export taxonomy "GPX (tracks/waypoints), PLT (tracks), WPT (waypoints)"; only the taxonomy touches this capability — WPT itself is codified in `waypoints`. Not codified: a waypoint GPX export — `build_waypoint_gpx_xml` exists in `export/gpx.rs` but no command or UI calls it (`export_gpx` exports track layers only).

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

### Requirement: Every track in the project exports in one step

The system SHALL provide an action that writes every track in every track layer
to a single user-chosen `.gpx` file, and SHALL report how many tracks it wrote.
A project with no tracks SHALL be refused rather than producing an empty file.

#### Scenario: Handing over a day of searching

- **WHEN** a project holds tracks across several layers, one per imported navigator file
- **THEN** one action writes all of them into one GPX, and the operator is told how many

#### Scenario: Nothing to hand over

- **WHEN** the project has no tracks
- **THEN** the export is refused with a reason and no file is written

### Requirement: A track exported to GPX and read back is the same track

A track written to GPX and imported again SHALL carry the same name, the same
division into segments, the same coordinates in the same order, the same point
timestamps, including their absence where there were none, and the same colour.

The segment boundary is part of this: a track whose sittings are joined on the
way back claims a straight line between them that was never walked.

The colour is carried as a GPX colour name, so what survives is the nearest
name rather than the exact bytes. A track that declares no colour, or one whose
declared name is not a GPX colour, SHALL keep the application's default rather
than be given a guess. Colours SHALL be matched to tracks by their order in the
document.

#### Scenario: A two-sitting track goes out and comes back

- **WHEN** a track of two segments, one of them timestamped, is exported to GPX and imported again
- **THEN** its name, its two segments, every coordinate in order and every timestamp are as they were

#### Scenario: A coloured track goes out and comes back

- **WHEN** a track whose colour is one of the GPX colour names is exported and imported again
- **THEN** it comes back that colour, not the default

#### Scenario: A file where only some tracks declare a colour

- **WHEN** a GPX holds a coloured track, then one with no colour, then another coloured one
- **THEN** each coloured track gets its own colour and the middle one keeps the default

#### Scenario: A colour name from another program

- **WHEN** an imported track declares a colour name that is not a GPX colour
- **THEN** the track keeps the default colour and the import succeeds

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

### Requirement: A day is handed over as one file

Exporting the day SHALL write every track and every waypoint in the project to
a single GPX file, and SHALL report how many of each it wrote. A project
holding waypoints and no tracks SHALL be exported, since a project of marks is
a day's work too; only a project holding neither SHALL be refused, and a
refusal SHALL leave no file behind.

#### Scenario: A day of routes and marks

- **WHEN** the operator exports the day from a project holding tracks and waypoints
- **THEN** one file carries both, and the operator is told how many tracks and how many marks were written

#### Scenario: A project of marks alone

- **WHEN** the project holds waypoints and no tracks
- **THEN** the export writes the marks rather than refusing

#### Scenario: Nothing to hand over

- **WHEN** the project holds neither tracks nor waypoints
- **THEN** the export is refused and no file is written

### Requirement: A comma in a name is replaced, not escaped

When writing OziExplorer files, a comma inside a text field SHALL be replaced
with a space rather than written as the `chr(209)` escape the format reserves.

The escape is a byte, and these files are Windows-1251, where that byte is
`С`. Reading it as a comma would put one inside «СТАРТ», which settles the
import side.

The export side rests on one thing that has not been checked: whether the
original substitutes that byte when reading a cp1251 file regardless of who
wrote it. If it does, a name with a `С` in it is mangled whatever this
application writes, and not escaping only avoids adding a second way to be
wrong. A round trip through a Russian OziExplorer would settle it; nobody has
run one.

#### Scenario: A track named with a comma

- **WHEN** a track named «ЛИСА15, вечер» is exported to PLT or WPT
- **THEN** the field carries «ЛИСА15  вечер» and the record is not broken

#### Scenario: A name carrying a Cyrillic С

- **WHEN** a mark named «СТАРТ» makes the trip out and back
- **THEN** it is still named «СТАРТ»
