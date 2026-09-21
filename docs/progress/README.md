# Progress gallery

One entry per merged slice, newest first. Each entry links before/after
screenshots so the state of the interface can be read in one page instead of
being reconstructed from commits.

Screenshots come from the screenshot matrix (`just shots`) once that lands;
until then they are cropped captures of the real window taken through the
native QA harness (`docs/native-qa-mcp.md`).

---

## 2026-09-21 — choosing what to download

A bundle carries print sheets and Android tile packs this app cannot open, and
on a phone tether they are most of the transfer. That is the owner's July note
asking for a type filter.

What is worth carrying is a field judgement — a crew that also runs
OziExplorer on a phone wants the Android pack; one working only here does not —
so the app does not decide. The loader shows what the bundle holds, with the
sizes the listing states, everything checked. Clearing an entry leaves it on
the server; clearing nothing fetches the whole bundle exactly as before.

| | |
|---|---|
| After | [the loader](2026-09-21-choose-what-to-download/after-contents.png) |
| Automated gates | `just ci` green (293 Rust, 339 frontend) |
| Test | a named entry is left on the server while everything else, including folders below the top level, still arrives |

---

## 2026-09-21 — handing the day over

A folder import makes one track layer per navigator file, so a day of searching
is twenty-odd layers. Handing that to the штаб meant opening the export dialog
once per layer — the same twenty-six-clicks shape the visibility toggles had
before the bulk controls.

One action now writes every track in the project into a single GPX and says how
many it wrote. An empty project is refused: a file that looks like a day's work
and contains nothing is worse than an error.

This is also the shape FTP upload will need — "the day's tracks" as one thing.

| | |
|---|---|
| After | [the tracks rail](2026-09-21-hand-it-over/after-tracks-rail.png) — with the layer selector finally reading "Треки" |
| Automated gates | `just ci` green (292 Rust, 339 frontend) |

---

## 2026-09-21 — start before the bundle finishes

The owner's July note said a bundle could not be opened while the rest still
downloaded. Checking it turned up the opposite: the backend has always set a
map's local path the moment its file lands, and opening it then reads from disk
instead of starting a second download. A Rust test pins that now.

What was missing was the sentence. The crew watched a file counter — the
16 MiB topo layer lands in seconds, the 185 MiB satellite layer takes minutes —
and waited for the whole thing. The first map of a download that becomes
openable is announced once, with the invitation to start on it while the rest
continues.

| | |
|---|---|
| Automated gates | `just ci` green (290 Rust, 339 frontend) |
| Not seen on screen | needs a real bundle download in flight; the owner's next one will show it |

---

## 2026-09-21 — the first screen

The cold-start route had never been examined: the workspace fixture carries an
active map, so the redirect always won. The stand grew a second state
(`?state=cold`) and the screen a crew actually launches into finally came up.

The bottom line read `Load projects from maps.lizaalert.ru` — the backend's own
status text, in English, as the first thing on screen. When the frontend knows
the answer it says so in the interface language now: `Проектов: 13440 —
выберите слева`, or that the list is loading, or that it is empty. Anything the
backend has that the frontend cannot derive — a transient message, a download's
phase, an error — still wins the line.

The layer selector said `Tracks` and `Waypoints`. The core creates those names
and they live in the saved file, so they are translated for display rather than
renamed: a project written today still opens in an older build, and an English
build still shows what it stored.

| | |
|---|---|
| Before | [the first screen](2026-09-21-first-screen/before-cold-start.png) |
| After | [the first screen](2026-09-21-first-screen/after-cold-start.png) |
| Automated gates | `just ci` green (289 Rust, 336 frontend) |

The stand also learned to emit `state-changed` after the commands whose real
implementations do. Without it the "refreshing…" hint sat there forever and the
first screen always looked mid-flight — a stand that stops halfway through the
app's own sequence is telling a half-truth.

---

## 2026-09-21 — numbers a crew reads

The first thing the stand was pointed at was the Track Inspector, which nobody
had examined since it was built. It reported `1.1 km · 5h 1m · 5 pts` and
`Начало 2026-07-08T09:00:00+00:00` in an otherwise Russian window, and the
segments table said `SEGMENT 1 · 3 PTS`, `Edit Mode`, and repeated the RFC3339
string under every point in the list.

