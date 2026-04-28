# Documentation Update — Design Spec

## Context

Project documentation was written at Phase 0 kickoff and not updated since. The codebase has evolved through Phases 1–10: UI stack migrated from egui to Tauri 2 + Svelte 5 + MapLibre GL 4, undo/redo changed from snapshot-based to delta-based, track editing and waypoint UI were implemented, PLT export added, drawing mode built. Documentation no longer reflects reality.

## Goal

Bring developer documentation in sync with the current codebase. Target audience: the developer (+ Claude Code) working on this project, and potential future contributors.

## Scope

### Update existing files (5)

1. **README.md** — reflect current feature set (track editing, waypoints, simplification, PLT export, drawing mode). Update Quick Start to `just dev`. Remove "Planned MVP" section (most items done). Mention current stack.

2. **docs/roadmap.md** — mark Phases 6–9 complete, Phase 10 partially complete (PLT export done, print deferred). Remove solved Open Questions. Add "What's Next" with remaining work.

3. **docs/architecture.md** — replace speculative module layout with real `src-tauri/src/` structure. Add UI stack section (Tauri IPC, Svelte, MapLibre). Add tile protocol section (sqlite://, ozi://). Update editing model to delta-based undo with command merging. Remove resolved "near-term decisions" and "risks".

4. **docs/requirements.md** — mark implemented requirements. Remove solved Open Questions. Add PLT export to supported formats.

5. **docs/testing-strategy.md** — remove "Known Gaps At Kickoff". Add real commands (`just test`, `just clippy`). Mention Vitest for frontend.

### New ADRs (2)

6. **docs/adr/adr-0017-delta-based-undo.md** — supersedes ADR-0005. Context: snapshot undo doesn't scale for drag operations. Decision: CommandDelta pairs (forward + reverse), apply_or_merge for drag coalescing, MAX_STACK_DEPTH=100.

7. **docs/adr/adr-0018-waypoint-symbols-as-strings.md** — Decision: `symbol: Option<String>` with a fixed display set, not a Rust enum. Reason: extensibility without data migration.

### New documents (2)

8. **docs/frontend-architecture.md** — UI stack overview, component table, state management (stores), API layer (api.ts), tile protocols, theme system (Catppuccin), key interaction modes (drawing, editing, simplification).

9. **docs/commands-reference.md** — table of ProjectCommand variants (name, description, undoable?), table of Tauri IPC commands (name, description, category). Brief intro to the command-driven editing principle.

## Out of scope

- User-facing documentation
- Inline code comments
- Auto-generated API docs
- Restructuring existing ADRs (except marking ADR-0005 as superseded)
