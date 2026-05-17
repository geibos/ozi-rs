## ADDED Requirements

### Requirement: User can add a waypoint by clicking on the map

The system SHALL allow the user to enter waypoint placement mode and click on the map to create a new waypoint in the active waypoint layer at the clicked coordinates as an undoable edit.

#### Scenario: Place a waypoint

- **WHEN** the user activates waypoint placement and clicks on the map
- **THEN** a new waypoint marker appears at the clicked coordinates, appears in the Waypoints panel, and the creation is reversible via undo

### Requirement: User can move a waypoint by dragging on the map

The system SHALL allow dragging waypoint markers; drag-and-release SHALL commit the new coordinates as an undoable edit, with drag coalescing per the `undo-redo` spec.

#### Scenario: Drag a waypoint

- **WHEN** the user drags a waypoint marker to a new location
- **THEN** the waypoint's coordinates update and a single coalesced undo step is added

### Requirement: User can rename a waypoint

The system SHALL allow renaming a waypoint via the Waypoints panel as an undoable edit; the map label SHALL update to reflect the new name.

#### Scenario: Rename a waypoint

- **WHEN** the user edits a waypoint name in the panel and confirms
- **THEN** the new name appears in the panel, on the map label, and is reversible via undo

### Requirement: User can delete a waypoint

The system SHALL allow deleting a waypoint as an undoable edit. The marker and panel row SHALL disappear; undo SHALL restore the full waypoint including its symbol.

#### Scenario: Delete and undo restores symbol

- **WHEN** the user deletes a waypoint that has a symbol set, then invokes undo
- **THEN** the waypoint reappears with its previous coordinates, name, and symbol intact

### Requirement: Waypoints support an optional symbol

The system SHALL allow assigning an optional symbol (e.g. flag, camp, danger, water, shelter) to each waypoint. Symbol changes SHALL be undoable, with the previous symbol value (including "no symbol") stored in the inverse command.

#### Scenario: Symbol picker commits choice

- **WHEN** the user picks a "shelter" symbol for a waypoint
- **THEN** the waypoint renders with the shelter glyph and undo restores the previous symbol (or absence of symbol)

### Requirement: Multiple waypoints render simultaneously with per-waypoint visibility

The system SHALL render all waypoints in visible layers simultaneously and SHALL allow toggling individual waypoint visibility; hidden waypoints SHALL NOT render on the map.

#### Scenario: Toggle a waypoint hidden

- **WHEN** the user toggles a waypoint's visibility off
- **THEN** its marker disappears from the map but the panel row remains and indicates hidden state

### Requirement: System exports waypoints to GPX

The system SHALL provide an "Export waypoints to GPX" action that writes all waypoints in a chosen layer to a user-selected `.gpx` file.

#### Scenario: Export waypoints to GPX

- **WHEN** the user exports waypoints from a layer with three named waypoints
- **THEN** the resulting `.gpx` contains three `<wpt>` elements with names, coordinates, and (where present) symbols

### Requirement: System exports waypoints to PLT

The system SHALL provide an "Export waypoints to PLT" action that writes waypoints to a user-selected `.plt` file in the OziExplorer PLT format.

#### Scenario: Export waypoints to PLT

- **WHEN** the user exports a waypoint collection to PLT
- **THEN** the resulting `.plt` file is valid OziExplorer PLT containing each waypoint

## References

- ADR-0018: Waypoint Symbols as Optional Strings
