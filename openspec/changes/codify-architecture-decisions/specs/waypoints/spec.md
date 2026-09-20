## ADDED Requirements

### Requirement: Waypoint symbol is an open optional string

The domain SHALL store a waypoint's symbol as an optional free-form string, not as a closed enumeration. An absent symbol (`"symbol": null` in the serialized waypoint) means "no symbol". Any string SHALL be accepted when setting a symbol and SHALL be stored and persisted verbatim; the system SHALL NOT reject, lower-case or otherwise normalise values outside the picker's set. The symbol picker SHALL offer a fixed set of keys — `flag`, `camp`, `danger`, `water`, `shelter`, `meeting-point`, `start`, `finish`, `viewpoint`, `parking` — and SHALL show its default glyph for an absent or unknown value. Extending the set SHALL require only a frontend change.

#### Scenario: Symbol text from GPX is kept verbatim

- **WHEN** a GPX file with a `<wpt>` carrying `<sym>Flag</sym>` is imported
- **THEN** the waypoint's symbol is the string `Flag` (not mapped to `flag`), and the Waypoints panel shows the default glyph for it because `Flag` is not a picker key

#### Scenario: Arbitrary symbol string is accepted

- **WHEN** a waypoint's symbol is set to `custom-key` through the set-symbol command
- **THEN** the command succeeds and the waypoint's symbol reads back as `custom-key`

#### Scenario: Clearing a symbol yields null

- **WHEN** a waypoint with symbol `camp` has its symbol cleared
- **THEN** the waypoint serializes with `"symbol": null` and the previous value `camp` is available to the inverse command

## MODIFIED Requirements

### Requirement: System exports waypoints to OziExplorer WPT

The system SHALL provide an "Export WPT" action for a waypoint layer (offered from the Waypoints panel layer menu, the Waypoint inspector and the command palette) that writes all waypoints of the chosen layer to a user-selected `.wpt` file in OziExplorer Waypoint File version 1.1 format. The file SHALL start with the four header lines `OziExplorer Waypoint File Version 1.1`, `WGS 84`, `Reserved 2`, `Reserved 3`, followed by one row per waypoint with 24 comma-separated fields: sequential number starting at 1, name (truncated to 14 characters, commas and line breaks replaced by spaces), latitude and longitude in WGS84 decimal degrees with six decimals, date `0`, symbol code, status `1`, map display format `0`, foreground colour `0`, background colour `65535`, empty description, `0`, `0`, `0`, altitude `-777`, font size `6`, font style `0`, symbol size `17`, `0`, `0`, `0` and three empty trailing fields. Waypoints model neither elevation nor timestamp, so altitude SHALL always be `-777` and date `0`. Every line SHALL be encoded as Windows-1251 and terminated with `\r\n`.

The symbol code SHALL be derived from the waypoint's symbol string: a string that parses as an integer SHALL be written as that number; known names SHALL map case-insensitively to OziExplorer codes — of the picker keys, `flag` → 9, `camp` → 18, `danger` → 19, `water` → 30 (further OziExplorer names such as `house`, `fuel`, `anchor`, `skull`, `star` are mapped as well); an absent symbol or any other string, including the picker keys `shelter`, `meeting-point`, `start`, `finish`, `viewpoint` and `parking`, SHALL be written as the default code `0`.

The export dialog SHALL pre-fill `<active bundle>/<layer name>.wpt` when a bundle is active and `<layer name>.wpt` otherwise; the user MAY choose any path. A failed export (layer not found, file cannot be created, write error) SHALL be reported as an error status message and SHALL NOT crash the application.

#### Scenario: Export waypoints with symbols to WPT

- **WHEN** the user invokes "Export WPT" on a layer with three waypoints whose symbols are `camp`, `flag` and none
- **THEN** the resulting `.wpt` file has the four header lines and exactly three rows of 24 fields, with symbol codes `18`, `9` and `0` respectively

#### Scenario: Cyrillic name is written as cp1251

- **WHEN** a waypoint named `Стоянка` is exported to WPT
- **THEN** its row contains the cp1251 byte sequence for `Стоянка`, the file contains no UTF-8 sequence for it, and every line ends with `\r\n`

#### Scenario: Picker key without a WPT code falls back to 0

- **WHEN** a waypoint with symbol `shelter` is exported to WPT
- **THEN** its symbol field is `0`

#### Scenario: WPT export dialog suggests `<bundle>/<layer>.wpt`

- **WHEN** the user invokes WPT export with a bundle active and a waypoint layer named `points`
- **THEN** the file picker pre-fills with `<bundle>/points.wpt`

#### Scenario: WPT export with no active bundle suggests filename only

- **WHEN** the user invokes WPT export with no active bundle
- **THEN** the file picker suggests `<layer>.wpt` only, without a directory
