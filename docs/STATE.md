# State

Read this first. Update it last. History lives in git; this page holds only
what an agent or a returning human needs to pick the work up.

## Where we are

Last merged slice: **when it fails** (2026-09-21) — offline the project list
says it is the saved one, and the failed-download path was verified rather than
assumed; after the stand replaying a bundle download, which found that the
download button did nothing at all on the path from the workspace, and after a map that is not downloaded being
marked as such in the Maps tab with its size in the tooltip, and after the app getting its padding back, where the unlayered global reset had
been beating every Tailwind spacing utility and fixing it uncovered the
inspector's segments card collapsing, and after
the loader letting the operator choose what a download fetches, the owner's
July type-filter note answered without the app deciding for them, and after
every track in the project exporting to one GPX, the shape FTP upload will
need, and after a map being
openable while the rest of its bundle downloads, which turned out to work
already and only needed saying, and after the first screen speaking Russian,
including the catalogue line that used to be the backend's own English status,
and after the units and instants pass
on the Track Inspector, and after the stand itself, which opens the real screens on fixtures in
a browser so a layout can be looked at without a build, an Appium session or a
window to catch, and after the fixtures the core writes,
whose first run exposed two default layers sharing one id in every fresh
project, and after
waypoints gaining a GPX export beside the OziExplorer WPT one,
the bundle download learning to state its weight, every map row stating its
size, slice 0.3 closing (timeouts, Esc discarding
a draw, `qa_observe`, exports returning errors), the project list marking
downloaded bundles, single-map downloads getting a panel and a cancel, and the
bundle flow slice (refusals carry a reason, loader failures are visible, the
progress panel belongs to the running download, a downloaded bundle is
recognised wherever its files sit, Cmd-K copes with the full catalogue),
Waypoints tab parity,
track triage, Russian by default, track search, catalogue repair, row density,
dev signing and 0.2 visible fixes. `main` is pushed and
`origin/main` is level with it. Automated gates are green: `just ci` runs
rustfmt, clippy, type-checks, 293 Rust tests and 340 frontend tests;
`just smoke` passes;
`cargo audit` is clean; the npm audit gate passes with one documented waiver.
GitHub Actions is green on `main` as of c08e05d — all seven jobs, including the
Windows NSIS bundle and the smoke builds on all three platforms.

The project is working through `openspec/changes/revive-ui-cycle`, which
rebuilds the UI development cycle rather than the UI: agents that change
screens cannot currently see them, most frontend tests assert on source text,
and there is one end-to-end scenario. `openspec/changes/codify-architecture-decisions`
is written and validating but not yet archived.
`openspec/changes/faster-track-triage`, `openspec/changes/waypoints-tab-parity`,
`openspec/changes/honest-bundle-flow` and `openspec/changes/see-what-is-downloaded`
have all their implementation tasks done and are ready to archive once the owner
has used them in the field.

## Next slice

Working goal, set by the owner on 2026-09-21: ozi-rs has to be good enough to
replace OziExplorer in the field, and working with tracks and bundles has to be
fast, comfortable and good-looking. The queue below is ordered by how much of
that it buys, and is meant to be re-read and re-ordered each session rather
than followed blindly.

1. **Archive the finished OpenSpec changes.** Ten are implemented, gated and
   waiting on the owner's review: archiving folds their requirements into the
   baseline specs, which is a step for a person, not for an agent.
2. **The screens of the field cycle have been walked** on the stand: the
   cold-start route, the Track Inspector, the Waypoints row menu, the shell at
   1024×640, a download in flight, a download that fails and a catalogue that
   cannot be reached. What is left there is upkeep — point it at whatever the
   next slice touches.

The rest of the bundle-flow survey is under "Bundle flow" in `docs/backlog.md`.

Slice 0.3 is done: bundles-root persistence, Esc discarding a draw without a
redo entry, map layer ids, `qa_observe`, HTTP timeouts, export errors reaching
the caller, `.kml` classified as unsupported, and the unused `lucide-svelte`
dependency dropped.

## Known broken

- ~~Slice 0.2 owes its customer-journey smoke.~~ `just smoke` passes again
  (18.5s, 2026-09-21). It had been failing since the app started opening in
  Russian: the journey was fine, the test was matching English labels — and
  the visible text at that. WKWebView publishes a control's `aria-label`, not
  the text inside it, so the matchers read the label now and accept either
  language.
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
