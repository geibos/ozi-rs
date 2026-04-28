# Agent Verification Protocol

This document is the rule for **how agents prove a desktop change works** in this
repository. It is referenced from `AGENTS.md` and `CLAUDE.md` and is binding on
any agent that claims a fix, an implementation, or that a feature "works."

If you are an agent and you are about to write "fixed", "works", "implemented",
"verified", or any equivalent — read this document first. Without the protocol's
artifacts, you do not have grounds to make that claim.

## TL;DR

> **No Tier 1 + Tier 2 evidence → no claim of "works".** No exceptions for desktop
> integration behaviour. Playwright is not evidence (see ADR-0024). Two failed
> verification cycles → stop and hand back to the user.

## When the protocol applies

- A change touches any UI component, Tauri IPC handler, custom protocol, file
  dialog, or window lifecycle: **full protocol required**.
- A change is purely backend (domain logic, infrastructure adapter, internal
  refactor) with no UI surface: **Tier 1 only is sufficient**, plus the relevant
  Rust unit tests.
- A change is purely frontend with no `invoke()` and no Tauri runtime concerns:
  **Vitest covers it**, but if there is any path to the running app, Tier 1 + Tier 2
  is still required.

If you are unsure which bucket applies, default to **full protocol**. The cost of
running it is minutes; the cost of skipping it is the bug class this protocol exists
to stop.

## The protocol

### Step 0 — Decide what you are verifying

State, in your response or PR description, the concrete claim you are making.
Examples:
- "Clicking 'Maps' opens the bundle list within 1 s on a cached bundle."
- "Importing `examples/short.gpx` adds a track named `short` to the active layer."

A claim must be falsifiable. "Maps now works" is not specific enough; "clicking
'Maps' opens a non-empty list of maps with at least one row" is.

### Step 1 — Build

Use `tools/ozi-rs-mcp`'s `build_app` tool. If the build fails, you do not have a
verifiable artefact yet — fix the build first, then come back here.

### Step 2 — Launch and observe baseline

`launch_app` to start the binary. Immediately call `qa_observe` to capture the
baseline state (logs + screenshot). The baseline screenshot is your "before"
artefact and proves the app reaches the screen where you intend to act.

### Step 3 — Drive the smoke check

Open the smoke document for the feature you changed: `docs/qa/smoke-<feature>.md`.
If the document does not exist yet, you are also responsible for creating it —
this is part of the deliverable, not an excuse to skip verification.

For each step in the smoke document:
- Use Appium tools (`appium_click`, `appium_type_text`) to perform the action.
- After each meaningful action, call `appium_screenshot` (or `qa_observe`) to
  record the resulting state.
- Compare each result against the smoke document's "expected outcome."

If any step's actual outcome does not match the expected outcome, the claim from
Step 0 is **not verified**. Do not write "works."

### Step 4 — Stop and attach evidence

`stop_app`. Then, in your response or PR description, list the absolute paths of
all artefacts produced (screenshots, logs). Do not paraphrase — link the paths.
A reviewer must be able to open them.

### Step 5 — If verification fails

You have one chance to retry. Diagnose, change the code, repeat the protocol from
Step 1 with a new evidence path.

If the second pass also fails, **stop**. Do not start a third attempt. Instead,
respond with:
- the original claim (Step 0),
- the smoke step that failed,
- both attempts' artefacts,
- your current hypothesis about the cause,
- what you would try next, framed as a question.

Hand back to the user. This is the anti-loop rule and it is mandatory.

## What is not allowed

- Claiming a desktop fix based on Playwright output. ADR-0024.
- Claiming a desktop fix based on `cargo test` alone when the change touches UI.
- Skipping Tier 2 because "Appium is optional in `docs/native-qa-mcp.md`." It is
  optional for one-shot exploratory checks; it is **not** optional for the
  verification protocol.
- Writing "should work" or "expected to work" instead of running the protocol.
- A third verification attempt after two failures without explicit user direction.

## Smoke document conventions

Each MVP feature has a smoke document under `docs/qa/`. Naming: `smoke-<feature>.md`.

Required sections:

```markdown
# Smoke: <feature>

## Preconditions
- (e.g.) A LizaAlert bundle with at least Topo + Satellite maps available locally
- App launched via `launch_app`

## Steps and expected outcomes
1. Action — Appium selector or coords
   Expected: visible state change, with measurable property
2. Action ...
   Expected: ...

## Known failure modes
- Failure pattern → likely cause → next diagnostic step
```

`Known failure modes` accumulates over time as bugs are caught and resolved.

## Promotion rule (smoke → Appium test)

When a feature's smoke check fails twice from regression — meaning a fix landed
and a later change broke it again — promote the smoke check from markdown to a
real Appium test under `tools/ozi-rs-mcp/tests/`. CI can then enforce it
automatically.

High-risk features (bundle open performance, large-track rendering) may be
promoted proactively without waiting for two regressions.

## Platform note

The protocol is currently macOS-only. Windows is the primary product platform but
QA tooling for Windows (WinAppDriver / FlaUI / pywinauto) is not yet selected;
the direction-B design spec records this as Future. Until then, Windows-only
issues must be reported by the user manually and reproduced on macOS where
possible.

## Related

- ADR-0024 — Playwright is not used for desktop QA
- `docs/native-qa-mcp.md` — MCP tool inventory and usage
- `docs/testing-strategy.md` — overall test layering
- `docs/superpowers/specs/2026-04-28-qa-debug-process-design.md` — direction-B spec
