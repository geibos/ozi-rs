## ADDED Requirements

### Requirement: System provides a Catppuccin theme selector with five options

The system SHALL apply one of the Catppuccin palettes — Auto (follow OS), Latte, Frappé, Macchiato, or Mocha — to the UI via CSS custom properties (`--ctp-*`). The Auto option SHALL track the OS light/dark preference dynamically.

#### Scenario: Pick a manual theme

- **WHEN** the user selects "Mocha" from the theme picker
- **THEN** the UI re-renders with the Mocha palette applied via `--ctp-*` variables

#### Scenario: Auto follows OS

- **WHEN** the user selects "Auto" and the OS toggles between light and dark mode
- **THEN** the UI switches between Latte (light) and Mocha (dark) accordingly

### Requirement: Theme choice persists across sessions via localStorage

The system SHALL persist the selected theme in browser localStorage and SHALL restore it on the next session. Theme is intentionally NOT stored in the Rust session file (see `project-persistence`).

#### Scenario: Theme survives restart

- **WHEN** the user selects "Frappé" and restarts the application
- **THEN** the UI starts in Frappé without prompting the user

### Requirement: Backtick key toggles an in-app developer console

The system SHALL toggle visibility of an in-app developer console whenever the user presses the backtick (`` ` ``) key while the application has focus.

#### Scenario: Open and close console

- **WHEN** the user presses backtick once and then again
- **THEN** the developer console appears on the first press and disappears on the second

### Requirement: F3 toggles an FPS counter overlay

The system SHALL toggle visibility of a frame-rate counter overlay whenever the user presses F3. The counter SHALL display real-time FPS computed from frame times.

#### Scenario: Toggle FPS overlay

- **WHEN** the user presses F3
- **THEN** an FPS overlay appears in a corner of the application window and updates continuously until F3 is pressed again
