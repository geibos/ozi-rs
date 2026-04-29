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
Status: **Resolved by commit `c25c2a9` (2026-04-29).** Verified by unit test
`appium_launch_session_includes_bundle_id_capability`, by direct curl POST to
`/session` with the same capability shape, and by live MCP re-verification on
2026-04-29: `appium_launch_session` created session
`d84a01b9-62e7-46af-9139-2f5b4600a0c7` for bundle
`ru.lizaalert.ozi-rs`.

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
Status: **Resolved by commit `1337926` (2026-04-29).** Verified by unit tests
`capture_screenshot_surfaces_tcc_denial_as_screen_recording_denied` (TCC
substring → `screen_recording_denied` + Screen Recording hint) and
`capture_screenshot_keeps_exit_code_error_kind_for_unrelated_failures`
(unrelated stderr keeps the original `exit_code` semantics).

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

Status: **Open — documented degraded-path/troubleshooting requirement.**

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

Status: **Not reproduced in 2026-04-29 live re-check.** After a fresh live MCP
`appium_launch_session`, `appium_screenshot` captured WebDriver screenshot
evidence successfully for session `d84a01b9-62e7-46af-9139-2f5b4600a0c7`, then
`appium_stop_session` removed the live session. Keep the finding open only for
the stale-session repro path below.

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

## F6 — `webdriver_request` read timeout was 5s, too short for Mac2 session create

Severity: **P0 — discovered while attempting live verification of F1 fix.**
Status: **Resolved by commit `998bc13` (2026-04-29), live MCP re-verified on
2026-04-29.** Mac2 session creation routinely takes 15-30s while the driver
attaches to the target app and probes Accessibility. The previous 5s read
timeout returned `EAGAIN` (`Resource temporarily unavailable`, os error 35)
before the driver could respond, masquerading as `server_unavailable` while the
Appium server was actually live and reachable via curl. Bumped to 60s read /
10s write. The live MCP re-check created session
`d84a01b9-62e7-46af-9139-2f5b4600a0c7` without hitting the timeout.

A drive-by test fix in the same commit replaces a hardcoded
`DEFAULT_APPIUM_SERVER_URL` reference in
`appium_launch_session_available_attempts_webdriver_and_reports_server_unavailable`
with a known-unreachable URL (127.0.0.1:9), so the suite stays fast and
deterministic on developer machines that happen to have Appium running.

## F7 — Mac2 driver does not respond to `/session` after extended use

Severity: P1 — observed during F1 live re-verification.
Status: **Partially resolved / needs focused stale-session follow-up.** Live
MCP verification of F1/F6 passed on 2026-04-29 after clearing a competing direct
WebDriver session. A new regression test covers the “connection accepted but no
HTTP/WebDriver response” failure class and surfaces it as
`webdriver_unresponsive` instead of `server_unavailable`, with a restart/stale
session hint.

After commits `c25c2a9` and `998bc13` landed and the MCP was reconnected,
`appium_launch_session` still returned `server_unavailable` (now correctly
hitting the 60s read budget instead of 5s). A direct curl POST to
`/session` with the exact capability shape the MCP now sends — including
`appium:bundleId: ru.lizaalert.ozi-rs` — also hung past 90s and returned
zero bytes. `/status` keeps reporting "ready" while `/session` accepts the
upload but never replies.

Vs. earlier in the same day: an identical curl POST during the original
F1 diagnostics (commit `2fc915b` window) returned a sessionId in ~5s. The
difference is duration: the session was created, never explicitly deleted
(or partially cleaned up after `brew services restart appium`), and the
driver appears to have entered a state where it cannot service new
sessions.

Hypotheses to test in a fresh session:
1. `brew services restart appium` clears it (most likely).
2. Killing leftover orphan sessions via the WebDriver protocol clears it.
3. The Mac2 driver has a reproducible deadlock when the target app is
   already running before session creation.

Until F7 is reproduced or cleared, F1/F6 fixes stand on:
- unit-test evidence (capability shape, timeout constant)
- the same-day curl evidence that proved bundleId was the missing key
- successful MCP `appium_doctor` confirming the install path works

## F5 — `mcp__computer-use__*` requires Accessibility + Screen Recording grants for its host process

Severity: P2 — alternate channel; not part of the verification protocol but
useful as a fallback when Appium is unhappy. Currently both grants are missing
for the helper process behind computer-use; `request_access` reports the panel
was shown but state stays denied.

Status: **Documented in `docs/native-qa-mcp.md` (2026-04-29).**

Fix sketch:
- Document the three independent macOS TCC grants (Terminal, Claude Code MCP
  host, computer-use helper) in `docs/native-qa-mcp.md` so a fresh install
  knows what to enable up front.

## Status summary (2026-04-29)

| ID | Severity | Status |
|----|----------|--------|
| F1 | P0 | Resolved (commit `c25c2a9`) — unit-tested + live MCP re-verified |
| F2 | P0 | Resolved (commit `1337926`) — unit-tested |
| F3 | P1 | Documented degraded path in `docs/native-qa-mcp.md` |
| F4 | P1 | Open — not reproduced in live re-check; stale-session repro remains |
| F5 | P2 | Documented TCC grant boundaries in `docs/native-qa-mcp.md` |
| F6 | P0 | Resolved (commit `998bc13`) — unit-tested + live MCP re-verified |
| F7 | P1 | Partially resolved — live gate cleared; unresponsive WebDriver now classified |

## Remaining remediation order

1. F4/F7 — keep one focused stale-session repro task: create a session, leave or
   invalidate it, then verify follow-up tools report `webdriver_unresponsive` or
   cleanly require a new session instead of misreporting `server_unavailable`.
2. Re-run `docs/superpowers/plans/2026-04-28-mvp-audit.md` from Task 1.

The application audit can resume as soon as the live verification gate of
F1/F6 passes once. F3/F4/F5/F7 do not block it because the verification
protocol's one-shot anti-loop rule already covers transient driver
weirdness — agents will record `broken/hidden/missing` and move on rather
than retry.
