# Persistence and Session Behavior

This document tracks what is persisted by project files, application settings, and planned
session restore work. It is intentionally conservative until implementation tasks update it
with verified behavior.

## Persisted

- Project files (`.ozp`) persist project data such as tracks, waypoints, layers, and map
  references that belong to the project model.
- Theme selection is persisted separately by the frontend.
- Planned in this reconciliation: remember the last project path and active map reference
  for startup restore when the referenced files still exist.

## Not Persisted

- Viewport/camera position.
- Selected tracks, waypoints, or points.
- Panel open/closed state.
- Bundle-loader window visibility or other window layout state.
- Unsaved project edits outside an explicit project save.

## Restore Flow

Planned in this reconciliation: on startup, load a saved session record, attempt to reopen
the last project path, then reactivate the previously active map if it is still available.
If restore cannot complete safely, the app should continue with a fresh state.

## Missing File Behavior

Missing project or map files must not panic. Planned behavior is to emit a warning or
diagnostic, skip the missing item, and keep the application usable.

## Tests

Planned in this reconciliation: tests should cover successful last-project/active-map
restore and graceful handling for missing project or map files.
