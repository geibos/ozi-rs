## ADDED Requirements

### Requirement: Removing a layer is undoable with its contents

Undoing the removal of a layer SHALL restore the layer together with
everything it held. Restoring the layer's name alone is a silent loss of the
work inside it.

#### Scenario: A layer of forty tracks removed and taken back

- **WHEN** a track layer holding forty tracks is removed and the operator undoes it
- **THEN** the layer is back with all forty tracks, their names, geometry and styles

#### Scenario: A waypoint layer removed and taken back

- **WHEN** a waypoint layer holding marks is removed and the operator undoes it
- **THEN** the layer is back with every mark, its symbol and its colour

### Requirement: Renaming a layer is undoable

A layer's name SHALL change through the command stack, so that renaming can be
undone and redone like every other edit.

#### Scenario: A rename taken back

- **WHEN** a layer is renamed and the operator undoes it
- **THEN** the layer carries the name it had before
