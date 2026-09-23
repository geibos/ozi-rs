# Tasks

- [x] 1.1 `direct_image.rs`: decode once, halve into a pyramid, cut tiles;
      refuse an oversized picture from its header alone. Six unit tests.
- [x] 1.2 `OziRasterTileSource` takes a backend — OZF2 or picture — so the
      level metadata and the tile shape are the same either way.
- [x] 1.3 `open_ozi_raster_tile_source` and `open_ozi_map_selection` accept
      `DirectImage(_)`; `.ozfx3` stays refused.
- [x] 1.4 JPEG, TIFF, BMP, GIF and WebP decoders enabled beside PNG.
- [x] 1.5 Two wiring tests: a `.map` beside a real PNG on disk opens and hands
      out tiles including a short edge tile; a `.map` naming a picture that is
      not there fails with the file's name.
- [x] 1.6 `just ci` green: 384 Rust, 588 frontend.
