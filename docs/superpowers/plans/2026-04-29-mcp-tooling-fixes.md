# MCP Tooling Fixes (F1, F2) Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Unblock the MVP audit (`docs/superpowers/plans/2026-04-28-mvp-audit.md`) by fixing the two P0 findings from `docs/qa/2026-04-29-tooling-audit-findings.md`: F1 — `appium_launch_session` omits `appium:bundleId`, and F2 — `capture_screenshot` does not surface a TCC error_kind when macOS denies Screen Recording.

**Architecture:** Two narrow, independent code changes inside `tools/ozi-rs-mcp` driven by TDD. F1 adds a `bundle_id` parameter (with env var default and protocol parameter override) and surfaces the Appium error body. F2 inspects `screencapture` stderr and remaps `exit_code` to a TCC-specific `error_kind` with install hints. The 13-tool inventory contract stays untouched — both are existing tools learning new behaviour, no tools added or removed. After landing, the MVP audit can resume from Task 1, Step 1.3.

**Tech Stack:** Rust 2024 in `tools/ozi-rs-mcp`, `rmcp` for the tool router, `serde_json` for capability JSON, `tempfile` + `FakeCommand` test harness (already used in `tests/native.rs` and `tests/appium.rs`).

---

## Preflight

- [ ] **Step P.1: Verify the working tree is clean and the workspace builds**

```bash
cd /Users/sobieg/Projects/ozi-rs
git status
just clippy
just test-rust
```

Expected: `git status` reports clean working tree (only this plan file may be staged); `just clippy` passes with `-D warnings`; `just test-rust` passes. If any of these fail before you start, stop — investigating preexisting failure is out of scope for this plan.

---

## Task 1: F1 — `appium_launch_session` accepts `bundle_id` parameter

**Files:**
- Modify: `tools/ozi-rs-mcp/src/appium.rs`
- Modify: `tools/ozi-rs-mcp/src/server.rs`
- Modify: `tools/ozi-rs-mcp/tests/appium.rs`

### Step 1.1 — Failing test for explicit `bundle_id` capability

- [ ] **Step 1.1: Write the failing test**

Open `tools/ozi-rs-mcp/tests/appium.rs`. Find the existing
`appium_launch_session_posts_to_webdriver_server` test (around line 149) — copy
its `FakeWebDriverServer` plumbing for the new test. Add this test just below
it:

```rust
#[test]
fn appium_launch_session_includes_bundle_id_capability() {
    let server = FakeWebDriverServer::with_bodies(vec![FakeResponse::json(
        200,
        r#"{"value":{"sessionId":"session-bundle","capabilities":{}}}"#,
    )]);

    let result = appium_launch_session_with_options(
        true,
        &server.url(),
        Some("ru.lizaalert.ozi-rs"),
    );

    assert!(result.ok, "{result:?}");
    let bodies = server.bodies();
    let posted = bodies.first().expect("at least one POST body");
    assert!(
        posted.contains("\"appium:bundleId\":\"ru.lizaalert.ozi-rs\""),
        "POST body missing bundleId capability: {posted}",
    );
}
```

This test depends on a test-only helper `FakeWebDriverServer::with_bodies`
that captures request bodies (the existing one captures only request lines)
and a public adapter `appium_launch_session_with_options` that accepts a
bundle override. Both are introduced in this task.

- [ ] **Step 1.2: Extend `FakeWebDriverServer` to record request bodies**

Still in `tools/ozi-rs-mcp/tests/appium.rs`, replace the existing
`FakeWebDriverServer` struct and impl with this version. The existing
`new(responses)` and `requests()` API stays so old tests keep compiling.

