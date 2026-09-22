## Why

ADR-0020 declares cropping a track by selection and it had never been built.
Crop by extent and crop by time exist; neither is the gesture a crew actually
has.

The commonest edit to a recording is that the first twenty minutes of it are
the drive to the start. Cropping by time does that — if they work out what time
the walking began. Cropping by extent does it — if the drive happens to be
outside a rectangle they can draw. What they have is the point: they can see
where the track stops being a road and starts being a search, on the map and in
the points table.

## What Changes

- Two controls on each point's row in the segments table: trim everything
  before this point, trim everything after it.
- The named point survives either way. It is where the walk starts or ends, and
  removing it would be off by one in the direction nobody checks.
- Both go through the existing `CropTrackPoints` command, so a trim is one
  undo step and the points come back exactly where they were.
- Trimming at the first or last point removes nothing, says so, and records no
  undo step: it is not an edit.

Two trims compose into a crop by selection — cut before the start, cut after
the end — which is the same result with half the interface, and each half is
useful on its own.

## Impact

- Affected specs: `track-editing`
- Affected code: `src-tauri/src/application/mod.rs`,
  `src-tauri/src/commands/mod.rs`, `src-tauri/src/lib.rs`,
  `src/lib/bindings.ts`, `src/lib/api.ts`, `src/lib/i18n.ts`,
  `src/components/inspector/TrackSegmentsTable.svelte`,
  `src/test/stand/tauri-core.ts`
