## Why

The map has the same overlapping-reload bug the library tabs had, and it is
more visible there: `refreshWaypointMarkers` runs from a slice effect and from
every waypoint action handler, so two runs overlap routinely, and the marker
set written last wins even when it was started first. A waypoint just added
could disappear again. The track geometry fetch beside it has the same shape —
two quick changes leave two fetches in flight, and the older one drawing last
puts the map behind the rail it is meant to match.

The map also read its waypoint layers one after another, on a path that runs on
every state change.

This is the fourth and fifth occurrence of one rule — take a token before the
first await, check it before writing, on the failure path too — written out by
hand each time. Written out by hand is how it comes to be missing.

## What Changes

- `createLatestRun` in `src/lib/latest-run.ts` holds the rule, with tests.
- The map's marker refresh and track geometry fetch use it.
- The map reads every waypoint layer at once instead of one after another.
- Both library tabs move onto it, replacing the counters written last slice, so
  there is one spelling of the rule rather than three.

## Impact

- Affected specs: `track-display`, `waypoints`
- Affected code: `src/lib/latest-run.ts` (new), `src/components/MapView.svelte`,
  `src/components/library/TracksTab.svelte`,
  `src/components/library/WaypointsTab.svelte`
