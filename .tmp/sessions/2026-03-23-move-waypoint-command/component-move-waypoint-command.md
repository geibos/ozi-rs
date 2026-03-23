# Component: Move Waypoint Command

Status: started

## Why this next
- The add-track and add-waypoint commands already exist, so repeating another insertion slice would duplicate completed work.
- The next useful Phase 1 step is a real edit of existing data rather than more creation scaffolding.
- Moving a waypoint is the smallest explicit edit that exercises entity lookup, mutation, command routing, and undo safety.

## This slice
- Add project support for moving a waypoint in a specific waypoint layer.
- Add an application command that routes waypoint movement through the undo/redo stack.
- Cover success, missing-layer, and missing-waypoint behavior with focused tests.

## Out of scope
- UI controls for waypoint editing
- waypoint renaming or deletion
- track-point editing and segment operations
