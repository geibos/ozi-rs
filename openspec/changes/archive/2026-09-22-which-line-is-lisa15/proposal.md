## Why

A day's recordings now draw in twelve colours, and the map carries no names:
on-map labels need SDF glyphs nobody has bundled, which is a repository-size
decision the owner has not been asked for. So "which line is ЛИСА15" is
answered by reading a colour off the list and hunting for it among eleven
others.

Selecting a row can answer it with no assets at all.

## What Changes

- The selected track gets a white casing under its own line, so it stands out
  from the others while keeping the colour that says whose it is. Stepping down
  the list lights each route in turn.
- The casing is a filtered layer whose filter is swapped, not a layer added and
  removed: its place in the stack — below the coloured line — is what keeps the
  colour true.
- Four effects in `MapView` that guarded on the map before reading their stores
  are turned around, and a guard fails on the shape. An effect is subscribed to
  what it actually reads, so a first run that takes the early exit leaves it
  subscribed to nothing and it never runs again. The highlight was written that
  way and was simply dead — the row lit up and the map did not.

## Capabilities

### Added Capabilities
- `track-display`: the selected track is picked out on the map.

## Impact

- **Frontend**: `lib/maplibre/tracks-layer.ts`, `MapView.svelte`,
  `src/test/effect-reads-before-guarding.test.ts` (new).
- **Backend**: none.
- **Risk**: low. One more layer in the stack, filtered to nothing until a track
  is selected.
