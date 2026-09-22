## ADDED Requirements

### Requirement: The project list says which bundles are on disk

Each project summary SHALL carry whether that bundle is already downloaded and
openable without a network, and the project list SHALL mark those projects. The
list SHALL offer a way to show only them. A summary restored from a stale local
cache SHALL default to "not downloaded" rather than claiming availability.

#### Scenario: Choosing a bundle with no signal

- **WHEN** the operator opens the project list offline and asks for downloaded bundles only
- **THEN** only the bundles present on disk are listed, each marked as downloaded

#### Scenario: A cache written before the flag existed

- **WHEN** the catalogue is restored from a local cache whose entries carry no availability flag
- **THEN** those projects are treated as not downloaded until the backend reports otherwise

### Requirement: The project filter matches what the operator types

The project filter SHALL match the query against both the project name and its
slug, and the list SHALL report how many projects are shown out of the total.

#### Scenario: Searching by date

- **WHEN** the operator types a date as it appears in the slug, such as `2026-09-21`
- **THEN** the projects of that date are listed, although their display names spell the date differently

#### Scenario: Reading how much the filter narrowed

- **WHEN** a query is active
- **THEN** the list reports the number shown out of the catalogue total

### Requirement: A single-map download is visible and can be stopped

Downloading one map package SHALL report a download identifier to the caller,
SHALL show the same progress panel a whole-bundle download shows, and SHALL be
cancellable. Both download paths SHALL announce when they stop, and a failure
SHALL be reported to the operator rather than only ending the progress display.

#### Scenario: Opening a map that is not on disk

- **WHEN** the operator opens a map package that has not been downloaded
- **THEN** the progress panel shows that download and offers to cancel it

#### Scenario: Cancelling a map download

- **WHEN** the operator cancels a running single-map download
- **THEN** the download stops, the partial file is removed, and the panel closes

#### Scenario: A download that fails

- **WHEN** a download ends in failure
- **THEN** the operator is told it failed, rather than only seeing the panel disappear

#### Scenario: Two downloads in flight

- **WHEN** one download finishes while another is still running
- **THEN** the panel keeps showing the running one
