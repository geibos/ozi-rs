# Task Context: Improve project sidebar FPS and search

Session ID: 2026-03-28-project-sidebar-perf-search
Created: 2026-03-28T07:31:00Z
Status: implemented_validated

## Current Request
Когда проекты загружены очень низкий FPS + нет поиска по ним.

## Context Files (Standards to Follow)
- /Users/sobieg/.config/opencode/context/core/standards/code-quality.md
- /Users/sobieg/.config/opencode/context/core/standards/test-coverage.md
- /Users/sobieg/.config/opencode/context/core/standards/code-analysis.md
- /Users/sobieg/.config/opencode/context/core/workflows/feature-breakdown.md
- /Users/sobieg/.config/opencode/context/ui/web/react-patterns.md

## Reference Files (Source Material to Look At)
- /Users/sobieg/Projects/ozi-rs/src/ui/mod.rs
- /Users/sobieg/Projects/ozi-rs/src/application/mod.rs
- /Users/sobieg/Projects/ozi-rs/src/infrastructure/lizaalert.rs

## External Docs Fetched
- None. Current investigation is within local UI/application code.

## Components
- sidebar repaint strategy
- project list rendering
- project search state and filtering
- validation

## Progress Notes
- Removed unconditional `ctx.request_repaint()` and replaced it with conditional repaint while background work is active.
- Added project search text input with case-insensitive matching on project name and slug.
- Avoided cloning the full project list on every frame.
- Added focused unit tests for project search matching.

## Constraints
- Prefer the smallest safe change that improves responsiveness.
- Keep domain/application boundaries intact; search state should stay in UI unless app behavior requires persistence.
- Avoid hidden background work or continuous repaint loops.

## Exit Criteria
- [x] Sidebar no longer forces continuous repaint when idle.
- [x] Loaded project list can be filtered by search input.
- [x] Relevant tests pass.
