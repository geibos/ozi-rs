# Issues

## Pre-existing (Known)
- `src/test/capabilities.test.ts` has TypeScript errors: `fs`, `path`, `__dirname` not found — pre-existing, not introduced by us
- `src/lib/stores.ts:6` — `update` declared but never used — pre-existing
- 11 clippy warnings in Rust code — being fixed in Task 1
- 14 `.lock().unwrap()` calls in `commands/mod.rs` — being fixed in Task 2
- `lsp_diagnostics` could not initialize because `rust-analyzer` is unavailable in the stable toolchain on this machine.
- `cargo test --all` initially surfaced a Tauri command handler quirk in `src-tauri/src/commands/mod.rs` around `?`; resolved locally with explicit lock handling and lint allowance so the suite could pass.
