# track-import Specification

## Purpose
Covers how track data enters a project from files: the single Import dialog (GPX, PLT, ZIP), recursive folder import, the archive path that unpacks ZIP entries, PLT text decoding, and the layers each import creates. Waypoints embedded in GPX files are imported here; their editing lives in `waypoints`, map bundles in `map-bundles` and `lizaalert-integration`.

### Decision history

- ADR-0007 (2026-03-28, accepted): archive import is three layers — `infrastructure::import::archive` reads ZIP entries and classifies them by extension only, format adapters (`gpx.rs`, `plt.rs`) parse, `application` orchestrates and registers results through `ProjectCommand`s; rationale: keep OZF2 raster risk out of GPX import and add a format by adding an adapter file. Codified as: ZIP archives import each GPX entry as a source file; GPX import creates track and waypoint layers per source file. Not codified: the layering itself (architecture, not behaviour); PLT/KML/WPT archive entries are classified but no adapter consumes them (`gpx.rs` keeps only `Gpx` entries); the zip-slip guard (`enclosed_name` in `archive.rs`) is only exercised by map-bundle staging in `lizaalert.rs` and belongs to those capabilities.
- Change `bootstrap-current-state`: first behaviour-level requirements; its "into the active track layer" wording never matched `application/import.rs`, which creates `Imported tracks: <path>` and `Imported waypoints: <path>` layers — replaced in `codify-architecture-decisions`.
- Change `fix-plt-import-encoding-detection` (2026-05-17): PLT bytes decode via BOM → strict UTF-8 → `chardetng` → Windows-1251 fallback; rationale: field PLT files from Russian Windows are cp1251, newer ones UTF-8 or UTF-16. Codified as: PLT import accepts Windows-1251 encoded text. GPX import has no such chain; it relies on the XML encoding declaration.
- CJ-3 (`docs/customer-journeys.md`) and the July 2026 import work: one Import dialog with a combined GPX/PLT/ZIP filter, `.zip` routed through the GPX import command, and a recursive "Import folder…" for per-date subfolders that reports per-file failures instead of aborting; rationale: volunteers bring a folder or a ZIP from several navigators, and the twin GPX/PLT buttons were indistinguishable. Codified as: Single import dialog accepts GPX, PLT and ZIP files; Recursive folder import of GPX and PLT files.

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