Units follow the interface language now, and an instant is formatted for a
person — `08.07.2026, 12:00`, in the operator's own clock.

| | |
|---|---|
| Before | [the inspector](2026-09-21-readable-numbers/before-inspector.png) |
| After | [the inspector](2026-09-21-readable-numbers/after-inspector.png) |
| Automated gates | `just ci` green (289 Rust, 334 frontend) |

The stand needed two fixes of its own in the process, both of the same class it
exists to catch. Its `get_waypoints` ignored the layer id and answered every
layer with the same waypoints, which listed each of them twice in the rail and
read exactly like an app defect until the call transcript showed two calls with
different ids. And tile requests threw, burying the screen under error toasts;
they answer with a transparent pixel now, cartographic fidelity being
explicitly out of scope.

Also: `bindings.ts` and the fixtures are prettier-ignored. Formatting them made
their own up-to-date tests fail on the next run, which cost a confusing minute
every time the frontend was formatted.

---

## 2026-09-21 — a stand, so a screen can be looked at

Every visual check in this repository has cost a full `just build`, an Appium
session that owns the screen and dims the display, and a window that has to be
caught in the right state before it moves on. Most of a working day went into
that, and twice the run was abandoned after two attempts per the project rule.

`just stand` serves the real frontend with the Tauri modules aliased to
fixture-backed stubs. The same screens open in a browser tab in a second, at any
size, with devtools. The data is the bytes the backend sends, because the
fixtures come from the same mappers the commands use.

An unanswered command throws with its name instead of returning `undefined` —
a screen that renders because a mock quietly answered nothing is the failure
this whole exercise exists to stop. The first run made the case for itself by
catching an unanswered `get_ozi_metadata` behind a red toast.

| | |
|---|---|
| Maps tab | [on fixtures](2026-09-21-stand/stand-maps.png) — sizes, cached badges, the active local map |
| Tracks tab | [on fixtures](2026-09-21-stand/stand-tracks.png) — a hidden track dimmed, the name-format warning, stats sublines |
| Automated gates | `just ci` green (289 Rust, 330 frontend) |

What it does not prove is in `src/test/stand/README.md`: nothing about the Rust
side, the IPC boundary, tiles or the packaged app. ADR-0024 stands — the smoke
gate is still what says the app works.

---

## 2026-09-21 — fixtures the frontend can trust

The frontend's tests mocked the backend by hand, and the mocks drifted. That is
how the Tracks tab came to filter for a geometry type the backend had stopped
emitting while 278 tests passed — the rail was empty on screen and green in CI.

The core now writes the fixtures. `src-tauri/src/fixtures.rs` builds a project
shaped like one search — two tracks (one hidden, one recorded in two sittings),
Cyrillic names, three waypoints with and without symbols, a catalogue where one
bundle is on disk and one is not — and serialises the wire snapshots through the
same mappers the commands use. `just fixtures` regenerates them, and
`fixtures_are_up_to_date` fails the Rust suite when a DTO change has not been.

Writing the generator found a defect nobody had noticed: `AppState::new` added
"Tracks" and "Waypoints" layers that `Project::default()` had already created,
so **every fresh project carried two layers of each kind, both with id 1**. The
layer selector listed "Tracks" twice, the second was unreachable because
everything addresses a layer by id, and saving then loading silently renumbered
it. It showed up as three lines of duplicated JSON the moment the first fixture
was written.

| | |
|---|---|
| Automated gates | `just ci` green (289 Rust, 325 frontend) |
| What this buys | a screen can be rendered against real backend data without a backend — the first half of giving agents eyes that do not need the GUI |

---

## 2026-09-21 — a waypoint that can leave the app

The waypoints spec has required a GPX export since it was written, and the XML
builder has been in the code the whole time with nothing calling it — a dead
`const _: fn(&[Waypoint]) -> String` kept the compiler quiet about it. So the
only way a mark left this app was OziExplorer's own WPT, which a phone, a
navigator and the other groups' software do not read. A crew that marks a
found object has to hand it over in the format the receiver has.

Both surfaces now offer both formats — the row menu in the Waypoints tab and
the Waypoint Inspector — the suggested file name follows the chosen format,
and a failed export reaches a toast instead of only the status line.

