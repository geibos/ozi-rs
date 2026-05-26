## Context

User-reported symptom: `just run-release` always recompiles Rust and re-bundles the frontend, even on back-to-back invocations with no source changes. The expected baseline is near-instant: Cargo's incremental cache, Vite's transform cache, and Tauri's `frontendDist` mtime comparison should make the second invocation a no-op.

## Investigation findings (read-only walkthrough)

1. `justfile:34-36` — `run-release` calls `npm run tauri build -- --no-bundle` and then runs `./target/release/ozi-rs`. The recipe itself adds nothing surprising; everything happens inside `tauri build`.

2. `src-tauri/tauri.conf.json:5-10` declares:
   - `"beforeBuildCommand": "npm run build"` → resolves to `vite build` (per `package.json:8`).
   - `"frontendDist": "../build"` → the directory `vite build` writes to.

   `tauri build` invokes `beforeBuildCommand` unconditionally on every call. There is no "skip if frontendDist is fresh" guard in Tauri 2's CLI. So `vite build` runs every time, even if `src/` is byte-identical.

3. `src/routes/+layout.ts:2` sets `export const prerender = true` and `svelte.config.js` uses `@sveltejs/adapter-static`. Each `vite build` therefore re-runs SvelteKit prerendering and emits a fresh `build/` directory. The output content is deterministic in principle, but:
   - SvelteKit's prerender step rewrites `index.html`, `200.html`, and `_app/` even when the bundle is byte-identical.
   - `tauri build` includes `build/` contents as inputs to the Rust crate via Tauri's asset embedding (`tauri::generate_context!()` reads the dist directory at compile time). If even one timestamp inside `build/` shifts, downstream Cargo cache keys for the embedded asset blob shift, forcing the final crate to relink.

   This is the dominant culprit: `vite build` always touches files in `build/`, which always invalidates the asset-embed input to `src-tauri`, which always forces at least a relink of the top-level Rust crate. The user perceives this as "Rust rebuilt", though strictly speaking only the final crate is recompiled, not the whole dependency graph.

4. `src-tauri/build.rs` is the trivial `tauri_build::build()` — no project-specific `rerun-if-changed` directives. `tauri_build::build()` internally declares a `rerun-if-changed` against `tauri.conf.json` and the icons / capabilities directories, which is correct. So `build.rs` is NOT the primary cause; it is fine as-is, but if option A is taken it becomes an even smaller surface.

5. Cargo / Rust workspace cache (`target/release/`): with a clean Vite output, `cargo build --release --manifest-path src-tauri/Cargo.toml` on an unchanged tree is genuinely a sub-second no-op. We can confirm this independently of `tauri build` by running `cargo build --release` directly twice — the second run reports "Finished".

6. No pre-commit hooks, no `cargo-deny` in the build path, no `.husky/` directory. CI runs the same `tauri build` but from a fresh checkout where rebuild-everything is correct behavior anyway, so CI is not relevant to this discussion.

7. `vite.config.ts:24` sets `minify: !process.env.TAURI_ENV_DEBUG ? "esbuild" : false` — esbuild minification is fast (sub-second), not a cost driver. `manualChunks` only re-runs when bundle inputs change, so it does not introduce nondeterminism.

**Conclusion**: the primary cause is that `tauri build` unconditionally re-runs `npm run build` (= `vite build`), which always re-emits `build/`, which always invalidates the Tauri asset-embed input to the final Rust crate, which always relinks `ozi-rs`. Secondary cost is `vite build` itself (~5-15s cold, ~2-5s warm), which is not free even when its output would have been identical.

## Decisions

### Decision 1: Take option A — split the recipe — as the primary remediation

Replace the body of `run-release` with two explicit steps so each cache layer is consulted on its own terms:

```just
run-release:
    npm run build               # vite build only; SvelteKit prerender writes to build/
    cargo build --release --manifest-path src-tauri/Cargo.toml
    ./target/release/ozi-rs
```

Then introduce a small guard so the `npm run build` step is skipped when no frontend input has changed:

