## ADDED Requirements

### Requirement: A `.map` beside an ordinary picture opens like an OZF2 one

The system SHALL open an OziExplorer `.map` file whose raster is an ordinary
image — JPEG, PNG, TIFF, BMP, GIF or WebP — and SHALL serve its tiles through
the same `ozi://` protocol, with the same level and tile geometry, as a `.map`
whose raster is an OZF2 file. Nothing above the raster adapter SHALL need to
know which of the two it was given.

Levels SHALL be built by halving the picture until its shorter side reaches one
tile, and tiles SHALL be 256 px square except at the right and bottom edges,
where a tile is only as large as the pixels that are there.

A picture whose pixel count exceeds what the application will hold in memory
SHALL be refused before its pixels are read, with a message naming the file and
its size. `.ozfx3` SHALL remain refused.

#### Scenario: A scan opens

- **WHEN** the operator opens a `.map` file that names `sheet.png` beside it
- **THEN** the map opens and its tiles are served, exactly as for an OZF2 pair

#### Scenario: Edge tiles are only as big as what is there

- **WHEN** the picture is 600 px wide and a tile in the third column is
  requested
- **THEN** that tile is 88 px wide, and a tile in a fourth column is an error

#### Scenario: A picture that is not there names itself

- **WHEN** a `.map` names a raster that does not exist beside it
- **THEN** the failure names the missing file rather than reporting an
  unsupported format

#### Scenario: A picture too large to hold is refused by size

- **WHEN** the named picture has more pixels than the application will hold
- **THEN** the open fails with a message giving the file and its dimensions,
  and no attempt is made to decode it
