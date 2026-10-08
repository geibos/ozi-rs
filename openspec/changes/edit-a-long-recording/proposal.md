## Why

A phone records a point a second, and the owner's recording of 2026-10-08
has an 18 000-point track that "opens slowly" in OziExplorer. On the stand
the same size of track showed where ozi-rs stood: the inspector opened in a
second and stepping through points was quick, but edit mode put a draggable
DOM marker on every point — 7.5 s to turn on, then 16.5 s for one pan of the
map.

## What Changes

- Edit mode puts handles only on the points inside the visible map, and
  again each time the map stops moving.
- With more than a thousand points in view it puts none, and says how many
  there are and to zoom in — moving one point needs the zoom anyway.
- The stand serves such a track with `?track=big` (18 000 points).

Measured on the stand, 2026-10-08, 18 000 points: edit mode on in 0.2 s
(was 7.5 s); a pan in 0.2 s with the whole track in view (was 16.5 s) and
0.6 s with 885 handles after zooming in.

## Impact

- Affected specs: `track-editing`
- Affected code: `src/lib/editable-points.ts` (new),
  `src/components/MapView.svelte`, `src/lib/i18n.ts`,
  `src/test/stand/tauri-core.ts`
