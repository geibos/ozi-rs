# State

Read this first. Update it last. History lives in git; this page holds only
what an agent or a returning human needs to pick the work up.

## Where we are

**2026-10-01: the owner answered the questions this page was waiting on**, and
the run that followed works through them. The decisions, with what became of
each:

- **Bundle open-source glyphs** — done: `names-along-the-route` (below).
- **A project is one search.** «Новый поиск» already existed (2026-09-23,
  `a-new-search`). Done on 2026-10-02 (`another-search-another-project`):
  opening a map of another search over a project with work in it starts a new
  project; unsaved work gets the close guard's three answers first, saved work
  needs none. The close guard's dialog was 312px wide and its Russian answers
  ran past it — widened.
- **Moving time and ascent/descent**: a stop is moving less than 25 m in two
  minutes, a climb counts only past 5 m of rise; both thresholds are settings.
  Done on 2026-10-02 (`settings-and-time-on-the-move`), shown in the track
  inspector.
- **No transparent no-data on OZI rasters** — rasters are drawn as they are.
  Recorded in `docs/backlog.md`.
- **The theme picker's place is ours to choose**: a Settings screen, which the
  FTP accounts and the thresholds need anyway; the palette keeps its entries.
  Done on 2026-10-02: the gear in the top bar, «Настройки…» in the palette,
  ⌘, — theme, language and the statistics thresholds.
- **FTP: credentials and hosts.** Bundles come from one account; results go to
  up to four endpoints, each with its own account. Passwords in the keychain.
  Done on 2026-10-02 (`ftp-accounts`): the FTP section of the settings screen
  holds one bundle account and any number of result endpoints, the passwords
  go to Keychain / Credential Manager under «ozi-rs FTP», and each account can
  be checked (connect, log in, change folder). Not yet: the FTP bundle listing
  and uploading results, which stand on this.
- **Commit and push after every green slice**, for this run.

**Desktop verification, 2026-10-02 at 10:30–10:40**, against a bundle built
from `fe06e6a` (and GitHub CI green on that commit, after nine red days — see
below):

- `smoke_cj5_draw_track` and `smoke_report_capture_moment` green.
- `smoke_cj3_layer_management` failed twice and was not run a third time, per
  the two-attempts rule: the first time the Mac2 driver died at session start,
  the second time its click on the Tracks tab landed on an inactive window and
  the tab never opened. The machine was in use at the time (input idle 0 s),
  which is the known way a Mac2 click goes astray. Owed: one run with nobody
  at the keyboard.
- MapLibre 6's worker runs in the packaged application. Proved without
  touching the interface: a throwaway build (not committed) waited for the
  `tracks` GeoJSON source to report itself loaded — which needs the worker to
  answer — and wrote a report folder saying so. It said `worker-ok` with the
  restored project's 26 tracks in the state; the folder was removed and the
  bundle rebuilt from the tree.
- `just build` failed at the DMG step (`bundle_dmg.sh`) in two builds of
  three, after the `.app` was already written — and a failed recipe never
  reached `sign-dev`, so the smoke would have run an unsigned bundle. The
  debug build makes the `.app` only now (`--bundles app`); release DMGs come
  from CI. Why `bundle_dmg.sh` fails here was not looked into.
- The FTP passwords' credential store, for real: `cargo test --lib
system_credential_store -- --ignored` files, reads back and deletes a
  password under «ozi-rs FTP» in the login keychain through `SystemSecrets`.
  Green. Not done: saving through the form in the packaged application.

**GitHub CI was red from 2026-09-23 to 2026-10-02** while `just ci` passed
locally, because the local gate runs on macOS: clippy on Linux failed on
`capture_rect`, used only on macOS. Since 2026-10-01 the npm audit gate also
failed, on a Next.js advisory reaching us through the `geist` font package.
Both fixed in `fe06e6a`; the Geist fonts stay vendored in `static/fonts/`.

**«Упростить» = фильтр Ozi с индексом 4** (`filter-like-ozi-index-4`,
2026-10-06). The owner recorded processing a real search in OziExplorer
(`docs/field-notes/2026-10-06-track-processing-in-ozi.md`) and gave eleven
original/filtered track pairs. Index 4, which the standard prescribes and
nobody could explain, is Douglas–Peucker at about 2 m: ours keeps 6671 points
where Ozi kept 6672 and agrees on 97–98 % of them. The panel opens at 2 m and
says so. Two defects fixed on the way: the perpendicular distance in raw
degrees (17 % long at latitude 60) and the track menu staying open over the
panel.

**Скачки и выбросы** (`jumps-and-outliers`, 2026-10-06), the next thing from
the same recording. The points table shows under each point the distance and
speed from the previous one — Ozi's Dist and KPH, which the wiki reads to find
an outlier. Above the table the inspector lists the track's jumps (a leg of
100 m and ten median steps) and outliers (a point the track goes out to and
comes straight back from); choosing one selects it, centres the map and the
table on it. A jump offers «Разделить здесь», an outlier «Удалить вершину» and
«Удалить и разделить» — the last a new command, `CutOutTrackPoint`, one undo
step. Thresholds measured on the owner's own cleaning of the eleven tracks:
26 of his 32 breaks (one more was already a GPX break) and 15 of his 20
deleted points are on the list, 112 entries over the eleven tracks, 59 of them
places he edited. The stand serves such a track with `?track=dirty`. Next from
the recording: name fix-up by the standard's transliteration, colour by group
type, each track to its own PLT in one go, a 20 000-point track.

**The screenshot matrix exists** (`finish-the-rebuild` group 1, 2026-10-02).
`just shots` photographs fifteen screen states — the launcher in all five
states, the library tabs, the track inspector clean and with jumps to clean,
the palette, settings — in both languages and both themes, inside the
Playwright Docker image so a Mac and the CI runner produce the same pixels,
and compares them with the 64 committed baseline shots in
`src/test/stand/baseline/`. It is part of `just ci` and a CI job of its own.
Two fresh runs matched 60/60 on 2026-10-02; on 2026-10-06 one of two runs
matched 64/64 and the other failed `library-tracks__error__en__light` (5736
pixels; its diff was overwritten before it was looked at), and an earlier run
that day timed out on `settings__loaded__ru__dark` («did not hold still») —
neither screen had changed, so the matrix has an intermittent miss somewhere
that is not yet understood. Accepting a
deliberate change is `just shots --update` in a commit of its own. It found a
defect on its first day: a project restored without a map opened in the
catalogue (`a-project-is-a-workspace`).

**Track names run along the route** (`names-along-the-route`, 2026-10-02).
Noto Sans Bold SDF glyphs ship in `static/glyphs/` (OFL, ~510 KB) and reach the
map through a `glyphs://` protocol that answers every range, an unshipped one
with an empty set — so a stray character cannot hold a tile up, which is how a
remote glyphs URL took the lines off the map in July. The names are a symbol
layer beside the line, repeated along it, so a long route is named on whatever
stretch is on screen; the single DOM label at the middle and its hand-written
declutter are gone. Walked on the stand; the desktop smoke is owed at the end
of the run.

