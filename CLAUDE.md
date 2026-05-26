# CLAUDE.md

Read `AGENTS.md` first — it contains the full project context, architecture, and conventions.

This file adds Claude Code-specific instructions only.

## Language

Communicate in Russian. Technical terms and code identifiers stay in English.

## Workflow

- Before making changes, read the relevant source files
- For user-visible behavior changes, read `openspec/specs/<capability>/spec.md` and follow the OpenSpec workflow described in `AGENTS.md` ("Behavioral changes via OpenSpec")
- Run `just ci` (or individually: `just clippy`, `just check`, `just lint`, `just test`) after code changes — GitHub Actions runs the same gates on every PR (see `docs/ci.md`); if any of these fail locally, the PR will be blocked
- Make sure `openspec validate <change> --strict` passes for every active change directory — the `openspec-validate` CI job will fail otherwise
- Keep TypeScript types (`src/lib/types.ts`) in sync with Rust structs manually
- All edits must go through `ProjectCommand` — see `docs/commands-reference.md`

## File deletion

- **Always use `rip` instead of `rm`** (recoverable trash; binary at `/opt/homebrew/bin/rip`). Same flags as `rm`: `rip <path>`, `rip -r <dir>`. Restore via `rip --unbury` if you delete the wrong thing.
- `rm` is intentionally not in the permission allowlist, so attempts will be blocked. Use `rip` for any unlink, including `find ... -exec rip {} \;`.

## Verification

Before claiming a desktop fix or feature works, follow `docs/agent-verification.md`.
Playwright is not acceptable evidence for desktop integration (ADR-0024). Two failed
verification attempts → stop and hand back diagnostic dump, do not retry a third time.

### After every native-QA run — ALWAYS clean up

The Mac2 driver Appium uses owns the screen during a session (the system shows a
recording / accessibility indicator and dims the surrounding UI) and a launched
`ozi-rs.app` does not quit on its own when the agent stops issuing actions. If
either is left running, the user sees a dimmed screen and an open app window long
after the verification turn ended. Cleaning up is mandatory, not optional.

At the end of every verification turn (success OR failure), run all three:

1. **`appium_stop_session`** — tear down the active WebDriver session. Without
   this, Mac2 keeps holding the accessibility/screen-recording grants and the
   screen stays dimmed.
2. **`stop_app`** — quit the running `ozi-rs.app`. If `stop_app` is not enough
   (the session indicator dump in `.sisyphus/evidence/native-qa/session.json` still
   shows `running: true` 1-2 seconds after), follow up with
   `pgrep -f "ozi-rs/Contents/MacOS" | xargs -I{} kill -9 {}`.
3. **Verify**: `pgrep -f "ozi-rs/Contents/MacOS"` returns nothing AND
   `curl -s http://127.0.0.1:4723/sessions` returns an empty list. Only then is
   the turn finished.

You do NOT need to stop the Appium server between turns — it is cheap to keep
running on `127.0.0.1:4723`. You DO need to stop sessions and the app every time.
