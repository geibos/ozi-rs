# Task Context: Fix parse_map_packages test

Session ID: 2026-03-28-lizaalert-parse-map-packages-test-fix
Created: 2026-03-28T07:40:00Z
Status: implemented_validated

## Current Request
почини тест

## Context Files (Standards to Follow)
- /Users/sobieg/.config/opencode/context/core/standards/code-quality.md
- /Users/sobieg/.config/opencode/context/core/standards/test-coverage.md
- /Users/sobieg/.config/opencode/context/core/standards/code-analysis.md
- /Users/sobieg/.config/opencode/context/development/principles/clean-code.md

## Reference Files (Source Material to Look At)
- /Users/sobieg/Projects/ozi-rs/src/infrastructure/lizaalert.rs

## External Docs Fetched
- None

## Components
- lizaalert HTML link parsing
- test fixture correction
- regression validation

## Progress Notes
- Identified that parser regex incorrectly matched escaped quotes instead of normal HTML attributes.
- Updated parser regex to match `href="..."` in real HTML source text.
- Corrected the regression test fixture to use actual HTML attribute quoting.
- Targeted validation now passes: `cargo test parse_map_packages_reads_sqlite_entries`.

## Constraints
- Apply the smallest safe fix.
- Preserve current parsing intent for live fetched HTML.
- Stop after validation and report any unrelated blocker.

## Exit Criteria
- [x] `parse_map_packages_reads_sqlite_entries` passes.
- [x] Parsing regex matches normal HTML attributes with quoted href values.
