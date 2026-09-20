## ADDED Requirements

### Requirement: User can sort a track's points by time as one undoable step

The system SHALL provide a "Sort by time" action for the selected track that reorders the points of every segment by timestamp, independently per segment, using a stable sort in which points without a timestamp come first in their existing relative order. The reorder SHALL be committed as a single `ReorderTrackPoints` command whose reverse restores the previous order exactly. When every segment is already in sorted order the action SHALL be a no-op that adds no undo entry.

#### Scenario: Shuffled track is sorted, untimed points first

- **WHEN** a segment holds points with timestamps `T3, T1, (none), T2` and the user invokes Sort by time
- **THEN** the segment order becomes `(none), T1, T2, T3` and the point set is unchanged

#### Scenario: Undo restores the previous order

- **WHEN** the user sorts a shuffled track and invokes undo
- **THEN** every segment shows its pre-sort point order

#### Scenario: Already sorted track adds no undo step

- **WHEN** the user invokes Sort by time on a track whose segments are already ordered
- **THEN** the project is unchanged and the undo depth does not grow

### Requirement: User can crop a track to the map extent or a time range

The system SHALL provide two crop actions for the selected track: crop to the current map viewport (after a confirmation dialog, and only once the map has published viewport bounds) and crop to a time range with optional lower and upper bounds. Crop to extent SHALL remove every point outside the bounding box; crop to time SHALL remove only timestamped points outside the range and SHALL always keep points without a timestamp. All removals SHALL be committed as a single `CropTrackPoints` command whose reverse re-inserts each removed point with its original id at its original index. The action SHALL report the number of removed points. When nothing falls outside the crop the action SHALL be a no-op with no undo entry. A crop that would remove every point of the track SHALL be rejected and leave the project unchanged.

#### Scenario: Crop to view removes outside points and reports the count

- **WHEN** four of a track's points lie outside the current viewport and the user confirms Crop to view
- **THEN** exactly those four points are removed, the polyline re-renders and the UI reports 4 removed points

#### Scenario: Crop by time keeps untimed points

- **WHEN** a track holds timestamped points before, inside and after the range plus points without a timestamp, and the user crops to that range
- **THEN** only the timestamped points outside the range are removed and every untimed point remains

#### Scenario: Crop of every point is rejected

- **WHEN** the requested range or extent contains none of the track's points
- **THEN** the crop fails, no point is removed and no undo entry is added

#### Scenario: Undo restores cropped points in place

- **WHEN** the user crops a track and invokes undo
- **THEN** each removed point is back with its original id at its original index and the point count equals the pre-crop count

## MODIFIED Requirements

### Requirement: User can split a segment at a chosen point

The system SHALL provide a "Split segment at point" action, offered for every point of a segment except its last one, that splits the containing segment into two consecutive segments as an undoable `SplitSegment` command. The chosen point SHALL remain the last point of the original segment; every following point SHALL move to a new segment (id `max segment id + 1` within the track) inserted directly after the original segment. No point SHALL be duplicated and no point id SHALL change. Splitting at the last point of a segment SHALL be rejected. The reverse of a split SHALL be the corresponding `JoinSegments`.

#### Scenario: Split at a midpoint

- **WHEN** the user splits a segment at one of its middle points
- **THEN** the track now contains two segments separated at that point; undo restores the single combined segment

#### Scenario: Split keeps every point exactly once

- **WHEN** a segment holds points `10, 11, 12` and the user splits at point `11`
- **THEN** the left segment holds `10, 11`, the right segment holds `12`, and the track's total point count is unchanged

#### Scenario: Last point cannot be a split point

- **WHEN** the user selects the last point of a segment
- **THEN** the split action is not offered, and a `SplitSegment` at that point is rejected by the command

### Requirement: User can join two adjacent segments

The system SHALL provide a "Join segments" action that merges two segments of the same track into one as an undoable `JoinSegments` command. The second segment SHALL be the one immediately following the first in the track's segment order; joining non-adjacent segments SHALL be rejected. Both segments SHALL be non-empty; joining an empty segment SHALL be rejected. The join SHALL append the second segment's points, in order and with their ids unchanged, to the first segment and remove the second segment. The reverse of a join SHALL be a `SplitSegment` at the first segment's former last point, so undo restores both segments exactly.

#### Scenario: Join two segments

- **WHEN** the user joins two adjacent segments
- **THEN** the track has one segment containing the union of points; undo restores the two original segments

#### Scenario: Non-adjacent segments cannot be joined

- **WHEN** the user attempts to join the first and third segments of a track
- **THEN** the command is rejected and the track is unchanged

#### Scenario: Split then join is exact

- **WHEN** the user splits a segment and then joins the two halves
- **THEN** the resulting segment's points and ids equal the original segment's points and ids

### Requirement: User can create new tracks by drawing on the map

The system SHALL provide a drawing mode that creates the new track in the active track layer through a `CreateEmptyTrack` command when the mode starts, appends one `InsertTrackPoint` command per single click on the map, and treats double-click or Enter as finishing the track. The Esc key SHALL cancel the in-progress draw by discarding every command issued during the draw (`CreateEmptyTrack` plus every insertion) from the command stack without pushing them onto the redo stack, and SHALL restore the project's dirty flag to its value before the draw started, so that a cancelled draw leaves no track, no redo entry and no unsaved-changes indicator.

#### Scenario: Draw a new track

- **WHEN** the user activates drawing mode, clicks four points on the map, and double-clicks to finish
- **THEN** a new track with four points appears in the active track layer

#### Scenario: Cancel drawing with Esc

- **WHEN** the user is mid-draw with two points placed and presses Esc
- **THEN** no track from the draw remains in the layer, Redo is unavailable for the cancelled draw, and the dirty indicator shows the pre-draw state

#### Scenario: Cancelling a draw on a saved project keeps it saved

- **WHEN** the project is saved, the user starts a draw, places one point and presses Esc
- **THEN** the project reports no unsaved changes and the undo stack is as it was before the draw
