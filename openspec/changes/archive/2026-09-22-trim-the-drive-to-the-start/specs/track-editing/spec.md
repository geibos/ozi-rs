## ADDED Requirements

### Requirement: A track can be trimmed at one of its points

The operator SHALL be able to remove everything in a track before a chosen
point, or everything after it, choosing the point in the points table. This is
the commonest edit to a recording — the first part of it is the drive to the
start — and the point is what the operator has, where a time or an extent is
something they would have to work out.

The chosen point SHALL be kept in either direction: it is where the walk begins
or ends.

A trim SHALL be a single undoable step that restores every removed point to its
place. A trim that removes nothing SHALL say so and SHALL NOT record a step.

#### Scenario: Cutting the drive to the start

- **WHEN** the operator trims everything before the point where the walking began
- **THEN** the earlier points are gone, that point remains, and one undo restores them all

#### Scenario: Trimming at an end

- **WHEN** the operator trims before the first point of a track
- **THEN** nothing is removed, they are told, and there is no step to undo
