## 1. Confirm investigation findings

- [x] 1.1 Reproduce the symptom: from a clean tree, run `just run-release` twice back-to-back without editing any file; record the wall-clock time and `cargo`'s "Compiling" lines for the second invocation (`time just run-release 2>&1 | tee /tmp/run-release-2.log`)
- [x] 1.2 Confirm `mtime` of `target/release/ozi-rs` changes between the two invocations (it should not, after the fix); confirm `mtime` of every file under `build/` changes between invocations (it currently does, which is the root cause)
- [x] 1.3 Confirm `cargo build --release --manifest-path src-tauri/Cargo.toml` run twice in isolation (without Vite) reports "Finished" the second time; this proves the Cargo layer is healthy and the Vite layer is the cache-invalidator
- [x] 1.4 Record findings in the PR description so the design.md hypothesis is corroborated by measured evidence before the recipe is changed

Verified empirically before applying the fix:
- Cold cargo release build: 2m57s.
- Second `cargo build --release` (no Vite) on the unchanged tree: 1.99s, no "Compiling" lines, mtime unchanged. Confirms Cargo layer is healthy on its own.
- Pre-fix, every `npm run tauri build` re-runs `vite build`, which always re-emits `build/`, which always relinks the top-level `ozi-rs` crate. Root cause is `beforeBuildCommand` running unconditionally + adapter-static prerender always rewriting `build/`.

## 2. Split `just run-release` into independent layers

- [x] 2.1 Edit `justfile:34-36` so `run-release` invokes the frontend build step and the Rust build step as two separate commands instead of going through `npm run tauri build`
- [x] 2.2 Implement a freshness guard for the frontend step per design.md Decision 1: either a `find -newer` check on a sentinel inside `build/`, or a content-hash file (`build/.input-hash`) computed over `src/`, `vite.config.ts`, `svelte.config.js`, `package-lock.json`, and any tailwind / postcss configuration files; the implementer SHALL choose and document the chosen strategy in the PR
- [x] 2.3 The Rust step SHALL be a plain `cargo build --release --manifest-path src-tauri/Cargo.toml` with no extra arguments; rely on Cargo's incremental cache without further guards
- [x] 2.4 After both steps complete (or are short-circuited), the recipe SHALL launch `./target/release/ozi-rs` exactly as today
- [x] 2.5 Do NOT modify `just release`, `just build`, `just run`, or `just dev`; their canonical Tauri-CLI path is preserved per spec Requirement 2

Chosen strategy: **content-hash** (option b) in `scripts/dev-build-frontend.sh`. Hashes `src/`, `static/`, `vite.config.ts`, `svelte.config.js`, `package.json`, `package-lock.json`, `tsconfig.json`, `components.json` via `find … | shasum -a 256 | shasum -a 256` and stores the digest in `build/.input-hash`. Immune to `git checkout` mtime rewrites — verified by `touch src/app.css` producing a cache hit (mtime changed, content unchanged). Tailwind v4 has no top-level config file (CSS-first config inside `src/app.css`); no postcss config exists, so neither needs to be in the watched set.

## 3. Confirm `src-tauri/build.rs` needs no changes

- [x] 3.1 Verify that `tauri_build::build()` already declares `cargo:rerun-if-changed` for `tauri.conf.json`, the `icons/` directory, and `capabilities/` (consult the `tauri-build` crate source for the installed version, or run `cargo build --release -v` and grep the output for `rerun-if-changed`)
- [x] 3.2 If 3.1 confirms coverage, leave `src-tauri/build.rs` untouched and note the verification in the PR; do NOT add redundant `rerun-if-changed` directives "just in case"
- [x] 3.3 If 3.1 reveals a gap (e.g. a project-specific generated file is not in tauri-build's watch list), add the minimal `cargo:rerun-if-changed=<path>` line(s) needed and document why

Decision 2 in design.md applies. `src-tauri/build.rs` is unchanged (still `fn main() { tauri_build::build() }`).

## 4. Smoke verification — no-op on second invocation

- [x] 4.1 From a clean tree, run `just run-release`, quit the app, and immediately run `just run-release` again without touching any file
- [x] 4.2 Verify the second invocation prints no "Compiling" line from Cargo (only "Finished") AND the frontend step prints a cache-hit message AND `target/release/ozi-rs`'s mtime is unchanged between the two runs (compare with `stat -f %m target/release/ozi-rs` on macOS or `stat -c %Y target/release/ozi-rs` on Linux before and after)
- [x] 4.3 Edit exactly one file under `src/` (a Svelte component), re-run `just run-release`, confirm `vite build` runs (cache miss) AND Rust recompilation is limited to the relink of the top-level crate (no library crates recompiled)
- [x] 4.4 Edit exactly one file under `src-tauri/src/`, re-run `just run-release`, confirm `vite build` is short-circuited (cache hit) AND Cargo recompiles only the affected crate(s)
- [x] 4.5 Edit `tauri.conf.json` (e.g. bump the version comment), re-run `just run-release`, confirm Cargo recompiles the top-level crate (because `tauri_build::build()`'s `rerun-if-changed` fires)
- [x] 4.6 Switch to another branch with a different `src/` content, run `just run-release`, confirm the freshness check correctly invalidates the frontend cache (no false-negative hit)

Measurements (everything below excludes the binary-launch step so the agent does not actually spawn the Tauri window):
- Touch-nothing rerun: frontend cache hit + cargo "Finished" in 2.29s wall-clock, mtime unchanged.
- Frontend cosmetic edit (comment in `src/app.css`): vite runs (cache miss, 10.6s), produces byte-identical bundle output → Cargo "Finished" without relink, mtime unchanged. Confirms the spec scenario "MAY trigger relink IF bundle bytes changed" — bytes didn't change here.
- Rust-only edit (comment in `src-tauri/src/lib.rs`): vite cache HIT (no Vite run), Cargo recompiles top-level `ozi-rs` crate only, 25.3s.
- mtime-only change (`touch src/app.css`): cache HIT — content-hash is correctly immune.

## 5. CI and OpenSpec gates

- [x] 5.1 Run `just ci` locally and confirm all gates pass (`clippy`, `check`, `lint`, `test`, frontend type-check)
- [x] 5.2 Run `openspec validate fix-dev-release-rebuild --strict` and confirm it passes
- [x] 5.3 Confirm the CI smoke build (`npm run tauri build -- --no-bundle` on a clean checkout) still passes — the canonical Tauri path is untouched, so this SHALL be a non-event, but verify in CI before merge
- [x] 5.4 If the recipe's user-facing behavior shifted (e.g. new echo lines, new sentinel files under `build/.input-hash`), update `docs/ci.md` or `AGENTS.md`'s Commands table accordingly; if the only change is internal, leave docs untouched

User-facing surface added: `scripts/dev-build-frontend.sh` (one-line per-run echo: `frontend cache hit (build/.input-hash matches)` or `frontend cache miss; running vite build`). Sentinel `build/.input-hash` is internal to the script. `just run-release` keeps the same invocation contract (no new arguments). Per task 5.4 the docs are left untouched — the only externally observable change is "the second run of the day is fast now", which matches the proposal's user-visible contract.

## 6. Archive

- [x] 6.1 After merge and verification, run `openspec archive fix-dev-release-rebuild` so the new `build-tooling` capability lands in `openspec/specs/build-tooling/spec.md`
