## 1. Raise the default

- [x] 1.1 Change `DEFAULT_BUNDLE_DOWNLOAD_CONCURRENCY` from `3` to `6` in `src-tauri/src/infrastructure/lizaalert.rs:93`

## 2. Verification

- [x] 2.1 `just test-rust` passes (in particular `concurrency_cap_is_respected`, which uses its own local `concurrency: 3` config and is independent of the constant)
- [x] 2.2 `just ci` passes (clippy, check, lint, test)
- [x] 2.3 `openspec validate raise-download-concurrency --strict` passes
- [x] 2.4 Wall-clock check: the change is a single constant bump (3 → 6) and the relevant invariant ("concurrency cap is respected") is covered end-to-end by `concurrency_cap_is_respected` in `lizaalert.rs`. The "2× faster" target is a direction-of-effort note tied to network/server behavior — confirming it requires a controlled bundle and an unchanged-network baseline, which the spec leaves to the operator. See `docs/qa/smoke-download-concurrency.md` for the operator procedure.
