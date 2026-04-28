# Eliminate `just run-release` Warnings

## TL;DR
> **Summary**: Make `just run-release` warning-clean for both the release build and bounded startup output, using evidence-first classification and root-cause fixes rather than blanket suppression.
> **Deliverables**:
> - Captured warning inventory for the current `just run-release` output.
> - Root-cause fixes for every confirmed warning-like line in build and startup phases.
> - Local warning-hygiene verification command or documented local gate for future checks.
> - Final evidence showing clean bounded `just run-release` output plus passing targeted tests.
> **Effort**: Medium
> **Parallel**: YES - 3 waves
> **Critical Path**: Task 1 → Tasks 2-5 → Task 6 → Final Verification Wave

## Context

### Original Request
User: "При запуске just run-release сыпется много варнингов. Я хочу от них избавиться"

### Interview Summary
- Success target: stricter local warning hygiene, not only cosmetic cleanup for one release command.
- Exact warning list: executor captures `just run-release` output as the first implementation task.
- Warning policy: fix root causes; do not hide unknown warnings. Config changes are allowed only when they address an understood tool/root cause and preserve future visibility.
- Test strategy: tests-after. Classify warnings first, then fix and verify with targeted checks plus final release run.
- Runtime scope: build warnings and warning/error-like output emitted after launching `./src-tauri/target/release/ozi-rs` are in scope.

### Metis Review (gaps addressed)
- Added a concrete warning definition: any line matching `warning`, `WARN`, `deprecated`, `security`, `CSP`, `bundle`, `error`, compiler warning formats, or Tauri/Vite advisory warnings during build or bounded startup must be classified.
- Added bounded startup capture so `just run-release` does not hang indefinitely after launching the GUI app.
- Added guardrails against blanket suppression, broad refactors, CI creep, and direct registry dependency patching.
- Added dependency-warning classification: app crate vs local path dependency vs registry dependency.
- Added platform scope: current macOS environment only unless user later requests cross-platform validation.

## Work Objectives

### Core Objective
Eliminate all unclassified warning-like output from `just run-release` on the current macOS development environment while preserving existing app behavior.

### Deliverables
- `.sisyphus/evidence/task-1-warning-inventory.md` with raw captured output, phase boundaries, and classification rows for every warning-like line.
- Targeted fixes for every confirmed warning class.
- A local warning-hygiene check that can be run by agents without manual inspection.
- Final evidence artifacts proving clean release build/startup output and passing tests.

### Definition of Done (verifiable conditions with commands)
- `just run-release` is captured with bounded runtime startup and exits/terminates cleanly under agent control.
- Captured final output contains no unclassified warning-like lines matching: `warning|warn|deprecated|security|csp|bundle|error` case-insensitively, excluding documented exact allowlisted non-warning lines if any remain.
- `npm run build` exits 0 and emits no unclassified warning-like lines.
- `just clippy` exits 0.
- `just test` exits 0.
- Any added local warning gate exits non-zero when fed a synthetic warning-like line and exits 0 on clean captured output.

### Must Have
- Evidence-first workflow: no fixes before warning capture and inventory.
- Every captured warning-like line has exact text, emitting phase/tool, root cause, fix decision, and verification command.
- Root-cause fixes preferred over suppression.
- Runtime startup output in release mode is quiet unless an issue is actionable.
- Development diagnostics remain available for dev/debug modes where practical.

### Must NOT Have (guardrails, AI slop patterns, scope boundaries)
- MUST NOT broadly refactor unrelated Rust, Svelte, Tauri, or build config code.
- MUST NOT globally disable warnings, raise thresholds without measured justification, mute all logging indiscriminately, or add blanket `allow`/suppression attributes.
- MUST NOT patch registry dependencies directly.
- MUST NOT add CI workflows; user asked for local warning hygiene.
- MUST NOT change product behavior, UI workflows, map rendering, persistence, IPC contracts, or domain models unless a captured warning's root cause strictly requires it.
- MUST NOT claim likely warnings exist until Task 1 captures them.

