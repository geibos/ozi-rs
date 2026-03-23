# Component: Command Stack Foundation

Status: completed

## Why this next
- The roadmap places command abstractions and reversible operations immediately after the initial core entities.
- The current project model exists, but edits still bypass the explicit command model required by the architecture.
- Undo/redo needs a concrete seam before persistence and UI editing workflows can land safely.

## This slice
- Introduce application-level project edit commands for layer creation.
- Add a reversible history stack with undo and redo.
- Cover the new behavior with focused application tests.

## Out of scope
- UI command wiring
- track and waypoint point editing
- persistence of undo history
