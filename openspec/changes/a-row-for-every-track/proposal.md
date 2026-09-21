## Why

The Tracks tab built its rows out of the map's GeoJSON. That is one call, so it
looked cheap, and it cost two things.

**Every coordinate of every track travelled to draw a list of names.** The row
model reads nine properties; the geometry beside them — a day's folder of
recordings is hundreds of thousands of points — was serialized in Rust, sent
over IPC, parsed, and dropped. Again on every change that reloads the tab.

**A track the map cannot draw had no row.** `build_tracks_geojson` omits a
track whose segments are all shorter than two points, which is right: there is
nothing to draw. The list inherited it. Such a track is in the project, counts
towards its size and exports with it, and there was no way to see it, rename it
or delete it. This is the second time a list derived from map geometry has gone
wrong this way — the first emptied the whole rail when the geometry type
changed.

## What Changes

- `list_tracks` returns `Vec<TrackSummaryDto>`: one row per track, every track,
  no geometry. It is typed, unlike the `JsonValue` the GeoJSON command returns.
- The Tracks tab reads it. `trackFeaturesFromGeojson` had no callers left and
  is gone.
- The fixtures gain `tracks-list.json`, written by the core beside the GeoJSON,
  and the sample project gains a one-point track — so the stand shows the case
  instead of inheriting the omission, and a test holds the two fixtures against
  each other.

## Impact

- Affected specs: `track-display`
- Affected code: `src-tauri/src/commands/mod.rs`, `src-tauri/src/lib.rs`,
  `src-tauri/src/fixtures.rs`, `src/lib/bindings.ts`, `src/lib/api.ts`,
  `src/lib/track-features.ts`, `src/components/library/TracksTab.svelte`,
  `src/test/fixtures/`, `src/test/stand/tauri-core.ts`