```rust
struct FakeWebDriverServer {
    url: String,
    request_rx: mpsc::Receiver<String>,
    body_rx: mpsc::Receiver<String>,
    handle: Option<thread::JoinHandle<()>>,
}

impl FakeWebDriverServer {
    fn new(responses: Vec<FakeResponse>) -> Self {
        Self::with_bodies(responses)
    }

    fn with_bodies(responses: Vec<FakeResponse>) -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").expect("fake webdriver listener");
        let url = format!("http://{}", listener.local_addr().expect("local addr"));
        let (request_tx, request_rx) = mpsc::channel();
        let (body_tx, body_rx) = mpsc::channel();
        let handle = thread::spawn(move || {
            for response in responses {
                let (mut stream, _) = listener.accept().expect("webdriver connection");
                let mut buffer = [0_u8; 4096];
                let read = stream.read(&mut buffer).expect("read request");
                let request = String::from_utf8_lossy(&buffer[..read]);
                let request_line = request.lines().next().expect("request line");
                let mut parts = request_line.split_whitespace();
                let method = parts.next().expect("method");
                let path = parts.next().expect("path");
                request_tx
                    .send(format!("{method} {path}"))
                    .expect("record request");
                let body = request
                    .split_once("\r\n\r\n")
                    .map(|(_, body)| body.to_owned())
                    .unwrap_or_default();
                body_tx.send(body).expect("record body");
                let status_text = if response.status == 200 {
                    "OK"
                } else {
                    "ERROR"
                };
                let response_text = format!(
                    "HTTP/1.1 {} {}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                    response.status,
                    status_text,
                    response.body.len(),
                    response.body
                );
                stream
                    .write_all(response_text.as_bytes())
                    .expect("write response");
            }
        });

        Self {
            url,
            request_rx,
            body_rx,
            handle: Some(handle),
        }
    }

    fn url(&self) -> String {
        self.url.clone()
    }

    fn requests(mut self) -> Vec<String> {
        if let Some(handle) = self.handle.take() {
            handle.join().expect("fake webdriver finished");
        }
        self.request_rx.try_iter().collect()
    }

    fn bodies(&self) -> Vec<String> {
        self.body_rx.try_iter().collect()
    }
}
```

- [ ] **Step 1.3: Add the public test-only import for the new helper**

At the top of `tools/ozi-rs-mcp/tests/appium.rs`, in the existing import
block from `ozi_rs_mcp::appium`, add `appium_launch_session_with_options`.
The block becomes:

```rust
use ozi_rs_mcp::appium::{
    AppiumCommandOutput, AppiumDoctorState, AppiumProbe, DEFAULT_APPIUM_SERVER_URL,
    appium_click_with_session, appium_click_with_session_id, appium_doctor_for_state,
    appium_doctor_with_availability, appium_doctor_with_probe,
    appium_launch_session_with_availability, appium_launch_session_with_options,
    appium_launch_session_with_server, appium_screenshot_with_fake_image,
    appium_screenshot_with_session, appium_stop_session_with_session,
    appium_stop_session_with_session_id, appium_type_text_with_session,
    appium_type_text_with_session_id,
};
```

- [ ] **Step 1.4: Run the failing test to verify it fails for the right reason**

```bash
cargo test --manifest-path tools/ozi-rs-mcp/Cargo.toml --test appium appium_launch_session_includes_bundle_id_capability -- --nocapture
```

Expected: compile error mentioning `appium_launch_session_with_options` is
unresolved (because the helper does not yet exist). That confirms the test
is wired and ready for implementation.

### Step 1.5 — Implement `appium_launch_session_with_options`

- [ ] **Step 1.5: Add bundle-id resolution and the new helper**

In `tools/ozi-rs-mcp/src/appium.rs`, add the env constant near
`DEFAULT_APPIUM_SERVER_URL`:

```rust
pub const DEFAULT_APPIUM_BUNDLE_ID: &str = "ru.lizaalert.ozi-rs";
```

Replace the existing `appium_launch_session` and
`appium_launch_session_with_server` block with this:

