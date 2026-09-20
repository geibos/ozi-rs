## ADDED Requirements

### Requirement: UI language defaults to Russian with a persisted, reachable switch

The UI SHALL render in Russian when no language preference is stored, regardless of OS locale. The ru/en switch SHALL be reachable from the workspace status bar and from the command palette, and the choice SHALL persist across restarts in `localStorage`. Every dictionary key SHALL exist in both `ru` and `en`; a unit test SHALL fail on a key missing from either dictionary.

#### Scenario: First launch is Russian

- **WHEN** the app starts with no stored language preference on an English-locale OS
- **THEN** the Library tabs read "Карты", "Треки", "Точки" and the status bar shows the language control

#### Scenario: Switch persists

- **WHEN** the user switches to English from the status bar and restarts the app
- **THEN** the UI starts in English

### Requirement: Icon-only controls carry a visible icon, an accessible label and a tooltip

Every button that renders an icon without visible text SHALL render a visible lucide icon, SHALL carry `aria-label` with the action name in the active language, and SHALL show a tooltip with the same text on hover or focus. A stand screenshot test SHALL fail when an icon button renders with no visible glyph (empty bounding box against the button background).

#### Scenario: Library row actions are legible

- **WHEN** the Tracks tab renders a row on the stand
- **THEN** each row action button shows its icon, exposes `aria-label` and a tooltip, and the screenshot assertion finds a non-empty glyph in each button

### Requirement: Every screen declares five states for the screenshot matrix

Each screen registered for the screenshot matrix SHALL define fixture data or transport configuration for the states `empty`, `loading`, `loaded`, `error` and `overflow` (lists larger than 1000 rows). Empty states SHALL include a primary next action; error states SHALL show the error toast or inline message; overflow states SHALL keep the UI responsive through virtualisation.

#### Scenario: Empty Tracks tab offers the next action

- **WHEN** the Tracks tab renders the `empty` state
- **THEN** it shows the message "Треков пока нет" (or its English equivalent) and an enabled Import button

#### Scenario: Overflow keeps the list virtualised

- **WHEN** the catalog screen renders the `overflow` state with 13 000 projects
- **THEN** fewer than 100 row elements exist in the DOM and scrolling repaints without dropped frames in the Playwright trace

### Requirement: Maps tab lists the active map independently of the current LizaAlert project

The Maps tab SHALL list the maps of the current LizaAlert project when one is loaded and SHALL additionally list the active map (`AppStateDto.active_map`) whenever it is set, including a locally opened OZI map that belongs to no catalog project. The empty state ("No maps in this project") SHALL appear only when there is neither a current project nor an active map.

#### Scenario: Local map is visible in the Maps tab

- **WHEN** the session restores a local OZI map and no LizaAlert project is current
- **THEN** the Maps tab shows one row for the active map, marked active, and no empty-state message

#### Scenario: Truly empty

- **WHEN** neither a current project nor an active map exists
- **THEN** the Maps tab shows the empty state with the "Open project…" action
