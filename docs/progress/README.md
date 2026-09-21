# Progress gallery

One entry per merged slice, newest first. Each entry links before/after
screenshots so the state of the interface can be read in one page instead of
being reconstructed from commits.

Screenshots come from the screenshot matrix (`just shots`) once that lands;
until then they are cropped captures of the real window taken through the
native QA harness (`docs/native-qa-mcp.md`).

---

## 2026-09-21 — the catalogue is not application state

I went looking for the backlog's note that the catalogue cache "only ever
grows", and found something larger on the way: `get_app_state` was carrying the
whole catalogue. Thirteen thousand rows of slug, name and a cached flag — by
the DTO's own shape, **about 1 MiB of JSON per call** — and that call is made
on every `state-changed`: once per file during a bundle download, on every
preview, on every edit. Building it also walked the bundles directory each
time, to compute a flag for rows nobody read.

One consumer wanted it, and only to seed a store that is already seeded twice:
synchronously from the `localStorage` cache when the module loads, and again
from the first `projects-chunk`, which `load_projects` emits from the disk
cache before it touches the network.

A catalogue is not application state. It is a stream, and it already had one.

The test pins the cost rather than the code: build a snapshot with five hundred
projects in the state, and assert the catalogue is not in it and the snapshot
is under 4 KiB. The real fixture is 2 908 bytes.

One consequence worth writing down: `LizaProjectSummaryDto` is no longer
generated, because specta generates from command signatures and the catalogue
now travels only as an event payload. It is hand-written in `types.ts`, which
`CLAUDE.md` already requires to be kept in step by hand.

| | |
|---|---|
| Evidence | a Rust test that a 500-project catalogue does not reach the snapshot, and that it stays under 4 KiB |
| Automated gates | `just ci` green (301 Rust, 367 frontend) |
| Customer-journey smoke | not run — the Mac2 driver host crashes at session creation (`docs/STATE.md`) |

---

## 2026-09-21 — walking the catalogue by keyboard

Thirteen thousand rows, and only the twenty on screen exist in the document.
There was no tab order over that and there never can be one, so finding a
search meant the trackpad, and a screen reader saw a handful of buttons out of
thirteen thousand.

Typing part of the name already narrowed the list. What was missing was the
rest of the gesture. Now: type, Down, Enter.

The shape matters here. Moving DOM focus row by row would fight the windowing —
most of the rows a position passes through are not rendered — so the list holds
focus itself and names the row it is on with `aria-activedescendant`, and the
position is an index into the filtered array rather than into what happens to
be drawn. Arrows move it, Page Up and Down by a screenful, Home and End to the
ends; it stops at both, and the list scrolls to keep it in view. Changing the
filter clears it, because it no longer points at anything the operator chose.

It also needed a mark of its own: the keyboard position is not DOM focus, and
it is not the selected row either. Three different things, three different
looks.

| | |
|---|---|
| Evidence | five behavioural tests: pointing, both ends, Enter, Enter before any row, walking in from the search box |
| Evidence | walked on the stand — End reaches the last search, its outline computes to 3px, and Enter fires `preview_project` for that slug |
| Automated gates | `just ci` green (300 Rust, 367 frontend) |
| Customer-journey smoke | not run — the Mac2 driver host crashes at session creation (`docs/STATE.md`) |

---

## 2026-09-21 — numbers that mean something

Two numbers on screen that say nothing.

The one-point track, now that it has a row at all, read `0.0 км · 0мин · 1 тчк`
— two measurements of nothing standing in front of the one measurement there
is. A track with fewer than two points has no length and no elapsed span, so it
says how many points it has and stops. English also says "1 pt" rather than
"1 pts" now.

And while the catalogue refreshes, the hint said only that it was refreshing.
The refresh can be stopped as of two slices ago, and the decision to stop rests
on exactly the fact the hint was keeping back: how much is already there. It
says "Обновление списка… уже 5 300" now. A crew that came for one search does
not need the other twelve thousand — but they do need to know whether theirs
has arrived.

| | |
|---|---|
| Evidence | tests on the short track in both languages, and the older `0.0 km · 0m · 0 pts` expectation updated with its reason |
| Automated gates | `just ci` green (300 Rust, 362 frontend) |
| Customer-journey smoke | not run — the Mac2 driver host crashes at session creation (`docs/STATE.md`) |

