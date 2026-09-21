## ADDED Requirements

### Requirement: The application state snapshot does not carry the catalogue

The application state snapshot SHALL NOT include the project catalogue. It is
fetched on every state change — once per file during a bundle download — and
the catalogue is thousands of rows, so carrying it there costs that payload on
every such change.

Building the snapshot SHALL NOT read the bundles directory, which was needed
only to mark catalogue rows.

The catalogue SHALL reach the interface as its own stream, and the interface
SHALL seed itself from its persisted cache, so that a cold start renders the
previous catalogue without waiting for either.

#### Scenario: A bundle download in progress

- **WHEN** a bundle download emits a state change for each file it finishes
- **THEN** no part of the catalogue is transferred with those state changes

#### Scenario: A cold start with a cached catalogue

- **WHEN** the application starts with a previously cached catalogue
- **THEN** the project list renders from the cache before any catalogue request completes
