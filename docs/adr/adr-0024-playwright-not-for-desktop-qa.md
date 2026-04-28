# ADR-0024: Playwright Is Not Used for Desktop QA

- Status: accepted
- Date: 2026-04-28

## Context

`docs/testing-strategy.md` and `docs/native-qa-mcp.md` already say that Playwright is
not the default for native desktop QA — `tools/ozi-rs-mcp` is. Despite that, agents
working on this repository have continued to validate desktop fixes through
browser-based Playwright runs. The browser does not exercise Tauri IPC, custom
protocol handlers (`sqlite://`, `ozi://`), or window lifecycle, so a passing browser
run can — and has — coincide with broken behaviour in the actual desktop app
(for example, the "Maps" button being unresponsive while the same UI renders fine
in a browser).

The cause is documentation drift: the constraint is buried in two reference docs
that agents may not read before claiming "verified." A dedicated ADR makes the
rule citable and refusable.

## Decision

Playwright is **not** an acceptable verification channel for desktop integration
behaviour in this repository. Agents and humans working on the application MUST NOT
claim that a desktop feature "works" or that a desktop bug is "fixed" based on
Playwright or any other browser-only test.

Playwright remains acceptable strictly for **isolated frontend unit checks** that do
not depend on Tauri IPC, custom protocols, native windowing, or backend state — for
example, a Svelte store-reactivity test that runs entirely in the browser without
calling `invoke()`.

All assertions about desktop integration (a click changes state, a panel opens, a
file dialog produces the expected file) MUST come from `tools/ozi-rs-mcp` runs,
either Tier 1 alone (when nothing in the change touches UI) or Tier 1 + Tier 2
Appium when UI is affected. The verification protocol is `docs/agent-verification.md`.

## Consequences

### Positive

- A single citable rule replaces two scattered mentions.
- Agents reviewing this ADR before declaring "fixed" can self-correct.
- Future "verification looked green but the bug is still there" reports have a
  clear root cause to check first.

### Negative

- Where Playwright tests exist for desktop-integration scenarios today, they need
  to be either deleted or downgraded to "frontend-only" coverage (no `invoke()`
  mocks pretending to be the backend).

## Out of scope

- Web/frontend testing strategy in general. Vitest stays as the unit/component
  layer; Playwright stays reserved for the isolated frontend cases described above.
- The Windows and Linux equivalents of `tools/ozi-rs-mcp` (Tier 2). Those are
  declared as Future in the direction-B design spec.

## Related

- ADR-0020 — MVP scope
- `docs/testing-strategy.md`
- `docs/native-qa-mcp.md`
- `docs/agent-verification.md` (verification protocol)
