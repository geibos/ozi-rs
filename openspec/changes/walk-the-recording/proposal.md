## Why

ADR-0020 puts a per-point walkthrough in the MVP. Points were click-selectable
and that was all: reviewing a recording meant clicking each row in turn, and
losing your place the moment the list scrolled or a page of points turned over.

## What Changes

- Previous and next controls in the points table's header, with the position
  shown as "3 из 128" so the operator knows where in the recording they are.
- Stepping takes the map with it. A walkthrough that left the map where it was
  would be a list.
- The order is the track's, across its segments: a recording made in two
  sittings is one walk, and stepping off the end of the first segment lands on
  the start of the second.
- With nothing selected, "next" lands on the first point and "previous" on the
  last. That is what they mean from nowhere.
- Each control is disabled at its end rather than wrapping around, because a
  silent jump from the last point to the first reads as a bug.

## Impact

- Affected specs: `track-display`
- Affected code: `src/lib/stores.ts`, `src/lib/i18n.ts`,
  `src/components/inspector/TrackSegmentsTable.svelte`
