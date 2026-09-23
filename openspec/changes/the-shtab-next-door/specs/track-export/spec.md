## ADDED Requirements

### Requirement: A comma in a name is replaced, not escaped

When writing OziExplorer files, a comma inside a text field SHALL be replaced
with a space rather than written as the `chr(209)` escape the format reserves.

The escape is a byte, and these files are Windows-1251, where that byte is
`С`. Reading it as a comma would put one inside «СТАРТ», which settles the
import side.

The export side rests on one thing that has not been checked: whether the
original substitutes that byte when reading a cp1251 file regardless of who
wrote it. If it does, a name with a `С` in it is mangled whatever this
application writes, and not escaping only avoids adding a second way to be
wrong. A round trip through a Russian OziExplorer would settle it; nobody has
run one.

#### Scenario: A track named with a comma

- **WHEN** a track named «ЛИСА15, вечер» is exported to PLT or WPT
- **THEN** the field carries «ЛИСА15  вечер» and the record is not broken

#### Scenario: A name carrying a Cyrillic С

- **WHEN** a mark named «СТАРТ» makes the trip out and back
- **THEN** it is still named «СТАРТ»
