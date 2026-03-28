# Task Context: Fix walkers attribution build break

Session ID: 2026-03-28-build-fix-walkers-attribution
Created: 2026-03-28T07:24:17Z
Status: implemented_validated

## Current Request
Посмотреть контекст нескольких параллельных агентских изменений и поправить сборку.

## Context Files (Standards to Follow)
- /Users/sobieg/.config/opencode/context/core/standards/code-quality.md
- /Users/sobieg/.config/opencode/context/core/standards/test-coverage.md
- /Users/sobieg/.config/opencode/context/core/standards/code-analysis.md
- /Users/sobieg/.config/opencode/context/core/workflows/external-libraries.md
- /Users/sobieg/.config/opencode/context/core/workflows/component-planning.md

## Reference Files (Source Material to Look At)
- /Users/sobieg/Projects/ozi-rs/Cargo.toml
- /Users/sobieg/Projects/ozi-rs/src/ui/sqlite_tiles.rs
- /Users/sobieg/Projects/ozi-rs/src/ui/mod.rs
- /Users/sobieg/Projects/ozi-rs/.tmp/external-context/walkers/attribution-api-migration.md

## External Docs Fetched
- walkers 0.52.0 attribution API: `Attribution` now contains only `text`, `url`, `logo_light`, and `logo_dark`.
- `AttributionType`, `logo_link`, and `attribution_type` were removed.
- `text` and `url` require `&'static str`, so the old dynamic `String` attribution no longer compiles.

## Components
- walkers attribution compatibility fix
- UI call site adjustment
- build validation

## Progress Notes
- Updated `SqliteTiles` attribution construction to match `walkers 0.52`.
- Removed the incompatible dynamic attribution string from the UI call site.
- Build/test validation now succeeds as part of the full `cargo test` run.

## Constraints
- Apply the smallest safe fix to restore compilation.
- Avoid leaking memory or introducing unsafe lifetime workarounds for dynamic attribution strings.
- Stop and report if validation reveals additional unrelated failures.

## Exit Criteria
- [x] `src/ui/sqlite_tiles.rs` matches walkers 0.52 attribution API.
- [x] UI call site no longer passes incompatible attribution data.
- [x] `cargo test` completes successfully, or the next blocker is reported explicitly.
