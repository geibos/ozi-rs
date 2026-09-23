## Why

Everything built between 2026-09-22 and 2026-09-23 — layer management, the
OziExplorer waypoint reader, a new search, drag and drop, the names on the
map, the area tools — was walked on the stand and never in the packaged
application. The stand proves how a screen behaves *given an answer*; only the
packaged application proves the answer comes back over real IPC.

The gate covered one journey, CJ-4's editing spine, and had since it was
written. The owner's standing instruction is that the customer journeys are
more complex than the smoke covers and want deeper checking.

Adding the second journey immediately found two things the first could not.

## What Changes

- A second journey in the gate: making a layer from the library, seeing it
  become active, and removing it again — `create_track_layer` and
  `delete_track_layer` over real IPC, in the packaged application.
- `just smoke` runs its journeys **one at a time**. Each drives a Mac2 session
  that owns the screen; run in parallel they click into each other's window,
  and the first time a second journey was added both failed — including the
  one that had passed on its own a minute earlier.
- The layer select's accessible name carries which layer is active. It said
  only "track layer", leaving the current value in a span WKWebView does not
  publish: a screen reader announced half the control, and nothing outside the
  application could see which layer was active. That is also what made the new
  journey unassertable.

## Impact

- Affected specs: `build-tooling`, `ui-shell`
- Affected code: `tools/ozi-rs-mcp/tests/smoke_core_workflow.rs`, `justfile`,
  `src/components/library/TracksTab.svelte`,
  `src/components/library/WaypointsTab.svelte`,
  `src/test/smoke-label-contract.test.ts`
