# Task Context: Command Stack Foundation

Session ID: 2026-03-23-command-stack-foundation
Created: 2026-03-23T17:31:51Z
Status: completed

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
- /Users/sobieg/Projects/ozi-rs/src/domain/project.rs
- /Users/sobieg/Projects/ozi-rs/src/domain/mod.rs
- /Users/sobieg/Projects/ozi-rs/src/application/mod.rs
- /Users/sobieg/Projects/ozi-rs/src/ui/mod.rs
- /Users/sobieg/Projects/ozi-rs/docs/roadmap.md
- /Users/sobieg/Projects/ozi-rs/docs/architecture.md
- /Users/sobieg/Projects/ozi-rs/docs/requirements.md
- /Users/sobieg/Projects/ozi-rs/.tmp/sessions/2026-03-23-core-project-model/context.md

## External Docs Fetched
- None for this slice.

## Components
- Application command abstraction for project edits
- Undo/redo history for reversible project mutations
- Tests for command execution and history behavior
- Repo-local task tracking so started work is visible

## Constraints
- Keep the domain layer UI-free and reusable.
- Start with command-driven layer creation before broader edit coverage.
- Keep the first undo/redo slice testable and deterministic.

## Exit Criteria
- [x] The application layer can apply a reversible project edit command.
- [x] Undo and redo restore project state deterministically.
- [x] Tests cover command execution and history behavior.
- [x] Started work is recorded in repo-local session and task files.
