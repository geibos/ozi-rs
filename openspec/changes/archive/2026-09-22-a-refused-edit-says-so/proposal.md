## Why

Ten failures on the editing path reached `console.error` and nothing else, and
the error reporter is disabled outside dev builds. In a release build they were
silent: the operator acted, the backend declined, and nothing said so.

They are not peripheral. Dragging a track point, deleting one, inserting one,
adding a waypoint, placing a point while drawing, cancelling a draw, fitting
the tracks on screen, and loading either inspector — that is the field workflow.

The track-point drag was worse than silent. On failure the marker stayed where
the operator dropped it while the data kept the old position, so the map went
on showing a point that was not there until something else reloaded it.

The loader was given this treatment in `honest-bundle-flow`. The map and the
inspectors never were, and nothing was watching.

## What Changes

- `reportEditFailure` raises a toast in the interface's language carrying the
  backend's own message, which is the only thing that says why.
- The ten sites use it. A failed track-point drag also reloads the points, so
  the map stops disagreeing with the data.
- A test scans every component for a `catch` that logs and tells nobody. Two
  entries are allowlisted with their reasons: OZF2 metadata, which is expected
  to fail for SQLite maps, and the per-file import failure, which is reported
  in aggregate after the loop so that one bad file does not bury the summary.

## Impact

- Affected specs: `track-editing`, `waypoints`
- Affected code: `src/lib/edit-failure.ts` (new), `src/lib/i18n.ts`,
  `src/components/MapView.svelte`,
  `src/components/inspector/TrackInspector.svelte`,
  `src/components/inspector/WaypointInspector.svelte`
