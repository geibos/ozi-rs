## ADDED Requirements

### Requirement: Track visibility can be changed for all tracks at once

The system SHALL provide commands to show every track and to hide every track in
the project, each applied as one operation rather than per-track toggles. The
Tracks tab SHALL expose both. Visibility remains a non-undoable style mutation,
so a bulk change SHALL mark the project dirty without adding undo entries.

#### Scenario: Hiding every track

- **WHEN** a project holds twenty-six visible tracks across several layers and the operator chooses hide-all
- **THEN** every track becomes hidden in a single state update, and the map redraws once

#### Scenario: Showing every track again

- **WHEN** some tracks are hidden and the operator chooses show-all
- **THEN** every track in every layer becomes visible

#### Scenario: Bulk visibility does not fill the undo stack

- **WHEN** the operator hides all tracks and then presses undo
- **THEN** the undo stack is unchanged by the bulk operation, consistent with per-track visibility toggles

### Requirement: One track can be isolated from the rest

The system SHALL provide an action that makes one track visible and hides every
other track in the project in a single operation, reachable from that track's
row.

#### Scenario: Isolating a track for inspection

- **WHEN** the operator picks "only this one" on the row for `20260709-ЛИСА15`
- **THEN** that track is visible and every other track in every layer is hidden

#### Scenario: Isolating a hidden track shows it

- **WHEN** the chosen track is itself hidden at the time of the action
- **THEN** it becomes visible while the others are hidden