## Verification Strategy
> ZERO HUMAN INTERVENTION - all verification is agent-executed.
- Test decision: Tests-after + existing Rust/Frontend framework (`just clippy`, `just test`, `npm run build`, `just run-release`).
- QA policy: Every task has agent-executed scenarios.
- Evidence: `.sisyphus/evidence/task-{N}-{slug}.{ext}`.
- Warning matching policy: case-insensitive scan for `warning|warn|deprecated|security|csp|bundle|error`, followed by inventory classification so real failures are not hidden and benign non-warning lines are documented by exact text only.
- Bounded app launch policy: run release startup long enough to capture initial logs, then terminate the app process cleanly. Suggested bound: 15 seconds after process start unless the app exits earlier.

## Execution Strategy

### Parallel Execution Waves
> Target: 5-8 tasks per wave. <3 per wave (except final) = under-splitting.
> Extract shared dependencies as Wave-1 tasks for max parallelism.

Wave 1: Task 1 evidence capture and inventory foundation.
Wave 2: Tasks 2-5 warning-class fixes can run in parallel after Task 1 assigns confirmed warnings by phase.
Wave 3: Task 6 local warning gate and full release verification after all fixes land.

### Dependency Matrix (full, all tasks)
| Task | Depends On | Blocks |
|---|---|---|
| 1. Capture and classify release warnings | None | 2, 3, 4, 5, 6 |
| 2. Fix confirmed frontend build warnings | 1 | 6 |
| 3. Fix confirmed Tauri build/config warnings | 1 | 6 |
| 4. Fix confirmed Rust compiler/dependency warnings | 1 | 6 |
| 5. Fix confirmed release startup warning/log output | 1 | 6 |
| 6. Add local warning hygiene gate and final release evidence | 2, 3, 4, 5 | Final Verification Wave |

### Agent Dispatch Summary (wave → task count → categories)
| Wave | Task Count | Categories |
|---|---:|---|
| 1 | 1 | deep |
| 2 | 4 | visual-engineering, quick, deep, unspecified-high |
| 3 | 1 | unspecified-high |

## TODOs
> Implementation + Test = ONE task. Never separate.
> EVERY task MUST have: Agent Profile + Parallelization + QA Scenarios.

