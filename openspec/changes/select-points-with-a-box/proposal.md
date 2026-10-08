## Why

Cleaning a real search's tracks on 2026-10-08, the owner reached for
OziExplorer's Selection Control on track after track: draw a box over the
active track's points, delete what is inside. A cluster of outliers, the walk
back to HQ, a tail left from the previous search — picking points 572 to 586
out of a list "is already inconvenient". ozi-rs deleted points one at a time,
from a context menu, in edit mode only.

## What Changes

- A box tool: «Рамка» in the points card and «Выделить точки рамкой» in the
  command palette. While it is on, dragging on the map draws a box instead of
  panning; the selected track's points inside it are chosen and drawn as red
  rings. Shift adds a second box to the first; a click without a drag clears.
- A bar over the map says how many points are chosen and offers «Удалить
  выбранные» and «Оставить только их» — each one undo step through the
  existing `CropTrackPoints` — and «Снять». Delete or Backspace deletes the
  chosen points; Esc puts the tool away.
- Removing every point of a track is refused, as cropping already refuses it.
- The box is judged in screen pixels: what the operator sees inside the box
  is what is chosen.

## Impact

- Affected specs: `track-editing`
- Affected code: `src-tauri/src/application/mod.rs`
  (`apply_remove_track_points`), `src-tauri/src/commands/mod.rs`
  (`remove_track_points`), `src-tauri/src/lib.rs`, `src/lib/api.ts`,
  `src/lib/box-select.ts` (new), `src/components/BoxSelect.svelte` (new),
  `src/components/MapView.svelte`, `src/components/CommandPalette.svelte`,
  `src/components/inspector/TrackSegmentsTable.svelte`, `src/lib/i18n.ts`,
  `docs/commands-reference.md`
- Evidence: `docs/field-notes/2026-10-08-forest-search-in-ozi.md`
