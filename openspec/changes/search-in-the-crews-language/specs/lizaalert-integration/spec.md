## ADDED Requirements

### Requirement: The catalogue filter accepts the crew's own language

The catalogue filter SHALL match a query written in Russian against project
names written in latin transliteration. Because no single transliteration is in
use, a Cyrillic letter SHALL match any of the latin spellings in common use for
it, and SHALL also match itself, so an entry written in Cyrillic is still
found. A query containing no Cyrillic SHALL behave exactly as before, matching
the name or the slug as a literal substring. A space in the query SHALL match
whichever separator the catalogue uses between words.

#### Scenario: Typing the Russian name of the search

- **WHEN** the catalogue holds a project named `2026 09 20 Schuvalovo` AND the operator types `Шувалово`
- **THEN** that project is listed, and the match count reflects it

#### Scenario: The same letter spelled two ways

- **WHEN** the catalogue holds both `Shuvalovo` and `Schuvalovo` AND the operator types `Шувалово`
- **THEN** both are listed

#### Scenario: A query that matches nothing

- **WHEN** the operator types a Russian name no project carries
- **THEN** the list is empty and the count says so, rather than falling back to everything

#### Scenario: A latin query is unchanged

- **WHEN** the operator types `Sagra`
- **THEN** the result is the same list the literal substring match produced before this change
