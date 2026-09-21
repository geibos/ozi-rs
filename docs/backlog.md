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
- **Moving time as a track statistic.** `duration_seconds` is the span between
  the first and last point, which reads oddly for multi-day recordings even now
  that it is formatted in days. Moving time needs a stop threshold the owner has
  not chosen.
- ~~Waypoint export to GPX.~~ Done on 2026-09-21: both the Waypoints tab row
  menu and the Waypoint Inspector offer GPX and WPT.

## Engineering

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
- **A single-map download has no panel and cannot be cancelled.** `open_selected_map`
  mints its own download id, never returns it and never sets busy, so neither
  the progress popup nor the cancel button appears; the row is disabled while it
  runs, so there is no way out.
- **The loader Sheet hides the download it started.** The Sheet overlay is
  `z-50`, the download popup `z-40`, and the toaster sits in the same corner.
  Opening the loader to queue the next map blocks the map and hides the running
  download.
- **The filter matches names only and miscounts.** Project names are latin
  transliterations, so a Cyrillic query finds nothing, and the count beside the
  box shows the catalogue total rather than the number of matches. No sort, no
  "recent", no jump to today's search.
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
- **The catalogue is walked in full at every launch**, holding the busy flag.
  The "no way to stop it" half is done on 2026-09-21 in
  `stop-waiting-for-the-catalogue`: the walk is cancellable, the loader offers
  a stop while it runs, and a stopped walk is not written over the cache. The
  hint states how many projects are listed so far, which is what the decision
  to stop is actually made on — a page number would have said less.

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
