## Why

Cleaning a track in OziExplorer is mostly finding the places where the GPS
lied: a jump, where the recording stopped and the track joins the gap with a
straight line, and an outlier, a point where nobody could have been. The
owner's recording of 2026-10-06 spends most of each track on this, and the
detachment's wiki says how: open the point list, read the Dist and KPH
columns, and delete only the "apex" of an outlier — deleting its neighbour
leaves the spike and bends the track. For a jump the owner deletes the point
and splits the track there, and would settle for just splitting.

ozi-rs had neither the columns nor a way to find these places other than
looking at the map, and deleting a point and splitting the track were two
actions in two places.

The owner's eleven tracks from that search (original GPX and the PLT after
his cleaning) say what a detector must find. Of his 32 segment breaks, one
was already a break in the GPX and 26 fall on a leg at least ten times the
track's median step and at least 100 m long; of his 20 hand-deleted points,
15 are apexes — both legs at least 30 m and three median steps, and the
neighbours on either side closer to each other than about a third of the way
there and back. With those thresholds the list holds 112 places over the
eleven tracks, 59 of them places he edited. The breaks it misses are mostly
short runs he cut out of a dense cluster of points.

## What Changes

- The points table shows, under each point, the distance from the previous
  point and the speed over it.
- The inspector lists a track's jumps and outliers in track order, each with
  the distances and speeds of its legs. Choosing one selects the point and
  takes the map there.
- A jump offers «Разделить здесь»; an outlier offers «Удалить вершину» and
  «Удалить и разделить».
- «Удалить и разделить» removes a point and splits its segment between the
  points on either side, as one undoable step (`CutOutTrackPoint`).
- The list is a review aid: it never edits a track by itself, and the
  thresholds are not settings yet.

## Impact

- Affected specs: `track-editing`
- Affected code: `src-tauri/src/application/commands.rs`,
  `src-tauri/src/application/mod.rs`, `src-tauri/src/commands/mod.rs`,
  `src-tauri/src/lib.rs`, `src/lib/api.ts`, `src/lib/track-jumps.ts` (new),
  `src/components/inspector/TrackSegmentsTable.svelte`,
  `src/components/inspector/TrackJumps.svelte` (new),
  `src/components/inspector/TrackInspector.svelte`, `src/lib/i18n.ts`,
  `docs/commands-reference.md`
- Evidence: `docs/field-notes/2026-10-06-track-processing-in-ozi.md`
