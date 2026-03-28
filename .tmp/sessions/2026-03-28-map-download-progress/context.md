# Task Context: Show map download progress

Session ID: 2026-03-28-map-download-progress
Created: 2026-03-28T07:45:00Z
Status: implemented_validated

## Current Request
Во-первых, хочу чтобы показывался прогресс загрузки карты

## Context Files (Standards to Follow)
- /Users/sobieg/.config/opencode/context/core/standards/code-quality.md
- /Users/sobieg/.config/opencode/context/core/standards/test-coverage.md
- /Users/sobieg/.config/opencode/context/core/standards/code-analysis.md
- /Users/sobieg/Projects/ozi-rs/docs/architecture.md
- /Users/sobieg/Projects/ozi-rs/docs/testing-strategy.md

## Reference Files (Source Material to Look At)
- /Users/sobieg/Projects/ozi-rs/src/application/mod.rs
- /Users/sobieg/Projects/ozi-rs/src/infrastructure/lizaalert.rs
- /Users/sobieg/Projects/ozi-rs/src/ui/mod.rs

## External Docs Fetched
- None

## Components
- background download progress messages
- application status updates
- sidebar progress display
- validation

## Progress Notes
- Refactored map download to stream response chunks to disk instead of buffering the whole response before writing.
- Added background progress messages carrying downloaded byte counts and optional total size.
- Updated application state handling so progress messages keep the background task marked busy while refreshing the status text.
- Full validation now passes with `cargo test`.

## Constraints
- Keep progress reporting in application/UI layers; do not leak transport details into domain.
- Prefer incremental status updates without overcomplicating the download path.
- Reuse existing sidebar status area where possible.

## Exit Criteria
- [x] Map downloads emit progress updates while in flight.
- [x] Sidebar shows meaningful progress state during download.
- [x] Relevant tests pass.
