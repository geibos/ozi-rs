## ADDED Requirements

### Requirement: Map text is drawn from glyphs shipped with the application

The map SHALL draw its text from SDF glyphs bundled with the application and
SHALL NOT request glyphs over the network. The shipped set SHALL cover Basic
Latin and Latin-1, Latin Extended, Cyrillic, general punctuation, and `№`.

A character outside the shipped set SHALL be skipped. It SHALL NOT stop the
track lines, or any other name, from rendering: a glyph request that fails
holds up the whole tile it belongs to, and the tile carries the lines.

#### Scenario: A field launch with no network

- **WHEN** the application starts with no network and a project with named tracks
- **THEN** the tracks carry their names and nothing was fetched from outside the application

#### Scenario: A name with a character the fonts do not have

- **WHEN** a track is named with a character outside the shipped ranges
- **THEN** its line is drawn, and its name is drawn without that character

## MODIFIED Requirements

### Requirement: A track carries its name on the map

Every visible track that has a name SHALL show that name on the map, drawn in
the track's own colour along the route, and repeated along it at intervals so
that a stretch of the route long enough to hold the name carries it whatever
part of the route is on screen.

A day of recordings is a dozen coloured lines. Without names the only way to
find one is to read its colour off the list and hunt — and a single name at the
middle of a route is off screen as soon as the operator zooms in on one end of
it.

Where along the route a name falls SHALL be decided by the route as it is drawn
on screen, not by its points: a crew that stopped for twenty minutes logs a
crowd of points at the rest stop, and the crowd must not pull the name to it.

#### Scenario: A day's routes on the map

- **WHEN** the operator looks at a day of imported tracks
- **THEN** each visible track has its name on it, in its own colour

#### Scenario: A track that stopped for a break

- **WHEN** a track's points are crowded at one end and sparse across the rest
- **THEN** its name is drawn along the walked line, and the crowd of points at the stop does not decide where

#### Scenario: Zoomed in on one end of a long route

- **WHEN** the operator zooms in on a stretch of a long route far from its middle
- **THEN** that stretch carries the route's name

### Requirement: A name sits on a line the crew actually walked

A track's name SHALL be drawn along one of that track's segments, and SHALL
NOT be drawn across the gap between two segments.

A track is split where the recording stopped. The straight line across the gap
was never walked, and a name floating over empty forest belongs to nothing.

#### Scenario: A track recorded in two stretches a kilometre apart

- **WHEN** a track has two segments with a kilometre of nothing between them
- **THEN** its name sits on one of the segments, not between them
