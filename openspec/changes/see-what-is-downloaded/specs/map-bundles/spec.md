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
