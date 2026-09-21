## ADDED Requirements

### Requirement: A catalogue link opens the search it names

A link to a project in the online catalogue SHALL open that project when the
operator pastes it, without their having to find the search by name. A link is
how a search reaches a crew, and its name is a transliteration they would
otherwise have to retype exactly.

Parsing SHALL tolerate what passing through a messenger does to a link — an
absent scheme, a missing or extra trailing slash, appended query parameters, a
percent-encoded name — and SHALL require the catalogue's own host, so that a
lookalike address is not treated as one.

Text that is not a catalogue link SHALL be treated as ordinary input rather
than guessed at.

A link naming a project the catalogue has not listed SHALL be reported as such,
naming it, rather than silently doing nothing.

#### Scenario: A link sent over a messenger

- **WHEN** the operator pastes a catalogue link for a listed project
- **THEN** that project is opened and identified by its name

#### Scenario: A link for a search not in the list yet

- **WHEN** the operator pastes a catalogue link naming a project the catalogue has not listed
- **THEN** they are told, and the name in the link is shown

#### Scenario: Ordinary text

- **WHEN** the operator types something that is not a catalogue link
- **THEN** no project is opened by it
