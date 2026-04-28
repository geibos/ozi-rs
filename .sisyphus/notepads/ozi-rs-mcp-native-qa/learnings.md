# Learnings

## 2026-04-27 - Task 1 workspace/crate foundation
- Root workspace conversion works with `members = ["src-tauri", "tools/ozi-rs-mcp"]` and resolver 3; `cargo metadata --no-deps --format-version 1` reports packages `ozi-rs` and `ozi-rs-mcp`.
- `src-tauri/Cargo.toml` did not need package metadata changes; its existing package name `ozi-rs`, edition 2024, and `ozi_rs_lib` library remained valid in the workspace.
- The new `ozi-rs-mcp` crate can stay stdout-quiet with an `anyhow::Result<()>` main while still testing startup sanity through a small `StartupSanity` helper.
- Running Tauri/Rust verification can refresh generated schema files; future tasks should check `git diff -- src-tauri/gen/schemas` and revert schema drift unless the active task explicitly owns it.

## 2026-04-27 11:34 UTC - Task 2 stdio-safe MCP skeleton
- `rmcp` 1.5 works with `#[tool_router(router = tool_router)]` plus `#[tool_handler(router = self.tool_router)]` for a small tools-only server while still allowing custom `ServerInfo`.
- Keeping `tool_inventory()` backed by `OziRsMcpServer::tool_router().list_all()` verifies the registered tool count/set, while the self-check can present the plan order explicitly.
- `--self-check` stays stdout-clean by parsing the flag before stdio serving and using only `serde_json::to_writer(std::io::stdout(), ...)`; normal MCP mode does not initialize stdout logging.

## 2026-04-27 11:52 UTC - Task 4 evidence fixtures
- Integration tests for `tools/ozi-rs-mcp` need a `src/lib.rs` target; the binary can stay thin by calling `ozi_rs_mcp::types::self_check()` and `ozi_rs_mcp::server::run_stdio_server()`.
- Evidence helpers now keep caller-supplied project roots separate from the fixed relative evidence root `.sisyphus/evidence/native-qa`, and reject absolute paths, `.` components, and `..` traversal before joining child paths.
- Deterministic process fixtures can use the `EvidenceCommand` trait plus `FakeCommand` to exercise stdout/stderr capture without invoking OS tools or contaminating MCP stdout.

## 2026-04-27 12:05 UTC - Task 3 Tier 1 native QA tools
- Repo-root detection is now testable without mutating process environment by using `find_repo_root_from(start, Option<&Path>)`; production still honors `OZI_RS_PROJECT_ROOT` first, then ancestor search for `justfile` plus `src-tauri/tauri.conf.json`.
- Native command construction is exposed through `RealCommand`/`EvidenceCommand`, so tests can assert arg arrays such as `["open", "-n", ".../Ozi RS.app"]` and avoid shell-concatenated command strings for paths containing spaces.
- Tier 1 MCP handlers now return one structured `NativeToolResult` shape with optional environment, evidence metadata, session state, artifact paths, and `error_kind`; Appium handlers remain Task 5 stubs.
- Deterministic Task 3 evidence generation is covered by `tools/ozi-rs-mcp/tests/native.rs`, writing parseable `.sisyphus/evidence/task-3-qa-environment.json` and `.sisyphus/evidence/task-3-missing-artifact.json` without running real Tauri builds or OS UI tools.
- Post-review hardening added preflight evidence file preparation before writes/native screenshot commands, rejects symlink redirects in evidence paths, and uses `osascript -e 'quit app "<app name>"'` derived from the launched `.app` instead of broad `pkill -f ozi-rs` for stop command construction.

## 2026-04-27 - Task 3 Tier 1 native macOS QA tools
- Existing Task 3 implementation covers repo-root detection, command construction without shell concatenation, environment probing, structured missing-artifact launch errors, build failure evidence, and idempotent stop behavior.
- `cargo test -p ozi-rs-mcp --test native -- --nocapture` passed 9/9 tests and refreshed Task 3 evidence files: `.sisyphus/evidence/task-3-qa-environment.json` and `.sisyphus/evidence/task-3-missing-artifact.json`.
- The environment evidence reports separate availability booleans for `just`, `open`, `log`, `screencapture`, and `appium`; Appium is currently unavailable, which remains valid for Tier 1.

## 2026-04-27 - Task 8 native smoke workflow
- Task 8 smoke coverage lives in `tools/ozi-rs-mcp/tests/smoke_workflow.rs` and uses fake `EvidenceCommand` backends so CI/local test runs do not require a real Tauri build, native app launch, screenshot permissions, logs permissions, or Appium.
- The workflow records the required phase order `qa_environment -> build_app -> launch_app -> capture_screenshot -> capture_logs -> stop_app -> appium_doctor` and writes phase artifacts under `.sisyphus/evidence/native-qa/`.
- Since Task 5 Appium handlers are still stubs, the smoke workflow records Appium as gated/degraded with `available: false`, `blocked: true`, and the `not_implemented("appium_doctor")` payload in both `.sisyphus/evidence/native-qa/appium_doctor/result.json` and `.sisyphus/evidence/task-8-appium-gated.json`.

