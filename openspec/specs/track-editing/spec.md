# track-editing Specification

## Purpose
Editing the geometry and structure of tracks that already exist in a project: moving, deleting and inserting points, splitting and joining segments, drawing new tracks point by point, Douglas–Peucker simplification with live preview, sorting points by time and cropping to the map extent or a time range. Every edit is a `ProjectCommand` applied through the `CommandStack`, so this spec states the per-operation behaviour and invariants while the `undo-redo` spec owns the stack itself (coalescing, depth, identifier stability, dirty tracking). Track styling and statistics live in `track-display`.

### Decision history

- ADR-0017 (2026-04-04, accepted): point drags are issued as `MoveTrackPoint` commands and coalesced by `apply_or_merge()`; every edit must carry enough data to compute its reverse at apply time; rationale: snapshot undo could not survive drag streams on large tracks. Codified as: User can move a track point by dragging on the map (the coalescing rule itself lives in `undo-redo`).
- ADR-0020 (2026-04-28, accepted): MVP scope lists segment splitting, sort by timestamp and crop "by current map extent, by time range, by selected points"; rationale: these are the operations SAR volunteers use to turn a noisy group track into a clean route. Codified as: User can split a segment at a chosen point; User can sort a track's points by time as one undoable step; User can crop a track to the map extent or a time range. Crop by selected points is not codified because it is not implemented (deferred pending multi-select, `docs/customer-journeys.md` CJ-4 status).
- Commit f5f44bb (2026-07-14, landed): split keeps the split point only in the left half and moves the following points to a new segment inserted right after it; join requires adjacency and non-empty segments so that `SplitSegment` and `JoinSegments` are exact inverses; rationale: the previous split duplicated the split point and every undo/redo cycle grew a phantom point with a duplicate id. Codified as: User can split a segment at a chosen point; User can join two adjacent segments.
- CJ-4 slice 1.2, commit c60dc6e (2026-07-15, landed; `docs/customer-journeys.md`): `ReorderTrackPoints` (per-segment stable sort, untimed points first, no-op when already sorted) and `CropTrackPoints` (extent or time range, untimed points always kept, all-points crop rejected, removed count reported), split/join UI in the Inspector segments table, track selection by clicking its line; rationale: cleaning a 5000-point track must take minutes and every step must be reversible. Codified as: User can sort a track's points by time as one undoable step; User can crop a track to the map extent or a time range.
- Drawing mode (as implemented in `src/components/MapView.svelte:129-160, 623-650`): the track is created by `CreateEmptyTrack` when the mode starts, each click is an `InsertTrackPoint` command, and Esc cancels by undoing all of them; no ADR records this. Codified as: User can create new tracks by drawing on the map (modified to describe the command-per-click model; the earlier wording "no project change is committed" did not match the code).
- Owner decision (2026-09-19): cancelling a draw with Esc discards the draw commands without a redo entry and restores the dirty flag; the previous undo-based cancel (commands left on the redo stack, project marked dirty) is replaced. Codified as the modified "User can create new tracks by drawing on the map"; implemented in `revive-ui-cycle` slice 0.3.

## Requirements

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

### Requirement: System supports Douglas–Peucker simplification with live preview

The system SHALL provide a simplification action with a configurable tolerance slider. While the slider moves, the system SHALL show a live preview of the simplified track on the map. Confirming the action SHALL commit the simplification as a single undoable edit; cancelling SHALL leave the track unchanged.

The tolerance SHALL be the deviation in **metres** that the slider states, at
every layer it passes through. The simplification algorithm works in
kilometres; the conversion SHALL happen once, at the boundary, in a parameter
whose name carries the unit.

#### Scenario: Preview and commit

- **WHEN** the user opens the simplify panel, adjusts tolerance, and clicks Apply
- **THEN** the simplified geometry is committed to the project and undo restores the original points

#### Scenario: Preview and cancel

- **WHEN** the user opens the simplify panel, adjusts tolerance, and cancels
- **THEN** the original track geometry is preserved and no undo step is added

#### Scenario: The slider's number is metres

- **WHEN** the operator simplifies a track at a tolerance of 10 m, and the
  track has a 50 m deviation in it
- **THEN** that deviation is kept, because it is five times the tolerance the
  operator set

#### Scenario: A tolerance that is noise

- **WHEN** the same track is simplified at 100 m
- **THEN** the 50 m deviation is removed

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

### Requirement: Bringing everything into view frames the data

Framing the camera on everything the project holds SHALL use the tracks and the
marks as stored, not the markers that have been drawn on the map so far, so
that the result does not depend on how much of an asynchronous redraw has
finished. A layer whose marks could not be read SHALL fall back to what is
drawn for it.

#### Scenario: Asking for everything right after opening a project

- **WHEN** the operator asks to see everything before the markers have finished being drawn
- **THEN** the camera frames the marks as well as the tracks

### Requirement: A refused edit tells the operator

An edit that the system declines SHALL be reported to the operator, carrying
the reason the backend gave, rather than only being written to a developer
console which is not present in a release build. This covers moving, deleting
and inserting a track point, placing a point while drawing, cancelling a draw,
and bringing the tracks into view.

Where the refused edit has already been shown on the map, the map SHALL be
brought back into agreement with the stored data rather than continuing to
show the edit that did not happen.

#### Scenario: A track point dragged to somewhere the system refuses

- **WHEN** a track point is dragged and the move is declined
- **THEN** the operator is told, with the reason, and the point is shown at its stored position

#### Scenario: Drawing a point that cannot be placed

- **WHEN** placing a point while drawing is declined
- **THEN** the operator is told, with the reason

### Requirement: Simplifying shows its cost before it is applied

The simplify control SHALL show how many points the track has and how many it
would keep, from the moment it opens, at whatever tolerance it opens with, and
from every entrance.

Without it the operator can apply a simplification having never been told what
it would remove, which is the one thing a live preview exists to prevent.

#### Scenario: Opened from the Track Inspector

- **WHEN** the operator opens the simplify control from the inspector
- **THEN** it shows the current and resulting point counts without them having to move anything

#### Scenario: Changing the tolerance

- **WHEN** the operator moves the tolerance
- **THEN** the counts follow it, and one settled change fetches one preview

### Requirement: An edit that changes a track's shape invalidates what is drawn from it

Every operation that changes a track's geometry SHALL invalidate the cached
track detail and the drawn line.

The statistics, the segment table and the map are three views of one track. An
operation that moves one and not the others reads as having done nothing,
which is worse than an error.

#### Scenario: Simplifying a track

- **WHEN** the operator simplifies a track
- **THEN** the statistics, the segment table and the line on the map all show the simplified track

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

### Requirement: Every click while drawing becomes a point

While drawing a track, each click on the map SHALL add exactly one point,
however quickly the clicks follow one another. A click SHALL NOT be discarded
because another click arrived, and points SHALL be added in the order they were
clicked.

A double-click SHALL finish the track and SHALL NOT add either of its own
clicks as a point.

A point the backend refuses SHALL be reported and SHALL NOT stop the points
clicked after it.

#### Scenario: A route plotted at speed

- **WHEN** the operator clicks six places on the map 90 ms apart
- **THEN** the track has six points, in the order they were clicked

#### Scenario: Finishing with a double-click

- **WHEN** the operator double-clicks to finish
- **THEN** the track is finished and neither half of the double-click became a
  point

#### Scenario: One point that could not be added

- **WHEN** one click in a burst fails to reach the project
- **THEN** the failure is reported and the clicks after it still become points
