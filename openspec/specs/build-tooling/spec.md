# build-tooling Specification

## Purpose

Define the `justfile` as the single entry point for developer and agent workflows: building and launching the desktop app (`just build`, `just release`, `just run`, `just run-release`), the pre-merge gate (`just ci` and its parts), the GUI-seizing end-to-end gate (`just smoke`) and, with `revive-ui-cycle`, the browser stand, fixture generation and screenshot matrix (`just stand`, `just fixtures`, `just shots`). The capability fixes which recipes are canonical, which may short-circuit work, and which must never launch a GUI, so that local runs, CI and the native QA MCP server (`tools/ozi-rs-mcp`, whose `build_app` tool shells out to `just build`) all go through the same commands.

### Decision history

- change `fix-dev-release-rebuild` (archived 2026-05-26, implemented): `just run-release` must be a no-op when no input changed while `just build` / `just release` keep going through `npm run tauri build`; rationale: the build-and-launch loop was rebuilding the whole app on every run, and bypassing the Tauri CLI on the bundled path would break parity with the CI smoke build. Codified as: `just run-release` SHALL be a no-op when inputs are unchanged, Canonical bundled build path SHALL remain `npm run tauri build`.
- ADR-0024 (2026-04-28, accepted; narrowed by ADR-0025 in `revive-ui-cycle`) together with the M1 `just smoke` gate (AGENTS.md "E2E gate"): the end-to-end gate for app-touching changes is an Appium Mac2 smoke run from `tools/ozi-rs-mcp/tests/`, invoked as `just smoke`, and it stays out of `just ci` because it seizes the screen; rationale: Playwright cannot exercise Tauri IPC, protocols or window lifecycle, and GUI-seizing runs must be batched, not run per check. Codified as: (revive-ui-cycle) Stand, fixtures and screenshot recipes are available through `just` — `just smoke [<N>]` runs the per-CJ smokes and `just ci` SHALL NOT include Appium smoke.
- `docs/native-qa-mcp.md` §"Registering with an MCP client" and `opencode.json` (2026-05, implemented for opencode only): the MCP server is started through `cargo run` so a stale prebuilt binary can never serve a session; rationale: `.mcp.json` still points at `target/release/ozi-rs-mcp`, and that binary (built 2026-05-26) served sessions for months after the July fixes. Codified as: (revive-ui-cycle, agent-workflow) The native QA MCP server is always built from source.
## Requirements
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

### Requirement: The stand answers in the shapes the bindings declare

The browser stand SHALL type each command's answer against the generated
bindings, so that an answer whose shape the application does not expect fails
the type check rather than the screen. Any command whose answer deliberately
differs SHALL be listed as an exception with its reason.

#### Scenario: A stub drifts from its DTO

- **WHEN** a stand answer is written with fields the command's DTO does not declare
- **THEN** the type check fails, naming the command and the missing fields

#### Scenario: A deliberate difference

- **WHEN** an answer differs from its binding on purpose, as the tile commands do
- **THEN** it is declared as an exception rather than widening every answer's type

### Requirement: The stand draws what an import produced

An import played on the stand SHALL reach every surface the real one reaches,
the map included, so that a decision about how tracks look can be checked
rather than reasoned about. A stand whose import changes one surface and not
another describes a state the application never has.

#### Scenario: A day's folder is imported on the stand

- **WHEN** a folder import is played
- **THEN** the new tracks appear in the list and are drawn on the map, in the colours the list shows

