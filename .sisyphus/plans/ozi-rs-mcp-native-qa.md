# Ozi RS MCP Native QA

## TL;DR
> **Summary**: Add a project-local Rust stdio MCP server named `ozi-rs-mcp` that gives OpenCode native desktop QA tools for the Tauri app. Tier 1 build/launch/log/screenshot/stop tools must work without Appium; Appium Mac2 tools add optional native interactions when local permissions and dependencies are available.
> **Deliverables**:
> - Root Cargo workspace with `tools/ozi-rs-mcp` crate and `ozi-rs-mcp` binary.
> - MCP tools for environment probing, app build, native launch/stop, log capture, screenshot capture, composite observation, and Appium Mac2 actions.
> - Root `opencode.json` registering the local MCP server without modifying `.opencode/oh-my-opencode.json`.
> - Tests and docs that make native MCP QA the default for desktop verification instead of browser/Playwright.
> **Effort**: Large
> **Parallel**: YES - 3 waves
> **Critical Path**: Task 1 → Task 2 → Task 3 → Task 5 → Task 8 → Final Verification

## Context
### Original Request
User requested: “Я хочу чтобы ты сделал MCP сервер для этого проекта и добавил в возможности opencode его. Чтобы тестирование было не через браузер — приложение предполагается как нативное, а не как браузерное.”

Confirmed decisions:
- MVP: `Native QA MCP`.
- Native QA level: `Tier 1 + Appium`.
- Binary/server name: `ozi-rs-mcp`.
- Preferred implementation: Rust stdio MCP, not TypeScript.

### Interview Summary
- Repo has `.opencode/oh-my-opencode.json`, but no documented root OpenCode `opencode.json` and no MCP registration.
- Existing test stack is Rust inline tests plus Node-environment Vitest; no Playwright config exists.
- App is a Tauri 2 native desktop app, so browser Playwright must not be the default end-to-end QA path.
- Appium Mac2 is desired, but Tier 1 tools must remain fully functional when Appium is absent.

### Metis Review (gaps addressed)
- Appium must be optional and gracefully degraded.
- Preserve `.opencode/oh-my-opencode.json`; create root `opencode.json` for MCP registration.
- Avoid stdout contamination for stdio MCP; logs go to stderr or evidence files.
- Verify workspace conversion does not break `just test`, `just clippy`, `just build`, and Tauri commands.
- Include edge cases: missing `just`, failed build, app exits immediately, screenshot/log permission issues, Appium installed without Mac2 driver, non-default Appium port, spaces in paths, dropped MCP connection leaving child processes.

## Work Objectives
### Core Objective
Create `ozi-rs-mcp`, a local stdio MCP server that OpenCode can call to perform native desktop QA for the Tauri app.

### Deliverables
- Root `Cargo.toml` workspace including `src-tauri` and `tools/ozi-rs-mcp`.
- `tools/ozi-rs-mcp` Rust crate with focused modules:
  - `src/main.rs` — stdio MCP entry point only.
  - `src/server.rs` — MCP tool registration and request dispatch.
  - `src/config.rs` — repo root/evidence directory/app path resolution.
  - `src/process.rs` — safe command runner with stdout/stderr capture.
  - `src/evidence.rs` — `.sisyphus/evidence/native-qa/...` artifact paths and metadata.
  - `src/native.rs` — Tier 1 macOS build/launch/log/screenshot/stop implementation.
  - `src/appium.rs` — optional Appium Mac2 doctor/session/action adapter.
  - `src/types.rs` — request/response structs for all tools.
- Root `opencode.json` with an `mcp.ozi_rs` local server entry invoking `cargo run --quiet -p ozi-rs-mcp --`. The MCP binary/server name remains `ozi-rs-mcp`; the OpenCode config key uses `ozi_rs` so generated tool prefixes avoid hyphens.
- Documentation updates to `AGENTS.md` and `docs/testing-strategy.md` instructing future agents to use native MCP QA for desktop verification.
- Unit tests and integration-style command-construction tests for MCP, native, and Appium degraded paths.

