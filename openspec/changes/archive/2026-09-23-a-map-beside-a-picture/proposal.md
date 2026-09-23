## Why

A headquarters is handed a map far more often as a picture than as an OZF2.

OziExplorer's own pairing is a `.map` calibration file beside an ordinary
image — `.jpg`, `.png`, `.tif`. OZF2 is the optimised form somebody produced
later with Img2Ozf, and it is what a bundle from maps.lizaalert.ru contains. A
scan of a paper sheet, a screenshot of a web map, a photograph of a sheet on a
table, a raster exported by somebody else's GIS: all of them arrive as the
first kind and OziExplorer opens every one without comment.

This application refused them all. The `.map` parser has known
`DirectImage(Jpeg)`, `DirectImage(Png)`, `DirectImage(Tiff)` and the rest for
months — it names them in its own vocabulary — and `open_ozi_raster_tile_source`
answered `UnsupportedRasterKind` for every one of them. So a coordinator with
a perfectly ordinary OziExplorer map pair got "unsupported raster kind" and no
way forward, which is most of what "I cannot open my map in this" meant.

It is also the ground the remaining blocker stands on. Calibrating an image
that has *no* `.map` — the case where a headquarters receives a picture and
nothing else — ends by writing a `.map` beside it, and there is no point
writing one the application cannot then open.

## What Changes

- A `.map` beside an ordinary picture opens: the picture is decoded once, a
  pyramid is built over it by halving, and tiles are cut from those levels —
  the same shape the OZF2 reader reports, so nothing above the adapter knows
  which it got.
- JPEG, TIFF, BMP, GIF and WebP decoders are enabled alongside PNG, matching
  the formats the `.map` parser already claims to recognise.
- A picture too large to hold in memory is refused by name and size before its
  pixels are read, rather than failing an allocation halfway through a launch.
- `.ozfx3` stays refused: it is encrypted and nothing here reads it.

## Impact

- Affected specs: `tile-rendering`
- Affected code: `src-tauri/src/infrastructure/import/direct_image.rs` (new),
  `src-tauri/src/infrastructure/import/ozi_raster.rs`,
  `src-tauri/src/application/mod.rs`, `src-tauri/Cargo.toml`
