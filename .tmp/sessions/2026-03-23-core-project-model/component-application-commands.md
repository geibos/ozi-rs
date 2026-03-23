# Component: Application Commands

Status: started

## Why this next
- The architecture requires non-trivial edits to go through explicit commands.
- The project now has enough domain structure to support concrete layer and entity editing commands.
- This creates the seam needed for later undo/redo improvements without pushing edit rules into the UI.

## This slice
- Extend the command model from layer creation to adding tracks and waypoints into layers.
- Return explicit application errors when commands reference missing layers.
- Keep history behavior predictable so failed commands do not pollute undo state.

## Out of scope
- Fine-grained inverse operations
- command validation for names and coordinates
- UI command wiring