---

## 2026-09-21 — nothing typed in English

Having found English on the library rows, I swept the rest of the components
rather than the rest of the screens. There was more, and one of it is on the
path a crew uses in the field: the map's point context menu, in edit mode, read
**"Delete Point"** and **"Insert Point After"**. Also the inspector rail's
heading and its empty state, the console's title, the symbol picker's "none",
the theme pack, the collapsed rails' placeholders, the live-preview label, and
two inspector sections' accessible names.

Two sweeps in one day for the same defect, so the third one is a test rather
than a sweep. It scans every component for a literal `aria-label`, `title` or
`placeholder` whose value contains a Latin letter. Anything computed,
translated or interpolated passes — it constrains where a label comes from, not
how it is built.

It earned itself on the first run: two offenders I had already missed twice,
both accessible names on inspector sections. The allowlist has one entry, an
inert mode group, with its reason written next to it.

Then the stand found two it still could not see, because they are assembled
rather than typed: the waypoint rows' `Symbol: flag` and the inspector rail's
pinned/unpinned ternary. So the guard grew a second rule — strip the `$t(…)`
calls, whose keys are Latin by design, and fail on English left in what
remains. The rows read "Значок: flag" and "Значок по умолчанию" now.

| | |
|---|---|
| Evidence | the guard test, red on its first run, naming both files it found |
| Evidence | the second rule, verified red by putting `` `Symbol: ${symbol}` `` back |
| Evidence | walked on the stand: "Значок: flag", "Значок по умолчанию", "Скрыть: ШТАБ" |
| Automated gates | `just ci` green (300 Rust, 360 frontend) |
| Customer-journey smoke | not run — the Mac2 driver host crashes at session creation (`docs/STATE.md`) |

---

## 2026-09-21 — the rows speak Russian

Walked the workspace on the stand, mostly to see the one-point track from the
last slice actually get a row. It does — "точка отсечки", third in the list,
with the two the map draws. The stand is the only channel of evidence open
while the Mac2 driver is down, and it answered the question the Rust tests
could only answer in principle.

It also showed what I had walked past: the library rows, the ones a crew
touches most, were English inside a Russian window. The eye button's tooltip
read "Hide". Its accessible name read "Hide 20260708_Veter2". The row menu was
"Actions", the swatch "Track color", the rail "Library". The Waypoints tab was
passing a translated label in; the Tracks and Maps tabs fell through to the
English default written into the component.

The tooltip is visible text. The rest is what a screen reader says — and on
this project it is also what the customer-journey smoke matches on, because
WKWebView publishes a control's `aria-label` rather than its inner text.

One thing the dictionary could not do literally: Russian declines the object of
"скрыть", and a name substituted raw stays nominative, so the honest
translation of `Hide {name}` would have read "Скрыть точка отсечки". It is
`Скрыть: {name}` — the colon sidesteps the case for any name, including the
ones that are dates and call signs rather than words.

| | |
|---|---|
| Evidence | walked on the stand: "Скрыть: точка отсечки", "Цвет трека", "Действия", "Библиотека" |
| Evidence | tests on the labels in both languages and on the declension |
| Automated gates | `just ci` green (300 Rust, 358 frontend) |
| Customer-journey smoke | not run — the Mac2 driver host crashes at session creation (`docs/STATE.md`) |

---

## 2026-09-21 — one rule, written once

Having found the overlapping-reload bug three times, I went looking for the
fourth. It was on the map, where it is worse than in a list.

`refreshWaypointMarkers` runs from a slice effect and from every waypoint
action handler, so two runs overlap as a matter of course, and the marker set
written last won even when it had started first: a waypoint just added could
disappear again. The track geometry fetch beside it has the same shape — two
quick edits leave two fetches in flight, and the older one drawing last puts
the map behind the rail it is supposed to match. The map was also reading its
waypoint layers one after another, on a path that runs on every state change.

Five places, one rule: take a token before the first await, check it before
writing anything, and check it on the failure path too. Written out by hand
five times is how it comes to be missing the sixth, so it is written once now,
in `latest-run.ts`, with the tests on it rather than on each caller. The two
tabs from the last slice moved onto it as well.