- [x] 1. Capture and classify `just run-release` warning output

  **What to do**:
  1. Check git status before running commands to identify unrelated dirty files.
  2. Run `just run-release` under a bounded capture that records stdout/stderr from both phases:
     - build phase: `npm run tauri build -- --no-bundle` via the recipe;
     - startup phase: `./src-tauri/target/release/ozi-rs` launched by the recipe.
  3. If the GUI process remains running after startup, terminate it cleanly after 15 seconds and record that termination method in evidence.
  4. Save raw output to `.sisyphus/evidence/task-1-run-release-raw.log`.
  5. Create `.sisyphus/evidence/task-1-warning-inventory.md` with one row per warning-like line: exact text, phase, emitting tool/module, classification, suspected root cause, fix owner task, verification command.
  6. Classify warning sources only after capture. Expected buckets are frontend/Vite/Svelte, Tauri config/platform, Rust app crate, Rust local path dependency, Rust registry dependency, and release startup runtime output.

  **Must NOT do**:
  - Do not fix any warning in this task.
  - Do not delete or edit source/build config files.
  - Do not assume likely warnings from planning research are present unless captured.

  **Recommended Agent Profile**:
  - Category: `deep` - Reason: needs careful root-cause classification across build, runtime, Rust, and frontend phases.
  - Skills: [`systematic-debugging`] - required for evidence-first investigation.
  - Omitted: [`test-driven-development`] - user chose tests-after, not TDD-style.

  **Parallelization**: Can Parallel: NO | Wave 1 | Blocks: 2, 3, 4, 5, 6 | Blocked By: none

  **References** (executor has NO interview context - be exhaustive):
  - Command: `justfile:34-36` - `run-release` runs Tauri release build without bundling, then launches the release binary.
  - Script: `package.json:10` - `tauri` script resolves to the Tauri CLI.
  - Build hook: `src-tauri/tauri.conf.json:5-10` - Tauri runs `npm run build` before building.
  - Frontend build: `package.json:8` - `build` maps to `vite build`.
  - Runtime logging: `src-tauri/src/lib.rs:11-16` - tracing subscriber initialization can affect startup log output.

  **Acceptance Criteria** (agent-executable only):
  - [ ] `.sisyphus/evidence/task-1-run-release-raw.log` exists and contains captured stdout/stderr from the run.
  - [ ] `.sisyphus/evidence/task-1-warning-inventory.md` exists and every warning-like line matching `warning|warn|deprecated|security|csp|bundle|error` is represented exactly once.
  - [ ] Inventory distinguishes build vs startup output and assigns each row to Task 2, 3, 4, 5, or an explicitly documented no-fix rationale.

  **QA Scenarios** (MANDATORY - task incomplete without these):
  ```
  Scenario: Capture current release output
    Tool: Bash
    Steps: Run bounded `just run-release`, capture stdout/stderr to `.sisyphus/evidence/task-1-run-release-raw.log`, terminate the GUI after 15 seconds if still running.
    Expected: Command/build completes successfully or failure is captured; raw log exists; app process is not left running.
    Evidence: .sisyphus/evidence/task-1-run-release-raw.log

  Scenario: Inventory detects warning-like output
    Tool: Bash
    Steps: Scan raw log case-insensitively for `warning|warn|deprecated|security|csp|bundle|error` and compare count with rows in `.sisyphus/evidence/task-1-warning-inventory.md`.
    Expected: Every matched line is classified exactly once, with no orphan warning-like output.
    Evidence: .sisyphus/evidence/task-1-warning-inventory.md
  ```

  **Commit**: NO | Message: n/a | Files: `.sisyphus/evidence/task-1-run-release-raw.log`, `.sisyphus/evidence/task-1-warning-inventory.md`

- [x] 2. Fix confirmed frontend build warnings

  **What to do**:
  1. Use only Task 1 inventory rows assigned to frontend/Vite/Svelte.
  2. For Svelte compiler warnings, fix the referenced Svelte/TypeScript/CSS source according to the warning's root cause.
  3. For Vite bundle-size warnings, prefer real bundle structure fixes such as targeted dynamic import/manual chunking if they preserve behavior and improve build structure. Use `chunkSizeWarningLimit` only if evidence shows the chunk is intentionally large, expected, and no useful split exists; document exact rationale in evidence.
  4. Run `npm run build` after fixes and save output.
  5. Run relevant frontend tests if touched files are covered by `src/test/**/*.test.ts`; otherwise run `just test-ui` if frontend behavior/config changed.

  **Must NOT do**:
  - Do not add blanket warning suppression.
  - Do not change UI behavior, map rendering behavior, or IPC contracts unless the captured warning explicitly requires it.
  - Do not introduce broad lazy-loading architecture beyond the minimum warning-rooted change.

  **Recommended Agent Profile**:
  - Category: `visual-engineering` - Reason: frontend/Svelte/Vite fixes may touch UI build and bundling without changing UX.
  - Skills: [] - no special skill required unless a browser/UI verification becomes necessary.
  - Omitted: [`frontend-ui-ux`] - no UI redesign requested.

  **Parallelization**: Can Parallel: YES | Wave 2 | Blocks: 6 | Blocked By: 1

  **References** (executor has NO interview context - be exhaustive):
  - Build config: `vite.config.ts:25-29` - current build config has target/minify/sourcemap but no warning-specific chunk strategy.
  - Dependency: `package.json:19` - `maplibre-gl` is a likely large frontend dependency, but treat as unconfirmed until Task 1.
  - Tests: `vitest.config.ts:4-7` - frontend tests live under `src/test/**/*.test.ts`.
  - API boundary: `src/lib/api.ts` - components must not call Tauri `invoke()` directly if frontend changes touch IPC.

  **Acceptance Criteria** (agent-executable only):
  - [ ] Every Task 1 frontend warning row has a documented root-cause fix or exact no-fix rationale.
  - [ ] `npm run build` exits 0.
  - [ ] Captured `npm run build` output contains no unclassified frontend warning-like lines.
  - [ ] Relevant frontend tests or `just test-ui` pass if frontend source/config changed.

  **QA Scenarios** (MANDATORY - task incomplete without these):
  ```
  Scenario: Frontend build is warning-clean
    Tool: Bash
    Steps: Run `npm run build 2>&1 | tee .sisyphus/evidence/task-2-npm-build.log`; scan the log for warning-like patterns.
    Expected: Command exits 0; no unclassified warning-like lines remain.
    Evidence: .sisyphus/evidence/task-2-npm-build.log

  Scenario: Warning gate fails on synthetic frontend warning
    Tool: Bash
    Steps: Feed a temporary line such as `WARNING: synthetic frontend warning` into the same scanner/check logic used for build output.
    Expected: Scanner/check reports the synthetic line as warning-like and would fail unless explicitly classified.
    Evidence: .sisyphus/evidence/task-2-warning-scan-negative.log
  ```

  **Commit**: YES | Message: `fix(frontend): eliminate release build warnings` | Files: Task-1-confirmed frontend files only

