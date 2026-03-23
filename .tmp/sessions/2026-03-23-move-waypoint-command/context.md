# Task Context: Move Waypoint Command

Session ID: 2026-03-23-move-waypoint-command
Created: 2026-03-23T17:37:20Z
Status: in_progress

## Current Request
Посмотри проект. Реши, какую фичу делать следующей и начни делать. Обязательно запиши, что начал ее делать, чтобы не повторяться.

## Context Files (Standards to Follow)
- /Users/sobieg/.config/opencode/context/core/standards/code-quality.md
- /Users/sobieg/.config/opencode/context/core/standards/test-coverage.md
- /Users/sobieg/.config/opencode/context/core/standards/documentation.md
- /Users/sobieg/.config/opencode/context/core/workflows/feature-breakdown.md
- /Users/sobieg/Projects/ozi-rs/docs/architecture.md
- /Users/sobieg/Projects/ozi-rs/docs/requirements.md
- /Users/sobieg/Projects/ozi-rs/docs/roadmap.md

## Reference Files (Source Material to Look At)
- /Users/sobieg/Projects/ozi-rs/src/domain/project.rs
- /Users/sobieg/Projects/ozi-rs/src/domain/waypoint.rs
- /Users/sobieg/Projects/ozi-rs/src/application/commands.rs
- /Users/sobieg/Projects/ozi-rs/.tmp/sessions/2026-03-23-command-stack-foundation/context.md
- /Users/sobieg/Projects/ozi-rs/.tmp/sessions/2026-03-23-waypoint-command-slice/context.md

## External Docs Fetched
- None for this slice.
- Validation follow-up: `walkers 0.52.0` now exposes `sources::Attribution` without `AttributionType`, `logo_link`, or `attribution_type`; current build is blocked by a pre-existing mismatch in `src/ui/sqlite_tiles.rs`.

## Components
- Domain support for targeted waypoint updates inside a waypoint layer
- Application command for moving an existing waypoint by id
- Focused tests for success and missing-target failures
- Repo-local tracking so the started work is visible and not repeated

## Constraints
- Continue from the already implemented add-track and add-waypoint command slices instead of duplicating them.
- Keep edits command-driven and reversible through the existing command stack.
- Keep domain logic UI-free and return explicit errors for missing layers or waypoints.
- Do not fix the unrelated `walkers` UI build break without explicit approval because validation hit a pre-existing dependency/API mismatch.

## Exit Criteria
- [ ] The application layer can move a waypoint inside a chosen waypoint layer through an explicit command.
- [ ] Missing layer or missing waypoint failures do not mutate project state or command history.
- [ ] Tests cover the successful and failing move-waypoint paths.
- [ ] Started work is recorded in repo-local session notes.