```rust
pub fn appium_launch_session() -> AppiumToolResult {
    appium_launch_session_with_options(
        command_available("appium"),
        &appium_server_url(),
        bundle_id_override(),
    )
}

pub fn appium_launch_session_with_availability(appium_available: bool) -> AppiumToolResult {
    appium_launch_session_with_options(appium_available, DEFAULT_APPIUM_SERVER_URL, None)
}

pub fn appium_launch_session_with_server(
    appium_available: bool,
    server_url: &str,
) -> AppiumToolResult {
    appium_launch_session_with_options(appium_available, server_url, None)
}

pub fn appium_launch_session_with_options(
    appium_available: bool,
    server_url: &str,
    bundle_id: Option<&str>,
) -> AppiumToolResult {
    if !appium_available {
        return appium_missing_result(
            "appium_launch_session",
            format!("Appium is not available on PATH; no session was started for {server_url}"),
        );
    }

    let resolved_bundle = bundle_id.unwrap_or(DEFAULT_APPIUM_BUNDLE_ID);
    let body = json!({
        "capabilities": {
            "alwaysMatch": {
                "platformName": "Mac",
                "appium:automationName": "Mac2",
                "appium:bundleId": resolved_bundle
            }
        }
    });

    match webdriver_request("POST", server_url, "/session", Some(&body)) {
        Ok(response) if response.status_code < 400 => {
            let session_id = extract_session_id(&response.body);
            if let Some(session_id) = session_id {
                let _ = persist_session(server_url, &session_id);
                AppiumToolResult {
                    ok: true,
                    tool: "appium_launch_session".to_owned(),
                    available: true,
                    error_kind: None,
                    missing: Vec::new(),
                    message: Some(format!(
                        "Started Appium Mac2 WebDriver session at {server_url} for {resolved_bundle}"
                    )),
                    session_id: Some(session_id),
                    install_hints: Vec::new(),
                    artifact_paths: Vec::new(),
                }
            } else {
                appium_failure_result(
                    "appium_launch_session",
                    "session_error",
                    format!("Appium server at {server_url} did not return a sessionId"),
                )
            }
        }
        Ok(response) => appium_failure_result(
            "appium_launch_session",
            "session_error",
            format!(
                "Appium server at {server_url} rejected session creation with HTTP {} (body: {})",
                response.status_code,
                truncate(&response.body, 400),
            ),
        ),
        Err(error) => appium_failure_result(
            "appium_launch_session",
            "server_unavailable",
            format!("Appium server at {server_url} is unreachable: {error}"),
        ),
    }
}

fn bundle_id_override() -> Option<&'static str> {
    static OVERRIDE: std::sync::OnceLock<Option<String>> = std::sync::OnceLock::new();
    OVERRIDE
        .get_or_init(|| env::var("OZI_RS_APPIUM_BUNDLE_ID").ok())
        .as_deref()
}

fn truncate(text: &str, max: usize) -> String {
    if text.len() <= max {
        text.to_owned()
    } else {
        let mut trimmed = text[..max].to_owned();
        trimmed.push_str("...");
        trimmed
    }
}
```

- [ ] **Step 1.6: Run the new and existing Appium tests**

```bash
cargo test --manifest-path tools/ozi-rs-mcp/Cargo.toml --test appium -- --nocapture
```

Expected: `appium_launch_session_includes_bundle_id_capability` passes;
the existing
`appium_launch_session_posts_to_webdriver_server`,
`appium_launch_session_reports_unreachable_server`, and
`appium_launch_session_available_attempts_webdriver_and_reports_server_unavailable`
all still pass.

### Step 1.7 — Wire `bundle_id` parameter through the rmcp tool

- [ ] **Step 1.7: Add the `bundle_id` parameter to the rmcp router**

In `tools/ozi-rs-mcp/src/server.rs`, find the existing
`AppiumClickParams` struct definition and add a new sibling struct above the
`tool_router` impl:

```rust
#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct AppiumLaunchSessionParams {
    bundle_id: Option<String>,
}
```

Then find the existing `appium_launch_session` tool method inside
`#[tool_router(router = tool_router)]`. Replace its signature and body:

```rust
#[tool(description = "Launch an Appium automation session")]
fn appium_launch_session(
    &self,
    params: Parameters<AppiumLaunchSessionParams>,
) -> Json<crate::appium::AppiumToolResult> {
    Json(appium::appium_launch_session_with_caller_options(
        params.0.bundle_id.as_deref(),
    ))
}
```

This relies on a thin wrapper `appium_launch_session_with_caller_options`
that bundles the live availability + URL probe with the caller-provided
`bundle_id`. Add it to `tools/ozi-rs-mcp/src/appium.rs` next to
`appium_launch_session`:

```rust
pub fn appium_launch_session_with_caller_options(
    caller_bundle_id: Option<&str>,
) -> AppiumToolResult {
    let resolved = caller_bundle_id.or_else(bundle_id_override);
    appium_launch_session_with_options(
        command_available("appium"),
        &appium_server_url(),
        resolved,
    )
}
```

- [ ] **Step 1.8: Run all `tools/ozi-rs-mcp` tests**

```bash
cargo test --manifest-path tools/ozi-rs-mcp/Cargo.toml -- --nocapture
```

Expected: every test passes. The 13-tool inventory contract test
(`tool_inventory_has_exact_required_names_in_order`) keeps passing because
the tool name `appium_launch_session` is unchanged — only its parameter set
grew.

