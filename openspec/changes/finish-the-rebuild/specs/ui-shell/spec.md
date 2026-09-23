## ADDED Requirements
### Requirement: Every screen declares five states for the screenshot matrix

Each screen registered for the screenshot matrix SHALL define fixture data or transport configuration for the states `empty`, `loading`, `loaded`, `error` and `overflow` (lists larger than 1000 rows). Empty states SHALL include a primary next action; error states SHALL show the error toast or inline message; overflow states SHALL keep the UI responsive through virtualisation.

#### Scenario: Empty Tracks tab offers the next action

- **WHEN** the Tracks tab renders the `empty` state
- **THEN** it shows the message "Треков пока нет" (or its English equivalent) and an enabled Import button

#### Scenario: Overflow keeps the list virtualised

- **WHEN** the catalog screen renders the `overflow` state with 13 000 projects
- **THEN** fewer than 100 row elements exist in the DOM and scrolling repaints without dropped frames in the Playwright trace

