# State

Read this first. Update it last. History lives in git; this page holds only
what an agent or a returning human needs to pick the work up.

## Where we are

**One OpenSpec change is open, and nothing in it is visible to an operator.**
`finish-the-rebuild` carries the three pieces of the development cycle that
were never built: the screenshot matrix, the view-model layer, and a smoke
journey for the six Customer Journeys that have none. Everything else is
archived — `codify-architecture-decisions` and `revive-ui-cycle` both closed on
2026-09-23, the first after reading all thirteen of its capability deltas
against the source, the second after moving its unbuilt requirements out rather
than archiving requirements the code does not meet.

**Verified after the review fixes, 2026-09-23 at 20:36.** The desktop gate
passed against a bundle built from the tree as it stands, and the stand was
re-walked for everything the fixes touched: pressing Рисование then Измерение
leaves only Измерение active; five clicks 90 ms apart make five points; a click
abandoned mid-window makes none and reorders nothing; the grid draws with every
label a sayable coordinate; the replay opens, moves its marker and says when the
moment falls in a silence.

**The desktop smoke gate is green** — both journeys, 2026-09-23 at 20:26,
against a bundle built from the current tree. It went down at 19:41 the same
evening and came back when the Accessibility grant was given again; the log
line that names that failure and the procedure for it are in
`docs/native-qa-mcp.md`, because it is the third time the grant has gone.

**All eight Customer Journeys walk end to end on the stand**, and
`docs/customer-journeys.md` now says what is true rather than what was true in
July. Two desktop journeys run green against a freshly built bundle. What is
still open there: CJ-1 and CJ-8 have not been walked at all, and six CJs have
no desktop journey — the blockers are one line each in
`tools/ozi-rs-mcp/tests/smoke_core_workflow.rs`.

Two OpenSpec changes were left — `codify-architecture-decisions`, which needs
the owner's review before its 63 requirements merge, and `revive-ui-cycle`.
Everything else is archived; three more closed on 2026-09-23
(`a-note-on-the-mark`, `walking-cj4`, `every-export-says-so`,
`a-tolerance-in-metres`).

Last merged slice: **a tolerance in metres** (2026-09-23) — the owner reported
that simplification cleans a track far too hard, and it did: the slider is
labelled «Допуск: {n} м» and its number went into a Rust parameter called
`tolerance` whose unit, recorded in a doc comment three files away, was
kilometres. One metre simplified at one kilometre; there was no setting at
which the control did anything but destroy the track. The unit now lives in the
name — `tolerance_m` / `toleranceM` out to the generated bindings — and a test
reads the bindings to keep it there. Two things had hidden it: the domain tests
all passed kilometres, correctly, so the algorithm was proved right about a
question nobody was asking; and the stand's preview kept every second point
whatever the slider said.

Before it, **every export says so** (2026-09-23): four of the five ways out of
the application were silent on success, and walking CJ-6 found it. The same
walk found three defects in the stand rather than the product — it emitted
`state-changed` after two commands where the application emits after
fifty-four, it served a fixed `project_dirty: false`, and its window registered
the close guard and never fired it. CJ-6 and CJ-7 now walk end to end, and
three tests read the Rust to keep the stand honest.

Fifty-five changes were archived on 2026-09-22 and their requirements are in
the baseline. Two are left: `codify-architecture-decisions`, which needs the
owner's review before its 63 requirements merge, and `revive-ui-cycle`, which
is the ongoing rebuild of the development cycle. What unblocked the batch was
the smoke gate coming back — forty-three of the changes were waiting on nothing
else — and four more were closed by verifying on screen what had been recorded
as unverified: the three on-map measuring tools' rendered pixels, and the
"this map is ready" announcement during a bundle download. Evidence in
`docs/progress/2026-09-22-verification/`.

