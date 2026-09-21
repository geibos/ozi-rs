## ADDED Requirements

### Requirement: A waypoint can carry a colour of its own

A waypoint SHALL be able to carry a colour, set and cleared by the operator,
and the map SHALL draw its marker in that colour. The symbol says what a mark
is; the colour says whose it is, and a search collects marks from every group
working it.

Having no colour SHALL NOT be the same as having the default one: a waypoint
that has never been coloured SHALL follow whatever the map draws waypoints
with, and clearing a colour SHALL return it to that rather than setting a
colour that resembles it.

Setting or clearing a colour SHALL be undoable. A project saved before
waypoints could carry a colour SHALL load with every waypoint uncoloured.

#### Scenario: Two groups' marks on one map

- **WHEN** one waypoint is given a colour and another is left without
- **THEN** the map draws the first in that colour and the second in the default

#### Scenario: Clearing a colour

- **WHEN** the operator clears a waypoint's colour
- **THEN** the waypoint follows the default again, and the change can be undone

#### Scenario: A project from before this existed

- **WHEN** a project saved without waypoint colours is loaded
- **THEN** it loads, with every waypoint uncoloured
