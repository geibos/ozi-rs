## ADDED Requirements

### Requirement: Two files of one bundle never share a partial path

A partial download SHALL be written to a path formed by appending a suffix to
the whole file name, so that two files of one bundle differing only in
extension never write to the same partial file.

#### Scenario: A bundle holding sheet.map and sheet.ozf2

- **WHEN** both files of that bundle are downloading at once
- **THEN** each writes to its own partial file and both land intact

#### Scenario: A partial path keeps what the file is

- **WHEN** a partial path is formed for `sheet.ozf2`
- **THEN** the original extension is still part of the name

### Requirement: A ready bundle file belongs to the map whose name it is

A file reported ready inside a bundle SHALL be matched to a map package by the
last component of its path, compared whole, rather than by whether the path
ends with the package's file name.

#### Scenario: A bundle holding map.ozf2 and bigmap.ozf2

- **WHEN** `bigmap.ozf2` finishes downloading
- **THEN** only `bigmap.ozf2` is marked available, and `map.ozf2` still needs downloading

### Requirement: A finished catalogue walk does not take the status line from a download

While a download is running, a catalogue walk that finishes SHALL report its
result to the diagnostics log without replacing the status line.

#### Scenario: The launch-time walk finishes mid-download

- **WHEN** the catalogue walk completes while a bundle is downloading
- **THEN** the status line still reports the download, and the walk's result is in the diagnostics log

#### Scenario: Nothing is downloading

- **WHEN** the catalogue walk completes with no download running
- **THEN** the status line reports how many projects were loaded

### Requirement: A cold launch with no link does not walk the catalogue

When the machine reports that it has no network at all, the application SHALL
NOT start the launch-time catalogue walk, and SHALL present the saved list as
saved rather than as the result of a refresh that failed.

The operator's own request to refresh SHALL run regardless: they can see the
state of the link better than the machine reports it.

#### Scenario: Launching in a field camp

- **WHEN** the application starts and the machine reports no network
- **THEN** no catalogue request is made and the catalogue is shown as the saved list with its age