Last merged slice: **what the review found** (2026-09-22) — an external
reviewer read the two-day run and the project as a whole
(`docs/reviews/2026-09-22/`), and sixteen of its findings survived a reading of
the code. The first was a data loss: partial downloads were named with
`with_extension("part")`, which replaces the extension instead of appending, so
`sheet.map` and `sheet.ozf2` of one bundle wrote to the same file — and the
four tests covering it recomputed the same wrong name, so they passed. The rest
were things that quietly lied (a failed read of a layer's marks erased its
markers; a finished catalogue walk wrote over a running download's status line;
a bundle file matched its map by `ends_with`), things that took the operator's
history (a refused command cleared redo; two drags of one point merged into one
undo step), and two broken promises (CJ-2 says a field launch makes no network
requests, and the app asked for OSM tiles and walked the catalogue on every
start). Four of the sixteen were introduced by the run under review. After
**which line is ЛИСА15** (2026-09-22) — twelve coloured
routes and no names on the map, so the selected track now carries a casing;
finding that it did not work uncovered a Svelte 5 trap in five effects; after a
day on the map, where the stand showed an
import in the list and not on the map, so the palette handed to imported tracks
had never been seen over a basemap; it has now, and it holds; after a colour
for every crew, where a day's folder of
GPX carries no colour, so twenty crews' routes imported as twenty identical red
lines, and each now takes its own from a palette; after a bar with nothing to
say, where the launch
screen's status bar reserved 80px for a download that is not running, an empty
progress track included, and is now one line until there is something to
report; after the stand cannot lie about shape, where two
stand answers in two days had shapes the app does not expect, so the answers
are typed against the generated bindings and the drift is a compile error;
after the menu speaks Russian, where nothing read the
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
rustfmt, clippy, type-checks, 352 Rust tests and 527 frontend tests;
**`just smoke` is green again** (2026-09-22, 19.3s, against a bundle built the
same hour) — the owner granted the Accessibility permission the Mac2 driver had
been timing out on, and the first two runs then failed on labels rather than
behaviour, which is now guarded by `src/test/smoke-label-contract.test.ts`;
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

Rewritten on 2026-09-22 at the end of a long autonomous run, because the
previous queue had been overtaken by the work done since. Ordered by what it
buys; meant to be re-read and re-ordered rather than followed blindly.

**Overtaken again on 2026-09-23.** Most of what follows is done. What the day
actually settled, so the queue below can be read against it:

- Both remaining OpenSpec changes closed. `codify-architecture-decisions`
  merged after all thirteen capability deltas were read against the source —
  seven said something the code does not, and the corrections are in that
  commit. `revive-ui-cycle` closed on what it delivered, with its three
  unbuilt pieces moved to `finish-the-rebuild` rather than archived as met.
- All eight Customer Journeys walk end to end on the stand. CJ-1 and CJ-8 were
  walked for the first time; CJ-8 found two defects and CJ-1 none.
- Everything the OziExplorer reference marks as core is built: calibrating a
  picture, opening a `.map` beside an ordinary image, a coordinate grid,
  distance between marks, track replay, and files attached to a mark.
- The desktop gate is green against a bundle built from the current tree.

What is left is in `finish-the-rebuild` — the screenshot matrix, the view-model
layer and a smoke journey for the six Customer Journeys that have none — plus
the owner decisions listed below. None of it is visible to an operator.

### Blocked on the owner

1. **Review and archive `codify-architecture-decisions`.** Fifty-five of the
   fifty-seven changes were archived on 2026-09-22; this one is left because
   its task 3.1 is the owner reading the proposal, the coverage table, the
   three new capabilities and their Purpose texts before 63 requirements enter
   the baseline. Its task 2.3 — walking each of those 63 against its cited
   evidence — is unfinished, and an agent can do it.

   The batch confirmed why that reading matters. The four "ADDED against a
   baseline that says the opposite" cases the backlog warned about had been
   fixed to MODIFIED except one, and it was in this change: it codified
   `.part` naming as replacing the extension, which is the exact data-loss
   defect fixed the same day. The wording is corrected now, but
   `openspec validate --strict` never saw it — it checks a change's shape, not
   whether it disagrees with the baseline or with the code.

2. **Decisions the code is waiting on**, each with its reasoning in
   `docs/backlog.md`:
   - the stop threshold for moving time, and the same question for ascent and
     descent — one decision covers both;
   - whether OZI rasters should key their no-data black to transparent, and per
     map or globally;
   - the catalogue's row and badge sizes;
   - where the theme picker goes, since `ui-shell` requires one and
     `ThemePicker.svelte` is imported by nothing;
   - whether a WPT name should carry `&#NNNN;` or `?` for a character cp1251
     cannot hold — needs someone with OziExplorer in front of them;
   - **is there such a thing as a new project?** Nothing creates an empty one,
     so a crew finishing one search and starting another keeps adding to the
     same project. Whether that is wrong depends on whether a project is a
     search or a machine's working set;
   - **bundled map glyphs**, which is the price of putting track names on the
     map: SDF PBFs for Cyrillic and Latin, checked in as binary assets, with a
     licence that allows it. A repository-size decision as much as a code one,
     and with a day's routes now drawn in twelve colours it is the largest
     remaining readability gap.
3. **FTP**, when the owner is ready: credentials from a settings form into the
   macOS keychain, an FTP listing adapter behind the interface the HTTP one
   implements, and a source preference. Track upload stands on the GPX and PLT
   round trips, both pinned, and on the day's export, which now carries the
   waypoints with the tracks.

### An agent can still do these