### Definition of Done (verifiable conditions with commands)
- `cargo test -p ozi-rs-mcp` passes.
- `cargo run --quiet -p ozi-rs-mcp -- --self-check` returns JSON with `server_name: "ozi-rs-mcp"`, `stdio_safe: true`, and no stdout noise before the JSON.
- `just test` passes.
- `just clippy` passes.
- `just build` still builds the Tauri app after workspace conversion.
- OpenCode MCP config validation succeeds by starting OpenCode/MCP Inspector against root `opencode.json` and listing `ozi_rs_*` tools.
- Appium absent path is verified: `appium_doctor` returns `available: false` with missing dependency details and Tier 1 tools still run.

### Must Have
- Tier 1 native tools work on macOS without Appium installed.
- MCP stdio stdout is protocol-only; human logs are stderr or artifact files.
- Appium Mac2 tools are dependency-gated and never panic when Appium/Xcode/permissions are missing.
- All evidence files live under `.sisyphus/evidence/native-qa/`.
- Paths with spaces are handled via `std::process::Command` args, never shell-concatenated strings.
- Root `opencode.json` is added for OpenCode MCP registration.

### Must NOT Have
- Do not add Playwright or browser E2E as the default QA path.
- Do not use `tauri-driver` for macOS MVP.
- Do not require Appium for build/launch/log/screenshot/stop.
- Do not modify `.opencode/oh-my-opencode.json`.
- Do not turn this into a generic cross-platform automation framework; macOS native QA is the first supported target.
- Do not change Tauri runtime behavior unless a task explicitly says so.

## Verification Strategy
> ZERO HUMAN INTERVENTION - all verification is agent-executed.
- Test decision: tests-after for this integration slice, using Rust unit/integration-style tests plus existing repo `just test` and `just clippy`.
- QA policy: Every task has agent-executed scenarios.
- Evidence: `.sisyphus/evidence/task-{N}-{slug}.{ext}` and `.sisyphus/evidence/native-qa/...`.
- Native UI policy: Tier 1 MCP tools are mandatory; Appium scenarios run only if `appium_doctor.available == true`, otherwise the expected pass condition is structured `available: false` with reasons.

## Execution Strategy
### Parallel Execution Waves
Wave 1: Task 1 workspace/crate foundation, then Task 2 MCP schema/stdio and Task 4 evidence/config helpers after Task 1 lands.
Wave 2: Task 3 Tier 1 native tools; Task 5 Appium adapter; Task 6 OpenCode config; Task 7 docs.
Wave 3: Task 8 end-to-end native QA scenarios; Task 9 repo-wide verification and hardening.

### Dependency Matrix (full, all tasks)
- Task 1 blocks Tasks 2, 3, 4, 5, 8, 9.
- Task 2 blocks Tasks 3, 5, 6, 8.
- Task 4 blocks Tasks 3, 5, 8.
- Task 3 blocks Task 8.
- Task 5 blocks Task 8.
- Task 6 blocks Task 8.
- Task 7 can run after Task 2 and must finish before Task 9.
- Task 8 blocks Task 9.

### Agent Dispatch Summary (wave → task count → categories)
- Wave 1 → 3 tasks → `quick`, `unspecified-high`.
- Wave 2 → 4 tasks → `unspecified-high`, `writing`.
- Wave 3 → 2 tasks → `deep`, `unspecified-high`.

## TODOs
> Implementation + Test = ONE task. Never separate.
> EVERY task MUST have: Agent Profile + Parallelization + QA Scenarios.

