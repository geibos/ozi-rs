# Progress gallery

One entry per merged slice, newest first. Each entry links before/after
screenshots so the state of the interface can be read in one page instead of
being reconstructed from commits.

Screenshots come from the screenshot matrix (`just shots`) once that lands;
until then they are cropped captures of the real window taken through the
native QA harness (`docs/native-qa-mcp.md`).

---

## 2026-09-22 — yesterday's work on the first screen

A saved `.ozp` could be opened from exactly one place: the command palette. The
launch screen offers «Открыть локальный бандл…» and «Папка для бандлов…» and
nothing else. So a crew arriving in the morning with yesterday's work saw a
catalogue of thirteen thousand LizaAlert searches and no way back to their own
project, unless somebody had told them about ⌘K.

This repo has already made this exact argument once, when the language switch
moved into the status bar: "a Russian-speaking crew had to know the palette
exists to get a Russian interface". A shortcut is not a route.

The first screen now offers «Открыть сохранённый проект…» and, under it, the
three most recent projects as one click each. The recents list existed already
— `yesterdays-project-is-one-key-away` built it — it just had nowhere to be
seen.

While wiring it: «проект» meant two different things and both buttons said so.
The Maps tab's «Открыть проект…» opens the LizaAlert catalogue; the palette's
opens a file. The loader's own list is headed «Каталог поисков», so the
catalogue already had a better word for itself, and the Maps tab now says
«Выбрать поиск…».

Third surface, so the dialog, the remembering, the framing and the dropping of
a stale path moved into one `openProjectFile` action rather than a third copy.

| | |
|---|---|
| Before | [no way back to a project](2026-09-22-first-screen/before-no-way-back.png) |
| After | [the recents where they are needed](2026-09-22-first-screen/after-recent-projects.png) |
| Change | `openspec/changes/yesterdays-work-on-the-first-screen/` |
| Automated gates | `just ci` green (320 Rust, 488 frontend) |
| Customer-journey smoke | still owed — the Mac2 driver cannot enable automation mode |

---

## 2026-09-22 — open a project and see it

The camera was fitted to the data in exactly one place: after an import. That
fix exists because of the owner's own finding — "imported tracks don't show on
the map" — and its comment says why: the tracks "render wherever they are, off
the active raster, and look like they failed to import".

Opening a saved project has the same hole, and it is the path that gets used
every morning. Both ways in — the Open dialog and the recents — load the file
and stop. The map stays where it was, so a crew reopening yesterday's search
gets an empty screen that is indistinguishable from a project that did not
load.

Two things came out of fixing it. The import fit only ever covered track
geometry, so a project whose content is a headquarters and a drop-off point
had nothing to fit — waypoints count now, and their positions come from the
markers already on the map, so it costs no round trip. And the bbox maths moved
out of `MapView`, which needs a MapLibre instance to mount and therefore had
none of this under test; on the way out it learned to skip a coordinate that is
not a finite number, rather than turning the whole frame into `NaN` because one
track could not be built.

A single point is centred at a readable zoom instead of fitted. `fitBounds` on
a box a metre across answers with its maximum zoom, which puts the crew inside
a building.

Measured on the stand: the scale bar went 50 m → 500 m and both track segments
and the waypoint markers came on screen.

| | |
|---|---|
| After | [framed on the data](2026-09-22-framing/after-framed-on-the-data.png) |
| Change | `openspec/changes/open-a-project-and-see-it/` |
| Automated gates | `just ci` green (320 Rust, 483 frontend) |
| Customer-journey smoke | still owed — the Mac2 driver cannot enable automation mode |

---

## 2026-09-22 — what the ground does

The track inspector carried a card headed «Высота» whose whole content was
«График высоты — будет в следующем изменении». `PointDetailDto` has carried
`elevation` since the first import path was written, so the data had been there
the entire time and the card was a promise shown to a crew instead of an
answer.

It matters for the work: a leg that climbs out of a river valley takes an hour
where the same distance on the flat takes twenty minutes, and reading somebody
else's recording is the one moment you cannot ask them.

The card draws the profile against distance, with the range beside the heading
— on the stand's fixture, `M0,40 L6.58,20 L8.76,0 L84.1,40 L100,20` in a 306×39
box and «30–32 м». A recording with no elevation, or one lonely reading, says
so rather than drawing a line through nothing. It costs no round trip: the
statistics card above it already loads the detail.

Two decisions worth naming. A point with no elevation still counts towards
distance, so a gap in the data does not slide the readings around it towards
each other — that one has a test, because it is the kind of thing that looks
right on a fixture where every point has a reading. And there is no ascent or
descent total: summing every rise in a GPS track sums its own noise, and the
threshold that fixes that is a decision about the data rather than about the
chart. It is in the backlog next to the moving-time threshold, which is the
same question asked about time.

| | |
|---|---|
| After | [the card with a profile](2026-09-22-elevation/after-elevation-card.png) |
| Change | `openspec/changes/what-the-ground-does/` |
| Automated gates | `just ci` green (320 Rust, 471 frontend) |
| Customer-journey smoke | still owed — the Mac2 driver cannot enable automation mode |

---

## 2026-09-22 — the button that was not there

Selecting a track opens the inspector. Opening the inspector put Save, Undo,
Redo and the ⌘K trigger underneath it — not crowded, *unreachable*. At 1024×640
with a track selected, `document.elementFromPoint` at the centre of each of
those four returned `HEADER.rail-header`. A click on Save went to the inspector.

So in an application for editing tracks, the way to save disappeared at the
moment a track was selected. I found it by looking at the screen, not by
reading code, which is the argument for looking at the screen.

The cause is one missing declaration. `.canvas-column` is a grid with
`grid-template-rows` and no `grid-template-columns`, so its implicit column is
`auto` — max-content. The bar wants about 710px for the inert mode chips plus
the actions; the column was 384px; `auto` made the column 742px wide and the
overflow ran under the inspector, which paints after it. Not an edge case of a
tiny window: with both rails open, a 1280px laptop leaves the bar 640px.

