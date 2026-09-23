# Tasks

- [x] 1.1 `read_ozi_map_text` decodes through `decode_plt_bytes` rather than
      falling back to lossy UTF-8; the lossy read stays only for bytes no
      supported encoding accepts, because a header of ASCII and numbers is
      still worth reading.
- [x] 1.2 A test with a real cp1251 `.map`: the title and the raster name come
      back intact and nothing is a replacement character.
- [x] 1.3 The round-trip test that found it —
      `a_calibrated_picture_serves_tiles_at_the_coordinates_it_was_given` —
      calibrates `Сагра.png`, opens the `.map` the application wrote, checks
      both corners and the middle against what the operator gave, and asks for
      a tile.
- [x] 1.4 `just ci` green: 401 Rust, 668 frontend.
