# Backlog

Things worth doing that are not in the current slice. Each entry says why it is
here rather than in the plan, so picking one up does not require re-deriving the
decision.

## Product

- **FTP as a second source for the catalogue.** `maps.lizaalert.ru` runs an FTP
  service ("220 DB Based FTP ready") that would give a machine-readable listing
  instead of scraped HTML — which is exactly what broke in September when the
  site changed its markup. Anonymous login is refused (`530 Authentication
  failed`), so it needs credentials. Owner decision (2026-09-21): credentials
  are entered by the user in application settings, never shipped in the build;
  the password belongs in the macOS keychain, not in a config file. Scope: a
  settings form (host, user, password), keychain storage, an FTP listing
  adapter behind the same interface the HTTP one implements, and a preference
  for which source to use.
- **Transparent no-data areas on OZI rasters.** A LizaAlert satellite sheet is
  cropped to the search area and the rest is filled solid black in the source
  file, so it hides the OSM basemap underneath. OziExplorer offers colour-keyed
  transparency for this; ozi-rs passes the pixels through unchanged. Would need
  a per-map setting, since black is legitimate image data elsewhere.
- ~~Shorter names for import-created track layers.~~ Already done, confirmed on
  2026-09-21: every layer-creating import path — single `.plt`, archived GPX
  tracks and archived GPX waypoints — names the layer through
  `source_file_label`, which keeps the file name and drops the path.
- **Ascent and descent as track statistics.** The elevation profile landed on
  2026-09-22 without them on purpose: summing every rise in a GPS recording
  sums its own noise, so a naive total reads high by a wide margin. It needs a
  threshold below which a change is not a climb — the same shape of decision as
  the moving-time threshold below, and best answered together with it.
- **Moving time as a track statistic.** `duration_seconds` is the span between
  the first and last point, which reads oddly for multi-day recordings even now
  that it is formatted in days. Moving time needs a stop threshold the owner has
  not chosen.
- ~~Waypoint export to GPX.~~ Done on 2026-09-21: both the Waypoints tab row
  menu and the Waypoint Inspector offer GPX and WPT.

## Engineering

- **ADR-0020 asks for waypoint export to PLT, which cannot mean what it says.**
  PLT is OziExplorer's track format — a track header and track rows, no
  waypoint record. Its waypoint format is WPT, which is implemented. Most
  likely the ADR meant WPT and the row in `docs/feature-status.md` has been
  reporting a gap that does not exist. Needs the owner to confirm, and then the
  ADR or the matrix corrected.

- **The `.ozp` format has no version field and no migration path** (CJ-8). Its
  entire compatibility story is that an older file still reads. An `Option`
  field gives that for free — serde reads a missing one as `None` without being
  told — so the attribute on such a field proves nothing; it is a
  **non-`Option`** field that needs `#[serde(default)]`. A test now enforces it
  (`a_project_from_an_early_build_still_loads_with_its_contents`). If a field
  ever genuinely cannot have a default, that test is where it will surface, and
  the answer then is a version and a migration rather than a weakened test.

- ~~Recent projects are recent maps.~~ Done on 2026-09-21 in
  `yesterdays-project-is-one-key-away`: `src/lib/recent-projects.ts` beside the
  map list, recorded on open and on both saves, offered in the palette, and a
  stale path is dropped when it fails.

- **A WPT name with a character cp1251 cannot hold becomes `&#NNNN;`.**
  `encoding_rs` does that for every legacy encoding; the code's comment claimed
  it becomes `?`, which was never true and is corrected. Which a legacy
  OziExplorer would rather read is a question for whoever has one in front of
  them: `&#128205;` keeps the character recoverable, `?` destroys it, and both
  look wrong in a waypoint list. Pinned by a test in `export/wpt.rs`. Note that
  cp1251 does carry `і`, `ї`, `є`, `ґ` and `ў`, so Ukrainian and Belarusian
  names are unaffected — it takes an emoji or a Latin-Extended character.
- **`get_ozi_tile` is registered but nothing calls it.** The map renders OZF2
  through `get_ozi_tile_projected`, via the `ozi://` protocol. Dead IPC
  surface, not a missing feature: it costs a generated binding and a line in
  the registry.
- ~~An annotated `let … = $state(null)` narrows to `null` in an inline
  `$derived`.~~ Hit a second time on 2026-09-22, in another file; a guard test
  now fails on the declaration. Scoped to `$state(null)` on purpose — an array
  narrows to `never[]`, which is assignable and harmless. The original note:** TypeScript's flow analysis, not a broken type: before the first
  assignment the variable is `null`, so the non-null branch of any inline
  derived is `never` and every property read on it fails. Function bodies are
  deferred and never see it, which is why it looks arbitrary. Declare with
  `$state<T>(null)` instead.
