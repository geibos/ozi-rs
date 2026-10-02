## ADDED Requirements

### Requirement: Settings live on one screen

The system SHALL provide a settings screen reachable from a control in the
workspace's top bar, from the command palette, and with ⌘, (Ctrl+, on
Windows). It SHALL hold the theme selector (the native themes and the
Catppuccin pack), the interface language, and the thresholds the track
statistics use. A changed value SHALL take effect at once and SHALL persist on
the machine across restarts.

Settings scattered over a palette list and a status-bar button are settings
nobody finds; the theme selector the `ui-shell` capability requires to be
reachable from settings had no settings to be reachable from.

#### Scenario: Opening settings

- **WHEN** the operator presses the gear in the top bar, chooses «Настройки» in the palette, or presses ⌘,
- **THEN** the settings screen opens with the current theme, language and thresholds shown

#### Scenario: A threshold survives a restart

- **WHEN** the operator sets the stop distance to 40 m and restarts the application
- **THEN** the settings screen shows 40 m and the statistics use it

#### Scenario: A value that is not a number

- **WHEN** the operator clears a threshold field or types something that is not a positive number
- **THEN** the previous value stays in force
