# track-import Specification

## Purpose
Covers how track data enters a project from files: the single Import dialog (GPX, PLT, WPT, ZIP) and the window's drop target, recursive folder import, the archive path that unpacks ZIP entries, PLT text decoding, and the layers each import creates. Waypoints embedded in GPX files are imported here; their editing lives in `waypoints`, map bundles in `map-bundles` and `lizaalert-integration`.

### Decision history

- ADR-0007 (2026-03-28, accepted): archive import is three layers — `infrastructure::import::archive` reads ZIP entries and classifies them by extension only, format adapters (`gpx.rs`, `plt.rs`) parse, `application` orchestrates and registers results through `ProjectCommand`s; rationale: keep OZF2 raster risk out of GPX import and add a format by adding an adapter file. Codified as: ZIP archives import each GPX entry as a source file; GPX import creates track and waypoint layers per source file. Not codified: the layering itself (architecture, not behaviour); PLT/KML/WPT archive entries are classified but no adapter consumes them (`gpx.rs` keeps only `Gpx` entries); the zip-slip guard (`enclosed_name` in `archive.rs`) is only exercised by map-bundle staging in `lizaalert.rs` and belongs to those capabilities.
- Change `bootstrap-current-state`: first behaviour-level requirements; its "into the active track layer" wording never matched `application/import.rs`, which creates `Imported tracks: <path>` and `Imported waypoints: <path>` layers — replaced in `codify-architecture-decisions`.
- Change `fix-plt-import-encoding-detection` (2026-05-17): PLT bytes decode via BOM → strict UTF-8 → `chardetng` → Windows-1251 fallback; rationale: field PLT files from Russian Windows are cp1251, newer ones UTF-8 or UTF-16. Codified as: PLT import accepts Windows-1251 encoded text. GPX import has no such chain; it relies on the XML encoding declaration.
- CJ-3 (`docs/customer-journeys.md`) and the July 2026 import work: one Import dialog with a combined GPX/PLT/ZIP filter, `.zip` routed through the GPX import command, and a recursive "Import folder…" for per-date subfolders that reports per-file failures instead of aborting; rationale: volunteers bring a folder or a ZIP from several navigators, and the twin GPX/PLT buttons were indistinguishable. Codified as: Single import dialog accepts a day's files whatever they are; Recursive folder import of GPX and PLT files. `.wpt` joined the filter on 2026-09-23 (`a-note-on-the-mark`), when the headquarters next door turned out to hand over OziExplorer's own waypoint format.

## Requirements

### Requirement: System imports OziExplorer PLT files

The system SHALL accept PLT files (OziExplorer Track Point File) and SHALL import each file as one `Track` in a new track layer named `Imported tracks: <source path>`. The track name SHALL come from the fourth field of the track properties line (line 5), falling back to the file stem when that field is empty; the visibility flag, line width and COLORREF colour SHALL be read from the same line. Point rows SHALL keep latitude, longitude, altitude (feet, `-777` meaning unknown) and the OLE date as timestamp; a point whose third field is `1` SHALL start a new segment. A file whose first line does not start with `OziExplorer Track Point File` SHALL be rejected.

#### Scenario: Single PLT file

- **WHEN** the user picks a PLT file whose point rows carry two segment-break flags
- **THEN** the track is imported into a new track layer with all its points and two segment boundaries preserved

#### Scenario: Missing header

- **WHEN** the user picks a `.plt` file whose first line is not the `OziExplorer Track Point File` signature
- **THEN** the import fails with an error and the project is unchanged

### Requirement: PLT import accepts Windows-1251 encoded text

The system SHALL decode PLT file content by detecting the encoding from a prioritized chain (BOM → strict UTF-8 → statistical detection via `chardetng` → Windows-1251 fallback) and SHALL convert the decoded text to UTF-8 before structural parsing. The decoder SHALL NOT introduce `U+FFFD` replacement characters for files whose bytes are a valid sequence in any supported encoding.

Supported encodings:

- UTF-8 (with or without BOM)
- UTF-16 LE / UTF-16 BE (when a BOM is present)
- Windows-1251 (cp1251) — used by legacy OziExplorer exports on Russian Windows
- Any other single-byte encoding `chardetng` can identify

#### Scenario: Cyrillic track name in Windows-1251 PLT

- **WHEN** the user imports a PLT file whose header contains a Cyrillic track name encoded as Windows-1251 (cp1251)
- **THEN** the imported `Track::name()` equals the original Russian string with all Cyrillic characters preserved and no `U+FFFD` replacement characters

