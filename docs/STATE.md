# State

Read this first. Update it last. History lives in git; this page holds only
what an agent or a returning human needs to pick the work up.

## Where we are

Last merged slice: **what survives the trip** (2026-09-21) — a track exported
to GPX and read back is pinned as the same track, which is the ground the
planned FTP upload stands on; after one flaky file not being the bundle, where
a
dropped transfer is retried, and the retry resumes with a `Range` request
instead of fetching the file again, and after a
refused edit saying so, where ten failures on
the track- and waypoint-editing path were silent outside dev builds, including
a drag that left the map showing a point that was not there, and after a search
that is gone leaving the list, where a
project taken down upstream finally stops being listed and cached, and the
stand got its catalogue back, and after the catalogue leaving application state,
where `get_app_state` had been shipping thirteen thousand catalogue rows, about
a mebibyte of JSON, on every state change, and after walking the catalogue by
keyboard, where type,
Down, Enter reaches a search in a thirteen-thousand-row virtualized list that
could never have a tab order, and after numbers that mean something, where a
one-point track stopped claiming a distance and a duration, and the catalogue
refresh says how far it has got, and after nothing typed in English, where the map's point
context menu and eight other literals were translated, and a test now fails on
the next one written into a component, and after the rows speaking Russian, where the library
rows' tooltips and accessible names were English inside a Russian window, which
is also what the customer-journey smoke reads, and after one rule written once,
where the same
overlapping-reload bug was found on the map, where a waypoint just added could
disappear again, and the rule extracted to `latest-run.ts` for all five
callers, and after only the newest list winning, where overlapping
reloads in both library tabs could put an older list back on screen, and the
Waypoints tab read its layers one round trip at a time, and after a row for every
track, where the Tracks tab
stopped building its rows out of the map's geometry, which cost every
coordinate of every track to draw a list of names and left a track the map
cannot draw with no row at all, and after the download speaking Russian, where
bundle progress reaches the status bar as a key and its arguments instead of an
English sentence, and after stopping waiting for the catalogue, where the
listing walk that holds the download button disabled for minutes after launch
can be stopped, without passing its first pages off as the whole catalogue,
and after the catalogue keeping your place, where closing
the loader to look at the map no longer throws away the search, the selection
and the place in the list, and after one preview at a time, where clicking through
several projects no longer lets an abandoned preview swap the map list back
under the operator, and a preview stopped releasing a busy flag it never took,
and after when it fails, where offline the project list
says it is the saved one, and the failed-download path was verified rather than
assumed, and after the stand replaying a bundle download, which found that the
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
rustfmt, clippy, type-checks, 309 Rust tests and 375 frontend tests;
`just smoke` cannot run on this machine right now — see "Known broken";
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

1. **Archive the finished OpenSpec changes.** Read the deltas against the
   baseline first: four of this session's were written as ADDED where a
   baseline requirement already said the opposite, and were corrected on
   2026-09-21. `openspec validate --strict` does not check for that. Eleven are implemented, gated and
   waiting on the owner's review: archiving folds their requirements into the
   baseline specs, which is a step for a person, not for an agent.
   `one-preview-at-a-time` is the twelfth and is implemented too, but its
   customer-journey smoke could not run, so it is not ready alongside them.
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
- **The Mac2 driver host still dies at session creation** (2026-09-21, late;
  the authentication block is cleared, the remaining cause is not found).
  `just smoke` fails with "'GET /status' cannot be proxied to Mac2 Driver
  server because its process is not running (probably crashed)".

  What was ruled out, each by running it:

  - *The stale system authentication.* Earlier the WebDriverAgent runner failed
    with `com.apple.LocalAuthentication Code=-4 "System authentication is
    running."` because a `coreautha` agent left over from 16:44 never released
    the session. That agent is gone and the runner now starts silently.
  - *A wedged Appium server.* It had been up since 11:21. Restarted; the
    failure is identical.
  - *The WebDriverAgent build.* The driver's own command,

        cd ~/.appium/node_modules/appium-mac2-driver
        xcodebuild build-for-testing test-without-building \
          -project WebDriverAgentMac/WebDriverAgentMac.xcodeproj \
          -scheme WebDriverAgentRunner COMPILER_INDEX_STORE_ENABLE=NO

    reports `** TEST BUILD SUCCEEDED **`, and WebDriverAgent opens its port
    within **four seconds** — checked by polling 127.0.0.1:10100 while it runs.
  - *A leftover process holding that port.* `lsof -nP -iTCP:10100` is empty and
    no `WebDriverAgentRunner` or `xcodebuild` process is left over.

  So every piece works when run by hand, and only fails when Appium spawns it.
  The next thing to look at is the environment the driver's child process
  inherits — the Appium server used here was started from an agent shell, which
  is not the same environment as a terminal's. **Worth trying first: start the
  server from your own terminal** (`! appium --address 127.0.0.1 --port 4723`)
  and run `just smoke` against that.

  The driver's xcodebuild output was suppressed through all of this, which is
  why the crash said nothing twice over. `appium:showServerLogs` is on now
  (`tools/ozi-rs-mcp/src/appium.rs`), so the next failed session carries the
  reason in the Appium log instead of only the symptom.

  Two attempts is the limit per `docs/agent-verification.md`, and both were
  spent, so the gate was not run a third time.
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
inspectors, the Maps tab, the command palette, bundle progress (which travels
as a key and its arguments) and — since 2026-09-21 — every tooltip and
accessible name on the library rows and the shell's landmarks. Two tests now fail on a
label written into a component — one on a literal `aria-label`, `title` or
`placeholder`, one on English assembled into the same attributes — so the next
one cannot reach a screen. What is still English:
the `AppState` status line's own messages, which no surface currently renders;
convert them the same way if one starts to. A test now holds both dictionaries to the same key set, so a
half-finished pass fails instead of falling back to English silently.