| | |
|---|---|
| Automated gates | `just ci` green (286 Rust, 317 frontend) |
| Tests | a GPX written from a real waypoint carries its position, its Cyrillic name and its symbol; both failure paths return an error; the suggested name follows the format |
| Not seen on screen | the menu items. Two attempts: the row dropdown holds focus and swallowed the following clicks, so the run stopped per `CLAUDE.md` rather than fighting it. |

---

## 2026-09-21 — what the bundle download weighs

One button fetches the project's whole directory tree — every subdirectory,
including print maps and Android packages this app cannot open — and the panel
counted files ("3 of 47") while saying nothing about bytes.

The scan already reads every directory listing to build the file list, and
those listings state each file's size, so the total was there to be had. It is
summed over what is actually about to be fetched (files already on disk are
skipped, so a resumed download does not re-announce the whole bundle) and the
workers' fetched bytes accumulate into the progress.

Two panel defects went with it: it sat at `z-40` under the loader sheet's
`z-50` overlay, so opening the loader to queue the next map hid the download it
had just started; and only the downloading phase reports byte totals, so the
figure blanked the moment extraction began — which is exactly when an operator
looks at it. The store keeps what the next phase does not restate.

| | |
|---|---|
| Seen on screen | the panel, per-file rows and sizes, over the loader |
| Not seen on screen | the aggregate byte line — on this link a bundle lands in a couple of seconds and the panel is gone before a screenshot catches it. Covered by tests: the scan announces the total, and the store keeps it through extraction. |
| Automated gates | `just ci` green (283 Rust, 317 frontend) |

---

## 2026-09-21 — how big is this download?

Opening a map commits to a download, and nothing said how big it was. In a
штаб running off a phone tether, the difference between a 16 MiB topo layer and
a 185 MiB satellite layer decides whether the map gets fetched at all.

The number was already in the listing the app parses — the cell beside each
file — and was being discarded. Every map row now carries it, in the loader and
in the Maps tab; a cached map reports the file on disk.

Three copies of a byte formatter lived in the loader, the download popup and
the cold-start status bar, printing English units into a Russian window and
disagreeing about MB versus MiB. One shared function now, in the app's
language, and the status bar's last English strings ("Downloading", "Cancel")
are translated.

| | |
|---|---|
| After | [sizes on every map](2026-09-21-download-size/after-map-sizes.png) |
| Automated gates | `just ci` green (282 Rust, 315 frontend) |

The size went on its own line after the first attempt put it beside the name
and the column — about 240px — cut it off.

---

## 2026-09-21 — which of these can I still open?

Thirteen thousand four hundred and forty projects, twenty-three of them on this
laptop. The backend had always known which — `is_project_cached` has been there
all along — but the summary sent to the screen carried a slug and a name, so the
one question a crew asks in a штаб with no signal had no answer.

The summary carries availability now, computed with a single read of the
bundles root rather than a question per row. The list marks what is on disk and
can show only that. The filter matches the slug as well as the name, because
names are transliterations and a date typed with dashes used to find nothing,
and the count reports matches — it used to show the catalogue total, which made
filtering look like it had done nothing.

A single-map download had no panel and no way out: it minted a download id
locally, never returned it and never set the busy flag, so nothing showed it and
the row simply went disabled until it was over. It now registers a cancel token
and hands its id to the same progress panel a whole-bundle download uses. Both
paths announce when they stop, which is also where a failed download finally
gets reported instead of just making the panel disappear.

| | |
|---|---|
| Before | [the catalogue, undifferentiated](2026-09-21-downloaded-bundles/before-catalogue.png) |
| After | [23 of 13440, each marked](2026-09-21-downloaded-bundles/after-downloaded-only.png) |
| Automated gates | `just ci` green (277 Rust, 311 frontend) |

Found on screen and fixed in the same slice: the filter input was `width: 100%`
and `flex-shrink: 0`, so the new toggle and the count were pushed out of the
280px rail entirely. They have their own line now.

---

## 2026-09-21 — the way to a map stops lying

A survey of the path from launching the app to having a map on screen found
that the parts a crew leans on say nothing when they fail and the wrong thing
when they succeed.

