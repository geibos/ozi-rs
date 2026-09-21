## ADDED Requirements

### Requirement: A project removed upstream is removed locally

A catalogue walk that ran to its end SHALL replace the known catalogue rather
than being merged into it, so that a project no longer listed upstream stops
being listed, cached and offered.

A walk that was stopped SHALL NOT remove anything, because it read only a
prefix of the catalogue.

The interface SHALL be told where a walk begins and whether it completed, and
SHALL count only the rows sent within that walk — not the cached rows sent
before it — when deciding what no longer exists.

#### Scenario: A search is taken down between two refreshes

- **WHEN** a complete catalogue walk does not list a project the previous walk listed
- **THEN** that project is no longer in the list or in the cache

#### Scenario: The operator stops the refresh

- **WHEN** a catalogue walk is stopped partway
- **THEN** no project is removed from the list

#### Scenario: The cached rows sent before the walk

- **WHEN** a refresh emits the cached catalogue before walking
- **THEN** those rows alone do not count as evidence that a project still exists