- [x] 1. Create Cargo workspace and `tools/ozi-rs-mcp` crate

  **What to do**: Add root `Cargo.toml` with workspace members `src-tauri` and `tools/ozi-rs-mcp`. Create the `tools/ozi-rs-mcp` crate with package name `ozi-rs-mcp`, binary name `ozi-rs-mcp`, edition 2024, a minimal compileable `main.rs`, and dependencies needed for async MCP, JSON serialization, command execution, temp paths, and tests. Keep `src-tauri/Cargo.toml` package metadata unchanged except workspace compatibility if Cargo requires it.
  **Must NOT do**: Do not merge MCP code into `src-tauri/src`; do not rename the existing Tauri package; do not edit `.opencode/oh-my-opencode.json`.

  **Recommended Agent Profile**:
  - Category: `unspecified-high` - Reason: workspace conversion can break existing Tauri/just flows.
  - Skills: [`superpowers:test-driven-development`] - Rust crate foundation should be covered by tests immediately.
  - Omitted: [`frontend-ui-ux`] - No UI design work.

  **Parallelization**: Can Parallel: YES | Wave 1 | Blocks: Tasks 2, 3, 4, 5, 8, 9 | Blocked By: none

  **References**:
  - Existing crate: `src-tauri/Cargo.toml:1-34` - Tauri package currently exists as a single crate.
  - Existing commands: `justfile:40-54` - build recipes that must keep working.
  - Existing tests: `justfile:72-87` - test recipes that must keep working.
  - External: `https://github.com/modelcontextprotocol/rust-sdk` - Rust MCP SDK.
  - External: `https://docs.rs/rmcp/latest/rmcp/` - `rmcp` crate documentation.

  **Acceptance Criteria**:
  - [ ] `cargo metadata --no-deps` shows workspace members `src-tauri` and `tools/ozi-rs-mcp`.
  - [ ] `cargo check -p ozi-rs-mcp` exits 0.
  - [ ] `cargo test -p ozi-rs-mcp` passes.
  - [ ] `cargo test --manifest-path src-tauri/Cargo.toml` still passes or fails only on pre-existing documented failures, with output captured to evidence.

  **QA Scenarios**:
  ```
  Scenario: Workspace detects both crates
    Tool: Bash
    Steps: Run `cargo metadata --no-deps --format-version 1` from repo root and save output to `.sisyphus/evidence/task-1-workspace-metadata.json`.
    Expected: JSON contains package names `ozi-rs` and `ozi-rs-mcp`.
    Evidence: .sisyphus/evidence/task-1-workspace-metadata.json

  Scenario: Tauri crate remains addressable
    Tool: Bash
    Steps: Run `cargo test --manifest-path src-tauri/Cargo.toml --no-run` and save output.
    Expected: Exit 0, or a failure explicitly matching a pre-existing non-MCP blocker; no failure caused by missing workspace/package metadata.
    Evidence: .sisyphus/evidence/task-1-tauri-no-run.txt
  ```

  **Commit**: YES | Message: `build(mcp): add ozi-rs-mcp workspace crate` | Files: [`Cargo.toml`, `tools/ozi-rs-mcp/**`]

