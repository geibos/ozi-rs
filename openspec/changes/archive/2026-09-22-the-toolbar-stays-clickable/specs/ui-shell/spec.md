## ADDED Requirements

### Requirement: The workspace actions stay reachable at every width

The workspace context bar SHALL remain within the canvas column at every window
width and in every combination of open rails, and its actions — undo, redo,
save and the command-palette trigger — SHALL remain clickable. When the bar is
too narrow for everything it holds, the inert mode placeholders SHALL be
dropped first and the labels on the actions second; no action SHALL be moved
out of reach, clipped or covered.

#### Scenario: A track is selected on a laptop screen

- **WHEN** the operator selects a track, which opens the inspector, on a window narrow enough that the bar cannot hold both the mode placeholders and the actions
- **THEN** the placeholders are gone and all four actions are on screen and receive their own clicks

#### Scenario: The bar is narrower still

- **WHEN** the bar has room for the actions but not for their labels
- **THEN** the labels are dropped and the controls stay, rather than the controls overflowing

#### Scenario: A wide window

- **WHEN** the bar has room for everything
- **THEN** the mode placeholders and the labels are both shown
