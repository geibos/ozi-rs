## Why

A search is cut into sectors, and the sectors go out over the radio as
coordinates. Every paper sheet a headquarters has ever used carries a grid for
exactly that, and OziExplorer draws one; this map had none.

Without it, reading a coordinate off the map means clicking a place and reading
it out of the status bar, one point at a time, and putting somebody else's
coordinate on the map means the same in reverse. A grid answers both at a
glance: the sector is "between 59°55' and 59°56', east of 31°40'", and it can
be said, written down and read back without touching the application.

It is the first of the four things `docs/backlog.md` recorded on 2026-09-23 as
present in OziExplorer and missing here, and the one judged most likely to be
missed.

## What Changes

- The map can draw a latitude/longitude grid, offered from the command palette
  and remembered between sessions.
- The spacing follows the zoom, always on a number a person can say: degrees,
  then 30/20/10/5/2/1 minutes, then the same seconds. A step of 0.15° would fit
  the screen better and be useless on the radio.
- Each line is named where it meets the edge — parallels down the left,
  meridians along the top — in the notation a map reader writes:
  `59°56'15"N`.
- The grid sits under the tracks and the measurements: it is the paper, not the
  work.

## Impact

- Affected specs: `tile-rendering`
- Affected code: `src/lib/graticule.ts` (new),
  `src/lib/maplibre/graticule-layer.ts` (new), `src/lib/stores.ts`,
  `src/components/MapView.svelte`, `src/components/CommandPalette.svelte`,
  `src/lib/i18n.ts`