Now the column is `minmax(0, 1fr)`, so the bar shrinks instead. What it sheds
is ordered: the inert mode placeholders go first and entirely, then the words
on the two labelled controls, and never a control — the actions are
`flex-shrink: 0`. The breakpoints are container queries on the bar itself,
because what takes the width away is the library rail and the inspector, not
the window.

Measured on the stand with a track selected:

| Window | Bar | Overflow past the column | Unreachable actions |
|---|---|---|---|
| 880 | 240px | 0px | none |
| 1024 | 384px | 0px | none |
| 1280 | 640px | 0px | none |
| 1300 | 660px | 0px | none |
| 1440 | 800px | 0px | none |

One thing I nearly left in: the first version also hid the "Сохранено" readout
on a narrow bar, and that rule silently did nothing — `.dirty-indicator`
declares its own `display` later in the same stylesheet, at equal specificity.
A rule that does not apply is worse than no rule, so it is gone; the readout
ellipsizes on its own, and "is my work saved" earns its 57 pixels.

| | |
|---|---|
| Before | [inspector over the toolbar](2026-09-22-toolbar/before-inspector-covers-toolbar.png) |
| After | [the bar inside its column](2026-09-22-toolbar/after-toolbar-fits.png) |
| Change | `openspec/changes/the-toolbar-stays-clickable/` |
| Automated gates | `just ci` green (320 Rust, 462 frontend) |
| Customer-journey smoke | still owed — the Mac2 driver cannot enable automation mode |

---

## 2026-09-22 — the third way to open a map

`open_selected_map` opens a map from disk when it is there and starts a
download when it is not, returning the download id in the second case so the
caller can show progress and offer a cancel. The bundle loader used it. The
Library Maps tab used it. The command palette threw it away.

So from the palette, asking for a map whose bytes were not on disk did this:
the palette closed, a download started, nothing appeared on screen, nothing
could be cancelled, and the map never opened — the palette's navigation is
gated on an `activeMap` that a running download has not set yet. The way in is
the palette's recent files, which name a map that may since have been removed,
moved, or downloaded onto a different machine.

Third call site, so the rule became a helper rather than a third copy — the
same move `latest-run.ts` got when the generation-stamp rule turned up five
times.

Finding it needed the stand to grow: its `open_selected_map` always answered
"already on disk", so this path had never been playable. It now plays a
single-map download for a map the fixture reports as not downloaded.

