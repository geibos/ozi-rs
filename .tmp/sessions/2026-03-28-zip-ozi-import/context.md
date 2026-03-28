# Task Context: ZIP-based Ozi and geodata import

Session ID: 2026-03-28-zip-ozi-import
Created: 2026-03-28T08:05:00Z
Status: application_workflow_validated

## Current Request
Карта должна открываться не только sqlite (мобильная), а также из `.zip` архивов с Ozi/геоданными форматами: `ozf2`, `map`, `wpt`, `plt`, `kml`, `gpx`.

## Context Files (Standards to Follow)
- /Users/sobieg/.config/opencode/context/core/standards/code-quality.md
- /Users/sobieg/.config/opencode/context/core/standards/test-coverage.md
- /Users/sobieg/.config/opencode/context/core/standards/documentation.md
- /Users/sobieg/.config/opencode/context/core/workflows/feature-breakdown.md
- /Users/sobieg/Projects/ozi-rs/docs/requirements.md
- /Users/sobieg/Projects/ozi-rs/docs/architecture.md
- /Users/sobieg/Projects/ozi-rs/docs/roadmap.md
- /Users/sobieg/Projects/ozi-rs/docs/testing-strategy.md
- /Users/sobieg/Projects/ozi-rs/docs/adr/adr-0001-initial-architecture.md

## Reference Files (Source Material to Look At)
- /Users/sobieg/Projects/ozi-rs/src/application/mod.rs
- /Users/sobieg/Projects/ozi-rs/src/application/commands.rs
- /Users/sobieg/Projects/ozi-rs/src/domain/project.rs
- /Users/sobieg/Projects/ozi-rs/src/domain/track.rs
- /Users/sobieg/Projects/ozi-rs/src/domain/waypoint.rs
- /Users/sobieg/Projects/ozi-rs/src/infrastructure/import/mod.rs
- /Users/sobieg/Projects/ozi-rs/src/infrastructure/import/archive.rs
- /Users/sobieg/Projects/ozi-rs/src/infrastructure/import/gpx.rs
- /Users/sobieg/Projects/ozi-rs/src/application/import.rs
- /Users/sobieg/Projects/ozi-rs/src/infrastructure/lizaalert.rs
- /Users/sobieg/Projects/ozi-rs/Cargo.toml
- /Users/sobieg/Projects/ozi-rs/tests/archive_inventory.rs
- /Users/sobieg/Projects/ozi-rs/tests/import_gpx_archive.rs
- /Users/sobieg/Projects/ozi-rs/tests/application_zip_gpx_import.rs

## External Docs Fetched
- `.tmp/external-context/rust-geodata-import/zip-geodata-import-support-matrix.md`
  - `zip` crate (`docs.rs/zip`) is the recommended first-slice archive reader.
  - `gpx` crate (`docs.rs/gpx`) is the recommended first-slice GPX parser.
  - `kml` crate is viable later, but broader and riskier than GPX.
  - Ozi `.plt` / `.wpt` / `.map` likely require custom parsers from published text specs.
  - `ozf2` remains a feasibility-risk item because no credible Rust crate/spec path was found.
- `.tmp/external-context/gpx/rust-gpx-import-adapter-api.md`
  - `gpx::read(Cursor::new(bytes))` is the entry point for in-memory parsing.
  - GPX track segment points are represented by `gpx::Waypoint`; coordinates come from `point().x()` = lon, `point().y()` = lat.
  - Minimal importer should preserve track/segment boundaries and return explicit parse errors for malformed XML or unsupported GPX structure.
- `.tmp/external-context/eframe-egui/app-trait-0-34-ui-migration.md`
  - `eframe 0.34.x` requires `App::ui(&mut Ui, &mut Frame)` instead of the old `update` trait item.
  - Background work that must run outside widget rendering should move to `App::logic`.

## Components
- ZIP archive ingestion and format detection
- track/waypoint import adapters for text-based formats
- Ozi map metadata import (`.map`)
- raster payload support, including separate `ozf2` investigation
- application orchestration and UI entry points
- parser and workflow validation