## 2026-04-27 - Task 9 full validation
- Final MCP validation passed: `cargo test -p ozi-rs-mcp`, `cargo run --quiet -p ozi-rs-mcp -- --self-check`, and `python3 -m json.tool .sisyphus/evidence/task-9-self-check.json` all exited 0; evidence is saved in `.sisyphus/evidence/task-9-cargo-test-ozi-rs-mcp.txt`, `.sisyphus/evidence/task-9-self-check.json`, and `.sisyphus/evidence/task-9-self-check.pretty.json`.
- Full repository gates passed with exit 0 for `just test`, `just clippy`, and `just build`; logs are saved as `.sisyphus/evidence/task-9-just-test.txt`, `.sisyphus/evidence/task-9-just-clippy.txt`, and `.sisyphus/evidence/task-9-just-build.txt`.
- Native smoke artifacts are present under `.sisyphus/evidence/native-qa/`; the Task 9 inventory is saved at `.sisyphus/evidence/task-9-native-smoke-evidence.txt`, and MCP tool/config visibility is saved at `.sisyphus/evidence/task-9-opencode-mcp-tools.txt`.
- `.opencode/oh-my-opencode.json` remained preserved for Task 9: `git diff -- .opencode/oh-my-opencode.json` produced no output in `.sisyphus/evidence/task-9-oh-my-opencode-preserved.txt`.

## 2026-04-27 - Task 5.1 Appium degraded adapter micro-slice
- The first compileable Appium adapter slice can stay independent of `server.rs`: `tools/ozi-rs-mcp/src/appium.rs` exposes `AppiumToolResult`, `DEFAULT_APPIUM_SERVER_URL`, PATH-based `appium_doctor()`, deterministic `appium_doctor_with_availability(false)`, and no-session `appium_click_with_session(false)` helpers.
- Deterministic Appium degraded-path tests live in `tools/ozi-rs-mcp/tests/appium.rs`; they do not start Appium, call WebDriver, or require Xcode/Accessibility permissions.

## 2026-04-27 - Task 5 optional Appium Mac2 adapter wiring
- All six Appium MCP handlers in `tools/ozi-rs-mcp/src/server.rs` now return `Json<crate::appium::AppiumToolResult>` and call `appium.rs` helpers instead of `not_implemented("appium_...")`, while preserving the exact 13-tool inventory.
- `tools/ozi-rs-mcp/src/appium.rs` remains deterministic and dependency-free for tests: fake doctor states cover absent Appium, missing Mac2 driver, permissions missing, and ready; action helpers return `session_missing` without a session; fake screenshot evidence is confined through `EvidencePaths` under `.sisyphus/evidence/native-qa/appium_screenshot/`.

## 2026-04-27 - Task 5 optional Appium Mac2 adapter
- Appium MCP handlers now route through `tools/ozi-rs-mcp/src/appium.rs` and return `AppiumToolResult` instead of `not_implemented`; the implementation stays deterministic and never starts real Appium/WebDriver.
- Launch is dependency-gated and reports a degraded optional adapter with the default URL `http://127.0.0.1:4723`; action helpers default to `session_missing` until a session is explicitly available in tests.
- Deterministic coverage in `tools/ozi-rs-mcp/tests/appium.rs` exercises absent Appium, missing sessions, degraded launch, fake doctor states, and fake screenshot evidence.

## 2026-04-27 - Task 8 native smoke refresh after Appium adapter
- Task 8 smoke evidence now uses `appium::appium_doctor_with_availability(false)` and serializes the real `AppiumToolResult` shape, so Appium absence is recorded as `available: false`, `error_kind: dependency_missing`, and `blocked: false` instead of a `not_implemented` stub.
- `cargo test -p ozi-rs-mcp --test smoke_workflow -- --nocapture` and full `cargo test -p ozi-rs-mcp` both passed; combined output is saved in `.sisyphus/evidence/task-8-native-smoke.txt`.

## 2026-04-27 - Task 6 OpenCode MCP registration
- OpenCode config substitution only documents `{env:VARIABLE_NAME}` and resolves missing variables to an empty string, so Task 6 uses the portable relative fallback `OZI_RS_PROJECT_ROOT: "."` instead of an unsupported inline fallback token or a user-specific absolute path.

