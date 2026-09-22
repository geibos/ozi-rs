# State

Read this first. Update it last. History lives in git; this page holds only
what an agent or a returning human needs to pick the work up.

## Where we are

Last merged slice: **the menu speaks Russian** (2026-09-22) — nothing read the
words between the tags, so every track row's actions menu was English, its
`Delete` included, along with the simplify dialog; after every toast speaks
Russian, where the third
instance of English reaching the crew became a guard over the shape, and the
guard found sixteen more on its first run, all of them the sentence shown at
the moment something failed; after the import speaks Russian, where the folder
import, which is how a day's recordings arrive, answered with an English
sentence that went straight into a toast; after the day's marks go with it,
where handing the
day over wrote the tracks and left out the waypoints, so the штаб got two files
and the marks arrived separately from the routes; after the colour comes back,
where a GPX round trip
dropped the track's colour, so a day's recordings from three navigators came
back as one red smear; after how old is this list, where the catalogue cache
had written a timestamp since the day it existed and nothing ever showed it, so
offline a list from this morning and one from three weeks ago looked the same;
after one wait does not block the other, where the
launch-time catalogue walk and a bundle download shared one `busy` flag, so the
only download button in the application was disabled for minutes after every
launch; after yesterday's work on the first screen, where a
saved `.ozp` could be opened only from the command palette, so a crew arriving
with yesterday's work had no route back to it from the screen they were looking
at; after open a project and see it, where the camera was
framed on the data only after an import, so opening a saved project left the
map wherever it was, which looks exactly like a project that failed to load;
after what the ground does, where the inspector's
elevation card promised a chart "in a follow-up change" while the elevation had
been in the DTO all along, and now draws it; after the button that was not
there, where the context
bar overflowed its grid column and ran under the inspector, so selecting a
track made Save, Undo, Redo and ⌘K unclickable; after the third way to open a
map, where the command
palette dropped the download id `open_selected_map` returns, so asking it for a
map that was not on disk started an invisible, uncancellable download and never
opened the map; after typing the name you were given, where the
catalogue is spelled in latin transliteration and the filter matched literally,
so a crew typing `Лаврово` got an empty list of thirteen thousand rows; after
the toast that sat on the download, where the
toaster and the download progress panel shared the bottom-right corner, and the
toast always won, hiding the panel's title, its Cancel button and its counters
for as long as it was up; after a waiver that cannot rot, where the MapLibre
advisory's waiver was re-checked against the code and its premise turned into a
test, and after walking the recording, where a track's points
became steppable with the map following, and after trimming the drive to
the start, where a track
became trimmable at a point in either direction as one undoable step, and
after a
link a coordinator sent, where a catalogue
link pasted into the search box opens that search, and after the three on-map
tools, where distance, a
geodesic radius ring and placing a waypoint by bearing and distance all
arrived, reachable from the command palette, and after an old project still opening, where
the `.ozp`
format's tolerance of older files became a test rather than an assumption,
and after whose mark is this, where a waypoint gained
a colour of its own, undoably, drawn by the map and the list alike, and after yesterday's project
being one key away, where the
palette offers the projects most recently opened or saved, and after the project
file becoming `.ozp`, where both dialogs
filtered on `json`, which made an `.ozp` project invisible in the open dialog,
and after a waypoint looking like what it is, where the
symbol a crew picks is drawn on the map marker, not only in the lists and the
exports, and after what survives the trip, where a track and a
waypoint exported to GPX and read back are pinned as the same track and
waypoint, which is the ground the planned FTP upload stands on, and after one flaky file not being the bundle, where
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
rustfmt, clippy, type-checks, 320 Rust tests and 431 frontend tests;
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

## What is left, and who has to do it

Working goal, set by the owner on 2026-09-21: ozi-rs has to be good enough to
replace OziExplorer in the field, and working with tracks and bundles has to be
fast, comfortable and good-looking.

Rewritten on 2026-09-21 after a long session, because the previous queue had
been overtaken. It is ordered by what it buys, and is meant to be re-read and
re-ordered rather than followed blindly.

### Blocked on the owner

1. **Archive the fifty-one OpenSpec changes.** Folding their requirements
   into the baseline is a step for a person. Read the deltas against the
   baseline first: four of this session's were written as ADDED where a
   baseline requirement already said the opposite, and were corrected on
   2026-09-21 — `openspec validate --strict` checks a change's shape, not
   whether it disagrees with the baseline.
2. **Decisions the code is waiting on**, each with its reasoning in
   `docs/backlog.md`: the stop threshold for moving time; whether OZI rasters
   should key their no-data black to transparent, and per map or globally; the
   catalogue's row and badge sizes; where the theme picker goes, since
   `ui-shell` requires one and `ThemePicker.svelte` is imported by nothing;
   and whether a WPT name should carry `&#NNNN;` or `?` for a character
   cp1251 cannot hold — that one needs someone with OziExplorer in front of
   them.
