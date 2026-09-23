## ADDED Requirements

### Requirement: The colour theme is chosen from the command palette

The command palette SHALL offer each colour theme as its own entry, mark the
one in use, and apply a chosen theme immediately as well as remembering it.

The theme selector this capability requires had no surface at all after the
workspace redesign: the picker component was mounted nowhere and the palette's
entry pointed at a sidebar that no longer existed.

#### Scenario: Choosing a theme

- **WHEN** the operator picks a theme from the palette
- **THEN** the interface changes to it at once, and the choice survives a restart

#### Scenario: Seeing which theme is in use

- **WHEN** the operator opens the palette
- **THEN** the theme currently in use is marked
