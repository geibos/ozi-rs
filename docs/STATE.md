# State

Read this first. Update it last. History lives in git; this page holds only
what an agent or a returning human needs to pick the work up.

## Where we are

Last merged slice: **Waypoints tab parity** (2026-09-21) — search, bulk
visibility, isolate, coordinates and a locate button on every row — after track
triage, Russian by default, track search, catalogue repair, row density, dev
signing and 0.2 visible fixes. `main` is pushed and
`origin/main` is level with it. Automated gates are green: `just ci` runs
rustfmt, clippy, type-checks, 269 Rust tests and 295 frontend tests;
`cargo audit` is clean; the npm audit gate passes with one documented waiver.
GitHub Actions is green on `main` as of c08e05d — all seven jobs, including the
Windows NSIS bundle and the smoke builds on all three platforms.

The project is working through `openspec/changes/revive-ui-cycle`, which
rebuilds the UI development cycle rather than the UI: agents that change
screens cannot currently see them, most frontend tests assert on source text,
and there is one end-to-end scenario. `openspec/changes/codify-architecture-decisions`
is written and validating but not yet archived.
`openspec/changes/faster-track-triage` and `openspec/changes/waypoints-tab-parity`
have all their implementation tasks done and are ready to archive once the owner
has used the triage controls in the field.

## Next slice

Working goal, set by the owner on 2026-09-21: ozi-rs has to be good enough to
replace OziExplorer in the field, and working with tracks and bundles has to be
fast, comfortable and good-looking. The queue below is ordered by how much of
that it buys, and is meant to be re-read and re-ordered each session rather
than followed blindly.

1. **Bundle flow.** Opening a project still means going through the cold-start
   route; the maps list and the download popup have not been looked at since
   the catalogue was repaired.
2. **Repair the native-QA click path.** `appium_click` reports success without
   the click landing, and `appium_screenshot` 404s against Appium 3.4.2, so the
   last two slices went in on automated tests alone. Window capture works
   (`screencapture -l <windowid>`); the missing piece is a click that lands and
   a screenshot endpoint that matches the server.

Then the correctness slice 0.3 (`revive-ui-cycle` tasks 1b.1 … 1b.8): bundles
root persistence, Esc discarding a draw, `qa_observe`, HTTP timeouts, export
errors surfaced.

## Known broken

- **Slice 0.2 still owes its customer-journey smoke.** Screenshots are done
  and in the gallery; the smoke run has not been driven yet.
- **Appium clicks do not land.** Sessions start again, but `appium_click`
  returns success for the rail's "Точки" tab twice without the tab changing.
  (`appium_screenshot` returned 404 in the same run, but against a session that
  had already terminated, so that is not evidence about the endpoint.) Window
  capture is fine:
  `screencapture -x -o -l <windowid>`, with the id from
  `CGWindowListCopyWindowInfo` (a five-line Swift script; `python3` here has no
  `Quartz`). A Mac2 session also quits the app when it ends, and its
  WebDriverAgent is one-shot — kill `WebDriverAgentRunner-Runner` before the
  next session.
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

Localization is done for the shell, the rails and the three library tabs. What
is still English: the Track Inspector's field labels, the command palette's own
entries, and the bundle loader. Those are the next translation pass.
