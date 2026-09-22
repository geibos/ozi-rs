## Why

The camera was fitted to the data in exactly one place: after an import. That
fix exists because of the owner's own finding — "imported tracks don't show on
the map" — and its comment says why: the tracks "render wherever they are, off
the active raster, and look like they failed to import".

Opening a saved project has the same hole and nobody had looked. Both ways in,
the Open dialog and the recent-projects list, load the file and stop. The map
stays wherever it was pointing, which for a crew reopening yesterday's search
on today's map is an empty screen — indistinguishable from a project that did
not load. It is the more common path of the two: a search is imported once and
reopened every morning.

The import fix also only ever fitted to track geometry. A project whose content
is a headquarters and a drop-off point has none, so importing waypoints alone
framed nothing.

## What Changes

- Opening a project frames what it contains, from the dialog and from the
  recents alike.
- The frame covers waypoints as well as tracks. Their positions come from the
  markers already on the map, so this costs no extra round trip.
- A single point, or an extent a few metres across, is centred at a readable
  zoom instead of being fitted to its own dot.
- The bbox maths moves out of `MapView` into `$lib/map-bounds`, where it can
  be tested — `MapView` needs a MapLibre instance to mount, so none of it ever
  ran under test. A coordinate that is not a finite number is now skipped
  rather than turning the whole frame into `NaN`.

## Capabilities

### Added Capabilities
- `project-persistence`: opening a project puts its contents on screen.

## Impact

- **Frontend**: `src/lib/map-bounds.ts` (new), `MapView.svelte`,
  `CommandPalette.svelte`, `stores.ts` (`all-tracks` → `all-data`),
  `library/TracksTab.svelte`.
- **Backend**: none.
- **Risk**: low. The camera moves in one more situation than before, and that
  situation is the one where the operator has just asked to see a project.
