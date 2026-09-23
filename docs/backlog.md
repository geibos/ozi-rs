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

### From the archiving and verification pass, 2026-09-22

- **The local `openspec` was ten minor versions behind CI's.** CI installs
  `@fission-ai/openspec@1.13.1` (`.github/workflows/ci.yml`); this machine had
  1.3.1, which lacks the check that a MODIFIED block must carry forward the
  scenarios the baseline still has — a MODIFIED requirement replaces the whole
  block, so anything left out is dropped. Fifty-five changes were archived
  under the older validator before that was noticed.

  The damage was two scenarios, both from
  `a-search-that-is-gone-leaves-the-list`, and both deliberately superseded:
  the requirement text that replaced them says so in as many words and covers
  both branches with new scenarios, so nothing was lost in meaning. A diff of
  every `#### Scenario:` heading in the baseline before and after the batch
  found no others, and no requirement was lost at all.

  Worth pinning the version the way `rust-toolchain.toml` pins rustc, so a
  machine cannot archive under a validator weaker than the gate's.

- **`MapView` is driven by no automated test.** It needs a MapLibre instance,
  so vitest cannot mount it, and the stand has no test runner wired into
  `just ci`. What exists is unit tests on the helpers (`edit-failure`,
  `latest-run`, `map-bounds`), class guards over the source, and hand-driven
  stand sessions. The gap was recorded as a permanently-open task in two
  changes; it belongs here instead. Closing it means a browser test runner
  against the stand — `revive-ui-cycle` task 4.1 already proposes Playwright
  as a dev dependency for the screenshot matrix, and the same runner would
  serve this.

- **The stand answers only the commands somebody has needed.** `add_waypoint`
  was missing until 2026-09-22, so placing a waypoint by bearing had never
  been walked past its form. It fails loudly rather than silently, which is
  the design working — but a feature nobody walks is a feature nobody sees.
  Worth a sweep: which commands in the bindings have no stand answer, and
  which screens does that make unreachable.

- **`?state=cold` cannot preview a project.** `previewedAppState()` re-slugs
  the fixture's `current_project`, and the cold fixture has none, so selecting
  a search from the cold screen sits on "Загрузка списка карт…" forever. The
  cold screen is the one a crew actually sees first.

- **Seeded content promised and absent.** `revive-ui-cycle` task 5.3 asks for
  `docs/backlog.md` seeded with, among others, the Meetily-inspired list; this
  file has no Meetily entry. Either it is elsewhere or it was never written.

### From the external review, 2026-09-22 — not yet acted on

Sixteen of the reviewer's findings were confirmed by reading the code and
fixed the same day (`openspec/changes/what-the-review-found`,
`docs/progress/README.md`). What is below is the remainder: each is the
reviewer's claim, checked only as far as the note says, and none is a verified
defect unless it says so. Reports: `docs/reviews/2026-09-22/`.

- **The baseline specs are stale, and `--strict` cannot see it.** Confirmed by
  reading: `openspec/specs/track-import/spec.md` still requires GPX to import
  "into the active track layer", while `application/import.rs` creates a layer
  per source file. This is not a spec-versus-code argument — the correct
  requirement is already written in
  `openspec/changes/codify-architecture-decisions`, which removes the stale one
  with its reason. It has simply never been archived. Fifty-seven unarchived
  changes is the actual finding: `openspec validate --strict` checks a change's
  shape, not whether it agrees with the baseline, so the baseline can say the
  opposite of both the code and the agreed change and stay green. Archiving is
  the owner's step and is already item 1 of "Blocked on the owner" in
  `docs/STATE.md`.

- **A spec freezes a temporary shell.** `openspec/specs/ui-shell/spec.md:403`
  requires the non-working mode chips to stay, and the measuring tool's author
  cites that requirement as the reason the action lives in the palette instead.
  A requirement that preserves scaffolding is worse than no requirement.
  Needs the owner to say whether the chips are the intended interface or a
  placeholder; the answer removes either the requirement or the scaffolding.

- **The shared mutex is held across save, import and export.** Not measured. A
  slow disk or a large import delays every other call into `AppState`. The
  reviewer's own advice is to measure on a real day's tracks first; splitting
  services without moving the lock boundary changes nothing.

- **Every state snapshot rebuilds all track summaries**, statistics included,
  and a snapshot goes out on every state change. Frontend filtering reduces
  redraws, not this. Same order: measure on a day's collection, then cache by
  revision.

- **CJ-7 is the least-supported journey.** Its criterion — do not lose an hour
  of marking — is stronger than the implemented contract, which explicitly
  excludes recovering unsaved edits (`project-persistence/spec.md:40`). The
  atomic save protects the last saved version, not the work after it. Needs a
  decision on recovery before it is worth designing.

