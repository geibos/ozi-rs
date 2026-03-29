# Task Context: ozf2-rs integration into ozi-rs

Session ID: 2026-03-29-ozf2-rs-integration
Created: 2026-03-29T06:35:00Z
Status: cached_project_ozi_utf8_safe_validated

## Current Request
Integrate the sibling `../ozf2-rs` project into `ozi-rs` so OZI maps that reference `.ozf2` payloads can start moving from deferred metadata-only handling toward actual in-app display.

## Context Files (Standards to Follow)
- /Users/sobieg/.config/opencode/context/core/standards/code-quality.md
- /Users/sobieg/.config/opencode/context/core/standards/test-coverage.md
- /Users/sobieg/.config/opencode/context/core/standards/documentation.md
- /Users/sobieg/Projects/ozi-rs/AGENTS.md
- /Users/sobieg/Projects/ozi-rs/docs/architecture.md
- /Users/sobieg/Projects/ozi-rs/docs/requirements.md
- /Users/sobieg/Projects/ozi-rs/docs/testing-strategy.md

## Reference Files (Source Material to Look At)
- /Users/sobieg/Projects/ozi-rs/.tmp/sessions/2026-03-28-zip-ozi-import/context.md
- /Users/sobieg/Projects/ozi-rs/.tmp/sessions/2026-03-28-ozi-map-display-first-slice/context.md
- /Users/sobieg/Projects/ozi-rs/src/infrastructure/import/mod.rs
- /Users/sobieg/Projects/ozi-rs/src/infrastructure/import/ozi_map.rs
- /Users/sobieg/Projects/ozi-rs/src/application/mod.rs
- /Users/sobieg/Projects/ozi-rs/src/application/import.rs
- /Users/sobieg/Projects/ozi-rs/src/domain/project.rs
- /Users/sobieg/Projects/ozi-rs/src/ui/mod.rs
- /Users/sobieg/Projects/ozi-rs/src/infrastructure/lizaalert.rs
- /Users/sobieg/Projects/ozi-rs/Cargo.toml
- /Users/sobieg/Projects/ozf2-rs/Cargo.toml
- /Users/sobieg/Projects/ozf2-rs/src/lib.rs
- /Users/sobieg/Projects/ozf2-rs/src/raster.rs
- /Users/sobieg/Projects/ozi-rs/example_data/2021-07-30_Murino/5-Ozi(Win&Android)_Topo_EEKO/Maps/2021-07-30_Murino_Topo_EEKO_z16.ozf2

## External Docs Fetched
- /Users/sobieg/Projects/ozi-rs/.tmp/external-context/ozf2/ozf2-ozfx3-raster-decoding-feasibility.md
  - Historical feasibility note said `ozf2/ozfx3` should stay gated and isolated.
  - Current integration changes that assumption only for `ozf2`, via the separate sibling crate boundary.

## Components
- infrastructure adapter over `ozf2-rs`
- OZI metadata classification updated for `ozf2` vs `ozfx3`
- application-level OZI/OZF opening flow
- UI display path for decoded OZF rasters

## Constraints
- Keep decoding isolated to the infrastructure layer.
- Treat `ozfx3` as unsupported until the sibling crate supports it.
- Preserve existing sqlite map flow and diagnostics behavior.
- Prefer small validated slices over a big-bang UI rewrite.

## Progress Notes
- Added `ozf2-rs = { path = "../ozf2-rs" }` as a sibling path dependency to keep the decoder isolated outside the main repo while making it available through a narrow adapter boundary.
- Added `src/infrastructure/import/ozi_raster.rs` with `decode_ozi_raster_image`, `DecodedOziRasterImage`, and explicit `OziRasterDecodeError` so `ozi-rs` can decode `.ozf2` through infrastructure without leaking sibling-crate types upward.
- Updated `OziRasterKind` to distinguish `Ozf2` from still-unsupported `Ozfx3`.
- Strengthened `.map` parsing to work with real LizaAlert fixtures instead of only simplified synthetic samples by scanning for projection and datum records rather than assuming fixed line numbers.
- Fixed raster-path resolution for Windows absolute paths embedded inside `.map` files so a path like `C:\LA\map\OZF\foo.ozf2` resolves to the local basename next to the `.map` on macOS/Linux.
- Added integration coverage for the real example bundle under `example_data/2021-07-30_Murino/...`, including `.map` parsing and `.ozf2` decode.
- Kept the binary `.ozf2` decode test as an ignored manual integration test because `example_data/` is local/untracked; the repo-safe suite still validates the adapter shape without requiring that local fixture.
- Validation passed with `cargo fmt --check`, `cargo test --test import_ozi_map_metadata`, `cargo test --test import_ozi_raster`, `cargo test --test import_ozi_raster -- --ignored`, and `cargo test parse_ozi_map_metadata --lib`.
- Added `ActiveMapKind` so the app can distinguish sqlite-backed maps from local OZI raster maps without coupling UI logic to infrastructure parsing details.
- Added `AppState::open_local_ozi_map(...)` with explicit `OpenLocalMapError` handling, local `.map` parsing, `ozf2`-only acceptance, map-layer registration, and diagnostics-friendly status updates.
- Updated the existing LizaAlert mobile-map flow to construct `ActiveMapSelection` with `ActiveMapKind::SqliteTiles`.
- Added a minimal UI path input and `Open OZI map` action in the sidebar, keeping the existing LizaAlert picker intact.
- Added a dedicated OZI render path in `src/ui/mod.rs` that parses the selected `.map`, decodes the `ozf2` raster through the infrastructure adapter, uploads it as an egui texture, and displays it in the central panel while preserving the sqlite/OSM fallback path.
- Added focused application tests covering successful local OZI opening and explicit rejection of unsupported `ozfx3` payloads.
- Extended cached LizaAlert project loading so mirrored `.map` files are discovered from the local project tree, surfaced as project map entries, and opened through the same project map selection flow as cached sqlite packages.
- Updated `lizaalert::build_active_map_selection(...)` to infer `ActiveMapKind` from the cached local path, allowing `.map` selections to route into the existing OZI renderer without introducing a second project-opening mode.
- Added focused coverage for cached OZI project entries in both infrastructure and application tests.
- Added shared `read_ozi_map_text(...)` handling so OZI `.map` metadata and UI loading tolerate mirrored non-UTF-8 map text instead of failing bundle load/open with a UTF-8 decoding error.
- Removed the manual sidebar `Open OZI map` controls so the active workflow remains centered on mirrored project bundles instead of ad hoc local map opening.
- Validation passed with `cargo fmt --check` and `cargo test --lib`.

## Exit Criteria
- [x] `ozi-rs` depends on `../ozf2-rs` through a narrow adapter boundary.
- [x] `.map` metadata distinguishes supported `ozf2` from still-unsupported `ozfx3`.
- [x] A decoded OZF raster can be opened through `ozi-rs` infrastructure with focused tests.
- [x] Application/UI wiring for OZI-backed maps is implemented for local `.map` + `ozf2` opening.
- [x] Cached mirrored project `.map` entries reuse the same application/UI OZI opening path.
- [x] Validation passes for the touched slice.
- [x] Cached mirrored `.map` files no longer require valid UTF-8 text encoding to be indexed and opened.
