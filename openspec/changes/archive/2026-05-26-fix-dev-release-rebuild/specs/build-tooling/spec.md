## ADDED Requirements

### Requirement: `just run-release` SHALL be a no-op when inputs are unchanged

A second invocation of `just run-release` immediately following a first successful invocation, with no modification to any tracked Rust source under `src-tauri/src/`, no modification to any tracked frontend source under `src/`, no modification to `Cargo.toml` / `Cargo.lock` / `package.json` / `package-lock.json` / `tauri.conf.json` / `svelte.config.js` / `vite.config.ts` / `tailwind` configuration files, and no change to capability or icon files referenced by `tauri.conf.json`, SHALL produce no observable build work. Specifically:

- No Rust compilation unit SHALL be recompiled (Cargo SHALL report "Finished" without emitting any "Compiling" line for project crates).
- No frontend re-bundle SHALL be performed (the `vite build` step SHALL be short-circuited, or SHALL itself report a no-op).
- The release binary at `target/release/ozi-rs` SHALL NOT be relinked (its mtime SHALL be unchanged between the two invocations).
- The wall-clock time of the second invocation SHALL be dominated by process startup, not by build work.

The recipe SHALL still launch `./target/release/ozi-rs` after the (potentially short-circuited) build, so the user observes "app starts" as the outcome in both the cache-hit and cache-miss case.

#### Scenario: Back-to-back invocations with no source changes

- **GIVEN** the user has just run `just run-release` to completion and quit the resulting binary
- **WHEN** the user runs `just run-release` again without modifying any source file
- **THEN** the recipe SHALL NOT recompile any Rust crate, SHALL NOT re-run `vite build` (or SHALL run a `vite build` that produces byte-identical output and is itself a measured no-op via tool-level caching), SHALL NOT relink `target/release/ozi-rs`, AND SHALL launch the existing binary directly

#### Scenario: Frontend source touched, Rust untouched

- **GIVEN** the user has a successful previous build and edits exactly one file under `src/`
- **WHEN** the user runs `just run-release`
- **THEN** the recipe SHALL re-run `vite build` (cache miss for the frontend layer) AND MAY trigger a relink of the top-level Rust crate if the embedded frontend asset bytes changed, AND SHALL NOT recompile project crates beyond that relink

#### Scenario: Rust source touched, frontend untouched

- **GIVEN** the user has a successful previous build and edits exactly one file under `src-tauri/src/`
- **WHEN** the user runs `just run-release`
- **THEN** the recipe SHALL recompile only the affected Rust crate(s) (Cargo incremental cache behavior) AND SHALL NOT re-run `vite build`

#### Scenario: `tauri.conf.json` touched

- **GIVEN** the user has a successful previous build and edits `tauri.conf.json`
- **WHEN** the user runs `just run-release`
- **THEN** `tauri_build::build()` SHALL pick up the change via its `rerun-if-changed` directive AND the top-level Rust crate SHALL be recompiled; frontend re-bundle SHALL be triggered only if `tauri.conf.json` changes affect frontend inputs

#### Scenario: Branch switch invalidates caches conservatively

- **GIVEN** the user has a successful previous build on branch A
- **WHEN** the user switches to branch B (with different source content) and runs `just run-release`
- **THEN** the freshness check SHALL detect content change (whether via mtime, content-hash, or Cargo's own fingerprint) AND SHALL trigger a full rebuild as needed; the implementation SHALL prefer false-positive cache misses over false-negative cache hits

### Requirement: Canonical bundled build path SHALL remain `npm run tauri build`

`just release` and `just build` SHALL continue to invoke `npm run tauri build` (with `--debug` for `just build`, no flag for `just release`). These recipes SHALL NOT be modified to bypass the Tauri CLI. This preserves parity between local bundle artifacts and the CI smoke build (`ci-pipeline`), and keeps codesigning / bundle hooks on their canonical path.

The optimization MAY be applied only to `just run-release` (and, at the implementer's discretion in a follow-up change, to `just run`), because those recipes are designed for the "build-and-launch-now" loop where bundle hooks are explicitly not needed.

#### Scenario: `just release` continues to invoke `tauri build`

- **WHEN** the user runs `just release`
- **THEN** the recipe SHALL invoke `npm run tauri build` (which runs `beforeBuildCommand` and produces a full bundle) AND SHALL NOT short-circuit any step

#### Scenario: `just build` continues to invoke `tauri build --debug`

- **WHEN** the user runs `just build`
- **THEN** the recipe SHALL invoke `npm run tauri build -- --debug` AND SHALL NOT short-circuit any step