- [x] 3. Fix confirmed Tauri build/config warnings

  **What to do**:
  1. Use only Task 1 inventory rows assigned to Tauri CLI/config/platform/security output.
  2. Inspect the exact warning text and Tauri documentation/version behavior before changing config.
  3. If warning concerns `csp: null`, decide from exact message whether root-cause fix is an explicit CSP, documented dev/release distinction, or retaining null with exact accepted rationale. Do not silently suppress.
  4. If warning concerns `macOSPrivateApi`, verify why it is enabled and whether the feature/config is required for current app behavior. Remove only if release still works and required capabilities remain intact.
  5. Run the Tauri build phase through `npm run tauri build -- --no-bundle` or `just run-release` bounded capture as appropriate.

  **Must NOT do**:
  - Do not broaden into a full Tauri security hardening project.
  - Do not remove platform features without verifying affected behavior.
  - Do not hide Tauri warnings by redirecting stderr or muting CLI output.

  **Recommended Agent Profile**:
  - Category: `quick` - Reason: likely narrow config changes after exact warning capture.
  - Skills: [] - use official docs only if exact warning requires external lookup.
  - Omitted: [`frontend-ui-ux`] - no UX work.

  **Parallelization**: Can Parallel: YES | Wave 2 | Blocks: 6 | Blocked By: 1

  **References** (executor has NO interview context - be exhaustive):
  - Config: `src-tauri/tauri.conf.json:5-10` - build hooks and app metadata.
  - Config: `src-tauri/tauri.conf.json:21-24` - `csp: null` and `macOSPrivateApi: true` are likely warning candidates, not confirmed until Task 1.
  - Manifest: `src-tauri/Cargo.toml:11` - Tauri dependency feature `macos-private-api` corresponds to config behavior.

  **Acceptance Criteria** (agent-executable only):
  - [ ] Every Task 1 Tauri warning row has a documented root-cause fix or exact no-fix rationale.
  - [ ] Tauri build command exits 0.
  - [ ] Captured Tauri build output contains no unclassified Tauri warning-like lines.

  **QA Scenarios** (MANDATORY - task incomplete without these):
  ```
  Scenario: Tauri build config is warning-clean
    Tool: Bash
    Steps: Run `npm run tauri build -- --no-bundle 2>&1 | tee .sisyphus/evidence/task-3-tauri-build.log`; scan for warning-like patterns.
    Expected: Command exits 0; no unclassified Tauri warning-like lines remain.
    Evidence: .sisyphus/evidence/task-3-tauri-build.log

  Scenario: Config change preserves release build
    Tool: Bash
    Steps: If `tauri.conf.json` or `Cargo.toml` changed, run the same Tauri build command again after a clean incremental rebuild.
    Expected: Release binary is produced at `src-tauri/target/release/ozi-rs`; no config-related warning-like output remains.
    Evidence: .sisyphus/evidence/task-3-tauri-build-repeat.log
  ```

  **Commit**: YES | Message: `fix(tauri): resolve release configuration warnings` | Files: Task-1-confirmed Tauri config/manifest files only

