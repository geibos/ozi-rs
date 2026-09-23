## ADDED Requirements

### Requirement: The map can draw a coordinate grid

The system SHALL be able to draw a latitude/longitude grid over the map,
turned on and off by the operator and remembered between sessions. The grid
SHALL be drawn beneath the tracks, the marks and the measurements.

Line spacing SHALL follow the zoom and SHALL always be a round quantity a
person can say aloud: a whole number of degrees, or 30, 20, 10, 5, 2 or 1
minutes, or the same in seconds. The spacing chosen SHALL be the coarsest that
still puts at least four lines across the view.

Each line SHALL be named where it meets the edge of the view — parallels at the
left, meridians at the top — in degrees, minutes and seconds with a hemisphere
letter, omitting the parts that are zero.

#### Scenario: Reading a sector off the map

- **WHEN** the grid is on over a view about two kilometres across
- **THEN** lines are drawn a minute apart and named `59°55'N`, `59°56'N` and so
  on

#### Scenario: The spacing follows the zoom

- **WHEN** the operator zooms from a fifty-metre view out to a ten-kilometre one
- **THEN** the spacing goes from seconds to minutes, and the number of lines on
  screen stays readable

#### Scenario: A line is named where it stands

- **WHEN** a parallel is drawn at 59.9375°
- **THEN** it is named `59°56'15"N`, and the name and the line are the same
  place however many steps were taken to reach it

#### Scenario: The grid is remembered

- **WHEN** the operator turns the grid on and reopens the application
- **THEN** the grid is on