- **A label looked up from a data table escapes the label guard.** The symbol
  picker carried ten English labels inside its own table; the guard checks what
  is written into an `aria-label`, not what a table hands it. Third miss of
  that kind. A guard on a syntactic shape catches the careless version, which
  is most of them, and is not worth widening until it misses something that
  reaches a screen — this one did, and is fixed.
- **Logic inside `MapView` has no tests, and that is where defects hide.** It
  needs a MapLibre instance to mount, so nothing in its 1200 lines runs under
  vitest. Two extractions so far — the generation stamps (still untested) and
  the bbox maths, which gained a `NaN` guard the moment it was testable. When
  touching that file, take the arithmetic out with the change.

- **There is no way to start a new project.** Nothing in the backend or the
  interface creates an empty one: a crew that finishes one search and starts
  another keeps adding to the same project, and the session restore brings it
  back on relaunch. Whether that is wrong depends on whether a "project" is a
  search or a machine's working set — an owner question, not a code one.
- **The launch screen's status bar reserves 80px for a download that is not
  running.** Four grid rows, three of them empty, including an outlined
  progress track that reads as a broken widget. The reservation avoids a jump
  when a download starts; permanent dead space on the first screen is the
  worse half of that trade.

- **A grid child can grow its own column.** `.canvas-column` had
  `grid-template-rows` and no `grid-template-columns`, so the implicit column
  was `auto` = max-content and the context bar widened it by 358px, under the
  inspector. Anywhere a grid holds a row that can be wider than its track,
  declare `minmax(0, 1fr)`; `min-width: 0` on the container is not enough,
  because it constrains the container, not the track.

- **The theme picker is unreachable.** `ThemePicker.svelte` is imported by
  nothing, and `ui-shell` requires a selector with five options and persistence
  across sessions. Both stores behind it work; only the placement is missing,
  and that is a design call — the toolbar, the palette, or a settings sheet
  that does not exist yet.

- ~~A GPX round trip loses the track's colour.~~ Done on 2026-09-22 in
  `the-colour-comes-back`: the second pass over the document exists, matching
  colours to tracks by position, with `xml-rs` — the parser `gpx` itself uses.
  What remains is a property of the format rather than a gap: GPX names its
  colours, so a custom shade returns as the nearest name, and a test says so.

- **A new requirement may already exist, saying the opposite.** Four deltas
  this session were written as ADDED against a baseline requirement that
  already covered the ground with a different rule — the catalogue merge, the
  catalogue cache, the track statistics, the `.part` file. `openspec validate
  --strict` checks a change's shape, not whether it disagrees with the
  baseline, so nothing catches it. Before writing a delta, read the
  capability's existing requirements; before archiving a batch, read them
  against each other.
- **Three guards now watch for a defect class rather than an instance**: a
  literal label in a component, English assembled into one, and a `catch` that
  logs and tells nobody. Each found something on its first run that the sweep
  which prompted it had missed. When a defect turns up twice, the third fix is
  a test over the shape, not another sweep.

- **An async reload that writes to shared state needs `createLatestRun`.**
  Found five times: the bundle preview, both library tabs, and the map's
  marker refresh and track-geometry fetch. Anything that reloads on
  `state-changed` overlaps with itself during a download, which emits once per
  file. `src/lib/latest-run.ts` holds the rule; take a token before the first
  await, check it before writing, and check it on the failure path too.
  Still open: the map's two guards have no test, because `MapView` needs a
  MapLibre instance to mount.
- **Nothing else derives a list from map geometry — check before adding one.**
  Twice now a list built from `build_tracks_geojson` has been wrong: once it
  emptied the whole rail when the geometry type changed, once it hid every
  track the map cannot draw. The map's features are shaped for drawing. A list
  wants its own command, typed, carrying only what a row shows.

- **MapLibre 4 → 6.** Carries a critical advisory (`GHSA-jrc7-96c5-q579`) whose
  fix is two majors ahead. The affected sink is not called here (popups use
  `setText`, no `innerHTML`), so it is waived in `scripts/npm-audit-gate.mjs`.
  The upgrade needs its own slice with visual verification.
- ~~Retries and timeouts on bundle downloads.~~ Done: timeouts in slice 0.3,
  retries on 2026-09-21 in `one-flaky-file-is-not-the-bundle` — three attempts
  per file, never for a cancellation, and the retry is reported. The retry resumes with a
  `Range` request, so a drop near the end of a large map does not cost it
  twice.
- ~~Partial bundle availability.~~ Checked on 2026-09-21: it already worked —
  a map whose file lands gets its local path and opens from disk. What was
  missing was telling the operator. Done, with a test pinning the capability.
