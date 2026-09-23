## Why

A recording is a list of places with times on them, and the question a
headquarters asks of it runs the other way: "where were they at half past two".
It comes up whenever a task has to be reconstructed — a crew that reports late,
a find whose time is known but not its place, two groups whose paths have to be
compared hour by hour.

Answering it from the points table means scrolling a few thousand rows looking
for a timestamp. OziExplorer has Track Replay for exactly this, and a
coordinator who has used it misses it here.

Third of the four things `docs/backlog.md` recorded as present in OziExplorer
and missing here.

## What Changes

- A track with times can be played back: a slider from its first moment to its
  last, the moment read out as a radio log writes it, and a marker on the map
  where the crew was.
- Between two recorded points the position is interpolated, because a crew
  walking at four kilometres an hour is sixty metres along by the half minute
  and the nearest point is the wrong answer by that much.
- A moment that falls in a silence longer than five minutes is marked as one,
  in the readout and on the marker: the straight line across a gap the
  navigator did not record is a guess, and a position read off it should not
  look like a fix.
- Play runs a minute of the recording per tenth of a second, so a six-hour walk
  plays in about a minute.

## Impact

- Affected specs: `track-display`
- Affected code: `src/lib/track-replay.ts` (new), `src/lib/stores.ts`,
  `src/components/inspector/TrackInspector.svelte`,
  `src/components/MapView.svelte`, `src/lib/i18n.ts`
