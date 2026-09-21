# State

Read this first. Update it last. History lives in git; this page holds only
what an agent or a returning human needs to pick the work up.

## Where we are

Last merged slice: **the bundle flow stops lying** (2026-09-21) — refusals
carry a reason, loader failures are visible, the progress panel belongs to the
running download, a downloaded bundle is recognised wherever its files sit, and
Cmd-K copes with a thirteen-thousand-row catalogue — after Waypoints tab parity,
track triage, Russian by default, track search, catalogue repair, row density,
dev signing and 0.2 visible fixes. `main` is pushed and
`origin/main` is level with it. Automated gates are green: `just ci` runs
rustfmt, clippy, type-checks, 276 Rust tests and 303 frontend tests;
`cargo audit` is clean; the npm audit gate passes with one documented waiver.
GitHub Actions is green on `main` as of c08e05d — all seven jobs, including the
Windows NSIS bundle and the smoke builds on all three platforms.

The project is working through `openspec/changes/revive-ui-cycle`, which
rebuilds the UI development cycle rather than the UI: agents that change
screens cannot currently see them, most frontend tests assert on source text,
and there is one end-to-end scenario. `openspec/changes/codify-architecture-decisions`
is written and validating but not yet archived.
`openspec/changes/faster-track-triage`, `openspec/changes/waypoints-tab-parity`
and `openspec/changes/honest-bundle-flow` have all their implementation tasks
done and are ready to archive once the owner has used them in the field.

## Next slice

Working goal, set by the owner on 2026-09-21: ozi-rs has to be good enough to
replace OziExplorer in the field, and working with tracks and bundles has to be
fast, comfortable and good-looking. The queue below is ordered by how much of
that it buys, and is meant to be re-read and re-ordered each session rather
than followed blindly.

1. **Say what is already downloaded.** The project list shows thirteen thousand
   identical rows; the backend knows which are cached but the summary does not
   carry the flag. Offline this is the difference between a usable list and a
   guess. See "Bundle flow" in `docs/backlog.md` for the rest of that survey.
2. **A single-map download needs a panel and a cancel.** It sets neither the
   busy flag nor a shared download id, so nothing shows it and nothing stops it.
3. **The rest of slice 0.3** (`revive-ui-cycle` tasks 1b.2, 1b.4, 1b.5): Esc
   discarding a draw without leaving a redo entry, `qa_observe` capturing
   again, and HTTP timeouts on both clients.

Done from slice 0.3 already: bundles-root persistence, export errors reaching
the caller, `.kml` classified as unsupported, the unused `lucide-svelte`
dependency dropped.

## Known broken

- **Slice 0.2 still owes its customer-journey smoke.** Screenshots are done
  and in the gallery; the smoke run has not been driven yet.
- **An Appium click only lands when the app window is frontmost.** A Mac2
  session starts the app but does not raise it, and a click on a background
  window reports success while the event goes to whatever is on top. Run
  `open <path to .app>` first. Other harness facts confirmed on 2026-09-21: the
  session quits the app when it ends; WebDriverAgent is one-shot, so kill
  `WebDriverAgentRunner-Runner` before the next session; window capture is
  `screencapture -x -o -l <windowid>` with the id from
  `CGWindowListCopyWindowInfo` (a five-line Swift script — the `python3` on this
  machine has no `Quartz`); `appium_type_text` into the search field did not
  reach it and triggered a shortcut instead, which is still open.
- **The native-QA MCP server now builds from source** (`.mcp.json` runs
  `cargo run -p ozi-rs-mcp --`, as `opencode.json` already did). Until
  2026-09-21 it ran a binary compiled in May, so July's fixes never reached
  agent sessions. The change takes effect in the next session.
- **MapLibre carries a critical advisory** (`GHSA-jrc7-96c5-q579`) whose fix is
  two major versions ahead. Waived with a reason in
  `scripts/npm-audit-gate.mjs`; the upgrade needs its own slice.
- **Human-facing docs still lie in places** — feature-status and roadmap mark
  sort, crop and split/join as absent although they exist; the command
  reference is missing several commands. Listed in the
  `codify-architecture-decisions` design under "Findings", fixed by its task 2.6.

Localization is done for the shell, the rails, the three library tabs, all four
inspectors, the Maps tab and the command palette. What is still English: the
backend's own status and progress text, which reaches the status bar verbatim
because there is no key-based channel for it. A test now holds both dictionaries to the same key set, so a
half-finished pass fails instead of falling back to English silently.
