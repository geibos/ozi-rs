## Why

A headquarters cutting the next outing into tasks places ten marks in a row:
the base camp, the finds, the hazards, the meeting point, where each group
starts. The mode for placing them switched itself off after every single one —
including after one that failed — so those ten marks cost ten trips back to the
toolbar between clicks on the map.

It was written down as a known defect of CJ-5 in July («режим самовыключается
после 1 точки») and was still there on 2026-09-23. CJ-5 step 2 says "поставить
вейпоинты", plural, and OziExplorer keeps its waypoint tool selected until
another tool is picked, which is what anybody who has used it expects.

The reason it was written that way is visible in the code: the reset sat in a
`finally`, so it was somebody making sure the mode could not get stuck. That
worry is right and the answer is the three ways out, not a mode that lasts one
click.

## What Changes

- Placing a mark leaves the mode armed. The operator places as many as they
  need and then leaves.
- Leaving is unchanged and now load-bearing: Esc, pressing the mode chip again,
  or starting to draw a track. Panning does not leave it, because a drag is not
  a click.

## Impact

- Affected specs: `waypoints`
- Affected code: `src/components/MapView.svelte`
