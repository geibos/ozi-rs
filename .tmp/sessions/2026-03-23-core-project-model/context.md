# Task Context: Core Project Model

Session ID: 2026-03-23-core-project-model
Created: 2026-03-23T17:28:20Z
Status: in_progress

## Current Request
Look through the project, decide which feature to build next, start implementing it, and record what has already been started.

## Context Files (Standards to Follow)
- /Users/sobieg/.config/opencode/context/core/standards/code-quality.md
- /Users/sobieg/.config/opencode/context/core/standards/test-coverage.md
- /Users/sobieg/.config/opencode/context/core/standards/documentation.md
- /Users/sobieg/.config/opencode/context/core/standards/security-patterns.md
- /Users/sobieg/.config/opencode/context/development/principles/clean-code.md
- /Users/sobieg/.config/opencode/context/core/workflows/feature-breakdown.md

## Reference Files (Source Material to Look At)
- /Users/sobieg/Projects/ozi-rs/Cargo.toml
- /Users/sobieg/Projects/ozi-rs/src/lib.rs
- /Users/sobieg/Projects/ozi-rs/src/domain/mod.rs
- /Users/sobieg/Projects/ozi-rs/src/application/mod.rs
- /Users/sobieg/Projects/ozi-rs/src/ui/mod.rs
- /Users/sobieg/Projects/ozi-rs/README.md
- /Users/sobieg/Projects/ozi-rs/docs/architecture.md
- /Users/sobieg/Projects/ozi-rs/docs/roadmap.md
- /Users/sobieg/Projects/ozi-rs/docs/requirements.md

## External Docs Fetched
- None for this slice.

## Components
- Domain project model with independent map, track, and waypoint layers
- Domain entities for tracks, segments, track points, and waypoints
- Application commands for explicit project edits
- Application-facing project summary derived from the domain state
- Minimal UI shell updates to surface the new model shape
- Documentation note that Phase 1 implementation has started

## Constraints
- Keep domain logic independent from the UI layer.
- Favor small, testable modules and explicit APIs.
- Implement one incremental slice only: project and layer entities, not command stack yet.

## Exit Criteria
- [ ] The domain model represents project, map layers, track layers, and waypoint layers as distinct concepts.
- [ ] Tests cover the initial project invariants for independent layer collections.
- [ ] The app shell reflects the new project state without crossing architecture boundaries.
- [ ] The started work is recorded in repo-local session notes.
