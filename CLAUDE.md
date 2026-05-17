# CLAUDE.md

Read `AGENTS.md` first — it contains the full project context, architecture, and conventions.

This file adds Claude Code-specific instructions only.

## Language

Communicate in Russian. Technical terms and code identifiers stay in English.

## Workflow

- Before making changes, read the relevant source files
- For user-visible behavior changes, read `openspec/specs/<capability>/spec.md` and follow the OpenSpec workflow described in `AGENTS.md` ("Behavioral changes via OpenSpec")
- Run `just clippy` and `just test` after code changes
- Keep TypeScript types (`src/lib/types.ts`) in sync with Rust structs manually
- All edits must go through `ProjectCommand` — see `docs/commands-reference.md`

## Verification

Before claiming a desktop fix or feature works, follow `docs/agent-verification.md`.
Playwright is not acceptable evidence for desktop integration (ADR-0024). Two failed
verification attempts → stop and hand back diagnostic dump, do not retry a third time.