#### Scenario: Cyrillic track name in UTF-8 PLT (no BOM)

- **WHEN** the user imports a PLT file whose header contains a Cyrillic track name encoded as UTF-8 without a BOM
- **THEN** the imported track name equals the original Russian string

#### Scenario: Track name in UTF-8 PLT with BOM

- **WHEN** the user imports a PLT file beginning with the UTF-8 BOM (`EF BB BF`)
- **THEN** the BOM is stripped and the file is decoded as UTF-8; the imported track name is correct

#### Scenario: Track name in UTF-16 LE PLT with BOM

- **WHEN** the user imports a PLT file beginning with the UTF-16 LE BOM (`FF FE`)
- **THEN** the file is decoded as UTF-16 LE and the imported track name is correct

#### Scenario: Pure ASCII PLT unchanged

- **WHEN** the user imports a PLT file whose bytes are all ASCII
- **THEN** all string fields decode identically to the previous (pre-detection) behaviour

### Requirement: Import failures surface as user-facing errors

The system SHALL convert import errors (unreadable file, malformed XML, unknown PLT version, etc.) into user-facing error messages and SHALL NOT crash the application or partially mutate the project on failure.

#### Scenario: Malformed GPX file

- **WHEN** the user attempts to import a file that fails GPX parsing
- **THEN** the application reports an import error and the project state is unchanged

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

### Requirement: A folder import reports in the operator's language

Importing a folder of recordings SHALL report what it did as counts — files
read, tracks imported, waypoints imported — and the interface SHALL put those
into words, rather than the backend sending a sentence to be displayed. Files
that could not be read SHALL be named, by file name rather than by path. A
folder in which some files were read and others were not SHALL be reported as a
success carrying a caveat, not as a failure.

#### Scenario: A day's archive imports

- **WHEN** the operator imports a folder and every file is read
- **THEN** the result is stated in the interface language, naming how many tracks and waypoints came from how many files

#### Scenario: One navigator's file is unreadable

- **WHEN** a folder imports with one file unread
- **THEN** the rest is reported as imported, and the unread file is named alongside rather than replacing the result

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

### Requirement: A waypoint file in another datum says so

When an imported OziExplorer waypoint file declares a datum that is not in the
WGS 84 family, the system SHALL warn that the coordinates may be displaced.

This application does not transform between datums, and does not intend to.
Taking the coordinates in silence puts another headquarters' marks 100–150 m
from where they meant them, and nobody finds out until a crew is standing
there.

#### Scenario: A file from a headquarters working in Pulkovo 1942

- **WHEN** a `.wpt` declaring a non-WGS 84 datum is imported
- **THEN** the marks are imported and a warning about the possible displacement is recorded

### Requirement: A mark's note is read from the file it came in

Importing marks SHALL read the note each one carries: `<desc>` from GPX and
field 11 from an OziExplorer waypoint file.

A headquarters that sends a mark sends the reason for it in the same file.
Reading the place and dropping the reason makes the exchange half useful.

#### Scenario: A GPX from another headquarters

- **WHEN** a GPX whose waypoints carry `<desc>` is imported
- **THEN** each mark carries its note

#### Scenario: An OziExplorer waypoint file

- **WHEN** a `.wpt` whose rows carry a description is imported
- **THEN** each mark carries its note

### Requirement: Single import dialog accepts a day's files whatever they are

The Tracks panel SHALL expose one "Import…" action that opens a native file dialog with a single filter covering the `gpx`, `plt`, `wpt` and `zip` extensions and allows selecting several files at once. The system SHALL import the selected files one after another, routing each by its extension (case-insensitive): `.plt` to the PLT importer, `.wpt` to the OziExplorer waypoint importer, and every other selected file to the GPX importer, which handles `.zip` itself. A failure in one file SHALL NOT stop the import of the remaining files; after the last file the system SHALL show one summary stating how many of the selected files were imported and naming the files that failed.

The routing itself is stated once, in "One dispatch behind every import surface": this requirement is about the dialog, not about a second copy of the rule.

#### Scenario: Mixed GPX, PLT, WPT and ZIP selection

- **WHEN** the user selects `a.gpx`, `b.PLT`, `c.zip` and `улики.wpt` in the Import dialog
- **THEN** `a.gpx` and `c.zip` are imported through the GPX importer, `b.PLT` through the PLT importer, `улики.wpt` through the waypoint importer, and one summary reports 4 of 4 files imported

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
