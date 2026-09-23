## Why

Drawing a route finishes on a double-click, so a single click cannot become a
point the moment it arrives — the second half of a double-click would already
have put one down. The application therefore held a click for 220 ms before
sending it.

It held *one*. A second click inside that window cancelled the first and
replaced it. So a coordinator plotting a route at any normal pace — three or
four clicks a second, which is what cutting up a search area looks like — got
roughly one point per burst, and nothing on screen said the others had been
dropped. Measured on the stand: six clicks 90 ms apart produced one point.

CJ-5 has carried the note «быстрые клики теряются, 220ms debounce» since July,
which reads as a delay. It was not a delay. The points were gone.

## What Changes

- Every click gets its own window. Nothing cancels a click but a double-click,
  which cancels every click still waiting — both of its own halves are inside
  the window by definition — and finishes the track.
- The commits are chained rather than overlapped: each is told the index to
  insert at, and two in flight would read the same one.
- A point the backend refuses no longer stops the points after it: the operator
  is still drawing.

## Impact

- Affected specs: `track-editing`
- Affected code: `src/lib/drawing-clicks.ts` (new),
  `src/components/MapView.svelte`
