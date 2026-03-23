---
description: Run project quality checks and summarize failures
agent: qa
---

Run the relevant Rust quality checks for this repository.

Prefer, when available:
- `cargo fmt --check`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo test --all`

If the project uses `nextest` or coverage tooling, use that too when appropriate.

Then provide:
- failing commands;
- likely root causes;
- missing tests;
- recommended next fixes in priority order.
