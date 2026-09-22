## ADDED Requirements

### Requirement: Only the most recently requested project preview may land

The system SHALL apply only the preview result belonging to the project
currently selected, and SHALL discard any earlier preview's result — whether it
succeeded or failed — rather than letting it replace the shown map list or
report its outcome. Previewing fetches the map list asynchronously and the
operator may start another preview before the first answers.

A preview SHALL NOT acquire the busy flag, so that a click during the catalogue
walk is not ignored, and SHALL NOT release it, so that a preview finishing
during a download does not let a second download start.

#### Scenario: Clicking through several projects in a row

- **WHEN** the operator previews one project and then previews another before the first answers
- **AND** the first project's map list arrives last
- **THEN** the shown map list is the second project's, and the first result is discarded

#### Scenario: An abandoned preview fails

- **WHEN** a preview the operator has moved on from fails
- **THEN** no failure is reported for it

#### Scenario: A preview finishes while a bundle download is running

- **WHEN** a preview started before a download completes during that download
- **THEN** the application is still busy and no second download may start

### Requirement: The loader identifies an arriving preview by slug

The loader SHALL decide that a requested preview has arrived by matching the
project's slug and not its display name, because display names are not unique
in the catalogue.

While a preview is outstanding the loader SHALL keep showing that it is
waiting. A timer SHALL NOT end the wait; after a long wait the loader SHALL say
the wait is running long, and the wait SHALL end only when the map list arrives
or the request fails.

#### Scenario: Two catalogue entries share a display name

- **WHEN** the operator previews a project whose display name matches another entry's
- **THEN** the wait ends only when that project's own slug arrives

#### Scenario: The server is slow to answer

- **WHEN** a preview has been outstanding for longer than the loader's patience
- **THEN** the loader says the map list is still loading rather than showing it as arrived
