## Why

Checking whether a group really walked into the marsh, the owner wanted to
point at a point on the map and see what it is, then find it in the list
(recording of 2026-10-08). The map said nothing about a point under the
cursor: its number, time and speed were in the points table, which had to be
scrolled to by hand.

## What Changes

- With a track selected and no tool on, the point of that track under the
  cursor (within 10 px) is named beside the cursor: its number of the
  track's total, its time, and the distance and speed of the leg into it.
- A click on it selects it; the points table centres on it.
- The search for the point is cut to a box around the cursor before anything
  is projected, so a phone's 18 000-point track keeps up with the mouse.

## Impact

- Affected specs: `track-display`
- Affected code: `src/lib/point-hover.ts` (new),
  `src/components/PointHover.svelte` (new), `src/components/MapView.svelte`,
  `src/lib/i18n.ts`
