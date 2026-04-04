# Issues

## Pre-existing (Known)
- `src/test/capabilities.test.ts` has TypeScript errors: `fs`, `path`, `__dirname` not found — pre-existing, not introduced by us
- `src/lib/stores.ts:6` — `update` declared but never used — pre-existing
- 11 clippy warnings in Rust code — being fixed in Task 1
- 14 `.lock().unwrap()` calls in `commands/mod.rs` — being fixed in Task 2
- `lsp_diagnostics` could not initialize because `rust-analyzer` is unavailable in the stable toolchain on this machine.
- `cargo test --all` initially surfaced a Tauri command handler quirk in `src-tauri/src/commands/mod.rs` around `?`; resolved locally with explicit lock handling and lint allowance so the suite could pass.

## [2026-04-04] History rewrite cleanup
- `git push --force-with-lease origin HEAD:main` initially failed with `(stale info)` because `origin/main` tracking ref was missing after `filter-repo`; resolved by `git fetch origin main` then retrying force-with-lease.

## [2026-04-04] Wave 2 command additions
- `clippy -D warnings` flags `ProjectCommand::move_track_point(...)` as `too_many_arguments`; kept required constructor shape and added a targeted `#[allow(clippy::too_many_arguments)]` on that constructor only.

## [2026-04-04] Task 35 follow-up dead_code fix
- `cargo clippy --all-targets --all-features -- -D warnings` reported dead code for PLT export because `infrastructure` is an internal module (`mod infrastructure;`), so `pub` items there are still considered unused unless re-exported from crate root.
- Removing the temporary wrapper in `infrastructure/export/mod.rs` was correct, but dead_code persisted until PLT export symbols were re-exported from `src-tauri/src/lib.rs`.
- `TrackPoint::timestamp()` returns `Option<chrono::DateTime<chrono::Utc>>` in current domain model, so `datetime_to_ole_date` should keep `DateTime<Utc>` input (no NaiveDateTime conversion needed).
