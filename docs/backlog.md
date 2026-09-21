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
- **Shorter names for import-created track layers.** Importing a folder creates
  one layer per file named `Imported tracks: /Users/.../20260708_Veter2.gpx`.
  In the layer selector that is a column of paths. The file name alone would do.
- **Moving time as a track statistic.** `duration_seconds` is the span between
  the first and last point, which reads oddly for multi-day recordings even now
  that it is formatted in days. Moving time needs a stop threshold the owner has
  not chosen.
- **Waypoint export to GPX.** The requirement exists in the `waypoints` spec and
  `build_waypoint_gpx_xml` exists in the code, but no command or UI reaches it.
  Owner wants it kept (2026-09-19); scheduled with CJ-6.

## Engineering

- **MapLibre 4 → 6.** Carries a critical advisory (`GHSA-jrc7-96c5-q579`) whose
  fix is two majors ahead. The affected sink is not called here (popups use
  `setText`, no `innerHTML`), so it is waived in `scripts/npm-audit-gate.mjs`.
  The upgrade needs its own slice with visual verification.
- **Retries and timeouts on bundle downloads.** Timeouts are in slice 0.3;
  retries are not planned yet. A stalled TCP connection currently freezes a
  download indefinitely.
- **Partial bundle availability.** `ready_bundle_files` exists in `AppState` but
  no screen uses it, so a bundle cannot be opened while the rest still downloads
  (owner's July note).
- **Choose what to download.** The owner rarely needs the Android `.sqlitedb`
  maps; a type filter would cut most of the transfer (owner's July note).
- **Remove the legacy element defaults from `app.css`.** They are in `@layer
  base` now so they no longer beat component utilities, but four rules still
  lean on them. Removing them needs the screenshot matrix to prove nothing
  regresses.
