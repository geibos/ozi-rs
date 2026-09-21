## Why

A search taken down upstream never left the list. Chunks are merged — append
what has not been seen, remove nothing — on both sides, so it stayed in the
project list, then in the cache, and offline it was still there to click and
fail on.

Pruning needs one fact the interface did not have: whether a walk ran to the
end. A stopped walk read a prefix of the catalogue, and pruning on that would
make "stop" mean "delete most of the list".

## What Changes

- A walk that completed replaces the backend's project list rather than merging
  into it; a stopped one changes nothing.
- The walk's boundaries are emitted: `catalogue-refresh-started` after the
  cached chunk and before the walk, `catalogue-refresh-finished` with whether
  it completed. Between them the interface collects the slugs the walk sent,
  and on a complete walk drops everything else.
- The cached chunk is emitted before the window opens on purpose: counting
  yesterday's cache as proof that a search still exists would defeat the point.

Also here, because the same change caused it: the catalogue leaving the state
snapshot left the stand with an empty project list. The core writes
`catalogue.json` now, and the stand plays the real sequence — cached chunk,
started, the walk's chunk, finished — so the stand exercises the pruning
instead of hiding it.

## Impact

- Affected specs: `lizaalert-integration`
- Affected code: `src-tauri/src/application/mod.rs`,
  `src-tauri/src/commands/mod.rs`, `src-tauri/src/fixtures.rs`,
  `src/lib/stores.ts`, `src/routes/+layout.svelte`, `src/test/fixtures/`,
  `src/test/stand/tauri-core.ts`
