## ADDED Requirements

### Requirement: User can move a track point by dragging on the map

The system SHALL allow the user to enter edit mode and drag individual track points on the map; release SHALL commit the point's new coordinates as an undoable edit.

#### Scenario: Drag a single point

- **WHEN** the user drags a track point in edit mode and releases
- **THEN** the point's coordinates update to the drop location, the polyline re-renders, and a single (coalesced) undo step is added per the `undo-redo` spec

### Requirement: User can delete a track point via context menu

The system SHALL provide a right-click context menu on track points that includes a "Delete point" action; invoking it SHALL remove that point as an undoable edit.

#### Scenario: Delete a point

- **WHEN** the user right-clicks a track point and selects "Delete point"
- **THEN** the point is removed, the polyline updates, and the deletion is reversible via undo

### Requirement: User can insert a new track point

The system SHALL allow inserting a new point into a track segment between two existing points as an undoable edit.

#### Scenario: Insert a point

- **WHEN** the user inserts a new point between two existing points
- **THEN** the polyline updates to pass through the new point, and undo removes the inserted point

### Requirement: User can split a segment at a chosen point

The system SHALL provide a "Split segment at point" action that splits the containing segment into two consecutive segments at the chosen point as an undoable edit.

#### Scenario: Split at a midpoint

- **WHEN** the user splits a segment at one of its middle points
- **THEN** the track now contains two segments separated at that point; undo restores the single combined segment

### Requirement: User can join two adjacent segments

The system SHALL provide a "Join segments" action that merges two adjacent segments of the same track into one as an undoable edit.

#### Scenario: Join two segments

- **WHEN** the user joins two adjacent segments
- **THEN** the track has one segment containing the union of points; undo restores the two original segments

### Requirement: User can create new tracks by drawing on the map

The system SHALL provide a drawing mode that treats single-click map interactions as appending a point to the in-progress track, and treats double-click or Enter as committing the track. The Esc key SHALL cancel the in-progress draw without creating a track.

#### Scenario: Draw a new track

- **WHEN** the user activates drawing mode, clicks four points on the map, and double-clicks to finish
- **THEN** a new track with four points appears in the active track layer

#### Scenario: Cancel drawing with Esc

- **WHEN** the user is mid-draw with two points placed and presses Esc
- **THEN** no track is created and no project change is committed

### Requirement: System supports Douglas–Peucker simplification with live preview

The system SHALL provide a simplification action with a configurable tolerance slider. While the slider moves, the system SHALL show a live preview of the simplified track on the map. Confirming the action SHALL commit the simplification as a single undoable edit; cancelling SHALL leave the track unchanged.

#### Scenario: Preview and commit

- **WHEN** the user opens the simplify panel, adjusts tolerance, and clicks Apply
- **THEN** the simplified geometry is committed to the project and undo restores the original points

#### Scenario: Preview and cancel

- **WHEN** the user opens the simplify panel, adjusts tolerance, and cancels
- **THEN** the original track geometry is preserved and no undo step is added

## References

- ADR-0017: Delta-Based Undo/Redo
- ADR-0021: Undo Stack — Reaffirm Delta-Based, Depth 100, No Hybrid
