# ADR-0005: Snapshot-Based Undo/Redo

- Status: accepted
- Date: 2026-03-23

## Context

The project model requires undo/redo for all editing commands. Two classical approaches
exist:

1. **Inverse commands** — each command stores enough data to reverse itself; undo
   applies the inverse operation
2. **Full snapshots** — before each command the entire project state is cloned and
   pushed onto an undo stack; undo restores the previous clone

## Decision

Use **full project snapshots** (`Vec<Project>`) in `CommandStack`.

`CommandStack` holds:
- `undo_history: Vec<Project>` — states before each applied command
- `redo_history: Vec<Project>` — states after each undone command

On `apply`: push a clone of the current project, then mutate it.
On `undo`: pop from undo stack, push current state to redo stack, replace project.
On `redo`: pop from redo stack, push current state to undo stack, replace project.

## Rationale

The snapshot approach was chosen because:
- It is trivially correct — no risk of inverse command getting out of sync with the
  forward command
- Every `ProjectCommand` automatically gets undo/redo without extra implementation work
- The `Project` type derives `Clone`, making snapshots cheap to implement
- Correctness is more important than memory efficiency at this stage of development

## Consequences

### Positive

- Every command gets undo/redo for free
- No risk of inverse command bugs
- Easy to test: apply N commands, undo N times, assert state equals initial state

### Negative

- Memory usage grows linearly with undo history depth
- For projects with large track collections (many thousands of points), each snapshot
  clone is expensive
- No upper bound on history depth is enforced currently (was removed in favour of
  unbounded history)

## Rejected Alternatives

### Inverse commands

Rejected because each command would need a hand-written inverse, and any divergence
between the forward and inverse operation would produce silent corruption. The
additional correctness risk was not justified at this stage.

## Follow-Up

- If memory usage becomes a problem, consider capping the undo stack depth
- For Phase 7 track point editing, evaluate whether per-point edits create too many
  snapshots during drag operations (may need to coalesce drag into a single command)