Honest about the gap: the map's two guards have no test. `MapView` needs a
MapLibre instance, and this slice did not build one. That is recorded in the
change's tasks rather than left to be assumed.

| | |
|---|---|
| Evidence | four tests on the rule itself; the two tabs' behavioural tests still green through the refactor |
| Not covered | the map's two guards — stated, not implied |
| Automated gates | `just ci` green (300 Rust, 356 frontend) |
| Customer-journey smoke | not run — the Mac2 driver cannot initialise UI testing on this machine (`docs/STATE.md`) |

---

## 2026-09-21 — only the newest list wins

The same bug as the bundle preview, in a second and a third place.

Both library tabs reload their rows whenever the app state changes, and during
a bundle download `state-changed` fires once per file. So reloads overlap, and
nothing ordered them: the one that started first could answer last and write
its rows to the screen. The list went backwards under the operator, and the
worse the link, the more likely it was.

The Waypoints tab carried a second cost on the same path. It read its layers
one after another, awaiting each, so a project of a dozen import-created layers
paid a dozen round trips in a row to draw a list that is rebuilt whole anyway.
It asks them all at once now.

The test for that one is the kind worth keeping: each layer's answer resolves
only once every layer has been asked, so the sequential version does not fail
an assertion — it deadlocks, and the rows never appear.

| | |
|---|---|
| Evidence | a held-open reload overtaken by a newer one, in both tabs; both red before the change |
| Evidence | a parallel-fetch test that cannot pass sequentially |
| Automated gates | `just ci` green (300 Rust, 352 frontend) |
| Customer-journey smoke | not run — the Mac2 driver cannot initialise UI testing on this machine (`docs/STATE.md`) |

---

## 2026-09-21 — a row for every track

The Tracks tab built its rows out of the map's GeoJSON. One call, so it looked
cheap. It was not.

Every coordinate of every track travelled so that a list of names could be
drawn. The row model reads nine properties; the geometry beside them — a day's
folder of recordings is hundreds of thousands of points — was serialized in
Rust, sent over IPC, parsed and dropped, again on every change that reloads the
tab.

And a track the map cannot draw had no row at all. `build_tracks_geojson`
omits a track whose segments are all shorter than two points, which is right —
there is nothing to draw. The list inherited it, so a one-point track sat in
the project, counted towards its size and went out with its export, with no way
to see it, rename it or delete it. That is the second time a list derived from
map geometry has gone wrong this way; the first emptied the whole rail when the
geometry type changed.

`list_tracks` returns rows now: one per track, every track, no geometry, and
typed instead of `JsonValue`. The old mapper had no callers left and is gone.

The stand would have inherited the same omission — it was deriving its answer
from the geometry fixture — so the core writes `tracks-list.json` beside it and
the sample project gained a one-point track. Three rows against two features,
and a test that holds the two fixtures against each other.

| | |
|---|---|
| Evidence | Rust tests that the listing includes what the map omits and carries no coordinates; frontend tests on the row mapper and on the gap between the two fixtures |
| Automated gates | `just ci` green (300 Rust, 349 frontend) |
| Customer-journey smoke | not run — the Mac2 driver cannot initialise UI testing on this machine (`docs/STATE.md`) |

---

## 2026-09-21 — the download speaks Russian

The app has opened in Russian for a while. The one part of it a crew actually
watches — a bundle coming down — did not, because those sentences are built in
Rust with `format!` and reach the status bar verbatim. "Downloading 3 files in
parallel", and the phase word beside it, sat in an otherwise Russian window for
the whole length of the download.

Twelve messages, which is all of them on the bundle path. Each carries a key
and its arguments now; the English wording still travels, as what a diagnostic
records and as the fallback — a build that meets a key it does not know shows
what the backend said, because a key on screen is worse than English.

Writing the substitution turned up a flaw in the first version of it: replacing
one argument at a time means the second pass can substitute into what the first
one inserted, and a bundle name is free text off a web listing. One pass now,
with a test that a name shaped like a placeholder survives.