## 2026-04-27 - Task 9 validation refresh after Appium adapter
- Refreshed stale Task 9 evidence after Task 5/Task 8 completion: `cargo test -p ozi-rs-mcp`, MCP self-check execution, and self-check JSON parsing all exited 0 in `.sisyphus/evidence/task-9-mcp-validation-summary.txt`.
- `.sisyphus/evidence/task-9-appium-blocker-scan.txt` now records the current Appium implementation state: `tools/ozi-rs-mcp/src/appium.rs` exists and no `not_implemented("appium_` source stubs remain under `tools/ozi-rs-mcp/src`.
- `.sisyphus/evidence/task-9-opencode-mcp-tools.txt` lists all 13 MCP tools from self-check, including the six `appium_*` tools; `.sisyphus/evidence/task-9-schema-drift-check.txt` is empty after the targeted generated schema diff check.

## 2026-04-27 - Task 7 native MCP QA docs
- `AGENTS.md` now directs future agents to use project-local `ozi-rs-mcp` native MCP tools by default for desktop Tauri QA while preserving `just test` and `just clippy` verification guidance.
- `docs/testing-strategy.md` now records Tier 1 native QA as build/launch/log/screenshot/stop/observe coverage that does not require Appium; Appium Mac2 remains optional and dependency-gated.
- Playwright/browser testing is documented as not the default for native desktop QA, while still allowed later for intentional isolated web/frontend experiments.
- Required search evidence was saved at `.sisyphus/evidence/task-7-docs-native-mcp.txt` and `.sisyphus/evidence/task-7-playwright-policy.txt`.

## 2026-04-27 - Task 6 OpenCode MCP discovery schema fix
- Root cause for `opencode mcp list` reporting `ozi_rs failed / Failed to get tools` was the Appium click/type handlers using `Parameters<serde_json::Value>`, which generated `AnyValue` input schemas without `type: object`; raw MCP probes tolerated this but OpenCode discovery rejected it.
- `tools/ozi-rs-mcp/src/server.rs` now uses typed Appium click/type parameter structs and a regression test that every registered tool input schema is an object; `opencode mcp list` now reports `ozi_rs connected` in `.sisyphus/evidence/task-6-opencode-mcp-list.txt`.

## 2026-04-28 - Task 8 recovery smoke failure coverage
- `tools/ozi-rs-mcp/tests/smoke_workflow.rs` now parameterizes fake smoke runs with `FakeSmokeScenario`, so the same deterministic helper covers happy path, build exit-code failure, and launch-exited-early handling while still collecting capture logs and Appium gate evidence.
- `cargo test -p ozi-rs-mcp --test smoke_workflow -- --nocapture` passed and refreshed `.sisyphus/evidence/task-8-native-smoke.txt`; Appium remains safely gated with `available: false` and `blocked: false`.

## 2026-04-28 - Task 9 full validation refresh
- Refreshed required Task 9 evidence after Task 8 recovery coverage: `cargo test -p ozi-rs-mcp`, MCP `--self-check`, JSON pretty parsing, `just test`, `just clippy`, and `just build` all exited 0.
- Self-check still reports `server_name=ozi-rs-mcp`, `stdio_safe=true`, exactly 13 tools, and includes `qa_environment`, `qa_observe`, and `appium_doctor`; stderr stayed empty.
- `opencode mcp list` reports `ozi_rs` connected with only the known `uint128` schema warnings; because the CLI does not print individual tools, `.sisyphus/evidence/task-9-opencode-mcp-tools.txt` also records the self-check-derived `ozi_rs_*` inventory.
- Native smoke evidence is present under tool-name directories in `.sisyphus/evidence/native-qa/`: `qa_environment`, `build_app`, `launch_app`, `capture_screenshot`, `capture_logs`, `stop_app`, `appium_doctor`, and `smoke_workflow`.
- Scoped preservation checks stayed clean: `git diff -- .opencode/oh-my-opencode.json` and `git diff -- src-tauri/gen/schemas` both produced empty evidence files.

## 2026-04-28 - Final-wave Appium and smoke repair
- The Appium adapter now keeps Tier 1 independent while making optional Mac2 checks concrete: production `appium_doctor()` runs `appium driver list --installed` and `appium driver doctor mac2`, mapping missing Appium, missing Mac2 driver, doctor/permission failure, and ready states into structured JSON.
- No new dependencies were added; the optional WebDriver path uses a minimal synchronous HTTP client over `std::net::TcpStream` so Appium remains dependency-gated and unavailable local servers return `server_unavailable` instead of panicking.
- Appium launch/actions now have deterministic fake-server coverage for session creation plus session-scoped click/type/stop HTTP calls, while default click/type/screenshot/stop still return `session_missing` when no persisted session exists.
- Smoke workflow evidence now writes the shared `native-qa/smoke_workflow/report.json` atomically only for the successful Tier 1 happy path; failure scenarios write scenario-specific reports and tests serialize shared evidence access with a mutex.

## 2026-04-28 - Final Verification Wave approvals
- F1-F4 all returned `VERDICT: APPROVE`; reviewers confirmed plan compliance, Appium/Tier 1 code quality, native smoke/manual QA evidence, and scope fidelity.
- The plan now has all 13 top-level checkboxes checked and no remaining `- [ ]` top-level tasks.
