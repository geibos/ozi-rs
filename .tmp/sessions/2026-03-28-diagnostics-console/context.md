# Task Context: Diagnostics console for network and map-loading errors

Session ID: 2026-03-28-diagnostics-console
Created: 2026-03-28T09:40:00Z
Status: implemented_validated

## Current Request
Show errors like `error sending request for url (https://maps.lizaalert.ru/maps/)` in an in-app console/log view instead of only a transient status line or stderr.

## Context Files (Standards to Follow)
- /Users/sobieg/.config/opencode/context/core/standards/code-quality.md
- /Users/sobieg/.config/opencode/context/core/standards/test-coverage.md
- /Users/sobieg/Projects/ozi-rs/AGENTS.md
- /Users/sobieg/Projects/ozi-rs/docs/architecture.md
- /Users/sobieg/Projects/ozi-rs/docs/requirements.md
- /Users/sobieg/Projects/ozi-rs/docs/testing-strategy.md

## Reference Files (Source Material to Look At)
- /Users/sobieg/Projects/ozi-rs/src/application/mod.rs
- /Users/sobieg/Projects/ozi-rs/src/application/import.rs
- /Users/sobieg/Projects/ozi-rs/src/ui/mod.rs
- /Users/sobieg/Projects/ozi-rs/src/infrastructure/lizaalert.rs
- /Users/sobieg/Projects/ozi-rs/tests/application_zip_gpx_import.rs

## External Docs Fetched
- None yet; current egui/eframe docs are not required for a minimal built-in diagnostics panel.

## Progress Notes
- Added a bounded diagnostics history in `AppState` using `VecDeque`, with explicit `DiagnosticLevel` and `DiagnosticEntry` types.
- Routed important status and error updates through shared helpers so the existing status label stays intact while recent events are also retained for inspection.
- Replaced the direct `eprintln!` map-open failure path in `src/ui/mod.rs` with application-level runtime error reporting.
- Added a simple in-app `Diagnostics` console in the left sidebar that renders recent entries newest-first with info/error coloring.
- Added focused application tests for missing-project error capture and bounded recent-history retention.
- Validation passed with `cargo fmt --check` and `cargo test --lib`.

## Components
- application-level diagnostics history
- UI diagnostics console rendering
- focused tests for surfaced errors and status events

## Constraints
- Keep GUI as an adapter over application state.
- Keep diagnostics state out of persisted domain entities.
- Prefer a minimal, deterministic, testable implementation.
- Preserve existing status text while adding a richer history view.

## Exit Criteria
- [x] Important status and error events are retained in recent history.
- [x] UI shows a simple diagnostics console for recent events.
- [x] Focused tests cover recent-history behavior.
- [x] Validation passes for the touched slice.