- The only download button is refused while the catalogue walk holds the busy
  flag — minutes, now that the listing paginates — and the refusal was
  indistinguishable from success. It carries a reason now, and the button is
  disabled and says it is waiting.
- Preview, download and local-bundle failures were swallowed by empty catches
  while the error reporter is off outside dev builds. They are toasts.
- The progress panel was never released when a download finished, so it came
  back with stale rows the next time anything made the app busy.
- Opening a map left the "show the loader" flag set, so the loader reopened
  over the map that had just been opened.
- The catalogue repair taught the remote walk to find `.sqlitedb` files in any
  subdirectory; the local mirror still read one fixed folder, so a bundle kept
  elsewhere read as missing and was downloaded a second time. The coordinates
  file is matched by pattern now too, as it already was online.

Cmd-K was handed every one of the ~13 000 catalogue projects and re-filtered
all of them on each keystroke; it now gets a filtered screenful, and "switch
project" actually selects the project in the loader instead of dropping the
operator at the top of the list with a toast.

Found while verifying: with a locally opened OZI map and no LizaAlert project,
the palette treated the workspace as a cold start and offered nothing but
Settings — save, undo, redo and the track search were all hidden with 26 tracks
on screen. It keys on the active map now.

| | |
|---|---|
| After | [command palette](2026-09-21-bundle-flow/after-palette.png), [maps tab in Russian](2026-09-21-bundle-flow/after-maps-tab.png) |
| Automated gates | `just ci` green (276 Rust, 303 frontend) |

Also translated in this pass: the Maps tab, the command palette, and all four
inspectors. What stays English is the backend's own status and progress text,
which reaches the status bar verbatim — there is no key-based channel for it
yet.

---

## 2026-09-21 — the Waypoints tab catches up

The Tracks tab had been brought up to field speed; the Waypoints tab listed the
same kind of object with none of it. It now has the same search field, clear
button and "shown of total" counter, the same show-all / hide-all pair, the same
"only this one" row action, and every row carries its coordinates and a locate
button that moves the map to that mark.

The matching rule is now one function both tabs call (`src/lib/name-filter.ts`),
so a query that finds a track finds a waypoint of the same name. Bulk visibility
is two new backend commands, not a loop of per-row toggles, and stays outside the
undo stack like every other visibility change.

The tests for this slice render the real component and assert on what is in the
DOM — five of them fail if the filter is removed. The tab's older tests read its
source text, which is the class of test that let the Tracks tab go empty
unnoticed.

| | |
|---|---|
| After | [the tab in use](2026-09-21-waypoints-parity/after-waypoints-tab.png) |
| Automated gates | `just ci` green (269 Rust, 295 frontend) |

Confirmed on screen: the search field, the show-all / hide-all pair, and every
row carrying its coordinates, a locate button and an actions menu.

The clicks only started landing once the app window was frontmost. A Mac2
session launches the app without raising it, and a click on a background window
reports success while the event goes to whatever is on top — two earlier
attempts failed silently that way. `open <path to .app>` before driving it.

---

## 2026-09-21 — one track at a time

Triage means looking at one track alone on the basemap. With twenty-six tracks
imported from the groups' navigators, the only control was a per-row toggle, so
isolating one meant hiding twenty-five by hand and restoring them afterwards.

The Tracks tab now has show-all and hide-all beside the search field, and every
row menu offers "show only this one". Each is a single backend command, so the
whole project changes in one round trip and one redraw. Visibility is still a
style mutation, so none of this touches the undo stack.

Import-created layers are named after the source file instead of its full path,
which makes the layer selector readable.

| | |
|---|---|
| Controls | [header](2026-09-21-bulk-visibility/after-controls.png) |
| Hide all | [one click, empty map](2026-09-21-bulk-visibility/after-hide-all.png) |
| Automated gates | `just ci` green (263 Rust, 286 frontend) |

---

## 2026-09-21 — Russian by default

The crew this is built for works in Russian, and the app opened in English
unless the operating system said otherwise, with the language switch buried in
the Cmd-K palette. It now starts in Russian whatever the system reports, and the
switch sits in the status bar as a RU/EN toggle. An explicit choice is still
remembered.