| | |
|---|---|
| Evidence | a Rust test pinning key, arguments and English for the message shapes; four frontend tests covering both languages, the fallback, the placeholder-shaped name and a missing argument |
| Stand | the played-out download carries keys, so it shows the translation rather than the fallback |
| Automated gates | `just ci` green (298 Rust, 349 frontend) |
| Customer-journey smoke | not run — the Mac2 driver cannot initialise UI testing on this machine (`docs/STATE.md`) |

---

## 2026-09-21 — stop waiting for the catalogue

Every launch walks the whole listing: up to a thousand pages, one request each,
and the walk holds the application busy for all of it. While it runs, the only
download button in the app is disabled. On a field link that is minutes, and a
crew that opened the app to fetch one bundle whose name they already know has
nothing to do but wait for an index of thirteen thousand searches they will not
read.

The list is already usable long before the walk ends — the cached one is
applied first and every page adds to it. What was missing was any way to say
that is enough. There is a stop beside the refreshing hint now.

The part worth being careful about is the cache. A stopped walk holds the first
few pages; writing those over the cached catalogue would leave a crew offline
tomorrow with the newest few dozen searches and nothing to say the rest ever
existed. It does not write. And the status says the refresh was *stopped* at
412 projects rather than that 412 were loaded — different facts.

| | |
|---|---|
| Evidence | a two-page test server, cancelled from inside the first page's callback: the second page is never requested, what was read stays, and the walk reports itself stopped |
| Evidence | a behavioural test: the stop appears only while refreshing, and reaches the backend |
| Automated gates | `just ci` green (297 Rust, 345 frontend) |
| Customer-journey smoke | not run — the Mac2 driver cannot initialise UI testing on this machine (`docs/STATE.md`) |

---

## 2026-09-21 — the catalogue keeps your place

The loader is a Sheet on the workspace route precisely so a crew can check the
map without leaving the catalogue. A Sheet unmounts its content, so doing that
threw away the search text, the "only downloaded" toggle, the selected project,
the contents they had unchecked and the place in a thirteen-thousand-row list.
They came back to an empty box at the top. Project names are latin
transliterations of Russian place names, so "just type it again" is not the
small thing it sounds like.

The position lives outside the component now and comes back with it. It is
session state on purpose: tomorrow starts on tomorrow's search.

The test mounts the real component, types a search, throws the component away
exactly as the Sheet does and mounts it again — and it fails on the old code,
which is the only reason to trust it. Mounting the loader at all needed
`$app/navigation` and `$app/paths` stubs; they are there now for the next
component that navigates.

| | |
|---|---|
| Evidence | a behavioural test across an unmount, red before the change |
| Automated gates | `just ci` green (296 Rust, 343 frontend) |
| Customer-journey smoke | not run — the Mac2 driver cannot initialise UI testing on this machine (`docs/STATE.md`) |

---

## 2026-09-21 — one preview at a time

No screenshot in this one, and that is the point: the defect was in *when*
things happened, not in how they looked. Scrolling a thirteen-thousand-entry
catalogue, a crew clicks several projects in a row — that is how you find the
right search. Each click previews a bundle in a thread of its own, and nothing
ordered them. Whichever answered last won, so the map list could swap, seconds
later, to a project the operator had already scrolled past, under the row they
were actually reading. A failure from an abandoned preview announced itself the
same way, naming a search nobody had asked about any more.

Two more faults sat on the same path. A preview deliberately does not take the
busy flag — a click during the catalogue walk must not be swallowed — but it
*cleared* the flag on the way out, releasing something it never held; a preview
landing mid-download let a second download start. And the loader decided the
list had arrived by comparing display names, which are not identity, while a
fifteen-second timer cleared the spinner outright with the request still in
flight: it said "done" about something that had not happened.

The newest preview is now the only one that may land, by slug. The timer says
the wait is running long rather than ending it; what bounds the request is the
HTTP read timeout added in slice 0.3, which reports a real failure.

| | |
|---|---|
| Evidence | three tests that fail on the old code: the abandoned preview lands and wins, its failure is reported, and it clears the busy flag |
| Stand | `preview_project` moves the previewed slug now, so the round trip closes instead of spinning forever |
| Automated gates | `just ci` green (296 Rust, 341 frontend) |
| Customer-journey smoke | not run — the Mac2 driver cannot initialise UI testing on this machine (`docs/STATE.md`) |

