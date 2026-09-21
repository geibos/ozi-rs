## ADDED Requirements

### Requirement: Waypoints can be found by name

The Waypoints tab SHALL offer a search field that narrows the listed waypoints to
those whose name contains the query, matched case-insensitively as a substring in
either alphabet. While a query is active the tab SHALL report how many waypoints
are shown out of the total and SHALL offer a way to clear the query. The matching
rule SHALL be the same one the Tracks tab uses.

#### Scenario: Narrowing a long list

- **WHEN** a project holds waypoints named `ШТАБ`, `Задача 1` and `Задача 2` and the operator types `задача`
- **THEN** only `Задача 1` and `Задача 2` are listed and the tab reports 2 of 3

#### Scenario: No waypoint matches

- **WHEN** the query matches no waypoint name
- **THEN** the tab says so, distinctly from the empty state shown when the project has no waypoints at all

#### Scenario: Clearing the query

- **WHEN** the operator clears the search field
- **THEN** every waypoint is listed again

### Requirement: Waypoint visibility can be changed for all waypoints at once

The system SHALL provide commands to show every waypoint and to hide every
waypoint in the project, each applied as one operation rather than per-waypoint
toggles, and SHALL provide an action that leaves one waypoint visible and hides
the rest. The Waypoints tab SHALL expose all three. Visibility remains a
non-undoable style mutation, so these operations SHALL mark the project dirty
without adding undo entries.

#### Scenario: Hiding every waypoint

- **WHEN** a project holds visible waypoints across several layers and the operator chooses hide-all
- **THEN** every waypoint becomes hidden in a single state update

#### Scenario: Isolating one waypoint

- **WHEN** the operator picks "only this one" on a waypoint's row
- **THEN** that waypoint is visible and every other waypoint in every layer is hidden

#### Scenario: Isolating a waypoint that does not exist

- **WHEN** the requested waypoint is absent from the project
- **THEN** visibility is left as it was rather than hiding everything

### Requirement: A waypoint row shows where it is and can put it on the map

Each waypoint row SHALL display the waypoint's coordinates, and SHALL offer a
control that moves the map to that waypoint without changing which waypoint is
selected for editing.

#### Scenario: Locating a waypoint

- **WHEN** the operator presses "show on map" on a waypoint row
- **THEN** the map centres on that waypoint's coordinates

#### Scenario: Reading a waypoint's position

- **WHEN** a waypoint sits at 55.75123, 37.61754
- **THEN** its row shows those coordinates to five decimal places, the same precision the track point list uses
