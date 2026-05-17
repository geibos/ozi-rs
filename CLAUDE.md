# CLAUDE.md

Read `AGENTS.md` first — it contains the full project context, architecture, and conventions.

This file adds Claude Code-specific instructions only.

## Language

Communicate in Russian. Technical terms and code identifiers stay in English.

## Workflow

- Before making changes, read the relevant source files
- For user-visible behavior changes, read `openspec/specs/<capability>/spec.md` and follow the OpenSpec workflow described in `AGENTS.md` ("Behavioral changes via OpenSpec")
- Run `just clippy`, `just check`, `just lint`, and `just test` after code changes (frontend type-check and lint are wired via SvelteKit + ESLint flat config)
- Keep TypeScript types (`src/lib/types.ts`) in sync with Rust structs manually
- All edits must go through `ProjectCommand` — see `docs/commands-reference.md`

## File deletion

- **Always use `rip` instead of `rm`** (recoverable trash; binary at `/opt/homebrew/bin/rip`). Same flags as `rm`: `rip <path>`, `rip -r <dir>`. Restore via `rip --unbury` if you delete the wrong thing.
- `rm` is intentionally not in the permission allowlist, so attempts will be blocked. Use `rip` for any unlink, including `find ... -exec rip {} \;`.

## Verification

Before claiming a desktop fix or feature works, follow `docs/agent-verification.md`.
Playwright is not acceptable evidence for desktop integration (ADR-0024). Two failed
verification attempts → stop and hand back diagnostic dump, do not retry a third time.
