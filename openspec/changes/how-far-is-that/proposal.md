## Why

ADR-0020 puts distance measurement in the MVP and it was never built. It is the
measurement a search crew takes constantly — how far is that from the task
point, how wide is this clearing, how long is the leg we are about to walk —
and without it they measure by eye or reach for another program.

I had this filed as blocked on where it goes, because `ui-shell` requires the
mode chips above the canvas to stay inert scaffolding. That was the wrong
reading: `product-scope` names the command palette as a place a workspace
action may live, and it is where the other verbs already are.

## What Changes

- "Измерить расстояние" in the palette turns the tool on, and the same entry
  turns it off — the same key a crew reached for is how they put the tape away.
- While it is on, a click on the map adds a point and the running total is
  shown over the canvas, not in the status bar: they read the number where they
  are clicking.
- Esc finishes, as it cancels a draw.
- A click while measuring is taken whole, so it does not also select a track or
  drop a waypoint.
- Metres under a kilometre, two decimals in the first ten, one beyond. A crew
  measuring the width of a clearing wants "180 м", and the difference between
  40 and 140 metres is the difference between two sides of a road.
- The points are scratch: never saved, discarded when the tool goes off. A
  measurement worth keeping is a track, which they can already draw.

The distance is computed in the frontend, with the same haversine and the same
earth radius as `domain/track.rs`, so a measured leg and a track's length are
the same number for the same two points. A round trip per click is what makes a
tool feel slow.

## Impact

- Affected specs: `product-scope`
- Affected code: `src/lib/geo.ts` (new), `src/lib/stores.ts`, `src/lib/i18n.ts`,
  `src/components/MapView.svelte`, `src/components/CommandPalette.svelte`