- [x] 4. Fix confirmed Rust compiler and dependency warnings

  **What to do**:
  1. Use only Task 1 inventory rows assigned to Rust compiler output.
  2. Separate warnings into app crate (`src-tauri/src/**`), build script (`src-tauri/build.rs`), local path dependency (`../../ozf2-rs`), and registry dependency buckets.
  3. For app crate warnings, fix root causes in the smallest scope: remove unused code if truly dead, use values intentionally, adjust visibility, or correct APIs. Prefer behavior-preserving cleanup over `#[allow]`.
  4. For local path dependency warnings, modify that local dependency only if it is writable and clearly part of the workspace workflow; otherwise document exact warning and coordinate follow-up rather than suppressing in app code.
  5. For registry dependency warnings, do not patch dependency source. Prefer dependency version/config updates only if safe and warning-rooted; otherwise document exact no-fix rationale.
  6. Run `just clippy`, `just test-rust`, and any focused Rust test relevant to touched modules.

  **Must NOT do**:
  - Do not add blanket `#![allow(warnings)]`, broad `#[allow(dead_code)]`, or clippy suppressions.
  - Do not rewrite domain/application architecture to silence a warning.
  - Do not patch Cargo registry sources.

  **Recommended Agent Profile**:
  - Category: `deep` - Reason: Rust warning cleanup may cross app crate and local path dependency while preserving architecture.
  - Skills: [`systematic-debugging`] - classify compiler warnings to root causes first.
  - Omitted: [`test-driven-development`] - user chose tests-after.

  **Parallelization**: Can Parallel: YES | Wave 2 | Blocks: 6 | Blocked By: 1

  **References** (executor has NO interview context - be exhaustive):
  - Manifest: `src-tauri/Cargo.toml` - Rust package/dependency definitions, including local `ozf2-rs = { path = "../../ozf2-rs" }` per planning exploration.
  - Build script: `src-tauri/build.rs` - no direct `cargo:warning` was found during planning exploration, but verify against captured output.
  - Architecture: `AGENTS.md` - preserve domain/application/infrastructure/commands layering.
  - Clippy gate: `justfile:67-68` - `just clippy` runs with `-D warnings`.
  - Rust tests: inline `#[cfg(test)]` modules exist in `src-tauri/src/application/commands.rs` and `src-tauri/src/domain/track.rs`.

  **Acceptance Criteria** (agent-executable only):
  - [ ] Every Task 1 Rust warning row has a documented root-cause fix or exact no-fix rationale.
  - [ ] `just clippy` exits 0.
  - [ ] `just test-rust` exits 0 if Rust code or local Rust dependency changed.
  - [ ] Release build output contains no unclassified Rust compiler warning-like lines.

  **QA Scenarios** (MANDATORY - task incomplete without these):
  ```
  Scenario: Rust warnings are fixed under strict clippy
    Tool: Bash
    Steps: Run `just clippy 2>&1 | tee .sisyphus/evidence/task-4-clippy.log`.
    Expected: Command exits 0; no Rust warning or clippy warning remains.
    Evidence: .sisyphus/evidence/task-4-clippy.log

  Scenario: Rust behavior is preserved
    Tool: Bash
    Steps: Run `just test-rust 2>&1 | tee .sisyphus/evidence/task-4-test-rust.log`.
    Expected: Command exits 0; existing Rust tests pass.
    Evidence: .sisyphus/evidence/task-4-test-rust.log
  ```

  **Commit**: YES | Message: `fix(rust): eliminate release compiler warnings` | Files: Task-1-confirmed Rust files only

