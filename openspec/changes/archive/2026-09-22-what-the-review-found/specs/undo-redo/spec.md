## ADDED Requirements

### Requirement: One gesture is one undo step

Commands SHALL be merged into a single undo step only when the caller states
that they belong to the same gesture — one continuous action as the operator
would describe it — and they touch the same entity. A command with no gesture
SHALL be its own undo step.

#### Scenario: The same point dragged twice

- **WHEN** a point is dragged, released, and dragged again
- **THEN** undo takes back the second drag and leaves the first

#### Scenario: A continuous drag

- **WHEN** one drag produces several commands under the same gesture
- **THEN** they form a single undo step

### Requirement: A refused command leaves the redo stack alone

The redo stack SHALL be cleared only after a command has been applied. A
command the system refuses SHALL leave the history exactly as it was.

#### Scenario: An edit the system declines

- **WHEN** a command fails to apply after something was undone
- **THEN** redo is still available and still holds what it held
