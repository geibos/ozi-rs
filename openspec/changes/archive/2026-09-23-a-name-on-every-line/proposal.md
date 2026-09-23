## Why

OziExplorer draws a track's name on the track. This application drew twelve
coloured lines and not one name, and the question "which line is ЛИСА15" was
answered by reading a colour off the list and hunting for it among eleven
others. Selecting a row gives the chosen track a casing, which helps with the
track you already found and not at all with the one you are looking for.

The reason was a dependency, not a decision. A MapLibre symbol layer refuses
to exist without a `glyphs` URL, and pointing the style at a remote one does
not merely lose the labels offline — a source that feeds a symbol layer stops
tiling when the glyph fetch hangs, so the *lines* disappear too. Bundling SDF
glyphs is a repository-size decision with a font licence attached, and it has
been "a follow-up" since the day the layer was written.

Names as DOM markers need no glyphs, no licence and no megabytes, and the
waypoint markers have worked that way all along.

## What Changes

- Every visible track with a name carries it on the map, in the track's own
  colour, at the point half way along the route **by length** — index-midpoint
  puts the name wherever the GPS logged densely, which is the rest stop.
- Names that would land on top of each other are dropped rather than stacked,
  which is the one thing a symbol layer would have done for free. A selected
  track keeps its name; after that the longer route wins, because a long route
  is the one a name helps to follow.
- Below the zoom where a whole district fits on screen there are no names at
  all: at that scale they are a smear, not information.
- The halo is white. The names are saturated hues over a pale topographic map,
  and a dark halo under a saturated hue turns it muddy at 11px.

## Impact

- Affected specs: `track-display`
- Affected code: `src/lib/track-labels.ts` (new),
  `src/components/MapView.svelte`, `src/lib/maplibre/tracks-layer.ts`
