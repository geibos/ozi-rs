## ADDED Requirements

### Requirement: Single import dialog accepts GPX, PLT and ZIP files

The Tracks panel SHALL expose one "Import…" action that opens a native file dialog with a single filter covering the `gpx`, `plt` and `zip` extensions and allows selecting several files at once. The system SHALL import the selected files one after another, routing each by its extension (case-insensitive): `.plt` files go to the PLT importer, every other selected file goes to the GPX importer, which handles `.zip` itself. A failure in one file SHALL NOT stop the import of the remaining files; after the last file the system SHALL show one summary stating how many of the selected files were imported and naming the files that failed.

#### Scenario: Mixed GPX, PLT and ZIP selection

- **WHEN** the user selects `a.gpx`, `b.PLT` and `c.zip` in the Import dialog
- **THEN** `a.gpx` and `c.zip` are imported through the GPX importer, `b.PLT` through the PLT importer, and one summary reports 3 of 3 files imported

#### Scenario: One broken file in a multi-file selection

- **WHEN** the user selects three files and the second one fails to parse
- **THEN** the first and third files are imported, and one summary reports 2 of 3 files imported and names the failed file

### Requirement: Recursive folder import of GPX and PLT files

The Tracks panel SHALL expose an "Import folder…" action. Given a directory, the system SHALL walk it and all of its subdirectories, collect every file whose extension is `.gpx` or `.plt` (case-insensitive), and import the collected files in sorted path order through the same single-file GPX and PLT importers. ZIP archives found in the folder SHALL NOT be expanded. A file that fails to import SHALL be recorded as skipped with its reason and SHALL NOT abort the remaining files; a subdirectory that cannot be read SHALL be skipped. The system SHALL report the number of imported files, tracks and waypoints and the names of skipped files in one summary. The action SHALL fail with an error when the directory cannot be read or contains no `.gpx` or `.plt` file at all.

#### Scenario: Per-date subfolders

- **WHEN** the user picks a folder `10-Tracks/` containing `2026-07-09/a.gpx` and `2026-07-10/b.plt`
- **THEN** both files are imported and the summary reports 2 files

#### Scenario: Broken file is skipped, not fatal

- **WHEN** the folder contains two valid GPX files and one `broken.gpx` that fails to parse
- **THEN** the two valid files are imported and the summary lists `broken.gpx` as skipped

#### Scenario: Folder without track files

- **WHEN** the user picks a folder that contains no `.gpx` or `.plt` file (for example only `.zip` archives)
- **THEN** the import fails with an error naming the folder and the project is unchanged

### Requirement: GPX import creates track and waypoint layers per source file

The system SHALL import a GPX file as `Track` entities with their original geometry preserved, placing all tracks of the file into a new track layer named `Imported tracks: <source path>`. A track without a `<name>` SHALL be named `<file stem> track <n>`. When the file contains `<wpt>` elements, the system SHALL place them into a new waypoint layer named `Imported waypoints: <source path>`, keeping each waypoint's name, coordinates and `<sym>` text verbatim; a waypoint without a name SHALL be named `<file stem> waypoint <n>`. A file with no tracks SHALL NOT create a track layer and a file with no waypoints SHALL NOT create a waypoint layer. Layer creation and every added track and waypoint SHALL be applied through `ProjectCommand`s on the project history (see `undo-redo`).

#### Scenario: Single-track GPX file

- **WHEN** the user imports `field.gpx` containing one track and no waypoints
- **THEN** a new track layer `Imported tracks: <path>/field.gpx` appears with that one track, and no waypoint layer is created

#### Scenario: Multi-track GPX file

- **WHEN** the user imports a GPX file containing multiple tracks
- **THEN** each track is imported as a separate `Track` entity in the same new track layer

#### Scenario: GPX file with waypoints

- **WHEN** the user imports a GPX file with two tracks and three `<wpt>` elements, one of them carrying `<sym>Flag</sym>`
- **THEN** a track layer with two tracks and a waypoint layer with three waypoints are created, and that waypoint's symbol is the string `Flag`

### Requirement: ZIP archives import each GPX entry as a source file

The system SHALL accept `.zip` archives through the Import dialog and SHALL route them to the archive importer even though they arrive through the GPX import path. The archive layer SHALL classify entries by file extension only (case-insensitive) without inspecting their content; directory entries SHALL be skipped. Every entry classified as GPX, at any folder depth inside the archive, SHALL be parsed and imported exactly as a standalone GPX file would be, using the entry path as its source path. Entries with any other extension (including `.plt`, `.kml`, `.map`, `.wpt` and raster payloads) SHALL be ignored by the track import. The system SHALL parse all GPX entries before applying anything to the project; if any entry fails to parse, the whole archive import SHALL fail and the project SHALL remain unchanged.

#### Scenario: ZIP archive of GPX files in nested folders

- **WHEN** the user imports `tracks.zip` containing `20260709/a.gpx` and `20260710/b.GPX`
- **THEN** two track layers are created, one per entry, each holding that entry's tracks

#### Scenario: PLT entries inside a ZIP are ignored

- **WHEN** the user imports a ZIP containing `a.gpx` and `b.plt`
- **THEN** only `a.gpx` is imported, the import succeeds, and `b.plt` is not imported

#### Scenario: Corrupt GPX entry fails the archive

- **WHEN** one GPX entry in the archive is malformed XML
- **THEN** the archive import fails with an error and no layer is added to the project

## MODIFIED Requirements

### Requirement: System imports OziExplorer PLT files

The system SHALL accept PLT files (OziExplorer Track Point File) and SHALL import each file as one `Track` in a new track layer named `Imported tracks: <source path>`. The track name SHALL come from the fourth field of the track properties line (line 5), falling back to the file stem when that field is empty; the visibility flag, line width and COLORREF colour SHALL be read from the same line. Point rows SHALL keep latitude, longitude, altitude (feet, `-777` meaning unknown) and the OLE date as timestamp; a point whose third field is `1` SHALL start a new segment. A file whose first line does not start with `OziExplorer Track Point File` SHALL be rejected.

#### Scenario: Single PLT file

- **WHEN** the user picks a PLT file whose point rows carry two segment-break flags
- **THEN** the track is imported into a new track layer with all its points and two segment boundaries preserved

#### Scenario: Missing header

- **WHEN** the user picks a `.plt` file whose first line is not the `OziExplorer Track Point File` signature
- **THEN** the import fails with an error and the project is unchanged

## REMOVED Requirements

### Requirement: System imports GPX files into the active track layer

**Reason**: The wording has been stale since the archive importer landed: `application/import.rs` creates a new track layer (`Imported tracks: <source path>`) and, when `<wpt>` elements are present, a new waypoint layer for every imported file; nothing is written into the active track layer. GPX waypoint import was not covered at all.

**Migration**: Replaced by "GPX import creates track and waypoint layers per source file" above.

### Requirement: System imports ZIP archives containing GPX or PLT files

**Reason**: Only GPX entries are consumed from archives; `.plt` entries are classified but skipped (`infrastructure/import/gpx.rs`), and the ZIP arrives through the unified Import dialog, not through "the same file pickers used for GPX and PLT", which no longer exist.

**Migration**: Replaced by "ZIP archives import each GPX entry as a source file" above.