- [ ] **Step 1.9: Verify the rmcp schema for `appium_launch_session` is still an object**

```bash
cargo test --manifest-path tools/ozi-rs-mcp/Cargo.toml tool_input_schemas_are_objects_for_opencode_discovery -- --nocapture
```

Expected: pass. The schema for the new params is `{ type: "object" }` because
`Option<String>` fields stay valid in `schemars::JsonSchema`.

- [ ] **Step 1.10: Smoke-check via self-check JSON**

```bash
cargo run --manifest-path tools/ozi-rs-mcp/Cargo.toml --release -- --self-check
```

Expected: `tool_count: 13`, `appium_launch_session` is the 9th name. The
self-check does not invoke the tool, only inventories it, so this confirms
discovery still works.

- [ ] **Step 1.11: Just-clippy guard**

```bash
just clippy
```

Expected: clean — no warnings, no errors.

- [ ] **Step 1.12: Commit**

```bash
git add tools/ozi-rs-mcp/src/appium.rs tools/ozi-rs-mcp/src/server.rs tools/ozi-rs-mcp/tests/appium.rs
git commit -m "fix(mcp): pass appium:bundleId in launch_session capabilities

Without bundleId the Mac2 driver returns HTTP 500 with 'Could not find
installed driver to support given caps', because the driver requires an
explicit application target. appium_launch_session now defaults to
ru.lizaalert.ozi-rs, accepts an OZI_RS_APPIUM_BUNDLE_ID env override,
and exposes a bundle_id parameter on the rmcp tool surface so callers
can target other bundles. The HTTP error path also surfaces the
upstream response body so subsequent failure modes are diagnosable
without curl.

Resolves F1 from docs/qa/2026-04-29-tooling-audit-findings.md.
"
```

---

## Task 2: F2 — `capture_screenshot` surfaces TCC denial

**Files:**
- Modify: `tools/ozi-rs-mcp/src/native.rs`
- Modify: `tools/ozi-rs-mcp/tests/native.rs`

### Step 2.1 — Failing test for TCC error_kind

- [ ] **Step 2.1: Write the failing test**

Open `tools/ozi-rs-mcp/tests/native.rs`. Find the existing
`capture_screenshot_creates_artifact_parent_before_command_runs` test (around
line 138). Add this test just below it:

```rust
#[test]
fn capture_screenshot_surfaces_tcc_denial_as_screen_recording_denied() {
    let repo = tempfile::tempdir().expect("repo fixture");
    create_repo_fixture(repo.path());
    let command = FakeCommand::new("screencapture")
        .stderr("could not create image from display\n")
        .exit_code(1);

    let result = capture_screenshot_with_command(repo.path(), &command)
        .expect("screenshot result");

    assert!(!result.ok);
    assert_eq!(
        result.error_kind.as_deref(),
        Some("screen_recording_denied"),
        "expected TCC-specific error_kind; got {:?}",
        result.error_kind,
    );
    let message = result.message.as_deref().unwrap_or_default();
    assert!(
        message.contains("Screen Recording"),
        "expected install hint to mention Screen Recording; got: {message}",
    );
}
```

- [ ] **Step 2.2: Run the failing test**

```bash
cargo test --manifest-path tools/ozi-rs-mcp/Cargo.toml --test native capture_screenshot_surfaces_tcc_denial -- --nocapture
```

Expected: FAIL. Without the fix, `error_kind` is `Some("exit_code")` and
`message` is `None`.

### Step 2.3 — Implement TCC remap in `capture_screenshot_with_command`

- [ ] **Step 2.3: Add the post-command stderr probe**

In `tools/ozi-rs-mcp/src/native.rs`, replace the existing
`capture_screenshot_with_command` function with this:

```rust
pub fn capture_screenshot_with_command(
    repo_root: &Path,
    command: &impl EvidenceCommand,
) -> anyhow::Result<NativeToolResult> {
    let paths = EvidencePaths::new(repo_root);
    let screenshot_path = paths.path_for("capture_screenshot", "screenshot.png")?;
    paths.prepare_file(&screenshot_path)?;
    let mut result = run_native_command(
        "capture_screenshot",
        repo_root,
        command,
        vec![paths.relative_display(&screenshot_path)?],
    )?;

    if !result.ok && result.error_kind.as_deref() == Some("exit_code") {
        if let Some(evidence) = result.evidence.as_ref() {
            let stderr_text = fs::read_to_string(repo_root.join(&evidence.stderr_path))
                .unwrap_or_default()
                .to_lowercase();
            if stderr_text.contains("could not create image from display") {
                result.error_kind = Some("screen_recording_denied".to_owned());
                result.message = Some(
                    "screencapture failed: macOS denied Screen Recording for the MCP host. \
                     Grant access in System Settings → Privacy & Security → Screen Recording, \
                     then restart the MCP client."
                        .to_owned(),
                );
            }
        }
    }

    Ok(result)
}
```

Note: `fs::read_to_string` is from `std::fs` — confirm `use std::fs;` is
already at the top of the file (yes it is, see line 1 of `native.rs`).

- [ ] **Step 2.4: Run the new test**

```bash
cargo test --manifest-path tools/ozi-rs-mcp/Cargo.toml --test native capture_screenshot_surfaces_tcc_denial -- --nocapture
```

Expected: pass.

- [ ] **Step 2.5: Run all native tests to confirm no regressions**

```bash
cargo test --manifest-path tools/ozi-rs-mcp/Cargo.toml --test native -- --nocapture
```

Expected: every test passes. The "happy path" test
`capture_screenshot_creates_artifact_parent_before_command_runs` still passes
because the TCC remap only triggers when `error_kind` is `exit_code`.

- [ ] **Step 2.6: Add a happy-path guard test**

Still in `tools/ozi-rs-mcp/tests/native.rs`, add this test just below the
TCC denial test:

```rust
#[test]
fn capture_screenshot_keeps_exit_code_error_kind_for_unrelated_failures() {
    let repo = tempfile::tempdir().expect("repo fixture");
    create_repo_fixture(repo.path());
    let command = FakeCommand::new("screencapture")
        .stderr("disk full or some unrelated error\n")
        .exit_code(2);

    let result = capture_screenshot_with_command(repo.path(), &command)
        .expect("screenshot result");

    assert!(!result.ok);
    assert_eq!(result.error_kind.as_deref(), Some("exit_code"));
    assert!(result.message.is_none());
}
```

```bash
cargo test --manifest-path tools/ozi-rs-mcp/Cargo.toml --test native capture_screenshot_keeps_exit_code -- --nocapture
```

Expected: pass. Confirms the TCC remap is narrow (only fires on the specific
TCC stderr substring), not a blanket replacement of `exit_code`.

- [ ] **Step 2.7: Just-clippy and full workspace test**

```bash
just clippy
just test
```

Expected: both clean. `just test` covers Rust + frontend.

- [ ] **Step 2.8: Commit**

```bash
git add tools/ozi-rs-mcp/src/native.rs tools/ozi-rs-mcp/tests/native.rs
git commit -m "fix(mcp): surface Screen Recording TCC denial as screen_recording_denied

screencapture exits 1 with 'could not create image from display' when
macOS denies Screen Recording for the calling process. The previous
capture_screenshot result returned error_kind=exit_code with no
message, leaving the agent and the human reviewer without a TCC hint.
The remap is narrow — it only fires when stderr matches the exact TCC
substring — and a guard test pins the unrelated-failure path to keep
returning exit_code with no synthetic message.

Resolves F2 from docs/qa/2026-04-29-tooling-audit-findings.md.
"
```

---

## Task 3: Smoke-verify both fixes against the live environment

**Goal:** Convince ourselves the patched MCP unblocks the original audit step
that failed. This is not part of the persistent test suite — it's a one-shot
manual verification before claiming the plan is done.

**Files:**
- No new files. Manual operations against the running MCP.

- [ ] **Step 3.1: Rebuild the MCP server in release mode**

```bash
cargo build --release --manifest-path tools/ozi-rs-mcp/Cargo.toml
```

Expected: compiles clean. Claude Code's `.mcp.json` already points at
`target/release/ozi-rs-mcp`.

- [ ] **Step 3.2: Reconnect the MCP from the host (if needed)**

In Claude Code, run `/mcp` and reconnect `ozi-rs-mcp` if it disconnected from
the previous session. The tool list must include `appium_launch_session` and
`capture_screenshot`.

- [ ] **Step 3.3: Verify F1 fix end-to-end**