The chrome that stayed English is translated: the rail tabs, the mode chips,
the palette button, the cached badge, and the whole Waypoints tab, which had no
translated string at all.

| | |
|---|---|
| After | [workspace in Russian](2026-09-21-russian-default/after-russian.png) |
| Automated gates | `just ci` green (259 Rust, 286 frontend) |

---

## 2026-09-21 — finding a track

A field project carries dozens of tracks named by date and call sign, spread
over one layer per imported file. There was no way to search them: the project
list had a filter, the track list did not, so finding one track meant scrolling
past all of them.

The Tracks tab now has a search field with a clear button and a "shown of
total" counter. Matching is a case-insensitive substring in either alphabet,
because operators search by call sign as often as by date. Typing `лиса1`
narrows 26 tracks to 8.

| | |
|---|---|
| After | [search in use](2026-09-21-track-search/after-search.png) |
| Automated gates | `just ci` green (259 Rust, 286 frontend) |

---

## 2026-09-21 — row density and dev signing

The name-format warning was a full line of orange text under most rows, so a
track row cost three lines and the names themselves were the least prominent
thing in the list. It is now a small yellow glyph beside the name, carrying the
same sentence as its tooltip and accessible label. A row is two lines again and
roughly twice as many tracks fit without scrolling.

Debug builds are signed with a local identity created by
`scripts/setup-dev-signing.sh`. Before this, every rebuild was a new program to
macOS, so the Documents-access prompt returned on each build and blocked the
window from opening — verified fixed: a rebuild now launches straight into the
workspace with no prompt.

| | |
|---|---|
| Before | [rows](2026-09-21-row-density/before-rows.png) |
| After | [rows](2026-09-21-row-density/after-rows.png) |
| Automated gates | `just ci` green (248 Rust, 281 frontend) |

---

## 2026-09-20 — visible fixes (slice 0.2)

Four defects the owner saw the first time the built app was opened.

- Straight red lines no longer run across the map. Track geometry is emitted
  as one `MultiLineString` part per segment, so the gap between segments is a
  gap, and `split` / `join` are visible on the map for the first time.
- Row buttons render their icons again. The legacy `button` reset in
  `app.css` was applying `padding: 4px 10px` and a border to 24px icon
  triggers, leaving no content box for the glyph and stretching the round
  colour swatch into a rectangle.
- The Maps tab lists the active map even when no LizaAlert project is loaded,
  so a locally opened OZI map no longer sits behind "No maps in this project".
- Durations of a day or more read as `26d 5h` instead of `629h 22m`, with a
  tooltip naming what the figure measures.

Evidence: [`2026-09-20-visible-fixes/`](2026-09-20-visible-fixes/)

| | |
|---|---|
| Before | [maps tab](2026-09-20-visible-fixes/before-maps-tab.png), [tracks tab](2026-09-20-visible-fixes/before-tracks-tab.png) |
| After | [maps tab](2026-09-20-visible-fixes/after-maps-tab.png), [tracks tab](2026-09-20-visible-fixes/after-tracks-tab.png), [row detail](2026-09-20-visible-fixes/after-tracks-rows-detail.png) |
| Automated gates | `just ci` green (248 Rust, 281 frontend), `cargo audit` clean, npm audit gate clean, GitHub Actions green |
| Customer-journey smoke | still owed |

Confirmed on screen (2026-09-21): no straight lines across the map; every row
carries its visibility eye, a round colour swatch, a locate button and the
actions menu; durations read `26d 5h` and `19d 7h`; the Maps tab lists the
active local map instead of claiming the project has none.

Two defects surfaced while verifying and are fixed in the same slice: the row
list went empty because the tab still filtered for `LineString` geometry, and
the icon utilities never applied because the legacy element defaults sat
outside a cascade layer and therefore beat them.

Still visibly wrong, not yet addressed: the orange `Format: YYYYMMDD_Callsign`
line repeats under most rows and dominates the list; the raster's no-data area
is filled solid black by the source map rather than left transparent.

The CJ smoke is still owed. The grant returned on 2026-09-21, but every
rebuild re-prompts for Documents access because the debug bundle is ad-hoc
signed, so the harness needs a stable signing identity before smoke runs can
be routine.
