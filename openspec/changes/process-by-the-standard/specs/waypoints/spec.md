## MODIFIED Requirements

### Requirement: System exports waypoints to OziExplorer WPT

The system SHALL provide an "Export WPT" action for a waypoint layer (offered from the Waypoints panel layer menu, the Waypoint inspector and the command palette) that writes all waypoints of the chosen layer to a user-selected `.wpt` file in OziExplorer Waypoint File version 1.1 format. The file SHALL start with the four header lines `OziExplorer Waypoint File Version 1.1`, `WGS 84`, `Reserved 2`, `Reserved 3`, followed by one row per waypoint with 24 comma-separated fields: sequential number starting at 1, name (truncated to 14 characters, commas and line breaks replaced by spaces), latitude and longitude in WGS84 decimal degrees with six decimals, date `0`, symbol code, status `1`, map display format `0`, foreground colour `0`, background colour — the mark's colour as OziExplorer packs it (red + green·256 + blue·65536), or `65535` for a mark with no colour —, the mark's note truncated to 40 characters (empty when it has none), `0`, `0`, `0`, altitude `-777`, font size `6`, font style `0`, symbol size `17`, `0`, `0`, `0` and three empty trailing fields. Waypoints model neither elevation nor timestamp, so altitude SHALL always be `-777` and date `0`. Field 11 is the note (`A mark carries a note`). Every line SHALL be encoded as Windows-1251 and terminated with `\r\n`. Reading a WPT file SHALL take a mark's colour from its background field, a background of `65535` meaning no colour.

The symbol code SHALL be derived from the waypoint's symbol string: a string that parses as an integer SHALL be written as that number; known names SHALL map case-insensitively to OziExplorer codes — of the picker keys, `flag` → 9, `camp` → 18, `danger` → 19, `water` → 30 (further OziExplorer names such as `house`, `fuel`, `anchor`, `skull`, `star` are mapped as well); an absent symbol or any other string, including the picker keys `shelter`, `meeting-point`, `start`, `finish`, `viewpoint` and `parking`, SHALL be written as the default code `0`.

The export dialog SHALL pre-fill the standard's file, `<active bundle>/10-Tracks/Waypoints_ГГГГММДД.wpt` with today's date (п. 25, 29), when a bundle is active, and `Waypoints_ГГГГММДД.wpt` otherwise; the user MAY choose any path. A failed export (layer not found, file cannot be created, write error) SHALL be reported as an error status message and SHALL NOT crash the application.

#### Scenario: Export waypoints with symbols to WPT

- **WHEN** the user invokes "Export WPT" on a layer with three waypoints whose symbols are `camp`, `flag` and none
- **THEN** the resulting `.wpt` file has the four header lines and exactly three rows of 24 fields, with symbol codes `18`, `9` and `0` respectively

#### Scenario: Cyrillic name is written as cp1251

- **WHEN** a waypoint named `Стоянка` is exported to WPT
- **THEN** its row contains the cp1251 byte sequence for `Стоянка`, the file contains no UTF-8 sequence for it, and every line ends with `\r\n`

#### Scenario: The note travels in field 11

- **WHEN** a mark named `улика` carrying the note `красная куртка, 200 м от просеки` is exported to WPT
- **THEN** field 11 of its row holds that note, with the comma replaced so the row keeps its 24 fields

#### Scenario: Picker key without a WPT code falls back to 0

- **WHEN** a waypoint with symbol `shelter` is exported to WPT
- **THEN** its symbol field is `0`

#### Scenario: WPT export dialog suggests `<bundle>/<layer>.wpt`

- **WHEN** the user invokes WPT export on 8 October 2026 with a bundle active
- **THEN** the file picker pre-fills with `<bundle>/10-Tracks/Waypoints_20261008.wpt`

#### Scenario: WPT export with no active bundle suggests filename only

- **WHEN** the user invokes WPT export with no active bundle
- **THEN** the file picker suggests `Waypoints_ГГГГММДД.wpt` only, without a directory

#### Scenario: A mark's colour goes out and comes back

- **WHEN** a red mark and an uncoloured one are written to WPT and read back
- **THEN** the first carries background `255` and comes back red; the second carries `65535` and comes back uncoloured

## ADDED Requirements

### Requirement: Every visible mark is saved into one WPT file

The system SHALL offer to write every visible mark of every waypoint layer
into one WPT file, suggesting `10-Tracks/Waypoints_ГГГГММДД.wpt`, because
marks arrive a file per navigator per day and the standard wants as few
waypoint files as possible (п. 31). Hidden marks SHALL stay out. With no
visible marks the export SHALL be refused.

#### Scenario: Two imports, one file

- **WHEN** two waypoint layers hold three marks, one of them hidden, and the
  operator saves every visible mark
- **THEN** one WPT file holds the two visible marks

### Requirement: A layer's notes can be cleared at once

The system SHALL offer to clear the note of every mark in the active waypoint
layer as one undoable step, and SHALL say how many were cleared.

#### Scenario: Addresses from a phone

- **WHEN** a layer imported from a phone carries a street address in each
  mark's note, and the operator clears the layer's notes
- **THEN** every note is empty, and one undo brings them all back

### Requirement: A mark is styled and named by the standard

The waypoint inspector SHALL offer the standard's mark colours (п. 28) —
red for what matters to the search and confirmed finds, sea green for
unconfirmed finds, green for groups' positions and features of the ground —
and lime for a drop-off point. It SHALL flag a Russian capital «С» in a
mark's name and offer to make it the latin «C», and SHALL name any character
п. 26 does not allow in a name.

#### Scenario: A Russian С

- **WHEN** a mark is named `Сарай`
- **THEN** the inspector offers to replace the «С», and doing so renames it
  with a latin «C»
