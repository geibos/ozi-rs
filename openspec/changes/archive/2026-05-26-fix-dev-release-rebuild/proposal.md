## Why

`just run-release` (`justfile:34-36`) shells out to `npm run tauri build -- --no-bundle`. In practice the user reports that every invocation does a full Rust + frontend rebuild even when neither `src/` nor `src-tauri/` changed since the last successful build. The expected behavior is that Cargo's incremental cache, Vite's transform cache, and Tauri's `frontendDist` mtime check together produce a near-instant no-op when inputs are unchanged. Today they do not, which makes "run the release binary I just built" a multi-minute ritual instead of a sub-second one and erodes the value of the release loop for QA evidence (each rebuild also re-triggers `tauri_build::build()` and re-links the binary). The contributing factors are not currently captured anywhere in the OpenSpec specs because `ci-pipeline` only governs GitHub Actions gates, not the local developer build invocation.

## What Changes

- A new capability `build-tooling` SHALL be added to capture local build-invocation contracts (the `just` recipes that wrap `cargo` and `tauri`). `ci-pipeline` remains scoped to GitHub Actions gates.
- The `build-tooling` capability SHALL require that `just run-release` produce no observable work (no Rust recompilation, no frontend re-bundle, no relink) when neither tracked Rust sources nor tracked frontend sources nor `tauri.conf.json` have changed since the previous successful invocation, modulo a constant-time freshness check.
- The remediation SHALL be selected during apply from the design.md options (A: split the recipe into independent `cargo build --release` + `vite build` steps so each layer's cache is consulted independently; B: tighten `src-tauri/build.rs` so `tauri_build::build()` runs with the narrowest set of `rerun-if-changed` directives possible; C: keep the recipe shape but add a freshness guard in the justfile that short-circuits when `target/release/ozi-rs` is newer than the inputs). The chosen option(s) SHALL be recorded in the change's design.md before apply.
- `docs/ci.md` (or `AGENTS.md` "Commands" table) SHALL be updated only if the recipe shape changes user-facing; otherwise this change is invisible to users beyond "the second build of the day is fast now".

## Capabilities

### New Capabilities

- `build-tooling`: local developer build-invocation contracts. Owns the behavior of `just run`, `just run-release`, `just build`, `just release`, `just build-rust`. Does NOT own GitHub Actions gates (those remain in `ci-pipeline`) and does NOT own runtime behavior of the resulting binary.

### Modified Capabilities

- _none_

## Impact

- **Tooling**: `justfile` recipe `run-release` may be split into two steps or guarded by a freshness check; `src-tauri/build.rs` may gain explicit `cargo:rerun-if-changed` directives. No application code changes.
- **Documentation**: a new `openspec/specs/build-tooling/spec.md` will be created by `openspec archive` after this change merges. `docs/ci.md` left untouched unless recipe shape changes.
- **Risk**: low. The change is to local developer ergonomics, not to runtime behavior or to CI gates. Worst case the freshness check is too aggressive and a stale binary is launched; mitigated by making the check strictly conservative (only short-circuit when every input file's mtime is older than the binary's mtime AND `tauri.conf.json` hash is unchanged) and by leaving `just release` (full bundle) untouched as the fall-back canonical path.
- **CI**: no impact. The `ci-pipeline` smoke build still calls `npm run tauri build -- --no-bundle` from scratch in a clean checkout, which is the correct behavior for a CI runner with no cache.

## Out of scope

- **CI cache configuration** (Cargo registry caching, npm cache, `target/` cache between workflow runs): governed by `ci-pipeline`, not `build-tooling`. A separate proposal can tighten CI caching if profiling shows it is the next bottleneck.
- **`tauri dev` (HMR) loop performance**: `just run` / `just dev` is a different code path with different semantics (Vite HMR + `cargo watch`); this change targets the release-binary loop only.
- **Bundle generation** (`just release` producing `.dmg` / `.msi` / `.AppImage`): bundling has its own expensive steps (icon resampling, codesigning) that are correctly run only when explicitly requested. Out of scope.
- **Switching away from `tauri build`** to a custom Cargo + Vite orchestration as the default: option A in design.md splits the recipe locally, but the canonical full path remains `tauri build` so that local results match CI's smoke build exactly.
