## Why

CJ-2 is a laptop switched on in a field camp with no link, and its success
criterion is a working map inside a minute. The restore that gets it there
read the session in one run of early returns: a `.ozp` that had been moved,
renamed or deleted aborted the whole restore before it reached the active map.

So a crew that had tidied their folder — or exported and deleted the project,
or put it on a drive that was not plugged in — met a blank screen, with the
raster sitting on the disk underneath it, unloaded, at four in the morning.
The same for a project file that exists and cannot be read.

The map is the ground; the project is the work on it. Losing the work is no
reason to lose the ground.

## What Changes

- The project and the active map are restored independently. A project that
  is missing or unreadable is reported and the restore carries on to the map.
- Each skip still says what it skipped and why, in the diagnostics, as before.

## Impact

- Affected specs: `project-persistence`
- Affected code: `src-tauri/src/application/mod.rs`
