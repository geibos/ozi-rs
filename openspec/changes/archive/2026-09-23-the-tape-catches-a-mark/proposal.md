## Why

"How far from the headquarters to the drop-off point" is the commonest
measurement a coordinator makes, and OziExplorer answers it with a dialog
listing waypoint pairs — Distance Between Waypoints.

The ruler here could already measure it, by clicking as close to each mark as
the hand manages. That is close enough for a sketch and not for a task:
fifteen pixels at a two-kilometre view is eighty metres, and the number goes
out over the radio to a crew that will walk it.

So rather than a dialog, the tape catches. A click that lands near a mark takes
that mark's exact position, and when both ends caught one the readout says
which two — which is the answer to the question in the words it was asked in,
and is also what makes the catching visible, since nothing else on screen would
show that the click moved.

Second of the four things `docs/backlog.md` recorded as present in OziExplorer
and missing here.

## What Changes

- While measuring, a click within about a fingertip of a mark takes that
  mark's position instead of the click's.
- With a mark at both ends, the readout names them: `ШТАБ → ЗАБРОС`.
- The reach is measured on screen rather than on the ground, because what the
  operator is aiming at is the marker under the cursor.

## Impact

- Affected specs: `waypoints`
- Affected code: `src/lib/measure-snap.ts` (new), `src/lib/stores.ts`,
  `src/components/MapView.svelte`
