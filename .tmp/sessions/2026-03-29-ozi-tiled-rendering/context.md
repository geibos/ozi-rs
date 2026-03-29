# Task Context: Tiled multi-level OZI rendering

Session ID: 2026-03-29-ozi-tiled-rendering
Created: 2026-03-29T08:40:00Z
Status: infrastructure_level_surface_validated

## Current Request
Use GIS/cartography best practices for OZI rendering: support multiple native OZF zoom levels, smooth intermediate zoom, and overzoom beyond the finest native level instead of relying on a single full-image preview texture.

## Context Files (Standards to Follow)
- /Users/sobieg/.config/opencode/context/core/standards/code-quality.md
- /Users/sobieg/.config/opencode/context/core/standards/test-coverage.md
- /Users/sobieg/Projects/ozi-rs/AGENTS.md
- /Users/sobieg/Projects/ozi-rs/docs/architecture.md
- /Users/sobieg/Projects/ozi-rs/docs/testing-strategy.md
- /Users/sobieg/Projects/ozi-rs/.tmp/sessions/2026-03-29-ozf2-rs-integration/context.md

## Reference Files (Source Material to Look At)
- /Users/sobieg/Projects/ozi-rs/src/ui/mod.rs
- /Users/sobieg/Projects/ozi-rs/src/infrastructure/import/ozi_raster.rs
- /Users/sobieg/Projects/ozi-rs/src/application/mod.rs
- /Users/sobieg/Projects/ozf2-rs/src/raster.rs
- /Users/sobieg/Projects/ozf2-rs/src/header.rs

## External Docs Fetched
- None yet; current local source is enough for the first rendering slice.

## Components
- infrastructure level metadata + level-aware decode surface
- UI zoom policy for native level selection and overzoom
- tiled OZI viewport rendering path
- focused tests for level selection and tile/render helpers

## Constraints
- Keep decoder-specific logic isolated behind infrastructure.
- Prefer level/tile-based rendering over full-image upload.
- Support nearest native source level plus smooth interpolation between levels.
- Allow overzoom beyond the finest native level even if the result is pixelated.
- Preserve existing sqlite/OSM map behavior.

## Progress Notes
- Added a renderer-friendly infrastructure surface in `src/infrastructure/import/ozi_raster.rs` for native OZF levels and per-tile RGBA decode.
- Added `OziRasterTileSource`, `OziRasterLevelMetadata`, and `DecodedOziRasterTile` so the next UI slice can reason about level dimensions, tile grids, and edge-tile visible sizes without depending directly on `ozf2-rs` internals.
- Preserved the existing `decode_ozi_raster_image(...)` helper by rebuilding it on top of the new tile-source opening path.
- Added focused unit tests for visible edge-tile sizing, out-of-bounds tile queries, indexed-to-RGBA conversion, and cropped edge-tile output.
- Added focused integration coverage in `tests/import_ozi_raster.rs` for unsupported non-OZF sources plus ignored real-fixture tests for native levels and tile decode.
- Validation passed with `cargo fmt --check`, `cargo test --lib ozi_raster`, and `cargo test --test import_ozi_raster`.

## Exit Criteria
- [x] Infrastructure exposes OZI level metadata needed for renderer decisions.
- [ ] UI selects an appropriate native source level for the current zoom.
- [ ] OZI rendering no longer depends on uploading one giant full-image texture.
- [ ] Intermediate zoom and overzoom are supported.
- [x] Focused tests cover level selection and critical rendering helpers.
