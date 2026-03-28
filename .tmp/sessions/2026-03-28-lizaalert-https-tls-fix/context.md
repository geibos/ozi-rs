# Task Context: LizaAlert HTTPS connectivity fix

Session ID: 2026-03-28-lizaalert-https-tls-fix
Created: 2026-03-28T09:55:00Z
Status: implemented_validated

## Current Request
Debug why requests to `https://maps.lizaalert.ru/maps/` fail and make sure the project still compiles cleanly after the fix.

## Context Files (Standards to Follow)
- /Users/sobieg/.config/opencode/context/core/standards/code-quality.md
- /Users/sobieg/.config/opencode/context/core/standards/test-coverage.md
- /Users/sobieg/.config/opencode/context/core/workflows/external-libraries.md
- /Users/sobieg/Projects/ozi-rs/AGENTS.md
- /Users/sobieg/Projects/ozi-rs/docs/architecture.md
- /Users/sobieg/Projects/ozi-rs/docs/testing-strategy.md

## Reference Files (Source Material to Look At)
- /Users/sobieg/Projects/ozi-rs/Cargo.toml
- /Users/sobieg/Projects/ozi-rs/src/infrastructure/lizaalert.rs
- /Users/sobieg/Projects/ozi-rs/src/application/mod.rs
- /Users/sobieg/Projects/ozi-rs/src/ui/mod.rs
- /Users/sobieg/Projects/ozi-rs/.tmp/sessions/2026-03-28-diagnostics-console/context.md

## External Docs Fetched
- /Users/sobieg/Projects/ozi-rs/.tmp/external-context/reqwest/default-features-false-https-blocking-tls.md — `reqwest 0.13.x` requires an explicit TLS backend when `default-features = false`; `blocking` alone is insufficient for HTTPS.

## Progress Notes
- Root cause identified in `Cargo.toml`: `reqwest` was configured with `default-features = false` and `features = ["blocking"]`, which leaves HTTPS requests without any TLS backend.
- Applied the minimal version-appropriate fix by enabling `rustls` explicitly: `features = ["blocking", "rustls"]`.
- Validation passed with `cargo fmt --check` and `cargo test --lib`.
- Validation introduced the expected lockfile updates for the new TLS dependency graph.

## Components
- reqwest dependency configuration
- LizaAlert HTTPS fetch path validation

## Constraints
- Keep the fix minimal and version-appropriate for current `reqwest 0.13.2`.
- Prefer deterministic compile/test validation over speculative code churn.
- Do not broaden scope beyond the connectivity blocker unless validation reveals a direct follow-up issue.

## Exit Criteria
- [x] `reqwest` has an explicit HTTPS-capable TLS backend.
- [x] The project compiles cleanly after the dependency fix.
- [x] The session context records the root cause and validation results.
