## Why

An external reviewer (`gpt-6-astra`, three slices, reports in
`docs/reviews/2026-09-22/`) read the two-day autonomous run and the project as
a whole. Sixteen of its findings survived a reading of the code. They are not
one theme; what they share is that each is invisible until the day it is not.

The one that had to be fixed first was a data loss. Partial downloads were
named with `Path::with_extension("part")`, which replaces the extension rather
than appending to it: `sheet.map` and `sheet.ozf2` of the same bundle both
wrote to `sheet.part`, so two parallel downloads of one bundle interleaved
their bytes into one file and both landed corrupt. The four tests that covered
partial paths computed the same wrong name themselves, so they passed.

The rest fall into three groups. Things that quietly lied to the operator: a
failed read of a layer's marks was indistinguishable from "this layer has no
marks" and erased the markers on the map; a finished catalogue walk wrote its
count over the one line reporting a running download; a bundle file matched its
map by `ends_with`, so `bigmap.ozf2` claimed `map.ozf2`. Things that took the
operator's history: a refused command cleared the redo stack before it failed;
two separate drags of the same point merged into one undo step. And two broken
promises: CJ-2 says a field launch makes zero network requests, and the app
asked for OpenStreetMap tiles and walked the catalogue on every start.

Four of the sixteen were introduced by the run the review was asked to check.

## What Changes

- Partial downloads append `.part` to the whole file name. The tests assert
  against the function instead of recomputing its answer.
- A read failure is `null`, not an empty list: a layer whose marks could not be
  read keeps the markers it has, and the operator is told.
- "Показать всё" frames the layer data rather than the markers that happen to
  have been drawn, so a click during the reconciler's work no longer leaves
  the marks off-camera.
- A finished catalogue walk keeps its count to the diagnostics log while a
  download is running.
- A ready bundle file matches the map whose file name it actually is.
- Applying the active map takes a run token, reports a failed metadata read,
  and counts as applied only once it has succeeded — so a failure no longer
  blocks every retry of that map for the session.
- Coalescing is scoped to a gesture. Nothing merges unless the caller says two
  commands are one action; `apply` never does.
- With no link, the app does not add the OpenStreetMap source and does not
  start the launch-time catalogue walk. Both are what the operator asked for
  the next time there is a link; the refresh button never stopped working.
- The MapLibre waiver states its facts correctly (the advisory covers <= 6.4.0,
  so 6.4.1 is the first fix; `^4` is a range and the lockfile is the pin) and
  its guard now covers `attribution`, which the advisory is actually about.
- The smoke gate's labels are under a contract test, after two of its three
  runs failed on wording rather than behaviour.

## Impact

- Affected specs: `lizaalert-integration`, `tile-rendering`, `undo-redo`,
  `waypoints`, `track-editing`
- Affected code: `src-tauri/src/infrastructure/lizaalert.rs`,
  `src-tauri/src/infrastructure/import/gpx.rs`,
  `src-tauri/src/application/mod.rs`,
  `src-tauri/src/application/commands.rs`,
  `src-tauri/src/commands/mod.rs`, `src/components/MapView.svelte`,
  `src/routes/+layout.svelte`, `src/lib/stores.ts`, `src/lib/api.ts`,
  `src/lib/map-bounds.ts`, `src/lib/bundle-url.ts`, `src/lib/i18n.ts`,
  `src/lib/network-reach.ts` (new), `scripts/npm-audit-gate.mjs`,
  `tools/ozi-rs-mcp/tests/smoke_core_workflow.rs`
