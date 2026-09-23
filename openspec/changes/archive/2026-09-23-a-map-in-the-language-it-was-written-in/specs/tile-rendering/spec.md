## ADDED Requirements

### Requirement: A `.map` is read in the encoding it was written in

The system SHALL decode `.map` files through the same encoding chain as `.plt`
files: a byte-order mark, then strict UTF-8, then statistical detection, then
Windows-1251. A `.map` whose title or raster file name is Cyrillic SHALL be
read with those names intact, and the raster it names SHALL be found.

A calibration this application writes SHALL be readable by this application,
whatever the picture is called.

#### Scenario: A map file from a Russian Windows

- **WHEN** a `.map` written in Windows-1251 names its raster `Сагра.png`
- **THEN** the name is read as written and the picture beside it is opened

#### Scenario: The round trip through calibration

- **WHEN** a picture called `Сагра.png` is calibrated and the resulting `.map`
  is then opened
- **THEN** the raster is served, and the corners resolve to the coordinates the
  operator gave