## Progress Notes
- Task breakdown created at `/Users/sobieg/Projects/ozi-rs/.tmp/tasks/zip-ozi-import/` with `task.json` and `subtask_01.json` … `subtask_12.json`.
- Recommended first delivery slice is ZIP inventory and format detection plus archived GPX import end-to-end.
- `ozf2` is explicitly separated behind a later feasibility gate due to decoder risk and unknown crate/spec support.
- External support matrix now confirms the first-slice dependency choice: `zip` + `gpx`.
- Staged architecture is now defined as: infrastructure archive reader -> infrastructure format detectors/adapters -> application import workflow -> UI trigger/reporting.
- The first implementation slice remains intentionally limited to archive inventory and supported-entry classification before GPX parsing is wired in.
- Implemented `infrastructure::import::archive::inventory_zip_entries` plus `ArchiveEntry` / `ArchiveEntryKind` classification for GPX, KML, Ozi text files, sqlite tiles, raster payloads, and unknown entries.
- Added deterministic archive inventory tests in `tests/archive_inventory.rs` plus focused unit tests for case-insensitive extension classification.
- Validation completed for this slice with `cargo fmt --check`, `cargo test --test archive_inventory`, and `cargo test classify_archive_entry --lib`.
- Added `infrastructure::import::gpx::import_gpx_entries_from_archive`, which reads only GPX entries from ZIP archives and converts them into domain `Track` and `Waypoint` collections.
- Reused the archive classifier through `classify_archive_path` so mixed-content archives skip non-GPX entries deterministically.
- Added integration tests for valid GPX import, mixed-content archive filtering, and explicit malformed-GPX failure reporting in `tests/import_gpx_archive.rs`.
- Validation completed for this slice with `cargo fmt --check`, `cargo test --test import_gpx_archive`, and `cargo test import_gpx --lib`.
- Added `application::import::import_gpx_archive_into_project`, which imports archived GPX results into independent track and waypoint layers through `ProjectCommand` and `CommandStack`.
- Added `ArchiveImportReport` and `ArchiveImportError` so application-level import results and failures stay explicit without UI coupling.
- Added workflow coverage in `tests/application_zip_gpx_import.rs` for successful imports, malformed GPX errors, preserved preexisting map layers, and non-regression of map-layer counts.
- Validation for the application slice initially exposed an unrelated UI dependency drift: `walkers 0.53.0` now pulls `eframe 0.34.x`, which requires `App::ui` instead of the legacy `update` trait method.
- Applied the minimal compatibility fix in `src/ui/mod.rs` by moving background orchestration into `App::logic`, keeping widget rendering in `App::ui`, and switching panel rendering to `show_inside` so validation can continue on the current dependency stack.
- Validation completed for this slice with `cargo fmt --check`, `cargo test --test application_zip_gpx_import`, and `cargo test import_gpx_archive --lib`.

## Constraints
- Preserve `domain` / `application` / `infrastructure` / `ui` boundaries.
- Treat archive and format parsing as infrastructure concerns.
- Prefer staged delivery; do not attempt all formats in one step.
- Keep imported maps, tracks, and waypoints as independent entities.
- Update docs as supported format scope changes.
- Use `zip` + `gpx` for the first archive-import slice; defer KML and Ozi text parsers until the archive boundary is stable.
- Keep `ozf2` out of implementation slices until a separate feasibility/legal decision is made.

## Exit Criteria
- [x] Feature is broken down into staged subtasks with dependencies.
- [x] First recommended implementation slice is identified and scoped tightly.
- [x] External format/library docs needed for implementation are identified.
- [x] ZIP archives can be enumerated into typed entries without touching UI code.
- [x] Supported-entry detection distinguishes text/map metadata files from unsupported raster/binary payloads.
- [x] Deterministic tests cover archive enumeration and unsupported entry classification.
- [x] A GPX entry from a ZIP archive can be parsed into project-facing track and waypoint data structures.
- [x] Parser behavior for malformed GPX remains deterministic and returns explicit errors.
- [x] Adapter tests cover tracks, waypoints, and mixed-content archives.
- [x] Application services can import GPX archive content into independent track and waypoint layers.
- [x] Import failures are surfaced as explicit application-level errors without UI coupling.
- [x] The workflow does not regress existing map layer registration behavior.
