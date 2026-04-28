# ADR-0017: Delta-Based Undo/Redo

- Status: accepted
- Date: 2026-04-04
- Supersedes: ADR-0005 (snapshot-based undo/redo)

## Context

ADR-0005 selected full project snapshots for undo/redo. By Phase 7, track point editing
revealed two problems with this approach:

1. **Drag operations** — moving a track point generates dozens of intermediate positions
   per second. Each would clone the entire project, creating unbounded memory growth
   during a single drag.

2. **Memory cost** — projects with large track collections (thousands of points across
   many tracks) make each `Project::clone()` expensive. With no depth limit, long
   editing sessions could consume hundreds of megabytes.

The follow-up section of ADR-0005 anticipated both issues.

## Decision

Replace snapshot-based undo with **delta-based undo** using forward/reverse command pairs.

### CommandDelta

Each applied command is stored as a `CommandDelta`:

```rust
struct CommandDelta {
    forward: ProjectCommand,
    reverse: ProjectCommand,
}
```

`reverse` is computed by `ProjectCommand::reverse(&self, project: &Project)` at the
moment of application, while the project is still in the pre-mutation state.

### CommandStack

```rust
struct CommandStack {
    undo_history: Vec<CommandDelta>,  // max depth: 100
    redo_history: Vec<CommandDelta>,  // cleared on new command
}
```

- **apply**: compute reverse, mutate project, push delta to undo history
- **undo**: pop delta, apply its `reverse`, push delta to redo history
- **redo**: pop delta, apply its `forward`, push delta to undo history

### Command Merging

`apply_or_merge(command, project)` checks if the new command targets the same entity
as the last undo entry (via `targets_same_entity()`). If so, the new command replaces
the `forward` in the existing delta while keeping the original `reverse`. This coalesces
an entire drag sequence into a single undo step.

### Max Depth

`MAX_STACK_DEPTH = 100`. When exceeded, the oldest delta is dropped.

## Rationale

- Drag coalescing is natural: same entity → merge, different entity → new delta
- Memory usage is proportional to command count, not project size
- Each command already knows how to reverse itself (structural requirement of the
  command pattern), so `reverse()` adds minimal implementation cost
- Testing is straightforward: apply → undo → assert state matches original

## Consequences

### Positive

- Drag operations use constant memory regardless of move count
- Bounded undo history (100 steps) prevents unbounded growth
- Coalescing is automatic and transparent to the UI

### Negative

- Each `ProjectCommand` variant must implement a correct `reverse()` — a bug in any
  reverse function causes silent state corruption on undo
- Reverse computation requires access to the pre-mutation project state, constraining
  when `reverse()` can be called

## Non-Undoable Mutations

Some style mutations bypass the command stack entirely:
- `SetTrackColor` — immediate visual feedback, non-destructive
- `SetTrackLineWidth` — same
- `ToggleTrackVisible` — same

These are applied directly to `AppState` without recording a delta.
