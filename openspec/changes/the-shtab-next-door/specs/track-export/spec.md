## ADDED Requirements

### Requirement: A comma in a name is replaced, not escaped

When writing OziExplorer files, a comma inside a text field SHALL be replaced
with a space rather than written as the `chr(209)` escape the format reserves.

The escape is a byte, and these files are Windows-1251, where that byte is
`С`. Writing it would put a comma inside every name the original reads back
that happens to contain that letter — which, in Russian, is most of them. One
character of a name is the price of not corrupting the rest.

#### Scenario: A track named with a comma

- **WHEN** a track named «ЛИСА15, вечер» is exported to PLT or WPT
- **THEN** the field carries «ЛИСА15  вечер» and the record is not broken

#### Scenario: A name carrying a Cyrillic С

- **WHEN** a mark named «СТАРТ» makes the trip out and back
- **THEN** it is still named «СТАРТ»
