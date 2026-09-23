## Why

A headquarters is sometimes handed an image and nothing else.

A screenshot of a web map with the search area on it. A photograph of a paper
sheet on a table. A raster somebody exported from their own GIS. It is a map of
exactly the ground the crews are walking, and it is useless here, because
nothing in it says where on the Earth it sits. The only answer this application
had was "calibrate it in OziExplorer first", which is the program the crew does
not have and the reason they are using this one.

OziExplorer calls it calibration: the operator says "this pixel is here" two or
more times, and from those an affine fit gives every other pixel. That is the
last thing in the reviewer's list of what a headquarters needs before this can
be used on a real search.

What a person looking at a screenshot of a web map can actually read off it is
its two opposite corners, so that is what the form asks for. Two corners fix
position and scale and say nothing about rotation, which is honest: a
screenshot is north-up, and a photograph of a sheet on a table is not something
two corners can straighten. The backend takes any number of points, so the
picking-on-the-map form that would handle a rotated photograph can be added
without moving anything underneath it.

Coordinates arrive in whatever form the person sending them had to hand —
`59.9311, 30.3609` off a phone, `59°55'52"N 30°21'39"E` off a screenshot,
`N 59 55.87 E 30 21.65` out of a navigator — so the fields take all of them. A
field that takes only one is a field that gets typed into wrong at four in the
morning.

## What Changes

- A picture with no `.map` can be tied to the Earth by giving its two opposite
  corners, and the application writes the `.map` beside it and opens the pair.
- The file written is a real OziExplorer one, in cp1251, so the same folder
  opens in OziExplorer on somebody else's laptop.
- Coordinate fields read decimal degrees, degrees and decimal minutes, and
  degrees-minutes-seconds, with hemisphere letters in either alphabet and in
  either order, and refuse anything that is not exactly one place.
- Calibration that cannot define a map — one point, or points all on one
  parallel — is refused with the reason, not with a file that is wrong.

## Impact

- Affected specs: `tile-rendering`
- Affected code: `src-tauri/src/infrastructure/export/ozi_map.rs` (new),
  `src-tauri/src/commands/mod.rs`, `src-tauri/src/lib.rs`,
  `src/lib/coordinates.ts` (new), `src/components/CalibratePicture.svelte`
  (new), `src/components/BundleLoader.svelte`, `src/lib/api.ts`,
  `src/lib/i18n.ts`
