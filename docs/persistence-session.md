# Persistence and Session Behavior

This document tracks what is persisted by project files, application settings, and startup
session restore. Session restore is intentionally bounded: it remembers only the last project
file path and the active map reference/path needed to reopen that map if the files still exist.

## Persisted

- Project files (`.ozp`) persist project data such as tracks, waypoints, layers, and map
  references that belong to the project model.
- Theme selection is persisted separately by the frontend.
- The app session file persists the last loaded/saved project path and the active map selection:
  map kind, display names, remote URL, local map path, map center, and base zoom.

## Not Persisted

- Viewport/camera position.
- Selected tracks, waypoints, or points.
- Panel open/closed state.
- Bundle-loader window visibility or other window layout state.
- Unsaved project edits outside an explicit project save.
- Undo/redo history.
- Theme selection inside the Rust session file or inside `.ozp` project files.

## Restore Flow

On startup, the Rust application creates a fresh `AppState`, then loads the session file before
Tauri command handlers expose state to the frontend. If the recorded project path exists and can
be loaded, it becomes the current project path. If the recorded active map path still exists and
has a supported map kind, it becomes the active map. The project file format is unchanged: session
data is stored separately and is never embedded in `.ozp` files.

## Missing File Behavior

Missing or corrupt session files, missing project files, missing map files, and unsupported stored
map kinds never panic. Missing session files are ignored. Corrupt sessions and missing referenced
files add diagnostics/status messages and keep the application usable. A missing project starts
fresh; a missing map leaves any successfully restored project loaded but skips active-map restore.

## Tests

Rust tests cover successful last-project/active-map restore and graceful handling for missing
project/map paths:

```bash
cargo test --manifest-path src-tauri/Cargo.toml session_restore_valid -- --nocapture
cargo test --manifest-path src-tauri/Cargo.toml session_restore_missing -- --nocapture
```
