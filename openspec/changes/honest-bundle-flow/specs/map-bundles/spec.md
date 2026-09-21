## ADDED Requirements

### Requirement: A refused bundle download says why

When the system cannot start opening a bundle it SHALL report the reason rather
than returning as if it had started. A refusal because another bundle operation
is still running SHALL be distinguishable from a refusal because the project is
unknown, and the loader SHALL tell the operator which it was. While the
application is busy, the control that starts a download SHALL be disabled and
SHALL state that it is waiting.

#### Scenario: Pressing download during the catalogue refresh

- **WHEN** the operator asks to open a bundle while the project list is still loading
- **THEN** the request is refused with a "still loading" reason and the operator is told to wait

#### Scenario: Asking for a project that is not in the list

- **WHEN** the requested project slug is absent from the catalogue
- **THEN** the request is refused as an unknown project rather than silently doing nothing

### Requirement: Loader failures are visible

The bundle loader SHALL surface a failure to preview a project, to start a
bundle download, or to open a local bundle, instead of discarding it.

#### Scenario: The project listing cannot be read

- **WHEN** previewing a project fails
- **THEN** the operator sees an error naming the failure, not an empty map list

### Requirement: The progress panel belongs to the running download

The bundle progress panel SHALL be shown for the download that is running and
SHALL be released when it ends, so that a later unrelated operation does not
bring back a finished download's progress.

#### Scenario: Refreshing the project list after a download finished

- **WHEN** a bundle download has completed and the operator later refreshes the project list
- **THEN** no progress panel for the completed download is shown

### Requirement: Opening a map closes the loader

Opening a map SHALL clear the request to show the bundle loader, so the loader
does not reopen over the map that was just opened.

#### Scenario: Switching maps from the Maps tab

- **WHEN** the operator opens the loader from the Maps tab and then opens a map
- **THEN** the workspace shows that map with the loader closed

### Requirement: A downloaded bundle is recognised wherever its files sit

The system SHALL find a bundle's cached `.sqlitedb` maps anywhere inside that
bundle's directory, and SHALL recognise the bundle's coordinates file by the
same pattern the online listing is matched by.

#### Scenario: Maps stored outside the conventional folder

- **WHEN** a downloaded bundle keeps its `.sqlitedb` files in a subdirectory other than the conventional one
- **THEN** those maps are reported as downloaded and are opened from disk rather than fetched again

#### Scenario: A differently named coordinates file

- **WHEN** a cached bundle's coordinates file is not named exactly `2-Coordinates.txt`
- **THEN** the bundle still counts as cached and opens offline
