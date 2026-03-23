# Testing Strategy

## Goals

Testing should protect domain invariants, command behavior, persistence boundaries, and regression-prone editing workflows.

The strategy favors deterministic tests that make architectural violations obvious.

## Test Layers

### Unit Tests

Focus on domain invariants and pure behavior:
- entity creation and validation;
- track and waypoint mutation rules;
- split/join behavior;
- geometry-like helper logic.

Expected qualities:
- fast;
- deterministic;
- no UI runtime required.

### Application Workflow Tests

Focus on command/use-case orchestration:
- explicit edit commands;
- undo/redo behavior;
- selection-driven operations where application state matters;
- cross-entity workflow correctness.

These tests should verify that the application layer preserves domain boundaries while still supporting realistic editing flows.

### Integration Tests

Focus on boundaries:
- import/export;
- project save/load;
- round-trip correctness;
- failure handling for malformed or unsupported input.

### Regression Tests

Every fixed bug should add a focused regression test.

The regression should capture:
- the triggering input or sequence;
- the expected safe behavior;
- the exact invariant that previously failed.

### Property-Based Tests

Use property-based tests where parsers or geometry-like transforms benefit from broader coverage, especially for:
- coordinate conversions or normalization logic;
- parser robustness;
- reversible transformations;
- import/export round-trips with constrained generators.

## Test Design Rules

- prefer deterministic tests;
- keep fixtures readable and reviewable;
- mock external boundaries only where that improves clarity;
- avoid UI-heavy tests for domain behavior that should stay below the UI layer.

## Quality Gates

Before considering work done:
- code compiles;
- relevant tests pass;
- architecture boundaries remain intact;
- docs are updated when behavior changes.

## Proposed Command Set

When the Rust project scaffolding exists, the standard verification path should be:
- `cargo fmt --check`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo test --all`

Add `cargo nextest` when the suite becomes large enough to benefit from it.

## Early Priorities

The first implemented features should get tests in this order:
1. domain entities and invariants;
2. command-driven edits and undo/redo;
3. save/load and format boundaries;
4. regression coverage for discovered editing bugs.

## Known Gaps At Kickoff

- no production code exists yet, so there is no executable test harness;
- no file-format matrix has been selected yet;
- external workflow-heavy OziExplorer add-on references still need structured extraction into concrete acceptance tests.
