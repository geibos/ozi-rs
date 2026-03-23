# Task Context: Map Layer Registration

Session ID: 2026-03-23-map-layer-registration
Created: 2026-03-23T17:35:46Z
Status: in_progress

## Current Request
Look through the project, decide which feature to build next, start implementing it, and record what has already been started.

## Context Files (Standards to Follow)
- /Users/sobieg/Projects/ozi-rs/AGENTS.md
- /Users/sobieg/.config/opencode/context/core/standards/code-quality.md
- /Users/sobieg/.config/opencode/context/core/standards/test-coverage.md
- /Users/sobieg/.config/opencode/context/core/standards/documentation.md
- /Users/sobieg/.config/opencode/context/development/principles/clean-code.md
- /Users/sobieg/.config/opencode/context/core/workflows/feature-breakdown.md

## Reference Files (Source Material to Look At)
- /Users/sobieg/Projects/ozi-rs/src/domain/project.rs
- /Users/sobieg/Projects/ozi-rs/src/application/mod.rs
- /Users/sobieg/Projects/ozi-rs/src/application/mod.rs
- /Users/sobieg/Projects/ozi-rs/src/application/commands.rs
- /Users/sobieg/Projects/ozi-rs/src/ui/mod.rs
- /Users/sobieg/Projects/ozi-rs/src/infrastructure/lizaalert.rs
- /Users/sobieg/Projects/ozi-rs/docs/requirements.md
- /Users/sobieg/Projects/ozi-rs/docs/roadmap.md
- /Users/sobieg/Projects/ozi-rs/README.md

## External Docs Fetched
- None for this slice.

## Components
- Domain map-layer metadata for locally opened maps
- Application command wiring so opened maps register inside the project model
- Tests for map registration and duplicate-open behavior
- Repo-local tracking so this started work is visible and not repeated

## Constraints
- Keep map registration command-driven and inside the application layer.
- Preserve the separation between transient UI selection and persisted project state.
- Do not replace existing active-map behavior; register loaded maps alongside it.

## Exit Criteria
- [ ] Opened local maps can be represented as project map layers with source-path metadata.
- [ ] AppState registers a downloaded map into the project model through an explicit command.
- [ ] Reopening the same local map does not duplicate the project map layer entry.
- [ ] Tests cover the map registration path.
- [ ] Started work is recorded in repo-local session and task files.