```just
run-release:
    @just _build-frontend-if-stale
    cargo build --release --manifest-path src-tauri/Cargo.toml
    ./target/release/ozi-rs

_build-frontend-if-stale:
    @if [ ! -d build ] || [ -n "$(find src vite.config.ts svelte.config.js package.json tailwind.config.js postcss.config.cjs -newer build/index.html 2>/dev/null | head -1)" ]; then \
        npm run build; \
    else \
        echo "frontend cache hit (build/ is up to date)"; \
    fi
```

The exact stale-detection script is illustrative; the implementer SHALL choose between (a) a `find -newer` mtime check rooted at `build/index.html`, (b) a content-hash file (`build/.input-hash`) compared on each invocation, or (c) deferring to a tool like `mtime-cache`. The contract that matters is "no work when inputs unchanged", which the spec captures.

**Trade-off accepted**: this bypasses `tauri build`'s `beforeBuildCommand` machinery, so we lose any future enhancements Tauri makes to that step (e.g. dist verification). Mitigated by keeping `just release` (full bundle) and `just build` calling `npm run tauri build` directly, so the canonical Tauri path is preserved for shipping artifacts. The split recipe is a local-developer-only optimization.

### Decision 2: Do NOT modify `src-tauri/build.rs`

`tauri_build::build()` already declares the right `rerun-if-changed` directives for `tauri.conf.json`, icons, and capability files (see `tauri-build` crate source). Adding our own narrows nothing. Option B from the proposal is therefore declined.

### Decision 3: Do NOT add a binary-mtime-vs-sources short-circuit at the top of `run-release` (option C declined)

A guard like "if `target/release/ozi-rs` is newer than all source files, just run it" is appealing but fragile:
- It needs to know about every input (Cargo lockfile, `tauri.conf.json`, icons, capability files, `src-tauri/src/`, `src/`, `package.json`, `package-lock.json`, ...).
- Drift between this list and the real Cargo / Vite input graph would silently launch a stale binary.

The split-recipe approach (Decision 1) delegates freshness checks to the tools that actually own the cache (Cargo, and our small Vite-output mtime guard), which is more conservative.

### Decision 4: Keep `just release` and `just build` calling `npm run tauri build`

The canonical path that matches CI's smoke build is `npm run tauri build`. Bundling (dmg / msi) and signing live there. We only optimize the `run-release` recipe — the one designed for "build it and launch it for QA right now" — not the shipping path.

### Decision 5: Capability scoping — introduce `build-tooling`, do not extend `ci-pipeline`

`ci-pipeline` is explicitly scoped to GitHub Actions gates in `openspec/specs/ci-pipeline/spec.md` ("the GitHub Actions workflows (`ci.yml`, `release.yml`)"). Stretching it to cover local `just` recipe semantics blurs that boundary and would force every local-tooling tweak to relitigate CI invariants. A new sibling capability `build-tooling` keeps the two concerns separable: `ci-pipeline` answers "what gates does a PR pass?", `build-tooling` answers "what does each `just` recipe contract to do locally?".

The initial `build-tooling` spec covers only `just run-release` (the file that motivated this proposal). Future requirements for `just run`, `just dev`, `just build`, and `just release` can be added incrementally.

## Risks / open questions

- **Risk: mtime-based staleness check is fooled by `git checkout` (which sets mtimes to checkout time, not commit time)**. After a branch switch, `find -newer` may say nothing is newer than `build/index.html` even though contents changed. Mitigation: if implementer chooses option (a), include `package-lock.json` and `svelte.config.js` in the watched set and document the limitation; if (b), use a content-hash file which is immune to mtime games. Recommend (b).
- **Risk: prerender output is non-deterministic across SvelteKit minor versions**. Not a concern for cache-hit (we only short-circuit when inputs are byte-identical), but worth a note that the cache is invalidated on any package upgrade.
- **Open: should `just build` (debug bundle) get the same treatment?** Out of scope here. If the user wants the same speed-up for the debug bundle loop, a follow-up change can extend the `build-tooling` capability.
- **Open: does the user want a `just run-release-fast` recipe alongside the existing `just run-release`?** This proposal replaces the existing recipe in place. If the user wants a tactile "I know what I'm doing" path that bypasses freshness checks entirely (for debugging the build itself), they can still call `npm run tauri build -- --no-bundle && ./target/release/ozi-rs` manually.
