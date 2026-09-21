# Progress gallery

One entry per merged slice, newest first. Each entry links before/after
screenshots so the state of the interface can be read in one page instead of
being reconstructed from commits.

Screenshots come from the screenshot matrix (`just shots`) once that lands;
until then they are cropped captures of the real window taken through the
native QA harness (`docs/native-qa-mcp.md`).

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
