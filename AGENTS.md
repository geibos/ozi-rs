# Rust Map Editor Project Rules

This repository contains a Rust desktop application for working with raster maps, tracks, and waypoints.

The product is a modern, predictable, testable replacement for the useful core of OziExplorer.
It is **not** a clone and must not reproduce legacy UX or data model limitations.

## Product intent

The app should support:
- opening raster maps;
- loading multiple tracks and waypoint collections;
- creating and editing tracks and waypoints;
- split / join operations on tracks;
- saving projects and exporting data.

The app should **not** include in MVP:
- datum management;
- advanced geodesy and projections beyond what is strictly required;
- GPS device sync;
- routes, events, live telemetry;
- special-case behavior like a privileged “Track 1”.

## Engineering principles

- Prefer correctness, clarity, and testability over cleverness.
- Keep domain logic independent from GUI.
- Treat GUI as an adapter over application services and domain commands.
- Every non-trivial feature should have tests.
- New functionality must be accompanied by documentation updates.
- Avoid hidden mutable global state.
- Avoid “append-only” UX models inherited from legacy GIS tools.
- Model maps, tracks, and waypoint collections as independent entities.

## Architecture constraints

Organize code so that the following boundaries remain explicit:
- `domain` / core business rules and entities;
- `application` / use-cases and command handling;
- `infrastructure` / import-export, persistence, file formats;
- `ui` / rendering, interaction, view state.

The domain layer must not depend on GUI types.
The domain layer should be serializable/testable without the UI runtime.

## Data model expectations

At minimum, the project model should have distinct concepts for:
- `Project`
- `MapLayer`
- `TrackLayer`
- `WaypointLayer`
- `Track`
- `TrackSegment`
- `TrackPoint`
- `Waypoint`
- selection and edit state outside persistent domain entities where practical

Do not bake UI-only state into persisted domain entities unless there is a strong reason.

## Editing model expectations

Use explicit commands for edits.
Examples:
- add waypoint
- move waypoint
- rename waypoint
- create track
- insert track point
- move track point
- split track segment
- join segments
- delete selection

The system should support undo/redo through a command stack or equivalent reversible operation log.

## Testing standards

Minimum expectation for every implemented feature:
- unit tests for domain invariants;
- integration tests for import/export and application workflows;
- regression tests for every fixed bug;
- property-based tests where parsers or geometry-like transforms are involved.

Prefer deterministic tests.
If snapshots are used, keep them readable and reviewable.

## Documentation requirements

Keep these documents current when the code changes materially:
- `README.md`
- `docs/requirements.md`
- `docs/architecture.md`
- `docs/testing-strategy.md`
- `docs/roadmap.md`
- `docs/adr/`

If an architectural decision changes, add or update an ADR.

## Delivery workflow

For substantial work, follow this order:
1. clarify requirements;
2. update or create plan;
3. propose affected files/modules;
4. implement in small coherent steps;
5. run tests/linters;
6. update docs;
7. summarize changes and risks.

## Review checklist

Before considering work done, verify:
- code compiles;
- tests relevant to the change pass;
- no accidental architectural boundary violations;
- docs updated;
- behavior matches product intent;
- no legacy OziExplorer constraints were reintroduced by accident.

## Rust-specific conventions

- Prefer stable Rust.
- Prefer small focused modules.
- Use `thiserror` for domain/application error types when appropriate.
- Use `serde` for persistence boundaries.
- Keep traits minimal and meaningful.
- Avoid premature abstraction.
- Add comments only where intent is non-obvious.
- Use `clippy` cleanly unless there is a documented exception.

## Agent usage

Use the OpenCode agents configured for this repository as the source of truth for role behavior, permissions, and command routing.

- `AGENTS.md` defines shared product, architecture, testing, and documentation rules for all agents.
- `opencode.json` defines which OpenCode agents and commands are available in this repository.
- `.opencode/agents/` contains role-specific prompts and constraints; keep detailed role behavior there instead of duplicating it here.
- `.opencode/commands/` contains task entrypoints such as kickoff, implementation, testing, and docs synchronization.
- If a local agent prompt conflicts with `AGENTS.md`, follow `AGENTS.md` for repository policy and adjust the agent prompt/config rather than weakening the project rules here.

If anything is unclear, prefer explicit assumptions over silent guessing, and record the assumption in docs.
