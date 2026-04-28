# ADR-0021: Undo Stack — Reaffirm Delta-Based, Depth 100, No Hybrid

- Status: accepted
- Date: 2026-04-28

## Context

During MVP scope brainstorming the question of undo depth was raised. A "hybrid"
scheme was floated: keep the last 20 actions densely, then thin out further history
(every 5/10/20 actions). The motivation was anticipation of long editing sessions
on tracks with tens of thousands of points.

ADR-0017 already established the current scheme: delta-based command stack with
`MAX_STACK_DEPTH = 100`, drag coalescing via `apply_or_merge`, and reverse commands
computed at apply time. Memory usage is proportional to command count rather than
project size.

## Decision

Keep ADR-0017 as-is for MVP **and beyond**. No hybrid scheme. No depth tuning.

The 100-step delta stack with drag coalescing is sufficient for the SAR-volunteer
workflow:

1. The workflow is not a graphics editor. A typical session is import a few tracks,
   make a small number of point edits, place waypoints, export. The number of distinct
   user-visible actions per session does not approach 100.
2. Drag operations already collapse into one delta via `apply_or_merge` — long drags
   do not consume stack slots.
3. Each delta is a forward+reverse command pair, not a project clone. Memory pressure
   from undo is negligible compared to the rendered tile cache.
4. A hybrid scheme is non-trivial: thinning history breaks the invariant that any
   prefix of the redo stack composes back to a valid earlier state. Reintroducing
   that invariant for thinned entries needs additional design.

## Consequences

### Positive

- No additional engineering load on MVP from undo concerns.
- Existing tests under `application/commands.rs` remain authoritative.

### Negative

- If a future workflow does generate dense action streams (for example, programmatic
  track filtering applied as many small edits), the 100-step ceiling can drop earlier
  history. This is a known and accepted trade-off until evidence contradicts it.

## Revisit when

- Real telemetry shows users hitting the 100-step ceiling regularly, or
- A new workflow class (programmatic edits, batch tools) generates dense streams.

In that case a follow-up ADR may introduce thinning, depth growth, or a hybrid
scheme. Until then the cost of changing it now exceeds the cost of leaving it.

## Related

- ADR-0017 — delta-based undo/redo (the design this ADR reaffirms)
- ADR-0020 — MVP scope (lists undo as MVP-must, references this ADR)