**One OpenSpec change is open.** `a-folder-for-one-moment` was archived on
2026-09-23, the day the owner asked for it, after the walk in the packaged
application found and closed a real defect in it. `finish-the-rebuild` carries the three pieces of the
development cycle that were never built: the screenshot matrix, the view-model
layer, and a smoke journey for the six Customer Journeys that have none —
nothing in it is visible to an operator. Everything else is archived — `codify-architecture-decisions` and `revive-ui-cycle` both closed on
2026-09-23, the first after reading all thirteen of its capability deltas
against the source, the second after moving its unbuilt requirements out rather
than archiving requirements the code does not meet.

**A moment can be captured into a folder** (`Shift+D`, or «Сохранить момент
для отчёта» in the palette). It writes
`~/Documents/ozi-rs-отчёты/<дата_время>/` holding `screenshot.png` of the
window, `diagnostics.txt` (the session's messages with the time and level),
`state.json` (the whole `AppStateDto`), `about.txt` (build, platform, moment)
and, if the operator writes one, `note.txt`. Deliberately not behind a debug
flag: the strangeness happens in the field, on a release build. The screenshot
is taken first and the description asked after, so nothing covers the screen.

It is walked in the packaged application by `smoke_report_capture_moment`,
which presses the chord and reads the folder back off disk — and that walk is
why the feature works. The first one produced a 588 KB picture of the owner's
**desktop wallpaper**: without the Screen Recording grant `screencapture` does
not fail, it exits zero and writes an image with every window missing, so the
check it had — does the file exist — was satisfied by it. The report would have
been handed over with the wrong picture in it and nobody the wiser. It now asks
`CGPreflightScreenCaptureAccess` before capturing and writes no screenshot at
all when the answer is no, saying in the toast to allow it and restart. Looked
at afterwards, not inferred: the folder holds the window itself, framed to its
edges.

Grant it once per machine — System Settings › Privacy & Security › Screen
Recording › ozi-rs — or the folders arrive complete but without their
pictures.

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

Answered on 2026-10-01 — see the top of this page. Two remain:

- the catalogue's row and badge sizes;
- whether a WPT name should carry `&#NNNN;` or `?` for a character cp1251
  cannot hold — needs someone with OziExplorer in front of them.

### An agent can still do these

4. **Packaged-app coverage beyond CJ-4.** The forty-five changes that carried
   an unchecked "smoke green" task are archived, but the gap they pointed at
   is not closed: `smoke_core_workflow` covers CJ-4's editing spine and
   nothing else; the import, the bundle download, the export and the session
   restore have no packaged-app coverage at all. `finish-the-rebuild` group 3.

5. ~~MapLibre 4 → 6.~~ Done on 2026-10-02: 6.11.2, the waiver gone with the
   advisory. Two things the upgrade needed that the changelog does not say
   in so many words: 6.x has no default export (`import * as maplibregl`),
   and it finds its worker from its own `import.meta.url`, which after
   bundling is a chunk and in the packaged app is `tauri://` — so the worker
   is bundled by Vite (`?worker&url`, `worker.format: "es"`) and handed over
   with `setWorkerUrl` (`src/lib/maplibre/worker.ts`). Walked on the stand;
   the packaged app is the open question — whether WKWebView starts a module
   worker from `tauri://` — and the desktop smoke answers it.
6. ~~`get_ozi_tile` is dead IPC surface.~~ Removed on 2026-10-02
   (`one-raster-tile-command`); the tile requirement names the two tile
   commands that exist.
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
- ~~MapLibre carries a critical advisory.~~ Upgraded to 6.11.2 on
  2026-10-02; the waiver is gone.
- ~~Human-facing docs still lie in places.~~ Fixed on 2026-09-21 (task 2.6 of
  `codify-architecture-decisions`), each claim checked against the code first.
- ~~`ThemePicker.svelte` is imported nowhere.~~ On the settings screen since
  2026-10-02.

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
