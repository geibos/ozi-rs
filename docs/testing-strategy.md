# Testing Strategy

## Goals

Testing should protect domain invariants, command behavior, persistence boundaries, and regression-prone editing workflows.

The strategy favors deterministic tests that make architectural violations obvious.

## Test Layers

### Unit Tests

Focus on domain invariants and pure behavior:
- Entity creation and validation
- Track and waypoint mutation rules (move, delete, insert, split, join)
- TrackStyle defaults and bounds
- Identifier uniqueness

Expected qualities:
- Fast, deterministic
- No UI runtime required
- Inline `#[cfg(test)]` modules alongside source

### Application Workflow Tests

Focus on command/use-case orchestration:
- ProjectCommand apply and reverse for all variants
- Undo/redo behavior (apply → undo → verify state restored)
- Command merging via `apply_or_merge()` for drag sequences
- Error cases (invalid IDs, out-of-bounds operations)
- Cross-entity workflow correctness

### Integration Tests

Focus on boundaries:
- Import/export round-trips (GPX, PLT)
- Project save/load (JSON)
- Format-specific edge cases (Windows-1251 encoding, OLE dates, COLORREF BGR)
- Failure handling for malformed or unsupported input

### Regression Tests

Every fixed bug should add a focused regression test capturing:
- The triggering input or sequence
- The expected safe behavior
- The exact invariant that previously failed

### Native Desktop QA

For desktop behavior in the Tauri app, use the project-local `ozi-rs-mcp` native MCP tools by default. Tier 1 native QA covers app build, native launch/stop, log capture, screenshot capture, and combined observation without requiring Appium. Appium Mac2 checks are optional and dependency-gated; unavailable Appium is an acceptable degraded path when Tier 1 native QA passes.

Playwright/browser testing is not the default for native desktop QA. It may still be used later for intentional isolated web/frontend experiments, but it should not replace native MCP verification for app behavior.

## Verification Commands

```bash
# Full verification (Rust + frontend)
just test

# Rust tests only
just test-rust

# Specific test by name
just test-filter <name>

# Frontend tests (Vitest)
just test-ui

# Strict clippy (all warnings are errors)
just clippy

# Type checking
just check
```

Underlying commands:
- `cargo test --manifest-path src-tauri/Cargo.toml`
- `cargo clippy --manifest-path src-tauri/Cargo.toml -- -D warnings`
- `npm test` (Vitest)

## Test Design Rules

- Prefer deterministic tests
- Keep fixtures readable and reviewable
- Mock external boundaries only where that improves clarity
- Avoid UI-heavy tests for domain behavior that should stay below the UI layer
- Rust tests live inline (`#[cfg(test)]` modules), not in separate test files
- Frontend tests in `src/test/` using Vitest

## Quality Gates

Before considering work done:
- Code compiles (`just check`)
- Clippy passes with zero warnings (`just clippy`)
- All Rust tests pass (`just test-rust`)
- Frontend tests pass (`just test-ui`)
- Full deterministic suite passes when applicable (`just test`)
- Native desktop QA uses `ozi-rs-mcp` Tier 1 tools for app behavior; Appium Mac2 runs only when its dependencies and permissions are available
- Architecture boundaries remain intact
- Docs are updated when behavior changes
