# Archived: the superpowers plans and specs

These eight documents are the plans and designs written between April and May
2026, before this repository used OpenSpec. They are kept because they hold the
reasoning behind decisions that are now requirements, and because four
documents elsewhere still cite them as the source of an audit or a process.

They are **not** normative. `openspec/specs/<capability>/spec.md` is; when one
of these disagrees with a requirement, the requirement governs, and this page
says which requirement absorbed it.

| Document | Where it went |
|---|---|
| `plans/2026-04-12-production-bugs-fix.md` | The fixes shipped; the ones that became rules are in `track-editing` and `project-persistence`. |
| `plans/2026-04-28-mvp-audit.md` | The audit itself. Its findings are `docs/qa/2026-04-28-audit-summary.md` and `docs/qa/triage.md`, which still cite this file as their source. |
| `plans/2026-04-29-mcp-tooling-fixes.md` | The native-QA server's behaviour is `docs/native-qa-mcp.md`; the evidence policy is `visual-verification`. |
| `specs/2026-04-08-documentation-update-design.md` | A one-off resync. The standing rule is `documentation` → "Navigation docs are dated and match the source tree". |
| `specs/2026-04-28-mvp-scope-design.md` | `product-scope`, in full — the "in scope" requirements and the non-goals. |
| `specs/2026-04-28-qa-debug-process-design.md` | The two-channel evidence policy, now `visual-verification`; `docs/agent-verification.md` still cites this as the direction-B spec. |
| `specs/2026-05-17-meetily-inspired-future-work.md` | `docs/backlog.md`. |
| `specs/2026-05-17-shadcn-ui-kit-svelte-design.md` | `ui-shell` — the UI kit, the icon set, the single toast and tooltip hosts. |

New decisions do not go here and do not go in a new ADR. They go in an OpenSpec
change's `design.md` and, once archived, in the capability's decision history
(`documentation` → "Decisions are recorded in OpenSpec, not in new ADR files").
