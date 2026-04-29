# 2026-04-29 — Tooling Audit Findings (pre-feature-audit)

The MVP feature audit defined in `docs/superpowers/plans/2026-04-28-mvp-audit.md`
could not start: every Tier-1 and Tier-2 channel of `tools/ozi-rs-mcp` failed to
deliver evidence end-to-end on the first attempt. Per the anti-loop rule in
`docs/agent-verification.md`, the run was halted after the fourth consecutive
infrastructure failure and the findings are recorded here.

These findings are about the verification *infrastructure*, not the application.
They block the application audit and must be resolved (or at least worked around
in a documented way) before re-running the plan.

## F1 — `mcp__ozi-rs-mcp__appium_launch_session` omits `appium:bundleId`

Severity: **P0 — blocks Tier-2 verification.**

The Mac2 driver requires `appium:bundleId` (or an `app` capability) to know which
application to attach to. The MCP currently sends only:

```json
{ "alwaysMatch": { "platformName": "Mac", "appium:automationName": "Mac2" } }
```

Result: Appium server returns HTTP 500 with "Could not find installed driver to
support given caps" (because the Mac2 driver refuses to attach without an app),
and the MCP surface error is the unhelpful "rejected session creation with HTTP
500."

Fix sketch (`tools/ozi-rs-mcp/src/appium.rs:208`):
- Read a default bundleId from env `OZI_RS_APPIUM_BUNDLE_ID` (default
  `ru.lizaalert.ozi-rs`).
- Add an optional `bundle_id` parameter to `appium_launch_session` so callers can
  override.
- Surface the precise Appium error body in the MCP response, not just the HTTP
  status, so the next failure mode is diagnosable without curl.

Workaround used during the run: `curl -X POST /session` with the full
capability JSON, then write the resulting `sessionId` into
`.sisyphus/evidence/native-qa/appium/session.json` so Tier-2 helpers
(`appium_click`, `appium_screenshot`, etc.) can pick it up. This is brittle and
not protocol-compliant.

## F2 — `mcp__ozi-rs-mcp__capture_screenshot` fails silently when Screen Recording is denied

Severity: **P0 — blocks Tier-1 baseline evidence.**

Calling `capture_screenshot` invokes `screencapture -x` from inside the MCP
server process. macOS denies screen access with "could not create image from
display" when the *MCP server's containing application* (here Claude Code) lacks
Screen Recording permission. The MCP returns `error_kind: "exit_code"` with
`exit_code: 1` but no human-readable hint about TCC.

Notes:
- Bash-launched `screencapture` works because Terminal already has Screen
  Recording.
- The MCP's containing process needs the same grant, granted to a different
  bundle id, in `System Settings → Privacy & Security → Screen Recording`.

Fix sketch (`tools/ozi-rs-mcp/src/native.rs`):
- On `exit_code: 1` from `screencapture`, parse stderr for "could not create
  image from display" and surface a TCC-specific error_kind with install hints
  pointing to the Screen Recording panel.
- Optionally, run a one-time TCC probe via `qa_environment` so the issue is
  diagnosed before the first failed shot.

## F3 — Appium server can launch before Mac2 driver is installed

Severity: P1 — startup ordering issue.

Homebrew starts Appium as a background service after first install. If the
Mac2 driver is installed *after* the server is up, the running Appium does not
re-scan and continues to report "Could not find a driver for automationName
'Mac2'." `brew services restart appium` clears it.

Fix sketch:
- Document this in `docs/native-qa-mcp.md` and link from the verification
  protocol's troubleshooting section.
- Optionally have `appium_doctor` warn when a Mac2 driver is installed but the
  running server's `/status` `build` predates the driver install timestamp.

## F4 — `mcp__ozi-rs-mcp__appium_screenshot` reported "Connection refused" with persisted session

Severity: P1 — needs reproduction.

After persisting a session JSON manually (workaround for F1), calling
`appium_screenshot` returned:

```
"server_url": "http://127.0.0.1:4723",
"error_kind": "server_unavailable",
"message": "Appium server is unreachable: Connection refused (os error 61)"
```

Direct `curl http://127.0.0.1:4723/status` succeeds in the same window. The
MCP's `webdriver_request` may be resolving 127.0.0.1 differently or hitting a
brief race after `brew services restart`. Needs a focused repro.

## F5 — `mcp__computer-use__*` requires Accessibility + Screen Recording grants for its host process

Severity: P2 — alternate channel; not part of the verification protocol but
useful as a fallback when Appium is unhappy. Currently both grants are missing
for the helper process behind computer-use; `request_access` reports the panel
was shown but state stays denied.

Fix sketch:
- Document the three independent macOS TCC grants (Terminal, Claude Code MCP
  host, computer-use helper) in `docs/native-qa-mcp.md` so a fresh install
  knows what to enable up front.

## Recommended remediation order

1. F1 — patch `appium_launch_session` (smallest, unblocks the largest fraction
   of the feature audit).
2. F2 — surface TCC errors clearly so further failures auto-diagnose.
3. F3 + F5 — documentation in `docs/native-qa-mcp.md`.
4. F4 — repro and fix.
5. Re-run `docs/superpowers/plans/2026-04-28-mvp-audit.md` from Task 1.

Until F1 and F2 are fixed, the feature audit cannot honestly produce
verification artefacts. Continuing with curl + manual screencapture would
mean abandoning the protocol — exactly the failure mode it exists to prevent.
