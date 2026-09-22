## ADDED Requirements

### Requirement: The selected track is picked out on the map

The map SHALL distinguish the selected track from the others without changing
the colour that identifies it, so that a row in the list and a route on the map
can be matched by eye while the map carries no names.

Selecting nothing SHALL leave every track drawn as it was.

#### Scenario: One route among a day's

- **WHEN** a track is selected while a day's recordings are on the map
- **THEN** that route is visibly marked out, and still drawn in its own colour

#### Scenario: Stepping down the list

- **WHEN** the operator moves from one track to the next
- **THEN** the mark follows the selection, one route at a time

#### Scenario: Nothing selected

- **WHEN** no track is selected
- **THEN** no route is marked
