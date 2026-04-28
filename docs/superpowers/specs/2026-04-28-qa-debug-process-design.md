# QA / Debug Process — Design Spec (direction B)

- Date: 2026-04-28
- Status: design accepted; audit not yet executed
- Authoritative decisions: ADR-0024, `docs/agent-verification.md`

## Purpose

This spec exists because the current MVP scope (ADR-0020) cannot be acted on. The
documentation says most MVP features are implemented; the user reports most of them
do not respond in the UI. Agents have repeatedly claimed "fixed" without changing
behaviour. This spec defines:

1. The **verification protocol** every agent follows before claiming a desktop
   change works.
2. The **audit** that produces the first verified status of every MVP feature.
3. The **smoke document** convention that turns repeatable QA into shareable
   knowledge.
4. The **promotion path** from manual smoke check to enforced Appium test.

## Problem statement

Three intertwined failures were observed:

- **Detection failure.** Tier 1 of `tools/ozi-rs-mcp` (build / launch / log /
  screenshot) reports green when the app starts. It does not exercise UI inputs.
  Bugs of the form "button click does nothing" pass undetected.
- **Channel mismatch.** Agents have used Playwright on the frontend dev server.
  This bypasses Tauri IPC and custom protocols entirely, so Playwright green can
  coexist with desktop red. ADR-0024 forbids this for desktop claims.
- **Loop pattern.** When verification cannot detect failure, an agent fixes a
  symptom, sees green, claims done, and the user reports the same bug. The agent
  retries the same approach. There is no rule that breaks the loop.

## Approach (selected: hybrid)

We start with manual smoke checks documented in markdown (low-cost, immediate
coverage), and promote the most failure-prone checks to Appium tests as evidence
accumulates. A hard verification gate already applies in the form of
`docs/agent-verification.md`; CI-level enforcement comes when Windows tooling
lands or when a feature's regression rate justifies it.

### Why not all-Appium from day one

- Each Appium test costs ~30–60 minutes to author and stabilise. Writing tests
  for ~25 MVP feature lines before knowing which actually need them is wasted
  effort if half the features turn out to be hidden in UI rather than broken.
- Appium reliability on Mac2 is moderate. False fails would erode the gate's
  authority before agents trust it.

### Why not pure markdown checklists forever

- Markdown checklists rely on agent discipline. They are necessary but
  insufficient for high-traffic features.
- Regressions in core flows (bundle open, track rendering) deserve automated
  detection.

## Components

### Verification protocol

`docs/agent-verification.md`. Binding on every agent. Five steps: state claim,
build, observe baseline, drive smoke, stop and attach evidence. Two-attempt
anti-loop rule: after a second failed verification, the agent stops and hands the
diagnostic dump to the user instead of retrying.

This document is referenced from `AGENTS.md` and `CLAUDE.md` so an agent reading
the canonical entry points cannot miss it.

### Smoke documents

`docs/qa/smoke-<feature>.md`. One per MVP feature line. Structure: preconditions,
steps with selectors and expected outcomes, accumulated known failure modes.
Owned by whoever last verified the feature; updated when failure modes are
observed.

Initial set produced as part of the audit (below). Anyone implementing a new MVP
feature must produce or update its smoke document as part of the change.

### Audit (one-shot)

A walk through every MVP feature line from ADR-0020 using `tools/ozi-rs-mcp`.
For each line:

1. Identify the UI entry point (button, menu, keyboard shortcut).
2. Run the verification protocol against it: build → launch → drive → observe →
   stop. The smoke document is created on first pass.
3. Classify the result as one of:
   - **works** — observed end-to-end, smoke passes
   - **partial** — works for happy path but fails edge cases recorded in the
     smoke document
   - **broken** — UI entry point is reachable but action does not produce
     expected outcome
   - **hidden** — backend exists but no UI entry point can be found
   - **missing** — neither UI nor backend
4. Capture artefacts under `tools/ozi-rs-mcp` evidence root; link from the
   feature-status row.

The audit deliverable is:

- updated `docs/feature-status.md` with the verified status column,
- a triage list ordered by criticality to the MVP workflow,
- 8–12 initial smoke documents (whichever MVP lines were exercised in the audit).

The audit is the first implementation step under direction B.

### Promotion path

Two triggers move a smoke document into a real Appium test under
`tools/ozi-rs-mcp/tests/`:

1. **Two regressions in a row.** A fix lands; a later change re-introduces the
   same failure. The cost of authoring the test is justified once the feature has
   demonstrated repeat fragility.
2. **High-risk features.** Three features are promoted proactively without waiting
   for regressions: bundle open / list (perf-sensitive), map switching (root cause
   of one of the user's known critical bugs), and track loading at scale (the
   tens-of-thousands-of-points performance constraint from ADR-0020).

### CI gate (Future, not now)

When automated Appium tests cover the high-risk set, the smoke workflow under
`tools/ozi-rs-mcp/tests/smoke_workflow.rs` becomes a required check on PRs. This
adds CI cost (a Mac runner) and is deferred until the protocol's value is proven
by the audit.

## Out of scope for direction B

- **Windows and Linux QA tooling.** Windows is the primary product platform but
  Tier 2 tooling for it (WinAppDriver / FlaUI / pywinauto) is not selected. Linux
  Tier 2 (at-spi / dogtail) is also deferred. Today's protocol is macOS-only.
  Until tooling lands, Windows-specific issues require manual reporting and
  cross-platform reproduction on macOS where possible.
- **Performance budgets and telemetry.** No SLOs, no perf gates beyond the
  qualitative "instant for bundle open, smooth for tens-of-thousands-of-points
  tracks" stated in ADR-0020. A later spec may quantify these with the audit's
  measurements as input.
- **Crash reporting / error budgets.** Out of scope.
- **Frontend unit-test strategy.** Vitest already covers what it covers; this
  spec does not change that. Playwright is constrained by ADR-0024 to isolated
  frontend cases.

## Acceptance for this spec

The spec is "done" when:

- ADR-0024 is filed and accepted.
- `docs/agent-verification.md` exists and is referenced from `AGENTS.md` and
  `CLAUDE.md`.
- The audit has executed and produced the deliverable described above.
- At least 8 smoke documents under `docs/qa/` cover the highest-risk MVP lines.
- One Appium test is promoted from a smoke document to demonstrate the path.

The first three are blocking for direction A. The smoke documents and the first
promotion can land progressively.

## Related

- ADR-0020 — MVP scope (the target)
- ADR-0024 — Playwright not for desktop QA (the rule)
- `docs/agent-verification.md` — the protocol
- `docs/native-qa-mcp.md` — the tooling
- `docs/testing-strategy.md` — overall layering