3. **FTP**, when the owner is ready: credentials from a settings form into the
   macOS keychain, an FTP listing adapter behind the interface the HTTP one
   implements, and a source preference. Track upload stands on the GPX round
   trip, which is pinned as of 2026-09-21.

### Blocked on this machine

4. **`just smoke`.** The Mac2 driver host dies at session creation; every
   piece works when run by hand and only fails when Appium spawns it. What has
   been ruled out, and by which command, is under "Known broken". Forty-two
   changes carry an unchecked "smoke green" task waiting on it.

### An agent can still do these

5. **MapLibre 4 → 6**, which clears the waived critical advisory. Deliberately
   not attempted while the E2E gate is down: the map is the product, and the
   stand stubs its tiles, so a major upgrade would ship verified only by type
   checks and a browser. Checked 2026-09-22: vulnerable `<=6.4.0`, fixed in
   6.10.0 only, no 5.x backport, so there is no smaller step. The waiver's
   premise — that nothing here reaches the vulnerable sink — is enforced by
   `src/test/maplibre-waiver.test.ts` rather than by remembering to look.
6. ~~Field tools declared in scope and absent.~~ All three — distance, radius
   ring, projection — were built on 2026-09-22 and live in the command palette.
   Their rendered pixels are unverified; see the change's task 6.1 for why the
   obvious check does not work. The home is the command palette, which
   `product-scope` names as a place a workspace action may live — the mode
   chips stay inert scaffolding per `ui-shell`, and that is not the obstacle it
   looked like.
7. ~~The remaining ADR-0020 items.~~ All of them landed: recent `.ozp` and
   waypoint colour on 2026-09-21; the three on-map tools, open-by-URL,
   trimming a track at a point and the per-point walkthrough on 2026-09-22.
   Trimming is crop-by-selection in two halves, each useful alone.
8. **Upkeep**: point the stand at whatever the next slice touches. Every screen
   of the field cycle has been walked on it at least once.

### Where the rest is

`docs/backlog.md` holds the bundle-flow survey, the engineering items and the
rules this session learned the hard way — the generation stamp for overlapping
reloads, not deriving a list from map geometry, and reading a capability's
existing requirements before writing a delta against it.

## Known broken

- ~~Slice 0.2 owes its customer-journey smoke.~~ `just smoke` passes again
  (18.5s, 2026-09-21). It had been failing since the app started opening in
  Russian: the journey was fine, the test was matching English labels — and
  the visible text at that. WKWebView publishes a control's `aria-label`, not
  the text inside it, so the matchers read the label now and accept either
  language.
- **The Mac2 driver cannot enable automation mode** (2026-09-22; the cause is
  now known and the fix is the owner's). `just smoke` fails at session
  creation with "'GET /status' cannot be proxied to Mac2 Driver server because
  its process is not running (probably crashed)". That is the symptom. The
  cause, visible for the first time because `appium:showServerLogs` is now on
  (`tools/ozi-rs-mcp/src/appium.rs`):

      Failed to initialize for UI testing: Error Domain=com.apple.dt.XCTest.XCTFuture
      Code=1000 "Timed out while enabling automation mode."

  **What this means.** Enabling automation mode is macOS asking for the
  Accessibility grant that lets a test runner drive the interface. It times out
  when the grant is missing for the *responsible* application — the one that
  spawned the chain — or when the permission dialog appeared and nobody
  answered it. The grant attaches to the application that started `appium`, not
  to `appium` itself.

  That explains everything observed. Run by hand from a shell that already has
  the grant, every piece works: `xcodebuild build-for-testing
  test-without-building` reports `** TEST BUILD SUCCEEDED **` and WebDriverAgent
  opens port 10100 in four seconds. Spawned by an `appium` that was started
  from a host without the grant, the same command times out here.

  **What to try, in order:**

  1. Start the Appium server from your own terminal — `! appium --address
     127.0.0.1 --port 4723` — and run `just smoke` against it. If a permission
     dialog appears, answer it.
  2. If it does not appear, add your terminal application (and, if it is listed,
     `node`) under System Settings → Privacy & Security → **Accessibility**, and
     restart the terminal so the grant takes effect.
  3. The stale `coreautha` block from 2026-09-21 is gone and is not this.

  Ruled out, each by running it: a wedged Appium server (restarted twice, one
  of them outside the agent's sandbox — identical failure); the WebDriverAgent
  build; a leftover process holding port 10100 (`lsof -nP -iTCP:10100` empty);
  and the earlier `com.apple.LocalAuthentication` block, which was a different
  error and is cleared.

  Forty-two changes carry an unchecked "smoke green" task waiting on this.
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
- ~~Human-facing docs still lie in places.~~ Fixed on 2026-09-21 (task 2.6 of
  `codify-architecture-decisions`), each claim checked against the code first.
- **`ThemePicker.svelte` is imported nowhere**, while `ui-shell` carries
  requirements for a theme selector and its persistence. Where the control
  belongs is a design call, so it is listed rather than placed. Noted in the
  `codify-architecture-decisions` Findings since 2026-09-19; confirmed still
  true on 2026-09-21.

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