- [x] 2. Implement stdio-safe MCP server skeleton and tool inventory

  **What to do**: Implement `main.rs`, `server.rs`, and `types.rs`. Support normal stdio MCP mode by default and `--self-check` as a non-MCP diagnostic command. Register these tool names exactly: `qa_environment`, `build_app`, `launch_app`, `stop_app`, `capture_logs`, `capture_screenshot`, `qa_observe`, `appium_doctor`, `appium_launch_session`, `appium_click`, `appium_type_text`, `appium_screenshot`, `appium_stop_session`. Stub tool handlers may return `not_implemented: true` only inside this task; later tasks must replace the relevant stubs.
  **Must NOT do**: Do not print tracing/logging to stdout in MCP mode. Do not register more tools than the listed set.

  **Recommended Agent Profile**:
  - Category: `unspecified-high` - Reason: protocol correctness and stdout hygiene are critical.
  - Skills: [`superpowers:test-driven-development`] - Tool inventory and self-check behavior need tests.
  - Omitted: [`playwright`] - Browser automation is explicitly out of scope.

  **Parallelization**: Can Parallel: YES | Wave 1 | Blocks: Tasks 3, 5, 6, 8 | Blocked By: Task 1

  **References**:
  - External: `https://modelcontextprotocol.io/docs/concepts/transports` - stdio transport rules.
  - External: `https://modelcontextprotocol.io/docs/tools/debugging` - stdout must be protocol-only; use stderr/log files.

  **Acceptance Criteria**:
  - [ ] `cargo test -p ozi-rs-mcp tool_inventory` passes and verifies exact tool names.
  - [ ] `cargo run --quiet -p ozi-rs-mcp -- --self-check > .sisyphus/evidence/task-2-self-check.json` produces JSON parseable by `python3 -m json.tool`.
  - [ ] Self-check JSON includes `stdio_safe: true`, `tool_count: 13`, and every required tool name.

  **QA Scenarios**:
  ```
  Scenario: Self-check is parseable
    Tool: Bash
    Steps: Run `cargo run --quiet -p ozi-rs-mcp -- --self-check > .sisyphus/evidence/task-2-self-check.json` then `python3 -m json.tool .sisyphus/evidence/task-2-self-check.json`.
    Expected: Both commands exit 0; JSON contains `"server_name": "ozi-rs-mcp"`.
    Evidence: .sisyphus/evidence/task-2-self-check.json

  Scenario: No stdout contamination in self-check
    Tool: Bash
    Steps: Parse `.sisyphus/evidence/task-2-self-check.json` as JSON and count top-level keys.
    Expected: File contains only one JSON document, no log prefixes, no warnings before `{`.
    Evidence: .sisyphus/evidence/task-2-stdout-clean.txt
  ```

  **Commit**: YES | Message: `feat(mcp): expose stdio-safe tool inventory` | Files: [`tools/ozi-rs-mcp/src/main.rs`, `tools/ozi-rs-mcp/src/server.rs`, `tools/ozi-rs-mcp/src/types.rs`]

- [x] 3. Implement Tier 1 native macOS QA tools

  **What to do**: Implement `config.rs`, `process.rs`, `evidence.rs`, and `native.rs` for `qa_environment`, `build_app`, `launch_app`, `stop_app`, `capture_logs`, `capture_screenshot`, and `qa_observe`. Resolve repo root from `OZI_RS_PROJECT_ROOT` if set, otherwise current directory ancestors containing `justfile` and `src-tauri/tauri.conf.json`. Use `just build` as default build command. Launch packaged/debug `.app` when found; otherwise return a structured `artifact_missing` error with build evidence path. Use `std::process::Command` with arg arrays for `open`, `log`, and `screencapture`. Persist session state under `.sisyphus/evidence/native-qa/session.json`.
  **Must NOT do**: Do not use shell-concatenated commands; do not require Appium; do not assume current working directory is repo root.

  **Recommended Agent Profile**:
  - Category: `unspecified-high` - Reason: process lifecycle and artifact handling need careful edge-case coverage.
  - Skills: [`superpowers:systematic-debugging`] - Native process failures require evidence-first diagnosis.
  - Omitted: [`frontend-ui-ux`] - No UI styling.

  **Parallelization**: Can Parallel: YES | Wave 2 | Blocks: Task 8 | Blocked By: Tasks 1, 2, 4

  **References**:
  - Existing build recipes: `justfile:40-46` - `just build` and `just release`.
  - Existing run recipes: `justfile:29-36` - dev/release execution patterns.
  - Tauri config: `src-tauri/tauri.conf.json` - app metadata and bundle output expectations.
  - External: `https://ss64.com/mac/open.html` - macOS `open` flags.
  - External: `https://ss64.com/mac/log.html` - macOS unified log capture.
  - External: `https://ss64.com/mac/screencapture.html` - screenshot capture.

  **Acceptance Criteria**:
  - [ ] Unit tests cover repo-root detection from repo root, subdirectory, and env var.
  - [ ] Unit tests cover command construction for paths with spaces.
  - [ ] `qa_environment` reports OS, repo root, `just` availability, `open` availability, `log` availability, `screencapture` availability, Appium availability separately.
  - [ ] `stop_app` is idempotent: second call returns `already_stopped` instead of error.
  - [ ] Build failure returns structured result with command, exit code, duration, stdout path, stderr path.

  **QA Scenarios**:
  ```
  Scenario: Tier 1 environment probe
    Tool: Bash
    Steps: Run MCP self-check or test helper that invokes `qa_environment` and writes `.sisyphus/evidence/task-3-qa-environment.json`.
    Expected: JSON contains `platform: "macos"`, repo root path, and separate availability booleans for `just`, `open`, `log`, `screencapture`, and `appium`.
    Evidence: .sisyphus/evidence/task-3-qa-environment.json

  Scenario: Missing app artifact is structured
    Tool: Bash
    Steps: Run test helper with a temp repo fixture where no `.app` exists and invoke `launch_app`.
    Expected: Result has `ok: false`, `error_kind: "artifact_missing"`, and does not panic.
    Evidence: .sisyphus/evidence/task-3-missing-artifact.json
  ```

  **Commit**: YES | Message: `feat(mcp): add native qa tier one tools` | Files: [`tools/ozi-rs-mcp/src/config.rs`, `tools/ozi-rs-mcp/src/process.rs`, `tools/ozi-rs-mcp/src/evidence.rs`, `tools/ozi-rs-mcp/src/native.rs`, `tools/ozi-rs-mcp/src/server.rs`, `tools/ozi-rs-mcp/src/types.rs`]

- [x] 4. Add evidence model and deterministic test fixtures

  **What to do**: Create test fixtures/helpers for fake command execution so native/Appium tests do not depend on the real OS tools. Define evidence metadata schema with fields `tool`, `started_at`, `duration_ms`, `command`, `exit_code`, `stdout_path`, `stderr_path`, `artifact_paths`, `status`, and `error_kind`. Add tests for evidence directory creation under `.sisyphus/evidence/native-qa/`.
  **Must NOT do**: Do not write evidence outside `.sisyphus/evidence`; do not require a real Tauri build for unit tests.

  **Recommended Agent Profile**:
  - Category: `quick` - Reason: focused helper/test infrastructure task.
  - Skills: [`superpowers:test-driven-development`] - Fixture behavior should be test-first.
  - Omitted: [`librarian`] - External research already completed.

  **Parallelization**: Can Parallel: YES | Wave 1 | Blocks: Tasks 3, 5, 8 | Blocked By: Task 1

  **References**:
  - Evidence target: `.sisyphus/evidence/` - existing project evidence directory.
  - Testing conventions: `AGENTS.md` testing section and `docs/testing-strategy.md`.

  **Acceptance Criteria**:
  - [ ] `cargo test -p ozi-rs-mcp evidence` passes.
  - [ ] Tests verify evidence paths are created under `.sisyphus/evidence/native-qa/`.
  - [ ] Tests verify command stdout/stderr are captured to files, not mixed into MCP responses.

  **QA Scenarios**:
  ```
  Scenario: Evidence path confinement
    Tool: Bash
    Steps: Run `cargo test -p ozi-rs-mcp evidence_path_confinement -- --nocapture` and save output.
    Expected: Test passes and printed evidence path starts with `.sisyphus/evidence/native-qa/`.
    Evidence: .sisyphus/evidence/task-4-evidence-paths.txt

  Scenario: Fake command captures stdout and stderr separately
    Tool: Bash
    Steps: Run `cargo test -p ozi-rs-mcp fake_command_captures_streams -- --nocapture`.
    Expected: Test passes; stdout and stderr fixture paths are distinct.
    Evidence: .sisyphus/evidence/task-4-stream-capture.txt
  ```

  **Commit**: YES | Message: `test(mcp): add native qa evidence fixtures` | Files: [`tools/ozi-rs-mcp/src/evidence.rs`, `tools/ozi-rs-mcp/src/process.rs`, `tools/ozi-rs-mcp/tests/**`]

- [x] 5. Implement optional Appium Mac2 adapter tools

  **What to do**: Implement `appium.rs` and server handlers for `appium_doctor`, `appium_launch_session`, `appium_click`, `appium_type_text`, `appium_screenshot`, and `appium_stop_session`. `appium_doctor` must run dependency probes for `appium`, `appium driver list --installed`, and `appium driver doctor mac2` when available. All Appium action tools must first check doctor/session state and return `available: false` or `session_missing` structured results instead of panicking. Default Appium server URL is `http://127.0.0.1:4723`; allow request override.
  **Must NOT do**: Do not start requiring Appium for Tier 1 tools; do not hard-code a non-local Appium server; do not fail all MCP startup when Appium is unavailable.

  **Recommended Agent Profile**:
  - Category: `unspecified-high` - Reason: optional external dependency with many failure modes.
  - Skills: [`superpowers:systematic-debugging`] - Must prove degraded paths with evidence.
  - Omitted: [`playwright`] - Not browser automation.

  **Parallelization**: Can Parallel: YES | Wave 2 | Blocks: Task 8 | Blocked By: Tasks 1, 2, 4

  **References**:
  - External: `https://github.com/appium/appium-mac2-driver` - Mac2 driver capabilities, doctor, locator strategies, screenshots, gestures.
  - External: `https://github.com/appium/appium-mac2-driver/blob/fb7d96037804b7bb8b1bcb34cc462f470a45da64/README.md#L19-L33` - macOS/Xcode/permissions requirements.
  - External: `https://github.com/appium/appium-mac2-driver/blob/fb7d96037804b7bb8b1bcb34cc462f470a45da64/README.md#L941-L943` - avoid parallel native UI tests.

  **Acceptance Criteria**:
  - [ ] With `appium` absent from PATH, `appium_doctor` returns `available: false`, missing dependency list, and install hints.
  - [ ] With fake Appium commands, tests verify Mac2 driver missing, permissions missing, and doctor success paths.
  - [ ] Appium action tools return `session_missing` before a session is launched.
  - [ ] Appium screenshot writes evidence under `.sisyphus/evidence/native-qa/` when fake backend reports image data.

  **QA Scenarios**:
  ```
  Scenario: Appium absent is graceful
    Tool: Bash
    Steps: Run `cargo test -p ozi-rs-mcp appium_absent_is_graceful -- --nocapture`.
    Expected: Test passes; result contains `available: false` and `missing: ["appium"]`.
    Evidence: .sisyphus/evidence/task-5-appium-absent.txt

  Scenario: Appium action without session is rejected structurally
    Tool: Bash
    Steps: Run `cargo test -p ozi-rs-mcp appium_click_requires_session -- --nocapture`.
    Expected: Test passes; response has `ok: false`, `error_kind: "session_missing"`.
    Evidence: .sisyphus/evidence/task-5-session-missing.txt
  ```

  **Commit**: YES | Message: `feat(mcp): add optional appium mac2 adapter` | Files: [`tools/ozi-rs-mcp/src/appium.rs`, `tools/ozi-rs-mcp/src/server.rs`, `tools/ozi-rs-mcp/src/types.rs`, `tools/ozi-rs-mcp/tests/**`]

- [x] 6. Register `ozi-rs-mcp` in root OpenCode config

  **What to do**: Add root `opencode.json` using OpenCode config schema. Configure `mcp.ozi_rs` as a local server with command `cargo run --quiet -p ozi-rs-mcp --`, enabled true, timeout 10000, and environment `OZI_RS_PROJECT_ROOT` set to the repo path via `{env:OZI_RS_PROJECT_ROOT}` fallback documentation. If OpenCode requires static env, document that agents should export `OZI_RS_PROJECT_ROOT=$(pwd)` before starting OpenCode. Add tool enablement so `ozi_rs_*` tools are available to implementation/QA agents and not globally disabled. Keep the binary/server name `ozi-rs-mcp`; only the OpenCode key uses `ozi_rs` for tool-prefix safety.
  **Must NOT do**: Do not modify `.opencode/oh-my-opencode.json`; do not add secrets or absolute user-specific paths.

  **Recommended Agent Profile**:
  - Category: `quick` - Reason: focused config addition with validation.
  - Skills: [] - No special skill required beyond config validation.
  - Omitted: [`frontend-ui-ux`] - No UI work.

  **Parallelization**: Can Parallel: YES | Wave 2 | Blocks: Task 8 | Blocked By: Task 2

  **References**:
  - Existing Oh My OpenCode config: `.opencode/oh-my-opencode.json:1-75` - preserve unchanged.
  - External: `https://opencode.ai/docs/mcp-servers/` - OpenCode MCP config.
  - External: `https://opencode.ai/docs/config/` - root/project config and env interpolation.

  **Acceptance Criteria**:
  - [ ] Root `opencode.json` exists and contains `$schema: "https://opencode.ai/config.json"`.
  - [ ] `opencode.json` contains `mcp.ozi_rs.type: "local"` and command array beginning `cargo`, `run`, `--quiet`, `-p`, `ozi-rs-mcp`, `--`.
  - [ ] `.opencode/oh-my-opencode.json` is unchanged by this task.
  - [ ] OpenCode or MCP Inspector can list `ozi_rs_qa_environment` and `ozi_rs_qa_observe`.

  **QA Scenarios**:
  ```
  Scenario: Config JSON validates
    Tool: Bash
    Steps: Run `python3 -m json.tool opencode.json > .sisyphus/evidence/task-6-opencode-json.txt`.
    Expected: Exit 0 and pretty-printed JSON includes `"mcp"`.
    Evidence: .sisyphus/evidence/task-6-opencode-json.txt

  Scenario: Existing Oh My OpenCode config preserved
    Tool: Bash
    Steps: Run `git diff -- .opencode/oh-my-opencode.json`.
    Expected: No diff output.
    Evidence: .sisyphus/evidence/task-6-oh-my-opencode-preserved.txt
  ```

  **Commit**: YES | Message: `chore(opencode): register ozi-rs mcp server` | Files: [`opencode.json`]

- [x] 7. Document native MCP QA policy for future agents

  **What to do**: Update `AGENTS.md` and `docs/testing-strategy.md`. State that desktop QA for the Tauri app should use `ozi-rs-mcp` native tools by default. Keep Rust/Vitest tests as deterministic gates. Explicitly say Playwright/browser testing is not the default for native app behavior, but may still be used for isolated web/frontend experiments if intentionally added later. Document Appium Mac2 as optional and dependency-gated.
  **Must NOT do**: Do not claim browser testing is impossible; do not remove existing test commands; do not overstate Appium availability.

  **Recommended Agent Profile**:
  - Category: `writing` - Reason: concise technical docs update.
  - Skills: [] - No extra skill required.
  - Omitted: [`playwright`] - Policy says not default.

  **Parallelization**: Can Parallel: YES | Wave 2 | Blocks: Task 9 | Blocked By: Task 2

  **References**:
  - Agent instructions: `AGENTS.md` testing and startup sections.
  - Testing strategy: `docs/testing-strategy.md`.
  - README commands: `README.md` Quick Start and command table.

  **Acceptance Criteria**:
  - [ ] `AGENTS.md` mentions `ozi-rs-mcp` and native MCP QA for desktop verification.
  - [ ] `docs/testing-strategy.md` lists MCP Tier 1 and Appium-gated native QA under quality gates.
  - [ ] Docs preserve existing `just test` and `just clippy` guidance.

  **QA Scenarios**:
  ```
  Scenario: Docs mention native MCP QA
    Tool: Bash
    Steps: Search `AGENTS.md` and `docs/testing-strategy.md` for `ozi-rs-mcp` and save results.
    Expected: Both files contain `ozi-rs-mcp` and `native` in the same relevant section.
    Evidence: .sisyphus/evidence/task-7-docs-native-mcp.txt

  Scenario: Browser default not introduced
    Tool: Bash
    Steps: Search changed docs for `Playwright`.
    Expected: Any Playwright mention says it is not default for native desktop QA.
    Evidence: .sisyphus/evidence/task-7-playwright-policy.txt
  ```

  **Commit**: YES | Message: `docs(testing): prefer native mcp qa for desktop app` | Files: [`AGENTS.md`, `docs/testing-strategy.md`]

- [x] 8. Add end-to-end native QA smoke workflow through MCP

  **What to do**: Add tests or a documented executable script under `tools/ozi-rs-mcp/tests/` or `tools/ozi-rs-mcp/scripts/` that drives the MCP server/tool handlers through: `qa_environment` → `build_app` → `launch_app` → `capture_screenshot` → `capture_logs` → `stop_app` → `appium_doctor`. Use real local commands when available and fake backends in automated tests. Save evidence under `.sisyphus/evidence/native-qa/`.
  **Must NOT do**: Do not require Appium success for the smoke workflow; Appium unavailable is a valid pass only if structured and Tier 1 completed.

  **Recommended Agent Profile**:
  - Category: `deep` - Reason: crosses MCP, build, native process lifecycle, evidence, and optional Appium.
  - Skills: [`superpowers:verification-before-completion`] - Evidence before success claims.
  - Omitted: [`frontend-ui-ux`] - No visual design work.

  **Parallelization**: Can Parallel: NO | Wave 3 | Blocks: Task 9 | Blocked By: Tasks 3, 5, 6

  **References**:
  - MCP tools from Tasks 2, 3, 5.
  - Evidence model from Task 4.
  - OpenCode config from Task 6.

  **Acceptance Criteria**:
  - [ ] Smoke workflow creates evidence files for environment, build, launch, screenshot, logs, stop, and appium doctor.
  - [ ] If app launches then exits immediately, workflow records `exited_early` with logs and exits non-zero.
  - [ ] If Appium unavailable, workflow records `available: false` and still exits zero after Tier 1 success.
  - [ ] If Tier 1 build fails, workflow exits non-zero with build stdout/stderr paths.

  **QA Scenarios**:
  ```
  Scenario: Tier 1 smoke succeeds or fails with structured evidence
    Tool: Bash
    Steps: Run the MCP native smoke command/script and save console output to `.sisyphus/evidence/task-8-native-smoke.txt`.
    Expected: Exit 0 only if environment/build/launch/screenshot/logs/stop all produce evidence; otherwise non-zero with structured error and evidence paths.
    Evidence: .sisyphus/evidence/task-8-native-smoke.txt

  Scenario: Appium gate is non-blocking when unavailable
    Tool: Bash
    Steps: Run smoke workflow on a machine without Appium or with PATH overridden to hide Appium.
    Expected: Workflow records `appium.available: false` and still passes if Tier 1 passed.
    Evidence: .sisyphus/evidence/task-8-appium-gated.json
  ```

  **Commit**: YES | Message: `test(mcp): add native qa smoke workflow` | Files: [`tools/ozi-rs-mcp/tests/**`, `tools/ozi-rs-mcp/scripts/**`]

- [x] 9. Run full repo validation and harden edge cases

  **What to do**: Run final validation commands and fix issues caused by this work. Specifically verify workspace conversion, clippy strictness, existing test suite, Tauri build, MCP self-check, OpenCode JSON, and native smoke evidence. Harden any edge case that fails: missing `just`, path spaces, screenshot/log command unavailable, dropped sessions, repeated stop, Appium unavailable.
  **Must NOT do**: Do not suppress clippy warnings; do not skip failing tests without documenting whether they are pre-existing; do not use `--no-verify` commits.

  **Recommended Agent Profile**:
  - Category: `unspecified-high` - Reason: final integration QA across repo.
  - Skills: [`superpowers:verification-before-completion`] - Required before claiming done.
  - Omitted: [`playwright`] - Browser QA not default.

  **Parallelization**: Can Parallel: NO | Wave 3 | Blocks: Final Verification | Blocked By: Tasks 1-8

  **References**:
  - `justfile:63-68` - `just check`, `just clippy`.
  - `justfile:72-87` - `just test`, targeted tests.
  - Plan acceptance criteria above.

  **Acceptance Criteria**:
  - [ ] `cargo test -p ozi-rs-mcp` passes.
  - [ ] `cargo run --quiet -p ozi-rs-mcp -- --self-check` passes and is JSON-parseable.
  - [ ] `just test` passes.
  - [ ] `just clippy` passes.
  - [ ] `just build` passes.
  - [ ] Native smoke workflow evidence exists under `.sisyphus/evidence/native-qa/`.
  - [ ] `git diff -- .opencode/oh-my-opencode.json` is empty.

  **QA Scenarios**:
  ```
  Scenario: Full validation pipeline
    Tool: Bash
    Steps: Run `cargo test -p ozi-rs-mcp`, `just test`, `just clippy`, and `just build`, saving outputs under `.sisyphus/evidence/task-9-*`.
    Expected: All commands exit 0, or any non-zero exit is proven pre-existing and unrelated with evidence.
    Evidence: .sisyphus/evidence/task-9-full-validation.txt

  Scenario: OpenCode/MCP integration visible
    Tool: interactive_bash / Bash
    Steps: Start OpenCode or MCP Inspector with root `opencode.json` and list MCP tools.
    Expected: Tools prefixed `ozi_rs_` include `qa_environment`, `qa_observe`, and `appium_doctor`.
    Evidence: .sisyphus/evidence/task-9-opencode-mcp-tools.txt
  ```

  **Commit**: YES | Message: `chore(mcp): validate native qa integration` | Files: [all final fixes from validation]

## Final Verification Wave (MANDATORY — after ALL implementation tasks)
> 4 review agents run in PARALLEL. ALL must APPROVE. Present consolidated results to user and get explicit "okay" before completing.
> **Do NOT auto-proceed after verification. Wait for user's explicit approval before marking work complete.**
> **Never mark F1-F4 as checked before getting user's okay.** Rejection or user feedback -> fix -> re-run -> present again -> wait for okay.
- [x] F1. Plan Compliance Audit — oracle
- [x] F2. Code Quality Review — unspecified-high
- [x] F3. Real Manual QA — unspecified-high (+ native MCP tools, not browser Playwright)
- [x] F4. Scope Fidelity Check — deep

## Commit Strategy
- Commit each task separately using the listed messages.
- Do not squash during task execution.
- Do not push unless the user explicitly requests it.
- Never commit secrets, local Appium credentials, absolute machine-specific paths, or generated screenshots/logs unless the user explicitly wants evidence tracked; evidence should normally remain untracked unless repo policy says otherwise.

## Success Criteria
- `ozi-rs-mcp` is discoverable from OpenCode as a local MCP server.
- Future agents can run native desktop QA through MCP tools without opening a browser test harness.
- Tier 1 native QA works without Appium.
- Appium Mac2 tools provide structured degraded results when unavailable and native interactions when available.
- Existing Rust, Vitest, Tauri build, and clippy workflows remain intact.
