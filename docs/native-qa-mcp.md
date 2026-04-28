# Native Desktop QA via `ozi-rs-mcp`

`tools/ozi-rs-mcp` is a project-local Model Context Protocol server that exposes 13 tools for driving the ozi-rs desktop app during automated QA. It is the **default** native QA channel for this project (see `docs/testing-strategy.md`); Playwright is not the default for desktop behavior.

## Why a local MCP server

Tauri apps expose no remote testing protocol on their own. The MCP server wraps platform commands (`open`, `log`, `screencapture`, `xcrun`, optionally `appium`) as MCP tools so an AI agent or other MCP client can build, launch, observe, and stop the app, capturing artifacts to a known evidence root.

## Layout

```
tools/ozi-rs-mcp/
├── Cargo.toml          # Edition 2024. Crates: rmcp, tokio, anyhow, schemars.
├── src/
│   ├── main.rs         # Tokio entry. `--self-check` prints inventory JSON and exits.
│   ├── lib.rs          # Module roots.
│   ├── server.rs       # rmcp ToolRouter; declares the 13 tools and `tool_inventory()`.
│   ├── native.rs       # Tier 1: build/launch/log/screenshot/observe.
│   ├── appium.rs       # Tier 2: optional Appium Mac2 session control.
│   ├── process.rs      # Subprocess + evidence capture helpers.
│   ├── evidence.rs     # Evidence-file metadata + paths.
│   ├── config.rs       # `repo_root()` resolution and `OZI_RS_PROJECT_ROOT` env.
│   └── types.rs        # `self_check` shape, `ToolMetadata`.
└── tests/              # Per-area integration tests + smoke_workflow.
```

## Tool inventory

Tier order matters: Tier 1 (native) must always be runnable; Tier 2 (Appium) is dependency-gated and may degrade gracefully.

### Tier 1 — Native (always available)

| Tool | Purpose |
|------|---------|
| `qa_environment` | Report platform, repo root, evidence root, and which optional tools (`just`, `open`, `log`, `screencapture`, `appium`) are detected. |
| `build_app` | Build the Tauri app (release, no bundle). Streams stdout/stderr to evidence. |
| `launch_app` | Launch the built binary. Records launch state. |
| `stop_app` | Terminate a previously launched binary. Records stop state. |
| `capture_logs` | Pull recent app logs via `log` (macOS unified logging). |
| `capture_screenshot` | Capture a window screenshot via `screencapture`. |
| `qa_observe` | Combined observation: log tail + screenshot in one call. |

### Tier 2 — Appium Mac2 (optional)

| Tool | Purpose |
|------|---------|
| `appium_doctor` | Diagnose Appium availability and driver readiness. |
| `appium_launch_session` | Start an Appium Mac2 session attached to the running app. |
| `appium_click` | Click a UI element. Optional `selector` parameter (XPath/Accessibility). |
| `appium_type_text` | Type text into the focused element. Optional `selector` and `text`. |
| `appium_screenshot` | Screenshot via the Appium session. |
| `appium_stop_session` | Tear down the Appium session. |

If Appium dependencies are missing, the Tier 2 tools return a structured error with `error_kind` set instead of failing loudly. Treat that as an acceptable degraded path.

## Repo-root resolution

`config::repo_root()` finds the ozi-rs repository:

1. If `OZI_RS_PROJECT_ROOT` is set and points to a directory containing both `justfile` and `src-tauri/tauri.conf.json`, use it.
2. Otherwise walk ancestors of the current working directory until both files are found.

This means the server can be launched from any subdirectory and still work, as long as it is inside the repository — or set the env var explicitly when running it from outside.

## Evidence

Every tool that runs a subprocess writes structured evidence (stdout, stderr, exit status, timing) under the evidence root reported by `qa_environment`. Returned `NativeToolResult` / `AppiumToolResult` objects include `artifact_paths` so the caller can fetch the captured artifacts.

The `.sisyphus/evidence/` tree in this repo holds historical evidence from past sessions; do not rely on its contents for new runs — generate fresh artifacts.

## Self-check

```
cargo run --bin ozi-rs-mcp -- --self-check
```

Prints a JSON document describing server identity (`ozi-rs-mcp`), `stdio_safe: true`, the 13 tool names in canonical order, and `tool_count: 13`. Use it to verify the build before wiring the server into a client.

## Registering with an MCP client

The repo includes `opencode.json` and `.opencode/oh-my-opencode.json` that wire `ozi-rs-mcp` into opencode. For other clients, run the binary over stdio:

```jsonc
{
  "mcpServers": {
    "ozi-rs-mcp": {
      "command": "cargo",
      "args": ["run", "--manifest-path", "tools/ozi-rs-mcp/Cargo.toml", "--bin", "ozi-rs-mcp"],
      "env": { "OZI_RS_PROJECT_ROOT": "/absolute/path/to/ozi-rs" }
    }
  }
}
```

Use a release build (`cargo build --release --manifest-path tools/ozi-rs-mcp/Cargo.toml`) and point at the produced binary if startup latency matters.

## Tests and contracts

- `tools/ozi-rs-mcp/tests/native.rs`, `tests/appium.rs`, `tests/evidence.rs`, `tests/smoke_workflow.rs` cover happy paths and degraded paths.
- `tool_inventory_has_exact_required_names_in_order` enforces the 13-tool contract: changing tools requires updating both `REQUIRED_TOOL_NAMES` (server.rs) and `EXPECTED_TOOLS` (main.rs tests). Drift will fail the build.
- `tool_input_schemas_are_objects_for_opencode_discovery` ensures every tool's input schema is a JSON object — opencode rejects non-object schemas at discovery time.

## When to use Tier 1 vs Tier 2

| Situation | Tier |
|-----------|------|
| Verify the app launches at all | Tier 1: `build_app` → `launch_app` → `qa_observe` → `stop_app`. |
| Log/screenshot regression after a change | Tier 1: `qa_observe`. |
| Drive a UI flow (click, type) | Tier 2: `appium_*`. Skip and document degraded path if `appium_doctor` reports missing deps. |
| CI without Appium installed | Tier 1 is sufficient for the testing-strategy gate. |

Playwright is not used for native desktop QA. It may still be added for *isolated* web/frontend experiments, but it does not replace this MCP for app-behavior checks.
