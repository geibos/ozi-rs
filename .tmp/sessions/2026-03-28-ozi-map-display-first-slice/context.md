# Task Context: OZI map display first slice

Session ID: 2026-03-28-ozi-map-display-first-slice
Created: 2026-03-28T10:10:00Z
Status: parser_component_validated

## Current Request
Allow the application UI to display OZI map content from folders such as `5-OZI*`, `6-OZI*`, `7-OZI*`, and other OZI-named directories.

## Context Files (Standards to Follow)
- /Users/sobieg/.config/opencode/context/core/standards/code-quality.md
- /Users/sobieg/.config/opencode/context/core/standards/test-coverage.md
- /Users/sobieg/.config/opencode/context/core/standards/documentation.md
- /Users/sobieg/.config/opencode/context/core/workflows/component-planning.md
- /Users/sobieg/Projects/ozi-rs/docs/architecture.md
- /Users/sobieg/Projects/ozi-rs/docs/requirements.md
- /Users/sobieg/Projects/ozi-rs/docs/roadmap.md
- /Users/sobieg/Projects/ozi-rs/docs/testing-strategy.md
- /Users/sobieg/Projects/ozi-rs/docs/adr/adr-0001-initial-architecture.md

## Reference Files (Source Material to Look At)
- /Users/sobieg/Projects/ozi-rs/.tmp/sessions/2026-03-28-zip-ozi-import/context.md
- /Users/sobieg/Projects/ozi-rs/.tmp/tasks/zip-ozi-import/task.json
- /Users/sobieg/Projects/ozi-rs/.tmp/tasks/zip-ozi-import/subtask_09.json
- /Users/sobieg/Projects/ozi-rs/.tmp/tasks/zip-ozi-import/subtask_10.json
- /Users/sobieg/Projects/ozi-rs/.tmp/tasks/zip-ozi-import/subtask_11.json
- /Users/sobieg/Projects/ozi-rs/src/domain/project.rs
- /Users/sobieg/Projects/ozi-rs/src/application/import.rs
- /Users/sobieg/Projects/ozi-rs/src/application/mod.rs
- /Users/sobieg/Projects/ozi-rs/src/ui/mod.rs
- /Users/sobieg/Projects/ozi-rs/src/ui/sqlite_tiles.rs
- /Users/sobieg/Projects/ozi-rs/src/infrastructure/import/mod.rs
- /Users/sobieg/Projects/ozi-rs/src/infrastructure/import/ozi_map.rs
- /Users/sobieg/Projects/ozi-rs/tests/import_ozi_map_metadata.rs

## External Docs Fetched
- /Users/sobieg/Projects/ozi-rs/.tmp/external-context/ozi-map/minimal-metadata-parser.md
  - `.map` is a text calibration/metadata format with raster filename references and fixed calibration rows.
  - MVP parser should extract title, raster reference, projection/datum names, populated calibration points, and border metadata where available.
- /Users/sobieg/Projects/ozi-rs/.tmp/external-context/ozf2/ozf2-ozfx3-raster-decoding-feasibility.md
  - `ozf2/ozfx3` decoding is not a good foundation for the in-repo MVP path.
  - Safe recommendation is to treat it as a separately gated legacy adapter or deferred capability.

## Components
- OZI `.map` metadata parser and raster reference resolution
- Application registration for OZI-backed map sources
- UI display for OZI maps when the referenced raster is a directly supported image format
- Explicit diagnostics for deferred `ozf2/ozfx3` raster payloads

## Constraints
- Preserve domain/application/infrastructure/ui boundaries.
- Keep the first slice independent from `ozf2/ozfx3` decode work.
- Prefer a working metadata + supported-raster path over speculative full OZI compatibility.
- Keep existing sqlite mobile map flow intact.

## Progress Notes
- Implemented `src/infrastructure/import/ozi_map.rs` as a dedicated infrastructure parser for OziExplorer `.map` metadata.
- Added explicit `OziMapMetadata`, `OziRasterKind`, `DirectImageFormat`, and `OziMapParseError` types so the parser can report supported image references, deferred `ozf2/ozfx3` payloads, and explicit malformed-input failures without UI coupling.
- Implemented deterministic raster path resolution relative to the `.map` file and basename fallback when the raster reference is absolute or tries to escape via `..`.
- Added focused integration tests in `tests/import_ozi_map_metadata.rs` for relative supported-image references, missing title failures, and deferred `ozfx3` classification.
- Added focused unit tests in `src/infrastructure/import/ozi_map.rs` for minimal valid parse, missing header, missing raster reference, deferred `ozf2`, and safe raster path resolution.
- Tightened calibration point filtering so only populated `PointNN` rows with explicit XY coordinates are retained.
- Validation passed with `cargo fmt --check`, `cargo test --test import_ozi_map_metadata`, and `cargo test parse_ozi_map_metadata --lib`.

## Exit Criteria
- [x] `.map` files can be parsed into project-facing map metadata with resolved raster references.
- [ ] The application can register an OZI map source without coupling parsing to the UI.
- [ ] The UI can display an OZI map when the raster payload is a directly supported image format.
- [ ] Unsupported `ozf2/ozfx3` references surface explicit diagnostics instead of silent failure.
- [x] Focused tests cover parser success/failure and first-slice workflow behavior.
