# ADR-0001: Initial Architecture Boundaries

- Status: accepted
- Date: 2026-03-22

## Context

The project aims to replace the useful core of OziExplorer with a modern Rust desktop application while avoiding legacy UX and data-model constraints.

The repository needs a clear architectural baseline before implementation begins so that editing features, persistence, and UI work can evolve without collapsing into a monolith.

## Decision

Adopt a four-layer architecture with explicit boundaries:
- `domain` for entities, invariants, and pure business logic;
- `application` for commands, use-cases, and undo/redo orchestration;
- `infrastructure` for import/export, persistence, and format adapters;
- `ui` for rendering, interaction, and transient view state.

All non-trivial edits must flow through explicit commands rather than ad hoc UI mutations.

UI-only state should remain outside persisted domain entities unless there is a documented reason to store it.

## Consequences

### Positive

- stronger testability for the core model;
- clearer undo/redo seams;
- easier isolation of file-format and persistence concerns;
- lower risk of recreating legacy special cases in the domain model.

### Negative

- more upfront design work before rapid UI experimentation;
- some features may feel slower to prototype because boundaries are enforced early;
- import/export code may require translation layers instead of reusing persistence structures directly.

## Rejected Alternatives

### UI-centric architecture

Rejected because it would make core editing logic harder to test and easier to entangle with transient interaction state.

### Persistence-driven domain model

Rejected because file formats and user-facing domain concepts have different responsibilities and should not be treated as identical.

### Legacy-compatible special cases

Rejected because privileged entities and append-only workflows conflict with the stated product direction.

## Follow-Up Decisions

- choose the command abstraction for reversible edits;
- define the first persisted project schema;
- choose first supported import/export formats;
- decide the minimal UI shell and technology stack once the core edit model is underway.
