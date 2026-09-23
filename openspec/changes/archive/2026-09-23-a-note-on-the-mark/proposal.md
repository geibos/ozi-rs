## Why

A mark called «улика» is the place. What a crew is sent to is the note beside
it — «красная куртка, 200 м от просеки». OziExplorer carries that note in
field 11 of a `.wpt` and GPX carries it in `<desc>`, and this application
dropped it in both directions: the place survived the exchange between
headquarters and the reason for it did not.

The waypoint reader landed on 2026-09-23 and read four fields. An outside
reviewer pointed out the same day that the fifth is the one that matters.

## What Changes

- A waypoint carries a note. `None` for a mark with nothing to say, which is
  not the same as an empty note somebody typed and cleared.
- It travels: read from and written to GPX `<desc>` and WPT field 11, with a
  round trip pinned in both formats. Forty characters is the WPT limit.
- The Waypoint Inspector has a field for it, committed on blur rather than on
  every keystroke — a field that writes per keystroke puts one undo step on
  the stack per letter.
- Setting it is undoable, like every other edit to a mark.
- Every project saved before this reads with its marks noteless.

## Impact

- Affected specs: `waypoints`, `track-import`, `track-export`
- Affected code: `src-tauri/src/domain/waypoint.rs`,
  `src-tauri/src/domain/project.rs`, `src-tauri/src/application/commands.rs`,
  `src-tauri/src/application/mod.rs`, `src-tauri/src/commands/mod.rs`,
  `src-tauri/src/lib.rs`, `src-tauri/src/fixtures.rs`,
  `src-tauri/src/infrastructure/import/{gpx,wpt}.rs`,
  `src-tauri/src/infrastructure/export/{gpx,wpt}.rs`, `src/lib/api.ts`,
  `src/lib/i18n.ts`, `src/components/inspector/WaypointInspector.svelte`
