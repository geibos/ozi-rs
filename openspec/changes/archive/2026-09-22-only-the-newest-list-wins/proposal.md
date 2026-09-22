## Why

Both library tabs reload their rows whenever the app state changes, and during
a bundle download `state-changed` fires once per file. Reloads therefore
overlap, and nothing ordered them: the reload that started first could answer
last and write its rows to the screen. The list went backwards under the
operator — the same shape of bug that let an abandoned bundle preview replace
the map list, found in a second place.

The Waypoints tab carried a second cost. It read its layers one after another,
awaiting each, so a project of a dozen import-created layers paid a dozen round
trips in a row to draw a list that is rebuilt whole anyway.

## What Changes

- Each tab stamps its reload and applies the result only if no newer reload has
  started since — for the rows and for the failure alike.
- The Waypoints tab asks every layer at once instead of one after another.

## Impact

- Affected specs: `track-display`, `waypoints`
- Affected code: `src/components/library/TracksTab.svelte`,
  `src/components/library/WaypointsTab.svelte`