4. **Walk the forty-five changes carrying an unchecked "smoke green" task.**
   The gate runs again, so the box can be ticked — but ticking it honestly
   means running the journey each change touched, not the one core journey.
   `smoke_core_workflow` covers CJ-4's editing spine and nothing else; the
   import, the bundle download, the export and the session restore have no
   packaged-app coverage at all. That gap is the reason two of today's three
   smoke runs failed on wording: nothing else was exercising those labels.

5. **MapLibre 4 → 6**, which clears the waived critical advisory. The gate is
   back, so the reason to defer it is now only its size: the map is the
   product, MapLibre 5 and 6 change the style spec and the marker API, and the
   stand stubs its tiles. The advisory covers `<=6.4.0`, so 6.4.1 is the first
   release without it — `npm audit` names 6.10.0 because that is the latest,
   not the earliest fix — and there is no 4.x or 5.x backport either way.
   `package.json` asks for `^4`; `package-lock.json` resolves 4.7.1. The
   waiver's premise is enforced by `src/test/maplibre-waiver.test.ts`, which
   since 2026-09-22 also covers `attribution`, the sink the advisory is
   actually about.
6. **`get_ozi_tile` is dead IPC surface** — registered, generated into the
   bindings, called by nothing; the map renders OZF2 through the `ozi://`
   protocol. Removing it costs a binding and a registry line.
7. **The legacy element defaults in `app.css`** are now all inside
   `@layer base`, so none of them overrides a component. Deleting them outright
   still needs a pass over the raw `<input>`/`<button>` sites that lean on
   them; the stand makes that cheap.
8. **Upkeep**: point the stand at whatever the next slice touches. Every screen
   of the field cycle has been walked on it, and its answers are typed against
   the generated bindings, so a stub that drifts from a DTO is a compile error.

### Where the rest is

`docs/backlog.md` holds the bundle-flow survey, the engineering items, and the
rules learned the hard way — the generation stamp for overlapping reloads, not
deriving a list from map geometry, reading a capability's existing requirements
before writing a delta, a grid child that can grow its own column, and an
`$effect` that returns before reading its dependencies.

Eight guards now watch defect _classes_ rather than instances: a literal label,
a computed English label, a `catch` that tells nobody, an annotated
`$state(null)`, a typed-in toast message, an effect that gives up before it
subscribes, and — since 2026-09-22 — the labels the customer-journey smoke
matches on, held against the dictionaries, and the map's three
overlapping-reload sites, each pinned to take its run token before the
asynchronous boundary and check it before writing. Each found something on its first
run that the sweep which prompted it had missed; the toast one found sixteen,
and the smoke-label one was written after the same break happened twice in an
afternoon.

## Known broken

- ~~Slice 0.2 owes its customer-journey smoke.~~ `just smoke` passes again
  (18.5s, 2026-09-21). It had been failing since the app started opening in
  Russian: the journey was fine, the test was matching English labels — and
  the visible text at that. WKWebView publishes a control's `aria-label`, not
  the text inside it, so the matchers read the label now and accept either
  language.
- ~~**The Mac2 driver cannot enable automation mode.**~~ Cleared on
  2026-09-22: the owner granted the Accessibility permission and `just smoke`
  passed. The diagnosis below is kept because the failure mode will come back
  on a new machine or after an OS update, and because the grant is the answer.

  Two further things the run taught, both now guarded:
  - The gate matches labels literally, in both languages, and two of the three
    runs failed on wording rather than behaviour — first because the drawing
    toggle's label had been reworded for the Russian plural, then because the
    map canvas selector was the one label never made bilingual, so every map
    click missed while the app was in its default language.
    `src/test/smoke-label-contract.test.ts` now holds the smoke's labels
    against the dictionaries, in a second rather than after a build and a
    launch.
  - `tools/ozi-rs-mcp/tests/smoke_bundle_and_maps.rs` is not a second gate. Its
    own `#[ignore]` says it: superseded, no UI interaction, launches by
    bundleId, which hangs on an unregistered debug bundle.

  The original diagnosis, for when it returns. `just smoke` failed at session
  creation with "'GET /status' cannot be proxied to Mac2 Driver server because
  its process is not running (probably crashed)". That is the symptom. The
  cause, visible for the first time because `appium:showServerLogs` is now on
  (`tools/ozi-rs-mcp/src/appium.rs`):

        Failed to initialize for UI testing: Error Domain=com.apple.dt.XCTest.XCTFuture
        Code=1000 "Timed out while enabling automation mode."

  **What this means.** Enabling automation mode is macOS asking for the
  Accessibility grant that lets a test runner drive the interface. It times out
  when the grant is missing for the _responsible_ application — the one that
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
