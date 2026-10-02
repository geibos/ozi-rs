## ADDED Requirements
### Requirement: Every screen declares its states for the screenshot matrix

Each screen registered for the screenshot matrix SHALL declare which of the states `empty`, `loading`, `loaded`, `error` and `overflow` (lists larger than 1000 rows) apply to it, and SHALL say for each how the stand reaches it. A screen SHALL declare every state an operator can actually meet on it: a list that can be empty declares `empty`, a list that comes from the network declares `loading` and `error`, a list that can hold thousands declares `overflow`. Empty states SHALL include a primary next action; error states SHALL show the error toast or inline message; overflow states SHALL keep the list virtualised.

A five-state grid for every screen would photograph states that do not exist — a settings screen has no overflow — and invent fixtures to fill them.

#### Scenario: Empty Tracks tab offers the next action

- **WHEN** the Tracks tab renders the `empty` state
- **THEN** it shows the message "Треков пока нет" (or its English equivalent) and an enabled Import button

#### Scenario: Overflow keeps the list virtualised

- **WHEN** the catalog screen renders the `overflow` state with 13 000 projects
- **THEN** no more than 100 row elements exist in the DOM, and the matrix fails if there are more
