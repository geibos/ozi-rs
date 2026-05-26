## 1. Confirm investigation findings

- [ ] 1.1 Reproduce the symptom: from a clean tree, run `just run-release` twice back-to-back without editing any file; record the wall-clock time and `cargo`'s "Compiling" lines for the second invocation (`time just run-release 2>&1 | tee /tmp/run-release-2.log`)
- [ ] 1.2 Confirm `mtime` of `target/release/ozi-rs` changes between the two invocations (it should not, after the fix); confirm `mtime` of every file under `build/` changes between invocations (it currently does, which is the root cause)
- [ ] 1.3 Confirm `cargo build --release --manifest-path src-tauri/Cargo.toml` run twice in isolation (without Vite) reports "Finished" the second time; this proves the Cargo layer is healthy and the Vite layer is the cache-invalidator
- [ ] 1.4 Record findings in the PR description so the design.md hypothesis is corroborated by measured evidence before the recipe is changed

## 2. Split `just run-release` into independent layers

- [ ] 2.1 Edit `justfile:34-36` so `run-release` invokes the frontend build step and the Rust build step as two separate commands instead of going through `npm run tauri build`
- [ ] 2.2 Implement a freshness guard for the frontend step per design.md Decision 1: either a `find -newer` check on a sentinel inside `build/`, or a content-hash file (`build/.input-hash`) computed over `src/`, `vite.config.ts`, `svelte.config.js`, `package-lock.json`, and any tailwind / postcss configuration files; the implementer SHALL choose and document the chosen strategy in the PR
- [ ] 2.3 The Rust step SHALL be a plain `cargo build --release --manifest-path src-tauri/Cargo.toml` with no extra arguments; rely on Cargo's incremental cache without further guards
- [ ] 2.4 After both steps complete (or are short-circuited), the recipe SHALL launch `./target/release/ozi-rs` exactly as today
- [ ] 2.5 Do NOT modify `just release`, `just build`, `just run`, or `just dev`; their canonical Tauri-CLI path is preserved per spec Requirement 2

## 3. Confirm `src-tauri/build.rs` needs no changes

- [ ] 3.1 Verify that `tauri_build::build()` already declares `cargo:rerun-if-changed` for `tauri.conf.json`, the `icons/` directory, and `capabilities/` (consult the `tauri-build` crate source for the installed version, or run `cargo build --release -v` and grep the output for `rerun-if-changed`)
- [ ] 3.2 If 3.1 confirms coverage, leave `src-tauri/build.rs` untouched and note the verification in the PR; do NOT add redundant `rerun-if-changed` directives "just in case"
- [ ] 3.3 If 3.1 reveals a gap (e.g. a project-specific generated file is not in tauri-build's watch list), add the minimal `cargo:rerun-if-changed=<path>` line(s) needed and document why

## 4. Smoke verification — no-op on second invocation

- [ ] 4.1 From a clean tree, run `just run-release`, quit the app, and immediately run `just run-release` again without touching any file
- [ ] 4.2 Verify the second invocation prints no "Compiling" line from Cargo (only "Finished") AND the frontend step prints a cache-hit message AND `target/release/ozi-rs`'s mtime is unchanged between the two runs (compare with `stat -f %m target/release/ozi-rs` on macOS or `stat -c %Y target/release/ozi-rs` on Linux before and after)
- [ ] 4.3 Edit exactly one file under `src/` (a Svelte component), re-run `just run-release`, confirm `vite build` runs (cache miss) AND Rust recompilation is limited to the relink of the top-level crate (no library crates recompiled)
- [ ] 4.4 Edit exactly one file under `src-tauri/src/`, re-run `just run-release`, confirm `vite build` is short-circuited (cache hit) AND Cargo recompiles only the affected crate(s)
- [ ] 4.5 Edit `tauri.conf.json` (e.g. bump the version comment), re-run `just run-release`, confirm Cargo recompiles the top-level crate (because `tauri_build::build()`'s `rerun-if-changed` fires)
- [ ] 4.6 Switch to another branch with a different `src/` content, run `just run-release`, confirm the freshness check correctly invalidates the frontend cache (no false-negative hit)

## 5. CI and OpenSpec gates

- [ ] 5.1 Run `just ci` locally and confirm all gates pass (`clippy`, `check`, `lint`, `test`, frontend type-check)
- [ ] 5.2 Run `openspec validate fix-dev-release-rebuild --strict` and confirm it passes
- [ ] 5.3 Confirm the CI smoke build (`npm run tauri build -- --no-bundle` on a clean checkout) still passes — the canonical Tauri path is untouched, so this SHALL be a non-event, but verify in CI before merge
- [ ] 5.4 If the recipe's user-facing behavior shifted (e.g. new echo lines, new sentinel files under `build/.input-hash`), update `docs/ci.md` or `AGENTS.md`'s Commands table accordingly; if the only change is internal, leave docs untouched

## 6. Archive

- [ ] 6.1 After merge and verification, run `openspec archive fix-dev-release-rebuild` so the new `build-tooling` capability lands in `openspec/specs/build-tooling/spec.md`
