# Task Context: Unified cached LizaAlert project workflow

Session ID: 2026-03-29-lizaalert-project-cache
Created: 2026-03-29T07:25:00Z
Status: bundle_utf8_and_progress_regression_fixed

## Current Request
Opening an online LizaAlert project should always use a unified UI flow: if the project is already cached locally, open it from the local copy; otherwise download the whole project structure once, store it locally, and then open it from that local cache without extra buttons.

## Context Files (Standards to Follow)
- /Users/sobieg/.config/opencode/context/core/standards/code-quality.md
- /Users/sobieg/.config/opencode/context/core/standards/test-coverage.md
- /Users/sobieg/.config/opencode/context/core/standards/documentation.md
- /Users/sobieg/Projects/ozi-rs/AGENTS.md
- /Users/sobieg/Projects/ozi-rs/docs/architecture.md
- /Users/sobieg/Projects/ozi-rs/docs/requirements.md
- /Users/sobieg/Projects/ozi-rs/docs/testing-strategy.md

## Reference Files (Source Material to Look At)
- /Users/sobieg/Projects/ozi-rs/.tmp/sessions/2026-03-23-map-layer-registration/context.md
- /Users/sobieg/Projects/ozi-rs/.tmp/sessions/2026-03-28-map-download-progress/context.md
- /Users/sobieg/Projects/ozi-rs/.tmp/sessions/2026-03-28-zip-ozi-import/context.md
- /Users/sobieg/Projects/ozi-rs/.tmp/sessions/2026-03-29-ozf2-rs-integration/context.md
- /Users/sobieg/Projects/ozi-rs/src/infrastructure/lizaalert.rs
- /Users/sobieg/Projects/ozi-rs/src/application/mod.rs
- /Users/sobieg/Projects/ozi-rs/src/application/commands.rs
- /Users/sobieg/Projects/ozi-rs/src/domain/project.rs
- /Users/sobieg/Projects/ozi-rs/src/ui/mod.rs

## External Docs Fetched
- None; current work stays within repo architecture and existing LizaAlert/OZI workflows.

## Components
- local cached project layout and source mirror
- infrastructure cached-or-download project materialization
- application unified project open orchestration
- diagnostics/progress updates that keep UI unchanged
- cached local OZI map discovery and opening through the same project UI

## Constraints
- Opening online vs local projects must not diverge in UI behavior.
- The first open should materialize a durable local project copy.
- Subsequent opens should prefer the local cached copy.
- Keep persistence/download details in infrastructure and orchestration in application.
- Preserve the current sqlite/OZI flows while creating the broader project cache foundation.

## Progress Notes
- Introduced a deterministic local cache root at `.tmp/lizaalert-projects/{project-slug}/source/` for mirrored online project contents.
- Added infrastructure `open_project(...)` logic that checks for a cached local copy, downloads the full remote project directory tree on first open, and then loads the project from local files.
- Mirrored remote project directories recursively via generic directory-list parsing so root files and nested folders are stored under the same local project structure.
- Switched cached project loading to read local `2-Coordinates.txt` and scan local `8-Android&iOS/*.sqlitedb` files instead of refetching remote metadata.
- Extended `LizaMapPackage` with `local_path` so the application can distinguish cached local map files from remote-only entries.
- Updated `load_project(...)` status text to distinguish `Downloading project ...` from `Opening cached project ...` without changing the UI entrypoint.
- Updated `open_selected_map(...)` to prefer cached local sqlite maps immediately instead of redownloading them.
- Extended cached project discovery so mirrored `.map` files are found recursively under the local project source tree, parsed through the existing OZI metadata parser, and exposed as project map entries alongside cached sqlite packages.
- Updated cached map selection so local `.map` entries open through the existing OZI flow while cached sqlite maps continue to use the sqlite-tile path.
- Updated the project sidebar wording from mobile-only packages to generic cached project maps because the same project list can now include mirrored OZI entries.
- Added focused tests for local cached project parsing, directory-entry parsing, remote package parsing, and cached map opening behavior.
- Added focused tests covering cached project loading with both sqlite and OZI maps plus cached OZI opening without a download thread.
- Fixed a regression where mirrored OZI `.map` discovery failed on non-UTF-8 map text by moving cached-project OZI reads to a shared lossy-safe map-text loader.
- Restored clearer project-open progress reporting by adding background `ProjectLoadProgress` messages during cached bundle download/indexing instead of only a single initial status line.
- Removed the manual `Local OZI map` controls from the main sidebar so the normal workflow stays centered on project bundles only.
- Validation passed with `cargo fmt --check` and `cargo test --lib`.

## Exit Criteria
- [x] A deterministic local cache layout exists for downloaded LizaAlert projects.
- [x] Project-open flow can detect cached projects vs missing local copies.
- [x] First open downloads and stores the project locally without extra UI actions.
- [x] Subsequent opens use the local cache path.
- [x] Focused tests cover the first cached-project slice.
- [x] Cached mirrored OZI `.map` files can be opened from the same project UI path as cached sqlite maps.
- [x] Cached bundle loading no longer fails when mirrored OZI `.map` files contain non-UTF-8 bytes.
- [x] Project-open progress/status is visible again during cached bundle download/indexing.