Call `mcp__ozi-rs-mcp__appium_doctor` → expect `available: true, ok: true`.
Then call `mcp__ozi-rs-mcp__appium_launch_session` (no parameters; the env
default is `ru.lizaalert.ozi-rs`).

Expected:
- `ok: true`
- `session_id` is non-null
- `message` mentions `ru.lizaalert.ozi-rs`

If it still returns `session_error`:
- Read the `message` field — it now contains the upstream Appium response
  body. Diagnose from there.

- [ ] **Step 3.4: Verify F2 fix end-to-end**

Pre-condition: leave Claude Code's Screen Recording grant denied (System
Settings → Privacy & Security → Screen Recording). Call
`mcp__ozi-rs-mcp__capture_screenshot`.

Expected:
- `ok: false`
- `error_kind: "screen_recording_denied"`
- `message` mentions System Settings → Privacy & Security

Now grant Screen Recording for Claude Code, restart the client, and call
`capture_screenshot` again.

Expected:
- `ok: true`
- `error_kind: null`
- `artifact_paths` lists the screenshot path.

- [ ] **Step 3.5: Stop the Appium session and clean up**

Call `mcp__ozi-rs-mcp__appium_stop_session`.

Expected: `ok: true`. The persisted `.sisyphus/evidence/native-qa/appium/session.json`
is removed (by the existing `remove_persisted_session` path).

- [ ] **Step 3.6: Update the tooling-audit findings doc**

Open `docs/qa/2026-04-29-tooling-audit-findings.md`. Strike through F1 and F2
with a "Resolved YYYY-MM-DD by docs/superpowers/plans/2026-04-29-mcp-tooling-fixes.md"
line under each finding. Do not delete the findings — the audit history is
what proves the loop was closed.

```bash
git add docs/qa/2026-04-29-tooling-audit-findings.md
git commit -m "qa: mark F1 and F2 as resolved

F1 (appium_launch_session bundleId) and F2 (TCC error surface for
capture_screenshot) are addressed by
docs/superpowers/plans/2026-04-29-mcp-tooling-fixes.md. F3, F4, F5
remain open — they are P1/P2 and do not block the MVP audit.
"
```

---

## Task 4: Update audit plan preflight to reference the patched server

**Files:**
- Modify: `docs/superpowers/plans/2026-04-28-mvp-audit.md`

- [ ] **Step 4.1: Add a note about Screen Recording grant**

Open `docs/superpowers/plans/2026-04-28-mvp-audit.md`. In the Preflight
section under Step P.3, append a new paragraph after the Appium-install
note:

> **Screen Recording grant.** macOS denies `screencapture` to processes
> without Screen Recording permission. `mcp__ozi-rs-mcp__capture_screenshot`
> now returns `error_kind: "screen_recording_denied"` with a hint when this
> happens (see ADR/finding history at
> `docs/qa/2026-04-29-tooling-audit-findings.md`). Grant access to the MCP
> host in *System Settings → Privacy & Security → Screen Recording* before
> running the audit.

- [ ] **Step 4.2: Commit**

```bash
git add docs/superpowers/plans/2026-04-28-mvp-audit.md
git commit -m "docs(plan): add Screen Recording grant to MVP audit preflight"
```

---

## Self-review checklist (executor reads before starting)

- [ ] **Spec coverage:** F1 → Task 1; F2 → Task 2; live verification → Task 3;
      audit plan reference update → Task 4. F3/F4/F5 are explicitly out of
      scope (P1/P2, not blocking).
- [ ] **No placeholders:** every step lists exact file paths, exact code
      snippets to copy, exact commands with expected output.
- [ ] **Type / signature consistency:** the new helper
      `appium_launch_session_with_options` is referenced by name in Task 1
      Step 1.1, defined in Step 1.5, used by `appium_launch_session_with_caller_options`
      in Step 1.7. The rmcp parameter struct `AppiumLaunchSessionParams`
      and its `bundle_id: Option<String>` field are consistent across
      Step 1.7 and the surrounding tool-method body.
- [ ] **Anti-loop discipline:** Task 3 is a one-shot live check. If it fails,
      the executor diagnoses from the new error surface (which is exactly
      what F1 and F2 were supposed to give us); they do not loop on
      "rebuild and retry."
- [ ] **No new tools added:** the 13-tool inventory contract is preserved.
      Both fixes change behaviour of existing tools only.
