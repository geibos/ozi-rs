## Why

`get_app_state` carried the entire LizaAlert catalogue — thirteen thousand
rows of slug, name and a cached flag. By the real DTO's own shape that is
about **1 MiB of JSON per call**, and the call is made on every
`state-changed`: once per file during a bundle download, on every preview, on
every edit that touches the project.

Building it also walked the bundles directory on disk every time, to compute
the `cached` flag for rows nobody was reading.

One consumer needed it, and only to seed a store that is already seeded twice
over: synchronously from the `localStorage` cache at module load, and again
from the first `projects-chunk`, which `load_projects` emits from the disk
cache before it touches the network.

A catalogue is not application state. It is a stream, and it already had one.

## What Changes

- `AppStateDto` loses `projects`, and `app_state_dto` loses the `cached_slugs`
  parameter it only needed for them — so `get_app_state` no longer reads the
  bundles directory either.
- `syncProjectsFromAppState` and its call site go; the store keeps its two
  existing sources.
- `LizaProjectSummaryDto` becomes a hand-written type in `src/lib/types.ts`.
  specta generates types for command signatures, and the catalogue now travels
  only as an event payload, so it is no longer generated. `CLAUDE.md` already
  requires that file to be kept in step with the Rust structs by hand.

## Impact

- Affected specs: `lizaalert-integration`
- Affected code: `src-tauri/src/commands/mod.rs`,
  `src-tauri/src/application/mod.rs`, `src-tauri/src/fixtures.rs`,
  `src/lib/bindings.ts`, `src/lib/types.ts`, `src/lib/stores.ts`,
  `src/lib/palette-projects.ts`, `src/routes/+page.svelte`,
  `src/test/fixtures/`
