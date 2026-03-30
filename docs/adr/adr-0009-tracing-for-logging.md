# ADR-0009: tracing + tracing-subscriber for Structured Logging

- Status: accepted
- Date: 2026-03-30

## Context

The application needed a way to surface diagnostic messages both to the in-app
developer console and to the terminal. Early implementation used `println!` /
`eprintln!` with hardcoded `[INFO]` / `[ERROR]` prefixes, which:
- could not be filtered by level or module
- had no standard way to configure verbosity
- was not composable with ecosystem crates that emit log events

## Decision

Use **`tracing` 0.1** for instrumentation and **`tracing-subscriber` 0.3** with the
`env-filter` feature for output.

`AppState::push_diagnostic` emits `tracing::info!` or `tracing::error!` for each
diagnostic entry, in addition to storing it in the in-app console ring buffer.

`main.rs` initializes the subscriber:

```rust
tracing_subscriber::fmt()
    .with_env_filter(
        tracing_subscriber::EnvFilter::try_from_default_env()
            .unwrap_or_else(|_| EnvFilter::new("info")),
    )
    .init();
```

Log level is controlled by the `RUST_LOG` environment variable (default: `info`).

## Consequences

### Positive

- Standard Rust ecosystem convention; ecosystem crates (reqwest, etc.) emit into
  the same subscriber
- Level filtering without recompilation (`RUST_LOG=debug`, `RUST_LOG=ozi_rs=trace`)
- Structured output with timestamps and module paths in the terminal
- No behavior change required for the in-app console (it reads from `AppState`, not
  from the subscriber)

### Negative

- `tracing-subscriber` adds compile-time dependencies (sharded-slab, thread-local,
  matchers, nu-ansi-term)
- The in-app console and stdout are now two separate outputs that must both be
  updated when a diagnostic is emitted (currently handled in `push_diagnostic`)

## Rejected Alternatives

### log + env_logger

Considered first but replaced by `tracing` which is the current ecosystem standard
for new Rust projects and provides richer span/event semantics for future use.

### Keep println!/eprintln!

Rejected because it cannot be filtered or redirected and does not integrate with
crates that use `log` or `tracing`.