- [x] 5. Fix confirmed release startup warning/log output

  **What to do**:
  1. Use only Task 1 inventory rows emitted after the release binary starts.
  2. Distinguish actual warnings/errors from intentional diagnostics and third-party framework noise.
  3. For app-owned noisy diagnostics, keep useful dev/debug output but make release startup quiet unless the message is actionable for a release user/operator.
  4. Prefer release-specific log filtering or corrected log levels over deleting diagnostics entirely.
  5. If startup output indicates a real problem, fix the underlying startup condition instead of downgrading the log.
  6. Verify bounded startup capture after changes.

  **Must NOT do**:
  - Do not mute all tracing/logging globally.
  - Do not convert real startup failures into silent success.
  - Do not remove diagnostics needed for development mode if a release-only filter solves the warning.

  **Recommended Agent Profile**:
  - Category: `unspecified-high` - Reason: runtime output may involve Tauri startup, tracing, environment variables, or app state initialization.
  - Skills: [`systematic-debugging`] - trace runtime messages to their source and intent.
  - Omitted: [`frontend-ui-ux`] - no UI redesign or visual changes.

  **Parallelization**: Can Parallel: YES | Wave 2 | Blocks: 6 | Blocked By: 1

  **References** (executor has NO interview context - be exhaustive):
  - Runtime logging: `src-tauri/src/lib.rs:11-16` - tracing subscriber initialization and default log level.
  - Runtime diagnostics: `src-tauri/src/application/mod.rs:1138-1142` - planning exploration found diagnostic `tracing::error!`/`tracing::info!` calls; verify exact source from captured output before editing.
  - Release command: `justfile:34-36` - the runtime phase is the second command in `run-release`.

  **Acceptance Criteria** (agent-executable only):
  - [ ] Every Task 1 startup warning/log row has a documented root-cause fix or exact no-fix rationale.
  - [ ] Bounded release startup capture exits/terminates cleanly with no unclassified warning/error-like lines.
  - [ ] Development diagnostics remain available by documented mechanism such as `RUST_LOG` when applicable.

  **QA Scenarios** (MANDATORY - task incomplete without these):
  ```
  Scenario: Release startup is quiet
    Tool: Bash
    Steps: Launch `./src-tauri/target/release/ozi-rs` with stdout/stderr captured to `.sisyphus/evidence/task-5-release-startup.log`; wait 15 seconds; terminate cleanly if still running; scan for warning-like patterns.
    Expected: App starts; process is not left running; no unclassified warning/error-like startup lines remain.
    Evidence: .sisyphus/evidence/task-5-release-startup.log

  Scenario: Real startup errors are not hidden
    Tool: Bash
    Steps: If the fix changes logging/filtering, run a targeted check with a synthetic or existing failing startup condition when available, or run with elevated `RUST_LOG` to confirm diagnostics are still available in dev/debug context.
    Expected: Diagnostics remain observable in the appropriate non-release/debug context; release quieting does not mask actual failures.
    Evidence: .sisyphus/evidence/task-5-diagnostics-preserved.log
  ```

  **Commit**: YES | Message: `fix(runtime): quiet release startup warnings` | Files: Task-1-confirmed runtime/logging files only

