# ADR-0015: Rust 2024 Edition

- Status: accepted
- Date: 2026-03-23

## Context

Rust editions define a small set of opt-in language changes that would otherwise be
backward-incompatible. The 2024 edition was stabilized in Rust 1.85 (February 2025).

The project was started in March 2026, when 2024 was the current stable edition.

## Decision

Use **`edition = "2024"`** in `Cargo.toml`.

## Key 2024 Edition Changes Used

- `if let` chains without extra blocks
- `impl Trait` in more positions
- `gen` keyword reservation (unused, but avoids future breakage)
- Lifetime capture rules for `impl Trait` return types (more precise)

## Consequences

### Positive

- Uses the current stable edition; no technical debt from an older edition
- Benefits from improved lifetime inference and `impl Trait` ergonomics
- Future language features will target 2024 compatibility first

### Negative

- Requires **Rust 1.85 or newer**; contributors with older toolchains will see
  compile errors
- Some tooling (older IDE plugins, nightly-only analysis tools) may have partial
  2024 support

## Minimum Supported Rust Version

The project currently has no explicit `rust-version` field in `Cargo.toml`. Effective
MSRV is Rust 1.85 (first stable release with 2024 edition support).
Consider adding `rust-version = "1.85"` to make this explicit.
