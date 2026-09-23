## Why

Nothing creates an empty project. A crew that finishes one search and starts
another keeps adding to the same document: yesterday's routes stay on the map
under today's, the Tracks tab mixes two operations, and the only way out is to
quit, delete the session file and start the application again.

The backlog has carried this as a question — "is there such a thing as a new
project?" — since 2026-09-19. The answer the code gives today is no, and that
answer makes the second search of a weekend harder than the first.

A project is one search. Starting the next one is an ordinary thing to do.

## What Changes

- A command empties the project: no tracks, no waypoints, the default layers
  back, the file path forgotten so the next save asks where, and the history
  cleared so Ctrl+Z cannot walk back into the search that is over.
- The loaded bundle and the active raster stay. The map is the ground; the
  project is the work on it. A second search in the same district should not
  blank the screen, and the maps are already on disk.
- The command palette offers it, and the frontend asks before discarding
  unsaved work — the same question the close guard asks, for the same reason.

## Impact

- Affected specs: `project-persistence`
- Affected code: `src-tauri/src/application/mod.rs`,
  `src-tauri/src/commands/mod.rs`, `src-tauri/src/lib.rs`, `src/lib/api.ts`,
  `src/lib/actions/project.ts`, `src/lib/i18n.ts`,
  `src/components/CommandPalette.svelte`
