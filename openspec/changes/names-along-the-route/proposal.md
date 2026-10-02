## Why

A track's name is on the map once, at the middle of its longest segment, drawn
as a DOM marker. That was the only way to draw text without SDF glyphs, and the
application bundled none: a remote glyphs URL does not merely lose the labels,
it stops the lines rendering offline.

One name per track is enough while the whole route is on screen. It is not
enough at the zoom a crew works at. Zoomed in on one stretch of a twelve
kilometre route, the middle is off screen, the name with it, and the question
"which line is ЛИСА15" is back to reading a colour off the list. A printed map
and OziExplorer both repeat a route's name along it.

The owner decided on 2026-10-01 to bundle open-source glyphs, which is what
this needed.

## What Changes

- Noto Sans Bold SDF glyphs (SIL OFL 1.1, from `openmaptiles/fonts` v2.0)
  ship with the application: Basic Latin and Latin-1, Latin Extended,
  combining marks, Cyrillic, general punctuation, and the letterlike block
  that holds `№`. About half a megabyte.
- The map style points `glyphs` at a local protocol. A range that is not
  shipped answers as an empty glyph set, so a character outside the set is
  skipped instead of failing the tile — the failure that made remote glyphs
  take the lines down with them.
- Track names become a MapLibre symbol layer: drawn along the line, repeated
  at intervals, collided and faded by the map itself. The selected track wins
  room first, then the track walked furthest, as before.
- The DOM label markers and their hand-written declutter go.

## Impact

- Affected specs: `track-display`
- Affected code: `static/glyphs/`, `src/lib/maplibre/glyphs-protocol.ts`
  (new), `src/lib/maplibre/tracks-layer.ts`, `src/lib/track-labels.ts`,
  `src/components/MapView.svelte`
- Repository size: about 510 KB of binary glyph ranges plus their licence.