Measured on the stand by sampling the DOM every 150 ms across the click: the
panel is absent before, present for 18 consecutive samples (~2.7 s, the
script's length), and gone on `download-finished`. Two earlier checks said the
panel never appeared — both had sampled after the script had already finished,
which is worth writing down: a one-shot check of a transient panel proves
nothing about whether it was there.

| | |
|---|---|
| Change | `openspec/changes/one-way-to-start-a-map/` |
| Automated gates | `just ci` green (320 Rust, 458 frontend) |
| Customer-journey smoke | still owed — the Mac2 driver cannot enable automation mode |

---

## 2026-09-22 — typing the name you were given

The catalogue names every search in latin transliteration — `2026-07-08_Lavrovo`,
`2026-09-20_Schuvalovo`, `2026-07-14_Sagra` — and the filter matched the query
as a literal substring. A crew that types `Лаврово`, which is the name on the
radio and the name of the place they are standing in, got an empty list of
thirteen thousand rows.

The instruction that would have made it work — "type it in English" — is not one
a crew should have to remember, and it is not one they could follow reliably
anyway: the catalogue itself writes `ш` as `sch` in one search and `sh` in the
next, `ж` as `zh` here and `j` there. So converting the query to a single
transliteration would have traded one empty list for another.

Instead the query becomes a pattern: each Cyrillic letter matches any of the
latin spellings in use for it, and itself, so a Cyrillic entry is still found.
A query with no Cyrillic takes the path it always took — the same results, and
no pattern built for the common case, which matters at thirteen thousand rows
per debounced keystroke.

Checked on the stand against the fixtures the Rust core writes:

| Typed | Listed | Count |
|---|---|---|
| `Шувалово` | 2026 09 20 Schuvalovo | 1 из 3 |
| `Лаврово` | 2026 07 08 Lavrovo | 1 из 3 |
| `Мурманск` | — | 0 из 3 |
| `Sagra` | 2026 07 14 Sagra | 1 из 3 |

The palette had it too. Its project search is the second way into the catalogue
and matched just as literally, so the two ways in would have disagreed about
what the catalogue contains. Same helper, same answer, and it still stops at a
screenful rather than filtering thirteen thousand rows into `cmdk`.

| | |
|---|---|
| Change | `openspec/changes/search-in-the-crews-language/` |
| Automated gates | `just ci` green (320 Rust, 454 frontend) |
| Customer-journey smoke | still owed — the Mac2 driver cannot enable automation mode |

---

## 2026-09-22 — the toast that sat on the download

One tail from the bundle-flow survey went unchecked until now: the toaster and
the download progress panel live in the same corner. I measured it on the stand
rather than reasoning about it, and they do collide. At 1024×640 the panel
occupied 692..1012 × 550..628 and an error toast covered 644..1000 × 542..616 —
and sonner's viewport carries `z-index: 999999999` against the panel's `60`, so
the toast wins every time. Hidden: the panel's title, its Cancel button, and the
files/bytes counters.

The pairing is not a coincidence. A toast during a bundle download is almost
always *about* the download — a file that failed its retries, a preview that
could not be read. So the operator learns one file failed and simultaneously
loses sight of how far the rest got, which is the number that decides whether
to keep waiting on a tethered phone.

The panel is the one that cannot move: the map keeps its controls in the left
corners and the panel has to stay above the bundle-loader sheet. So the toaster
steps aside, and only while there is something to step aside for. `DownloadPopup`
publishes its measured height; the layout derives sonner's `offset` from it.

Measured after the change, same viewport: panel 522..628, toast lifted to
437..511, an 11 px gap, no overlap; when the download finishes the height store
returns to 0 and the toast drops back to its usual 534..608.

One thing the tests would not have caught and the measurement did: a panel
measured mid-mount can report a height of a few px, which would have put a toast
*closer* to the edge than normal. The offset is clamped to sonner's own edge
offset, and the test for that failed before the clamp existed.

| | |
|---|---|
| Change | `openspec/changes/progress-stays-readable/` |
| Measured | on the stand, before and after, geometry above |
| Automated gates | `just ci` green (320 Rust, 439 frontend) |
| Customer-journey smoke | still owed — the Mac2 driver cannot enable automation mode |

---

## 2026-09-22 — a waiver that cannot rot

I have declined the MapLibre 4 → 6 upgrade three times, each time for the same
reason, and never once checked whether the grounds for declining still held.
That is how a waived critical advisory turns into an unwaived one without
anyone noticing.

Its own recheck condition names the code I had spent the day changing: "if any
code starts calling `Popup.setHTML()`, Marker with custom HTML, or custom
attribution strings". Waypoint glyphs, waypoint colours, the measuring layer —
all of it touches markers.

It still holds. Popups use `setText`, markers set `textContent`, there is no
`innerHTML` and no custom attribution anywhere in `src/`.

But "we promise not to call it" is not a defence, so the premise is a test now.
It fails on `setHTML`, `innerHTML` or `customAttribution` anywhere in the
source, naming the file and line, and says in its failure message that the
alternative is to remove the waiver and upgrade. I checked it bites by writing
`el.innerHTML = waypointGlyph(...)` in the marker and watching it name the
line.

The version facts, so nobody has to re-derive them: `npm audit` says vulnerable
`<=6.4.0`, fixed in **6.10.0 only**. There is no 5.x backport — the 5 line ends
at 5.24.0 and is still vulnerable — so the pinned 4.7.x cannot be patched into
safety and the route really is two majors. The waiver in
`scripts/npm-audit-gate.mjs` now carries that, dated, with the command that
produced it.

| | |
|---|---|
| Evidence | the premise re-checked against the code; the guard, verified red |
| Evidence | `npm audit --json`: `range <=6.4.0`, `fixAvailable 6.10.0`, `isSemVerMajor` |
| Automated gates | `just ci` green (320 Rust, 431 frontend); the audit gate passes with one waiver |

---

## 2026-09-22 — walking the recording

The last ADR-0020 item, and not the onboarding tour the word "walkthrough" had
me expecting: stepping through a track's points one at a time. Points were
click-selectable and that was all, so reviewing a recording meant clicking each
row and losing your place the moment the list scrolled.

Previous and next in the points table's header, with "3 из 128" beside them so
the operator knows where in the recording they are. Stepping takes the map with
it — a walkthrough that left the map where it was would be a list. The order is
the track's own across its segments, because a recording made in two sittings
is one walk and stepping off the end of the first should land on the start of
the second. From nothing selected, next goes to the first point and previous to
the last; that is what they mean from nowhere. Neither wraps: a silent jump
from the last point to the first reads as a fault.

**And the backlog paid for itself.** The first `$derived` over `trackDetail`
failed with "Property 'segments' does not exist on type 'never'" — the exact
trap I had recorded two slices earlier, in a different file, after losing time
to it once. This time it took a minute, because the note said what it was:
`let x: T | null = $state(null)` is narrowed to `null` by flow analysis at
every point before its first assignment, which is where every inline derived
sits.

Four other declarations carried it latently — compiling only because nothing
read them in a derived yet. They are converted, and there is a guard now,
scoped to `$state(null)`: an array narrows to `never[]`, which is assignable to
anything and harmless, and a guard that flagged those would be one somebody
turns off.

| | |
|---|---|
| Evidence | walked on the stand — nothing selected, then "1 из 5", "3 из 5", back to "2 из 5", crossing the segment boundary |
| Evidence | the guard, verified red by putting one declaration back |
| Automated gates | `just ci` green (320 Rust, 430 frontend) |
| Customer-journey smoke | not run — the Mac2 driver host crashes at session creation (`docs/STATE.md`) |

---

## 2026-09-22 — trimming the drive to the start

ADR-0020 declares cropping a track by selection; crop by extent and crop by
time exist, and neither is the gesture a crew actually has.

The commonest edit to a recording is that the first twenty minutes of it are
the drive to the start. Cropping by time does that — if they work out what time
the walking began. Cropping by extent does it — if the drive happens to fall
outside a rectangle they can draw. What they *have* is the point: they can see
where the track stops being a road and starts being a search, on the map and in
the points table.

So: two controls on each point's row. Trim everything before this point, trim
everything after it. The named point survives either way — it is where the walk
starts or ends, and removing it would be off by one in the direction nobody
checks.

No new command in the core. `CropTrackPoints` already takes the exact points to
remove and restores them to their places on undo; what was missing was a caller
that knows *order*, which a per-point predicate cannot. The plumbing takes an
`FnMut` now and the rule walks the track flipping once it reaches the point.

Two trims compose into a crop by selection — cut before the start, cut after
the end — which is the same result with half the interface, and each half is
useful alone.

A trim at the first or last point removes nothing, says so, and records no undo
step. An edit that changes nothing should not be something to undo.

| | |
|---|---|
| Evidence | Rust tests: both directions, the point survives, one undo restores all five, and a trim at an end leaves the mutation count untouched |
| Automated gates | `just ci` green (320 Rust, 429 frontend) |
| Customer-journey smoke | not run — the Mac2 driver host crashes at session creation (`docs/STATE.md`) |

---

## 2026-09-22 — a link a coordinator sent

`product-scope` declares opening a bundle directly by URL and it had never been
built. A link is how a search arrives: someone sends
`https://maps.lizaalert.ru/maps/2026-09-21_Vesta/` over a messenger. Typing the
name back in instead is error-prone in the way that costs time in the field —
the names are latin transliterations of Russian place names, and the one in the
link is the one that is exactly right.

**No new control.** Paste it into the loader's search box, which is where a
paste naturally lands; text that is not a link is still just a filter, and the
box keeps the readable name afterwards rather than the URL, so the operator can
see what they landed on.

The parsing is lenient about what a messenger does to a link — a lost scheme, a
missing trailing slash, a `?utm_source`, a percent-encoded Cyrillic name — and
strict about the host, so `evil.example/maps.lizaalert.ru/maps/x/` is not one.
Anything that is not a catalogue link yields nothing rather than a guess: a box
that jumped to a project because someone typed a word with a slash in it would
be worse than one that did nothing.

The case that will actually happen: a link sent minutes after the search was
created, for a project the catalogue has not walked yet. That says so and names
the slug, because it is not an error and "nothing happened" would be the wrong
thing to show.

| | |
|---|---|
| Evidence | six tests on the parsing, including the lookalike host and the catalogue root |
| Evidence | walked on the stand — a pasted link previews that slug and leaves "2026 09 20 Schuvalovo" in the box; an unlisted one shows the message and previews nothing |
| Automated gates | `just ci` green (318 Rust, 429 frontend) |
| Customer-journey smoke | not run — the Mac2 driver host crashes at session creation (`docs/STATE.md`) |

---

## 2026-09-22 — the three on-map tools

ADR-0020 puts distance measurement in the MVP and it had never been built. It
is the measurement a search crew takes constantly — how far is that from the
task point, how wide is this clearing, how long is the leg we are about to walk
— and without it they measure by eye or open another program.

I had it filed as blocked on *where it goes*, because `ui-shell` requires the
four mode chips above the canvas to stay inert scaffolding. That was a
misreading of my own note: `product-scope` names the command palette as a place
a workspace action may live, and it is where the other verbs already are. The
block was mine, not the spec's.

Cmd-K, "Измерить расстояние". Click points, read the number over the canvas —
not in the status bar, because a crew reads it where they are clicking. Esc
finishes, as it cancels a draw. The same palette entry puts the tape away,
labelled for the state it is in.

Three decisions worth stating. A click while measuring is taken **whole**, so
it does not also select a track or drop a waypoint. The points are **scratch** —
never saved, gone when the tool goes off; a measurement worth keeping is a
track, which they can already draw. And the arithmetic is the frontend's, using
the same haversine and the same earth radius as `domain/track.rs`, so a
measured leg and a track's length are the same number for the same two points —
a tool that argued with the list beside it would be worse than none, and a
round trip per click is what makes a tool feel slow.

Metres under a kilometre. "180 м", not "0.2 км": the difference between 40 and
140 metres is the difference between two sides of a road.

I deferred the drawn line as "decoration" and that was wrong. A crew clicking
on a map at night has to see where the points went — whether the click
registered, whether it landed on the road or beside it, what shape the thing
they are measuring has. A running total with nothing under it is a number they
cannot check. So: a dot at each click, a dashed line through them, in an orange
that is neither a track colour nor a waypoint colour, so the tape never reads
as something in the project. And Backspace takes back the last point, because
misclicks happen and starting over because of one is worse than the misclick.

That last one wanted `isEditableTarget`, which lived inside the layout. Copied,
the two handlers would have had their own ideas of what "editable" means, which
is how they come to disagree about a single keypress. It is a module now, with
its own tests.

One thing I could not verify, stated rather than glossed: the tape's **pixels**.
MapLibre does not set `preserveDrawingBuffer`, so reading the canvas back after
a frame is presented returns an empty buffer — I got zero orange pixels, and
that zero means nothing either way. What is pinned instead is the GeoJSON the
tape hands MapLibre, including the `lon, lat` order a map gets wrong once.

And the other on-map tool ADR-0020 declares, in the same slice because it
shares everything the tape built: the **radius ring**. Everything within five
hundred metres of the last known position is a thing a search draws constantly.
Click a centre, click a radius, click again to move it — a crew drawing rings
draws several and should not reach for the palette between them.

The ring is geodesic, and that is the whole point of it. A circle drawn flat in
screen pixels is right only at the equator: at 60°, where these searches are, a
"500 m" circle drawn flat is half a kilometre north-south and a kilometre
east-west, and the crew standing in it is looking in the wrong place. The tests
hold every point of a 500 m ring and a 25 km ring to its radius at that
latitude, and check that a ring near the antimeridian does not come out as a
band around the world.

The two tools take the same click, so they cannot both be listening. Turning
one on turns the other off and discards what it held — a stale point left
behind would be measured into the next measurement, which is the kind of wrong
number nobody questions.

And the third, which completes what `product-scope` asks for: **placing a
waypoint by bearing and distance**. Unlike the other two it is not click-driven,
because "from the task point, 240° and 1.2 kilometres" is dictated over a radio,
not pointed at. Click the origin, type the two numbers, and the point lands
there — previewed on the map first, because a bearing heard over a radio is easy
to mishear and seeing it is how that gets caught. "Place" stays disabled until
there is a distance to place at.

All three take the map's clicks, so only one listens at a time, and switching
discards what the previous one held.

Found beside it: the default name for a waypoint placed by clicking was
`Waypoint N`. English, in the app's own Russian window, on a path I had walked
past twice.

| | |
|---|---|
| Evidence | walked on the stand end to end — the palette turns it on, the readout reads "0 м · Клик — мерить · Esc — закончить", clicks on the canvas make it "101 м" and then "146 м", Esc takes it away |
| Evidence | the ring on the stand: "Клик — центр", then "Клик — радиус", then "52 м" |
| Evidence | the projection on the stand: the hint, then the fields, then 240° and 1200 m placing "Точка 4" at the computed coordinates |
| Evidence | tests on the distance, the formatting, the tape's GeoJSON and what counts as an editable target |
| Not verified | the tape's rendered pixels — see above |
| Automated gates | `just ci` green (318 Rust, 423 frontend) |
| Customer-journey smoke | not run — the Mac2 driver host crashes at session creation (`docs/STATE.md`) |

---

## 2026-09-22 — an old project still opens

Yesterday's waypoint colour turned up a class rather than an instance. The
whole `.ozp` format rests on `#[serde(default)]` being in the right places: a
field added to any persisted struct without one turns **every project a crew
has ever saved** into a load error, and the only place that shows up is a load.
Nothing in the build says a word.

That is not a nicety. The format has no version field and no migration path —
tracked as CJ-8 and still open — so "the older file still loads" is the entire
compatibility story.

What existed guarded the project shell: a legacy file with no layers at all.
Nothing guarded what is inside them. There is now one file carrying a track of
one segment and two points and a waypoint, with only the fields that have
always existed, and it asserts what absence is supposed to *mean* — a track
with no style recorded is visible, a waypoint with no symbol has none.

It passed on the first run, which is the finding: the format is genuinely
tolerant, and nobody had checked. Now it stays that way. I removed one
`#[serde(default)]` — from `Track.style` — and watched the test name the field
it could no longer read.

Then the same guard for the session file, which fails worse: `load_app_session`
returns the parse error rather than `None`, so a session that will not read
takes the restored project and the active map with it — the crew opens the app
to an empty workspace and no explanation. A corrupt session is now distinguished
from an absent one, because absent is a first run.

**And a correction to the paragraph above.** Writing that test I removed the
`#[serde(default)]` from `bundles_root` expecting it to fail, and it passed.
Serde reads a missing `Option` field as `None` whether you tell it to or not.
So the attribute on `bundles_root`, and the one on `Waypoint.color` I made much
of yesterday, are belt-and-braces; what they are credited with, those fields
have anyway. The rule is narrower than I wrote it: it is a **non-`Option`**
field that needs the default, `Track.style` is one, and that is the removal
that did make the guard fire.

Two code comments and yesterday's write-up said the wrong thing and now say
this.

| | |
|---|---|
| Evidence | the guard, verified red by removing the default from `Track.style` — which is not an `Option` and so depends on it |
| Evidence | the session guards, and the check that disproved my own claim |
| Automated gates | `just ci` green (318 Rust, 395 frontend) |

---

## 2026-09-21 — whose mark is this

The symbol says what a mark is; the colour says whose it is. Group A's marks
against group B's, on one map, in a field HQ, at night — a search collects
marks from every group working it, and symbols alone cannot carry that. Tracks
have had per-track colour from the beginning. Waypoints had none, and ADR-0020
declares it in scope.

The care is all in one distinction: **having no colour is not having the
default one.** A waypoint that has never been coloured follows whatever the map
draws waypoints with, so changing that default later moves every uncoloured
waypoint with it; clearing returns a waypoint to that rather than to a colour
that happens to look like it. `Option<[u8; 4]>` end to end, `#[serde(default)]`
so a project written before this loads with every waypoint uncoloured — which
is what it meant.

Two things this cost that were worth the finding.

`svelte-check` refused `waypoint?.color` with "Property 'color' does not exist
on type 'never'", which reads like a broken type and is not. `let waypoint:
WaypointData | null = $state(null)` is narrowed by TypeScript's flow analysis
to `null` at every point before the first assignment — which is every inline
`$derived` in the file, so the non-null branch is `never`. Ordinary functions
never see it because their bodies are deferred. `$state<T>(null)` declares the
type instead of narrowing to it. The declaration says so now.

And beside the symbol handler, `toast.error("Failed to change symbol")` —
English, in a file I had translated twice. The silent-failure guard let it
through because it *does* toast; the label guard does not look inside `toast`
calls. Fourth miss, same shape as the others: a guard on syntax catches the
careless version.

A colour that does not survive being saved is a lie, and the round trip that
would have caught it was over a *bare* waypoint — a round trip over defaults
proves only that defaults survive. It carries a symbol, a colour and a
visibility flag now.

The case that actually costs a crew their work is the other one: a `.ozp`
written before the field existed. It rests on a single `#[serde(default)]`,
and a field added without one turns every older project into a load error.
There is a test, and it is built from this build's own output with the `color`
key deleted rather than from JSON I wrote — my hand-written version failed on a
field I had not known the format carried, which is the whole argument against
hand-written fixtures. Both tests go red when the field is made
`#[serde(skip)]`; I checked.

Then the obvious hole in my own feature: the colour was on the map and in the
inspector, and the **list** — the thing a crew actually reads to find what they
are looking at on the map — knew nothing about it. The row's symbol button is a
disc now, the same disc the marker draws, from the same conversion. Three
surfaces, one function, so they cannot disagree.

| | |
|---|---|
| Evidence | a Rust test over set, set again, undo, redo, and clearing back to the default |
| Evidence | walked on the stand: 🏁 on `rgb(37, 99, 235)` in the row *and* on the marker, the others on the default in both |
| Evidence | tests on the shared conversion, including alpha and a single-digit channel |
| Evidence | the project round trip and an older `.ozp` — both red when the field is made `#[serde(skip)]` |
| Automated gates | `just ci` green (315 Rust, 395 frontend) |
| Customer-journey smoke | not run — the Mac2 driver host crashes at session creation (`docs/STATE.md`) |

---

## 2026-09-21 — yesterday's project is one key away

The item I went looking for last slice, built. Reopening yesterday's search
meant finding the file in a dialog again; the palette's recents were recent
*maps*, which answers a different question.

A crew comes back to the same search for days, and the one they want is almost
always the one they had open last. Cmd-K now offers the last eight, by file
name with the path beneath.

Two details that matter more than the list itself. **Both saves record**, not
just the open dialog: a save is where a never-saved project first gets a path
at all, and the project just written is the one most likely wanted next — and a
failed save records nothing, so a read-only volume does not plant an entry that
never opens. And **a path that fails is dropped**, with a message, rather than
sitting in the list failing every time it is chosen. Paths go stale: the file
moved, the disk is not mounted, the crew is on the other machine.

Not merged with `recentFiles.ts`, and the module says why: the records differ,
the shared part is forty lines of storage plumbing around a working feature
with its own tests, and two is not yet a pattern.

| | |
|---|---|
| Evidence | seven tests on the list, three on the save wiring including the failed save |
| Evidence | walked on the stand: the palette shows "Недавние проекты" with the file name above its path |
| Automated gates | `just ci` green (313 Rust, 392 frontend) |
| Customer-journey smoke | not run — the Mac2 driver host crashes at session creation (`docs/STATE.md`) |

---

## 2026-09-21 — the project file is `.ozp`

Went looking for the declared-but-absent "recent `.ozp`" and found something
before it. The recents in the palette are recent *maps*, not projects — so the
item is genuinely absent — but on the way there, both project dialogs turned
out to filter on `json`.

The format is `.ozp`: `AGENTS.md` says so, `docs/project-map.md` says so, and
`project-persistence` requires a save "to a user-chosen `.ozp` file". So what a
crew saved was not the format the documentation names, and — the part that
costs them something — **an `.ozp` was invisible in the open dialog.** A filter
hides what it does not match. A project from an older build, from a colleague,
from a USB stick was simply not there to select.

The divergence was known: the capability's own Purpose carried a reality note
about it. Owner decision 1 says the code is fixed where the owner wants the
behaviour, and the requirement is unambiguous.

There were two dialogs and no shared value, which is how they came to disagree
with the documentation and agree with each other. One module now. Open still
accepts `json`, because everything this app has saved until today carries it,
and a filter that hid those would lose a crew their work far more surely than
the wrong extension ever did.

A CJ-7 test had `extensions: ["json"]` written into it as the contract. It was
pinning the defect. What it pins now is that the filter comes from the shared
module; the value is covered behaviourally beside the dialog.

| | |
|---|---|
| Evidence | tests on what Save offers, what Open accepts, and that a cancelled dialog saves nothing |
| Automated gates | `just ci` green (313 Rust, 382 frontend) |
| Customer-journey smoke | not run — the Mac2 driver host crashes at session creation (`docs/STATE.md`) |

---

## 2026-09-21 — the last of the findings

Two lines left on the code-findings list, both now answered rather than
carried.

**`get_ozi_tile` registered but unused** is true and harmless: the map renders
OZF2 through `get_ozi_tile_projected`, via the `ozi://` protocol. Dead IPC
surface, not a missing feature. Worth knowing before someone reads it as "OZF2
maps do not display".

**cp1251 unrepresentable characters** turned out to be a finding about a
comment. The code says `encode()` substitutes `?` "which matches what legacy
OziExplorer expects"; `encoding_rs` substitutes an HTML numeric reference for
every legacy encoding, so it was never true, and the second half of the
sentence is a claim about a third-party program that this code cannot check.
Both are gone. The behaviour is pinned instead, and the question of which a
real OziExplorer would rather read is in the backlog for whoever has one.

The test caught me first. I wrote it with a Ukrainian `і`, assuming that is
outside cp1251. It is not — cp1251 carries `і`, `ї`, `є`, `ґ` and `ў` — and the
character came through intact, which is how the assumption was caught rather
than recorded. It takes an emoji or a Latin-Extended character to reach the
replacement path, and an emoji typed on a phone is the likelier one anyway.

| | |
|---|---|
| Evidence | a waypoint named with an emoji: the file carries `&#128205;`, the Cyrillic around it untouched |
| Automated gates | `just ci` green (313 Rust, 379 frontend) |

---

## 2026-09-21 — a waypoint looks like what it is

Every waypoint on the map was the same yellow dot. The symbol a crew picks —
flag, camp, danger, water, meeting point — was stored, listed in the Waypoints
tab and carried into GPX, and the one place it did not appear was the place
they look at. A search area collects the task point, what was found, where the
danger is and every group's marks; ten identical dots distinguish none of it.

The table lived inside the picker, which is why only the picker knew anything.
It is a module now, read by both, and the marker draws the glyph on the disc
rather than instead of it — a glyph alone over aerial imagery is unreadable,
and a marker has to be findable before it can be identified. 22px instead of 14
to fit one.

Two smaller things fell out. The applied-marker record now carries the symbol,
so changing a symbol redraws the marker instead of waiting for the waypoint to
move. And a symbol this build does not know — from a GPX another tool wrote —
falls back to the default pin rather than leaving the waypoint undrawn: it is
still there and must still be findable.

The picker's ten labels turned out to be English literals inside its table.
The label guard did not see them, and could not: they are not written into the
attribute, they are looked up from data. That is the third case the guard has
missed, which is roughly what a guard on a syntactic shape is worth — it
catches the careless version, not every version.

| | |
|---|---|
| Evidence | walked on the stand: the flagged waypoint draws 🏁, the one without a symbol 📍, both computed 22×22 |
| Evidence | tests on the lookup, including the unknown-symbol fallback |
| Automated gates | `just ci` green (312 Rust, 379 frontend) |
| Customer-journey smoke | not run — the Mac2 driver host crashes at session creation (`docs/STATE.md`) |

---

## 2026-09-21 — re-reading the findings before the owner does

`codify-architecture-decisions` carries two lists of findings the owner reads
at review. They were written on 19 September, and a fortnight of slices has
happened to them. Re-checked against the code, every line:

Of the three "spec contradicts code" items, two are closed by the code having
been fixed, which is the owner's own decision 1 — bundles-root persistence
landed in slice 0.3, and waypoint GPX export is registered and reached from two
surfaces. The third stands: the theme picker is still imported by nothing.

Of the code findings, six are closed. One **was never true as written**: "zip-slip
guard absent on the track-import archive path". `extract_zip_entries_to_directory`
has always used `enclosed_name()`, which refuses an entry that escapes its
directory, and both extraction paths — track import and cached bundle archives
— go through it.

What was actually missing there was a test. A ZIP is untrusted input: a crew
imports one another group sent them, and an entry named `../../something`
would, extracted naively, write over whatever the path reaches. The guard is a
single call with nothing visible depending on it, which is exactly what a
refactor drops without noticing. Two tests depend on it now — one that the
extraction refuses such an entry and writes nothing, one that the inventory
shown to the operator beforehand lists it rather than hiding it, so the listing
and the refusal agree.

| | |
|---|---|
| Evidence | a crafted archive with a `../escaped.gpx` entry: refused by name, nothing written beside the destination |
| Automated gates | `just ci` green (312 Rust, 375 frontend) |

---

## 2026-09-21 — the docs stop lying

`docs/STATE.md` has carried "human-facing docs still lie in places" under Known
broken since the review. It is task 2.6 of `codify-architecture-decisions`, and
it is not owner-gated — it just needed someone to check each claim against the
code rather than against the last person who wrote it down.

Every one below was verified before it was edited, and four of them were worse
than stale — they said a feature does not exist:

- **`sort_track_points` "does not exist".** It is a registered command with an
  `api.ts` wrapper and a caller in the Track Inspector.
- **Crop "no backend commands".** `crop_track_to_extent` and
  `crop_track_to_time` exist and are reached. Crop by selection genuinely does
  not, and the row says so now instead of lumping all three together.
- **Split/join "no UI entry point".** The segments table calls both.
- **ZIP import "not reachable, the picker filters to .gpx/.plt".** The picker
  takes `["gpx", "plt", "zip"]`.

The rest were plain untruths: "no async runtime" (there is a tokio one, scoped
to the download path, and the rule about not holding the mutex across an await
is not decorative there any more); "no Ctrl+Z bindings" (the layout binds
Cmd/Ctrl+Z and Shift for redo); "`ozf2-rs`, local crate" (it is `ozf2` 0.1 from
crates.io); and the glossary's "DTO mirrored manually", which is true only of
the ones that travel as event payloads — the rest are generated.

Seven rows of the status matrix said `TBD`, which is not in the vocabulary that
capability allows. They say `pending`.

The command reference was missing nine commands. It now also says what it is:
the registry in `lib.rs` is the list, this page describes it, and when they
disagree the registry is right. A page that claims to be exhaustive goes stale
the day someone adds a command; one that says where the truth lives does not.

Found on the way, not fixed because the placement is a design call: **`ThemePicker.svelte`
is imported nowhere.** The `ui-shell` spec has requirements about a theme
selector and its persistence; the component is unreachable. I localized its
labels yesterday, which was polish on dead code.

| | |
|---|---|
| Evidence | each claim checked against the code before editing — five commands traced from `lib.rs` through `api.ts` to their callers, the import filter read, `Cargo.toml` read for `ozf2` and `tokio`, the keydown handler read |
| Automated gates | `just ci` green (310 Rust, 375 frontend) |

---

## 2026-09-21 — what survives the trip

FTP upload of tracks is planned, and the file that goes up is a GPX this app
writes. Both halves of that were tested apart — the writer's XML, the reader's
parse — and nothing checked that a track handed to one comes back from the
other. For a search record that matters in specifics: the Cyrillic name, the
break between two sittings, every coordinate in order, and the times that make
a track a timeline rather than a shape.

It passes. All of that survives, and is now pinned.

What does not is the colour: the writer emits `gpxx:DisplayColor`, the reader
drops it, because the `gpx` crate does not surface extensions at all and
reading one needs a second pass over the XML. Nothing about the track is lost
by that — the colour lives in the `.ozp`, and GPX is the interchange format,
not the record — so the gap is pinned rather than closed, and the test fails
when someone closes it.

The first version of that test asserted against red. Red is the default track
colour, so it would have passed whether or not the colour survived: it would
have recorded a fact that was not one. It caught itself only because I ran it
expecting it to fail and it did not.

Waypoints got the same treatment, since they are the half of a search record
that names things — the task point, what was found, where the danger is. Name,
coordinates and symbol all survive, and a waypoint that went out without a
symbol comes back without one, which matters: acquiring a default would put a
mark on the map nobody placed.

Then I surveyed the other two formats rather than assuming, because the point
of this exercise is to know. **PLT** — what OziExplorer itself reads — already
round-trips, name in Cyrillic, colour and width included; better covered than
GPX was. **WPT** has no importer, deliberately, and `product-scope` says so;
its output is pinned line by line instead, exact v1.1 header, CP1251, CRLF, six
decimal places.

So the interchange story is now known rather than assumed, in all four
directions.

| | |
|---|---|
| Evidence | round trips over a two-segment track and over two waypoints; the colour gap against a colour that is not the default |
| Automated gates | `just ci` green (310 Rust, 375 frontend) |
| Customer-journey smoke | not run — the Mac2 driver host crashes at session creation (`docs/STATE.md`) |

---

## 2026-09-21 — reading the specs I had been writing against

Yesterday's slice caught one of my own changes contradicting a baseline
requirement instead of modifying it. One is a slip; twenty-six unarchived
changes made by the same hand is a reason to look. So I read the baselines for
every capability this session has touched, against every requirement it
declares.

Three more contradictions, all of them mine:

- **The catalogue cache.** `Catalog cache merges refresh deltas without
  dropping known entries` says the cache "SHALL accumulate the union of entries
  the frontend has ever observed" and is "a superset of any single refresh",
  with a scenario spelling out that five missing entries are not removed. The
  pruning slice does exactly the opposite. Modified now, and it says why the
  old rule was right when written: a complete refresh could not then be told
  from an interrupted one.
- **Track statistics.** `System computes and surfaces per-track statistics`
  says the row shows distance and point count in every case; a one-point track
  now shows the count alone. Modified, with the reason — those tracks had no
  row at all until the list stopped coming from the map's geometry.
- **The partial file.** `codify-architecture-decisions`, written but not
  archived, codifies "a network error or cancellation mid-stream SHALL remove
  the `.part` file", with a scenario asserting none remains. Resumable retries
  keep it between attempts. Amended there rather than left to contradict, since
  archiving both in sequence would leave the baseline saying two things.

The pattern behind all four: writing a change against the code in front of me
rather than against the requirement that already covered it. Nothing catches
that — `openspec validate --strict` checks a change's shape, not whether it
disagrees with the baseline — so it is in the backlog as something to read for,
deliberately, before an archive.

| | |
|---|---|
| Evidence | four deltas rewritten from ADDED to MODIFIED or amended in place; `openspec validate --changes --strict` green across 26 |
| Automated gates | `just ci` green (307 Rust, 375 frontend) |

---

## 2026-09-21 — one flaky file is not the bundle

Slice 0.3 gave the transfers timeouts, so a stalled connection fails instead of
hanging for ever. It did not give them a second try, and one file's failure
failed the whole bundle — after however many files had already come down.

That is the wrong trade for the link a bundle is fetched over. Hundreds of
megabytes across a handful of files, pulled over a tethered phone at the edge
of coverage: a dropped connection there is ordinary, not exceptional. Losing
the whole download to one, and starting again, is how a crew ends up without a
map.

Three attempts per file, half a second between them — short, because the
operator is standing there. Cancellation is never retried: it is the operator's
instruction, not a transport failure, and retrying it would keep the link busy
after they asked it to stop. And a retry says so through the keyed progress
channel, because a retry that looks like a stall is a stall to the person
watching the bar.

And then the part that matters on the transfer that actually needs a retry: it
resumes. Starting the file again is no use when the connection drops near the
end of a 185 MiB map — that costs the map twice, and on a link that keeps
dropping it may never land at all. The second attempt asks for the rest with a
`Range` request, so the partial file is kept between attempts and removed only
once the file is given up on. A server that ignores the range answers 200
instead of 206; then what is on disk is worthless and the file starts again,
which is the only safe reading of that answer.

That reverses a contract the old test spelled out — "a stalled transfer SHALL
clean up its .part file". Deliberately, and the test says why now. Nothing
treats a `.part` as a map, so keeping one does not make the bundles root look
complete when it is not.

Writing this up also caught a mistake in the slice before it: the catalogue
pruning was written as an ADDED requirement while the baseline said the
opposite — "entries absent from a refresh SHALL NOT be removed". It is a
MODIFIED requirement now, and says why the old rule was right when it was
written: there was no way then to tell a complete refresh from an interrupted
one.

| | |
|---|---|
| Evidence | a mock server that fails once and then serves: the file lands whole, after exactly one retry |
| Evidence | a raw server that sends half and hangs up: the second request carries `Range: bytes=512-`, the file is 1024 bytes, and the progress bar never goes backwards |
| Evidence | a cancelled transfer is not retried; a file given up on leaves no `.part` |
| Automated gates | `just ci` green (307 Rust, 375 frontend) |
| Customer-journey smoke | not run — the Mac2 driver host crashes at session creation (`docs/STATE.md`) |

---

## 2026-09-21 — a refused edit says so

The backlog had run thin on things that are not the owner's to decide, so I
audited the one field path never audited: editing a track.

Ten failures reached `console.error` and nothing else, and the error reporter
is disabled outside dev builds. In a release build they were silent. Not
peripheral ones either — dragging a track point, deleting one, inserting one,
adding a waypoint, placing a point while drawing, cancelling a draw, fitting
the tracks on screen, loading either inspector. That list is the field
workflow.

The track-point drag was worse than silent. On failure the marker stayed where
the operator dropped it while the data kept the old position, so the map went
on showing a point that was not there.

The loader was given exactly this treatment months ago, in `honest-bundle-flow`
— "every failure in the loader was swallowed by an empty `catch`". The map
never was, and nothing was watching. Now something is: a test that scans every
component for a `catch` that logs and tells nobody. Two are allowed through,
each with its reason written next to it — OZF2 metadata, which is meant to fail
for SQLite maps, and the per-file import failure, which is reported in
aggregate afterwards so one bad file does not bury the summary.

That is the third guard of this shape this session. They keep finding things
the sweep that prompted them had already missed.

| | |
|---|---|
| Evidence | tests on the reporter itself; the guard, verified red by putting one `console.error` back |
| Not covered | the ten call sites — `MapView` and the inspectors need a MapLibre instance |
| Automated gates | `just ci` green (303 Rust, 375 frontend) |
| Customer-journey smoke | not run — the Mac2 driver host crashes at session creation (`docs/STATE.md`) |

---

## 2026-09-21 — a search that is gone leaves the list

The thing the backlog note meant, done. A search taken down upstream never
left: chunks are merged — append what has not been seen, remove nothing — on
both sides, so it stayed in the list, then in the cache, and offline it was
still there to click and fail on.

The whole difficulty is one fact: whether the walk ran to the end. A stopped
walk read a prefix, and pruning on that would make "stop" mean "delete most of
the list". So the walk's boundaries are emitted now, and between them the
interface collects what the walk sent; on a complete walk it drops the rest.
The cached chunk goes out *before* the window opens, deliberately — counting
yesterday's cache as proof that a search still exists would defeat the point,
and there is a test for exactly that.

And a correction to the last slice. Taking the catalogue out of the state
snapshot left the stand with an empty project list, which I did not see because
I had stopped the stand before making the change. The core writes
`catalogue.json` now, and the stand plays the real sequence — cached chunk,
started, the walk's chunk, finished — so it exercises the pruning rather than
hiding it. Three rows, "3 из 3", not emptied by the complete walk.

| | |
|---|---|
| Evidence | two Rust tests (complete walk replaces, stopped walk does not) and five on the store, including the chunk that arrives before the window |
| Evidence | walked on the stand: the catalogue is back and survives a complete walk |
| Automated gates | `just ci` green (303 Rust, 372 frontend) |
| Customer-journey smoke | not run — the Mac2 driver host crashes at session creation (`docs/STATE.md`) |

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
