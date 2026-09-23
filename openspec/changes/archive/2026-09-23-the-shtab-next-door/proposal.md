## Why

The headquarters next door runs the original OziExplorer, and what it hands
over is a `.wpt` — its native waypoint file. We have written that format since
ADR-0022 and never read it. Taking marks from a second штаб meant finding
something that converts them to GPX first, which is not a thing a coordinator
does at four in the morning.

Writing a format without reading it also meant the round trip had never been
checked, and it was wrong. Both exporters replace a comma in a name with a
space, and a test pinned that as intended. The format reserves `chr(209)` for
a comma inside a text field and turns it back on reading — but `chr(209)` is a
*byte*, and these files are Windows-1251, where byte 209 is `С`. Honouring the
escape would put a comma inside «СТАРТ». So the escape is wrong for these
users in both directions, and the right answer is to say so rather than to
discover it in the field.

## What Changes

- `.wpt` files import: the four fields that mean something here (name,
  latitude, longitude, symbol), into a waypoint layer named after the file,
  like every other import. A second штаб's marks stay theirs, so they can be
  hidden or handed back.
- The import dialog accepts `.wpt` beside GPX, PLT and ZIP.
- A file that is not a waypoint file is refused rather than read as a scatter
  of marks in the Gulf of Guinea. Placeholder rows for empty GPS slots and
  blank lines are skipped without failing the import.
- The `chr(209)` escape is documented as deliberately not used, in either
  direction, with the Windows-1251 reason. A comma in a name is spent; a name
  with a `С` in it is not corrupted.
- The round trip out and back through WPT is a test, which it could not be
  before.

## Impact

- Affected specs: `track-import`, `track-export`
- Affected code: `src-tauri/src/infrastructure/import/wpt.rs` (new),
  `src-tauri/src/infrastructure/import/mod.rs`,
  `src-tauri/src/infrastructure/export/wpt.rs`,
  `src-tauri/src/infrastructure/export/plt.rs`,
  `src-tauri/src/application/import.rs`, `src-tauri/src/application/mod.rs`,
  `src-tauri/src/commands/mod.rs`, `src-tauri/src/lib.rs`, `src/lib/api.ts`,
  `src/lib/i18n.ts`, `src/components/library/TracksTab.svelte`
