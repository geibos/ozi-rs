## ADDED Requirements

### Requirement: A waypoint's symbol is drawn on the map

A waypoint's symbol SHALL be shown on its map marker, not only in the lists and
the exports, because the map is where a crew reads which mark is which — the
task point, what was found, where the danger is.

The marker SHALL remain findable as a marker: the symbol is drawn on a shape
that stands out against imagery rather than alone. A symbol the build does not
recognise SHALL fall back to the default marker rather than leaving the
waypoint undrawn.

Changing a waypoint's symbol SHALL redraw its marker.

#### Scenario: Marks of different kinds on one search

- **WHEN** the map shows a waypoint with a symbol and one without
- **THEN** the first draws that symbol's glyph and the second the default one, and both are legible as markers

#### Scenario: A symbol from another tool

- **WHEN** a waypoint carries a symbol this build does not know
- **THEN** its marker is drawn with the default symbol
