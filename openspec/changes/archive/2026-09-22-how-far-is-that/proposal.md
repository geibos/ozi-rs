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

## The radius ring, in the same slice

ADR-0020's other on-map tool, and the one a search draws constantly:
everything within five hundred metres of the last known position. It shares
everything the tape built — the palette entry, the scratch geometry, the layer
— so it is here rather than in a slice of its own.

Click a centre, click again to set the radius; a further click moves the
radius, because a crew drawing rings draws several and should not reach for the
palette between them.

The ring is **geodesic**, not a circle in screen pixels. A flat circle is wrong
everywhere except the equator and worse the further north the search is: at
60°, where these searches happen, a "500 m" circle drawn flat is half a
kilometre north-south and a kilometre east-west, and the crew standing in it is
looking in the wrong place.

The two tools take the same click, so they cannot both be listening. Turning
one on turns the other off and discards what it held — a stale point left
behind would be measured into the next measurement.

## Projection, completing the three

`product-scope` asks for "placing a waypoint by projection (azimuth plus
distance) from a selected point" — and unlike the other two, it is not
click-driven. "From the task point, 240° and 1.2 kilometres" is dictated over a
radio, not pointed at.

So: click the origin, type the bearing and the distance, and the point is
placed where they land. The result is previewed on the map before it is
committed, because a bearing heard over a radio is easy to mishear and seeing
the point is how that gets caught.

All three tools take the map's clicks, so only one listens at a time and
switching discards what the previous one held.

Found beside it: the default name for a waypoint placed by clicking was
`Waypoint N` — English, in the app's own Russian window.

## Impact

- Affected specs: `product-scope`
- Affected code: `src/lib/geo.ts` (new), `src/lib/stores.ts`, `src/lib/i18n.ts`,
  `src/components/MapView.svelte`, `src/components/CommandPalette.svelte`
