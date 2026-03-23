# Component: Core Project Model

Status: started

## Why this next
- The roadmap says Phase 1 should begin with the core model and explicit architectural boundaries.
- The repo already has a UI shell, but the domain still only models a project name.
- Independent map, track, and waypoint layers are a prerequisite for commands, persistence, and import/export.

## This slice
- Introduce stable project and layer entities in the domain.
- Add tests for empty/default state and independent layer collections.
- Surface project counts through the application and UI shell.

## Out of scope
- Undo/redo
- edit commands
- persistence and external formats