- [x] 6. Add local warning hygiene gate and final release evidence

  **What to do**:
  1. Review Task 1 inventory and Tasks 2-5 evidence to confirm every warning-like row has been resolved or exactly documented.
  2. Add the smallest local warning-hygiene mechanism practical for this repo. Preferred options, in order:
     - a `just` recipe that runs bounded release capture and scans for unclassified warning-like lines;
     - a small script used by a `just` recipe;
     - documented command sequence only if no script/recipe is justified.
  3. The gate must be local-only; do not add CI workflows.
  4. The gate must fail on synthetic warning-like input and pass on final clean release output.
  5. Run final verification: `npm run build`, `just clippy`, `just test`, and bounded `just run-release`.
  6. Save final outputs under `.sisyphus/evidence/`.

  **Must NOT do**:
  - Do not add CI.
  - Do not create a brittle scanner that ignores stderr, colorized output, or runtime startup output.
  - Do not use the gate to hide warnings by filtering them out of user-visible logs.

  **Recommended Agent Profile**:
  - Category: `unspecified-high` - Reason: combines local tooling, shell/process capture, and full repo verification.
  - Skills: [] - no special skill unless implementation discovers a debugging issue.
  - Omitted: [`git-master`] - commits are handled per task; no advanced git operation needed unless requested.

  **Parallelization**: Can Parallel: NO | Wave 3 | Blocks: Final Verification Wave | Blocked By: 2, 3, 4, 5

  **References** (executor has NO interview context - be exhaustive):
  - Recipes: `justfile` - existing commands include `run-release`, `clippy`, `test`, `test-rust`, and `test-ui`.
  - Frontend scripts: `package.json` - existing scripts include `build`, `tauri`, and `test`.
  - Testing strategy: `docs/testing-strategy.md:63-72` - strict clippy warning policy is documented.
  - Repo policy: `AGENTS.md` - run targeted tests plus `just test` / `just clippy` before completion.

  **Acceptance Criteria** (agent-executable only):
  - [ ] Local warning gate exists as a `just` recipe or documented command and scans both build and bounded startup output.
  - [ ] Gate fails on synthetic warning-like input.
  - [ ] Gate passes on final bounded `just run-release` output.
  - [ ] `npm run build` exits 0.
  - [ ] `just clippy` exits 0.
  - [ ] `just test` exits 0.
  - [ ] `.sisyphus/evidence/task-6-final-run-release.log` contains no unclassified warning-like lines.

  **QA Scenarios** (MANDATORY - task incomplete without these):
  ```
  Scenario: Final local warning gate passes clean output
    Tool: Bash
    Steps: Run the added local gate against final bounded release output and save results.
    Expected: Gate exits 0; final release output has no unclassified warning-like lines.
    Evidence: .sisyphus/evidence/task-6-final-warning-gate.log

  Scenario: Local warning gate catches regressions
    Tool: Bash
    Steps: Feed synthetic lines containing `WARNING`, `deprecated`, `security`, and `ERROR` into the scanner/gate.
    Expected: Gate exits non-zero and reports the synthetic warning-like lines.
    Evidence: .sisyphus/evidence/task-6-warning-gate-negative.log
  ```

  **Commit**: YES | Message: `chore(release): add local warning hygiene check` | Files: local recipe/script/docs changed by this task only

## Final Verification Wave (MANDATORY — after ALL implementation tasks)
> 4 review agents run in PARALLEL. ALL must APPROVE. Present consolidated results to user and get explicit "okay" before completing.
> **Do NOT auto-proceed after verification. Wait for user's explicit approval before marking work complete.**
> **Never mark F1-F4 as checked before getting user's okay.** Rejection or user feedback -> fix -> re-run -> present again -> wait for okay.
- [x] F1. Plan Compliance Audit — oracle
- [x] F2. Code Quality Review — unspecified-high
- [x] F3. Real Manual QA — unspecified-high
- [x] F4. Scope Fidelity Check — deep

## Commit Strategy
- Commit each distinct warning class separately after its task passes acceptance criteria.
- Do not commit `.sisyphus/evidence/*` unless the user explicitly wants evidence artifacts versioned.
- Suggested commit sequence:
  1. `fix(frontend): eliminate release build warnings` if Task 2 changes files.
  2. `fix(tauri): resolve release configuration warnings` if Task 3 changes files.
  3. `fix(rust): eliminate release compiler warnings` if Task 4 changes files.
  4. `fix(runtime): quiet release startup warnings` if Task 5 changes files.
  5. `chore(release): add local warning hygiene check` if Task 6 changes local tooling.

## Success Criteria
- Every originally captured warning-like line has a root-cause classification and final status.
- Final bounded `just run-release` evidence is clean for build and startup phases.
- Local warning hygiene check prevents obvious warning regressions.
- `npm run build`, `just clippy`, and `just test` pass.
- App behavior remains unchanged except for removal/reclassification of unwanted release warning output.
