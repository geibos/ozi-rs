# Task Context: OZF2/OZFX3 separate crate requirements

Session ID: 2026-03-28-ozf2-crate-requirements
Created: 2026-03-28T10:55:00Z
Status: drafted

## Current Request
Prepare a requirements handoff document for a separate experimental Rust crate that can read legacy OziExplorer raster formats `ozf2` and `ozfx3`. The document should be suitable for passing into another session that will build the crate in a separate subdirectory.

## Context Files (Standards to Follow)
- /Users/sobieg/.config/opencode/context/core/standards/documentation.md
- /Users/sobieg/Projects/ozi-rs/docs/architecture.md
- /Users/sobieg/Projects/ozi-rs/docs/roadmap.md
- /Users/sobieg/Projects/ozi-rs/docs/adr/adr-0001-initial-architecture.md

## Reference Files (Source Material to Look At)
- /Users/sobieg/Projects/ozi-rs/.tmp/external-context/ozf2/ozf2-ozfx3-raster-decoding-feasibility.md
- /Users/sobieg/Projects/ozi-rs/.tmp/tasks/zip-ozi-import/subtask_10.json
- /Users/sobieg/Projects/ozi-rs/.tmp/sessions/2026-03-28-zip-ozi-import/context.md
- /Users/sobieg/Projects/ozi-rs/.tmp/sessions/2026-03-28-ozi-map-display-first-slice/context.md
- /Users/sobieg/Projects/ozi-rs/example_data/2021-07-30_Murino/5-Ozi(Win&Android)_Topo_EEKO/Maps/2021-07-30_Murino_Topo_EEKO_z16_ozf.map

## Constraints
- Treat `ozf2/ozfx3` support as experimental, read-only, and isolated from the main app.
- Do not make the decoder a required MVP dependency for `ozi-rs`.
- Keep the crate usable as a standalone library first; app integration comes later.
- Assume `OZF4` is out of scope.

## Exit Criteria
- [x] A handoff requirements document exists for a separate `ozf2/ozfx3` crate.
- [x] The document states scope, non-goals, risks, architecture boundary, and acceptance criteria.
- [x] The document is suitable for another session to begin implementation planning.
