## Why

The owner reported on 2026-09-23 that simplification cleans a track far too
hard — "нормализация трека работает некорректно, очень сильно чистится".

It does. The slider is labelled «Допуск: {n} м» and runs from 1 to 1000. Its
number was handed to a Rust parameter called `tolerance`, whose unit was
recorded in one doc comment three files away: kilometres. So the gentlest
setting the interface offers — one metre — simplified at one kilometre, and a
day's walk around a search area came back as two or three points. There was no
setting at which the control did anything useful.

The unit was lost the way units are always lost: a parameter named for what it
is rather than what it is in. Nothing between the slider and the algorithm had
to state it, and so nothing did.

Two other things kept it hidden. The stand's simplify preview kept every second
point whatever the slider said, so on the stand the numbers under the slider
never moved with it. And the domain function's own tests all passed tolerances
in kilometres — correctly, since that is its unit — so the algorithm was proved
right about the question nobody was asking.

## What Changes

- The tolerance crosses the boundary in metres, in parameters named
  `tolerance_m` / `toleranceM` all the way out to the generated bindings, and
  the conversion to kilometres happens once, on the Rust side.
- `domain::track::simplify_track_points_m` is the metre-taking entry point;
  `simplify_track_points` keeps its kilometres and its tests.
- The stand's preview answers a count that falls as the tolerance rises.

## Impact

- Affected specs: `track-editing`
- Affected code: `src-tauri/src/domain/track.rs`,
  `src-tauri/src/application/mod.rs`, `src-tauri/src/commands/mod.rs`,
  `src/lib/api.ts`, `src/lib/bindings.ts`, `src/lib/stores.ts`,
  `src/components/library/TracksTab.svelte`,
  `src/components/inspector/TrackInspector.svelte`, `src/test/stand/tauri-core.ts`
