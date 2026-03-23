# Component: Track And Waypoint Entities

Status: started

## Why this next
- The requirements explicitly call out `Track`, `TrackSegment`, `TrackPoint`, and `Waypoint` as distinct concepts.
- Layer collections are in place, but they still need domain payloads before commands and persistence can be meaningful.
- This keeps the next command-model work grounded in real entities instead of placeholder counts.

## This slice
- Add core domain types for tracks, segments, track points, and waypoints.
- Attach `Track` collections to `TrackLayer` and `Waypoint` collections to `WaypointLayer`.
- Add focused unit tests for collection shape and ordering.

## Out of scope
- Geometry validation rules
- Editing commands
- Undo/redo