- **`get_ozi_tile` is dead IPC surface** — see `docs/STATE.md`.

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
  narrows to `never[]`, which is assignable and harmless. The original note:\*\* TypeScript's flow analysis, not a broken type: before the first
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
- ~~The launch screen's status bar reserves 80px for a download that is not
  running.~~ Done on 2026-09-22 in `a-bar-with-nothing-to-say`: one line until
  there is something to report, four rows while a download runs, with a height
  transition so the growth reads as opening rather than as a jump.

- ~~The stand's answers are not typed against the bindings.~~ Done on
  2026-09-22 in `the-stand-cannot-lie-about-shape`, after two wrong shapes in
  two days. The handler table is typed from `commands` in `bindings.ts`, keyed
  by the wire's snake_case name; the tile commands' `ArrayBuffer` is the one
  stated exception. It found four loose answers on the way in.

- **On-map track labels wait on bundled glyphs, and now they are worth it.**
  `tracks-layer.ts` adds its label layer only when the style has glyphs, which
  a raster basemap does not: the basemap's own names are baked into the tiles.
  With a day's twenty routes drawn in twenty colours, the missing half is which
  colour is whose without going back to the list. The cost is real and is why
  it has not been done: SDF glyph PBFs for at least Cyrillic and Latin ranges,
  generated with fontnik or similar, checked into the repository as binary
  assets, with a font whose licence allows it — and `style.glyphs` pointing at
  them so it works with no network, which is the whole point. That is a repo-
  size decision as much as a code one.

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
- **An `$effect` that returns before reading its dependencies is dead.** Svelte
  5 subscribes an effect to what it actually reads, so `if (!map) return;`
  ahead of a `$store` read leaves the first run subscribed to nothing. It cost
  a feature that looked written and was not, and four more effects in `MapView`
  had the shape and worked only because they are declared after `onMount`.
  Guarded by `effect-reads-before-guarding`; a `$state` local counts as a read.
- **Six guards now watch for a defect class rather than an instance**: a
  literal label in a component, English assembled into one, a `catch` that logs
  and tells nobody, an annotated `$state(null)`, a toast message typed in
  rather than looked up, and an effect that gives up before it subscribes. Each found something on its first run that the sweep
  which prompted it had missed — the last one found sixteen. When a defect
  turns up twice, the third fix is a test over the shape, not another sweep.
  And each guard covers one shape only: `no-untranslated-labels` watches
  attributes, so it said nothing about toasts for months.

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
  backend's English wording is the fallback. The folder import's summary was
  the one that _was_ being rendered, and it was converted on 2026-09-22 in
  `the-import-speaks-russian`. Still English: the `AppState` status line's own
  44 messages, which no surface currently renders — convert them the same way
  if one starts to. Worth a guard over the shape rather than a third sweep:
  a backend string that reaches a toast.
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

- **A camp router with no uplink.** The OSM backdrop is skipped at launch when
  `navigator.onLine` is `false`, which covers a laptop with no interface at
  all. It does not cover the commoner field case: connected to a camp router or
  a phone hotspot that has no way out. There `navigator.onLine` says `true`,
  the map asks OpenStreetMap for every visible tile, and the requests hang.

  The application already probes the network at launch — the catalogue walk —
  and its failure is stronger evidence than `navigator`. Acting on it needs the
  backend to say *why* the walk failed: a transport error means no reach, a 500
  from maps.lizaalert.ru means the link is fine and the backdrop should stay.
  Until that distinction exists, removing the backdrop on any catalogue error
  would take it away from an operator whose link works. Noticed walking CJ-2 on
  2026-09-23.

## What OziExplorer has and this does not

Re-checked against `docs/reference/oziexplorer.md` on 2026-09-23, after the
calibration and field-tool work. Everything the reference marks **[ЯДРО]** is
now built except these four, none of which was ever agreed into scope — they
are written down so the choice is the owner's rather than an omission.

- ~~**Сетка координат на карте.**~~ Built on 2026-09-23
  (`a-grid-to-read-sectors-off`): lat/lon lines on a step that follows the zoom
  and always lands on a number a person can say, named at the edges, offered
  from the command palette and remembered. OziExplorer also draws UTM and
  national grids; those are not built and were not the part that mattered.
- ~~**Distance Between Waypoints.**~~ Built on 2026-09-23
  (`the-tape-catches-a-mark`): rather than OziExplorer's dialog of waypoint
  pairs, the ruler catches hold of a mark the click lands near, and names both
  when a measurement has a mark at each end.
- **Track Replay.** Playing a recording back in time. "Where was the group at
  half past two" is answered today by reading the points table, which has the
  times in it — slower, but not missing.
- **Вложения к точке.** A photo attached to a mark — the found item, the
  footprint. This needs files inside the `.ozp`, which is a change to the
  format rather than a feature on top of it, and the format is the thing two
  headquarters exchange.

Deliberately not listed: Track Move (a track is a recording, not a drawing),
Routes, Events, Point Sets, live GPS, printing, DEM relief, and the GIS import
formats — all recorded as non-goals in `product-scope`.