- ~~Choose what to download.~~ Done on 2026-09-21: the loader lists the
  bundle's top level with sizes and everything checked; clearing an entry
  leaves it on the server. The app does not decide what to skip.
- **Remove the legacy element defaults from `app.css`.** Everything is inside
  `@layer base` now — the `button` rules, the `input`/`select` pair and the
  global `*` reset, which had been beating every spacing utility in the app
  until 2026-09-21 — so none of them overrides a component any more. Deleting
  them outright still needs a pass over the raw `<input>`/`<button>` sites that
  lean on them; the stand makes that cheap.

## Bundle flow — from the 2026-09-21 survey

Found while reviewing the path from "launch the app" to "a map on screen".
The refusal reporting, the progress-panel lifetime, the cached-bundle
detection and the palette's project list were fixed in `honest-bundle-flow`
and the palette slice; these were not.

- **The project list does not say what is already on disk.** `LizaProjectSummaryDto`
  carries only slug and name, although the backend has `is_project_cached`.
  Offline, a crew cannot tell which of thirteen thousand projects they can
  actually open. Needs a `cached` flag on the summary and a badge in the list.
- **No size before committing to a download.** Map packages carry no byte size,
  and the prominent button downloads the whole project directory recursively,
  not the selected map. On a tethered phone that is potentially gigabytes with
  no estimate and no disk-space check.
- ~~A single-map download has no panel and cannot be cancelled.~~ The backend
  returns the id; the last caller that ignored it, the command palette, was
  fixed on 2026-09-22 in `one-way-to-start-a-map`, and the rule now lives in
  `src/lib/actions/open-map.ts` so a fourth caller cannot get it wrong. The
  stand plays a single-map download, which is what made the gap visible.
- **The loader Sheet hides the download it started.** The Sheet overlay is
  `z-50`, the download popup `z-40`, and the toaster sits in the same corner.
  Opening the loader to queue the next map blocks the map and hides the running
  download.
- ~~The filter matches names only and miscounts.~~ Both halves are done: the
  count reads `N of M` and the row carries a `cached` badge, and a Cyrillic
  query finds a latin name as of 2026-09-22 in `search-in-the-crews-language`
  — `src/lib/translit.ts` spells each Russian letter every way the catalogue
  spells it, because the catalogue is not consistent with itself. Still open:
  no sort, no "recent", no jump to today's search.
- ~~The preview timeout is a lie.~~ Done on 2026-09-21 in
  `one-preview-at-a-time`: only the newest preview may land, a preview no
  longer releases a busy flag it never took, the loader matches by slug rather
  than display name, and the fifteen-second timer says the wait is running long
  instead of ending it.
- ~~Filter and selection do not survive closing the loader.~~ Done on
  2026-09-21 in `keep-the-place-in-the-catalogue`: the filter, the "only
  downloaded" toggle, the selection, the cleared contents and the scroll offset
  live in `bundleLoaderView` and come back when the Sheet mounts the loader
  again. Session state only — tomorrow starts on tomorrow's search.
- ~~Backend progress text is English.~~ Done on 2026-09-21 in
  `progress-in-the-crews-language`: the bundle path sends a key and its
  arguments, both status surfaces translate the message and the phase, and the
  backend's English wording is the fallback. Still English: the `AppState`
  status line's own 44 messages, which no surface currently renders — convert
  them the same way if one starts to.
- ~~The catalogue is walked in full at every launch, holding the busy flag.~~
  Both halves are done. It is stoppable since 2026-09-21
  (`stop-waiting-for-the-catalogue`), and since 2026-09-22 it holds a flag of
  its own (`one-wait-does-not-block-the-other`), so it blocks only another
  refresh — a download may start while it runs. Still open, and smaller now:
  the walk still happens at every launch rather than when the cache is stale.
  Since 2026-09-22 the list's date is on screen (`how-old-is-this-list`), which
  is what any "skip the walk" rule would have to stand on — and it needs a
  staleness threshold, which is an owner decision like the moving-time one.
  Worth weighing against data usage: a thousand pages of HTML on a phone
  tether, at every launch.

  ~~A project deleted upstream is never removed.~~ Done on 2026-09-21 in
  `a-search-that-is-gone-leaves-the-list`: a complete walk replaces the list,
  a stopped one changes nothing, and the walk's boundaries are emitted so the
  interface knows which it was.
- **Small targets.** 28px rows and 10px badges in the catalogue. The keyboard
  half of this is done on 2026-09-21 in `walk-the-catalogue-by-keyboard`: the
  list is a listbox with an `aria-activedescendant` position, walked with the
  arrows and opened with Enter, so a screen reader is no longer looking at a
  handful of buttons out of thirteen thousand. The row and badge sizes are
  still a design decision the owner has not made.