---

## 2026-09-21 — when it fails

The stand learned the two failures a field crew actually meets: the listing
unreachable, and a download that dies.

With no network the project list still shows, from the cache — the right
answer. Nothing said so. A crew could not tell today's list from one saved days
ago, and a bundle made this morning missing from it would read as "no such
search". The loader marks the list as the saved one now, and the status line
says the refresh did not happen.

The failed download turned out to behave already: the error reaches a toast
carrying the backend's own message, the panel closes rather than hanging, and
the map that did land stays openable. Verified rather than assumed.

| | |
|---|---|
| Offline | [the saved list, marked](2026-09-21-when-it-fails/offline-catalogue.png) |
| Failed download | [the toast](2026-09-21-when-it-fails/failed-download.png) |
| Automated gates | `just ci` green (293 Rust, 340 frontend) |

---

## 2026-09-21 — a download, played out

The progress panel, the byte totals, the "this map is ready" announcement and
the panel closing had all been built without ever being seen: catching them
needs a bundle in flight, and on this link a bundle lands in seconds. The stand
now plays the event sequence a real download emits — same names, same payload
shapes — slowly enough to watch.

The first press proved the idea in the worst way: **`load_project` was never
called at all.** `handleOpenBundle` reads the slug of the row the operator
clicked, and on the path from the workspace — Maps tab, "Открыть проект…" — a
project is already previewed and no row has been clicked. The handler returned
before calling anything, so the only download button in the app sat there doing
nothing, silently. The project now carries its slug and the handler falls back
to it.

With that fixed, one run confirmed everything at once: the panel appears, its
total reads `165 Б / 201.0 МиБ` and climbs to `201.0 МиБ / 201.0 МиБ`, the file
count walks 0/3 → 3/3, the per-file bars arrive one by one, the ready-map
announcement fires when the topo layer lands, and the panel closes when the
download finishes.

| | |
|---|---|
| The panel, mid-flight | [captured](2026-09-21-download-in-flight/panel-in-flight.png) |
| Automated gates | `just ci` green (293 Rust, 340 frontend) |

`window.__stand.calls` now exposes the IPC transcript, which is how "the button
does nothing" became "the command never fired" without guessing.

---

## 2026-09-21 — what a click will cost

Walking the screens at the restored padding, at a laptop-sized window: the
Maps tab marked a downloaded map "В КЭШЕ" and a map that is not downloaded
with nothing at all. Blank reads as "nothing to say", while clicking that row
starts a download — 185 MiB for the satellite layer on this bundle.

It carries a marker now, and the tooltip names the size.

| | |
|---|---|
| After | [the maps rows](2026-09-21-not-downloaded-marker/after-maps-rows.png) |
| Automated gates | `just ci` green (293 Rust, 339 frontend) |

The rest of the pass found nothing: the workspace, the tracks and waypoints
rails and the inspector all hold together at 1024×640.

---

## 2026-09-21 — the app gets its padding back

Opening a row menu on the stand showed the labels flush against the box's
edge. Measuring in the browser rather than guessing: a dropdown item computed
`padding: 0px`, and so did the menu itself.

`* { margin: 0; padding: 0 }` in `app.css` sat outside any layer, and an
unlayered rule beats every `@layer` whatever its specificity. Every Tailwind
spacing utility in the app had been losing to it — the same landmine that once
left a 24px icon trigger with no room for its glyph, in its most general form.
Inside `@layer base` the reset still replaces the browser's defaults and the
utilities decide from there.

Restoring padding everywhere then uncovered a defect it had been masking: the
segments card was the only shrinkable child of the inspector column, so the
moment the rail ran short of room that card — and only it — collapsed, hiding
the points behind the next card. It does not shrink now; its own cap and the
rail's scroll do the work.

| | |
|---|---|
| Before | [a row menu](2026-09-21-padding-restored/before-menu.png) |
| After | [the same menu](2026-09-21-padding-restored/after-menu.png), [the inspector](2026-09-21-padding-restored/after-inspector.png) |
| Automated gates | `just ci` green (293 Rust, 339 frontend) |

Both were found in about ten minutes on the stand. Neither would have been
visible in a test, and both had been shipping for months.

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
