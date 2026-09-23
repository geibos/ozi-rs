## ADDED Requirements

### Requirement: PLT export writes the OziExplorer 2.1 track layout

PLT export SHALL write exactly this layout, in this order:

1. `OziExplorer Track Point File Version 2.1`
2. `WGS 84`
3. `Altitude is in Feet`
4. `Field 1 = Lat, Field 2 = Lon, Field 3 = Code, Field 4 = Alt, Field 5 = Date, Field 6 = Stop, Field 7 = Bearing`
5. the track properties line with ten comma-separated fields in the order OziExplorer and the PLT importer expect: visibility flag `0` (shown), line width, colour as COLORREF, track name, then the fixed tail `0,0,2,8421376,-1,0` (skip value, track type, fill style, fill colour, closed flag, reserved)
6. the total point count
7. one row per point: latitude and longitude with six decimals, segment flag, altitude, OLE date with seven decimals, `DD-MM-YYYY` date and `HH:MM:SS` time

The segment flag SHALL be `1` on the first point of every segment and `0` otherwise. Altitude SHALL be metres converted to feet and rounded, or `-777` when the point has no elevation. A point without a timestamp SHALL get OLE date `0.0000000` and empty date and time fields. Line width SHALL be rounded and clamped to the range 1–7. Commas and line breaks in the track name SHALL be replaced by spaces so the properties line keeps ten fields.

#### Scenario: Exact header and properties line

- **WHEN** a track named `Direct` with colour RGB `#112233` and width 3.6 is exported to PLT
- **THEN** lines 1–4 are the four header lines above and line 5 is `0,4,3351057,Direct,0,0,2,8421376,-1,0`

#### Scenario: Segment starts are flagged

- **WHEN** a track with two segments of two points and one point is exported to PLT
- **THEN** the first row of each segment has segment flag `1` and the remaining row has `0`

#### Scenario: Export re-imports through the PLT importer

- **WHEN** a track named `20240601_Иванов` with width 3 and colour `#11AA33` is exported to PLT and the file is imported again
- **THEN** the imported track has the same name, width 3, colour `#11AA33` and the same number of points

### Requirement: PLT export text is Windows-1251 with CRLF line endings

Every line of a PLT export SHALL be encoded as Windows-1251 (cp1251) and terminated with `\r\n`; the file SHALL NOT contain UTF-8 sequences for non-ASCII text.

#### Scenario: Cyrillic track name

- **WHEN** a track named `Иванов` is exported to PLT
- **THEN** the file contains the cp1251 byte sequence for `Иванов` and does not contain its UTF-8 encoding

#### Scenario: Line endings

- **WHEN** any track is exported to PLT
- **THEN** every line ends with `\r\n` and no bare `\n` occurs
