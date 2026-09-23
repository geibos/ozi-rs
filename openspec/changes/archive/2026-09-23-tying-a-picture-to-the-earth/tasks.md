# Tasks

## 1. Writing the calibration

- [x] 1.1 `export/ozi_map.rs`: build a `.map` from calibration points, with the
      two-point axis-aligned fit and the least-squares plane for three or more;
      refuse too few, degenerate and off-Earth points with the reason.
- [x] 1.2 Round-trip test: what is written reads back through
      `parse_ozi_georeference` as the same places, including a rotated
      three-point fit.
- [x] 1.3 Written in cp1251 beside the picture, named after it.

## 2. Reading what a coordinator writes

- [x] 2.1 `src/lib/coordinates.ts`: decimal degrees, degrees and decimal
      minutes, degrees-minutes-seconds; hemisphere letters in either alphabet,
      before or after, in either order; a decimal comma when nothing else could
      be the separator.
- [x] 2.2 Sixteen tests, including what must be refused: one number, three,
      a latitude off the Earth, two letters on the same axis, words.

## 3. The form

- [x] 3.1 `read_raster_size` and `calibrate_raster` commands, registered and
      wrapped in `api.ts`.
- [x] 3.2 `CalibratePicture.svelte` under the launcher's footer: pick a
      picture, see its size, give two corners, calibrate and open.
- [x] 3.3 A corner that was not understood marks its field and keeps the
      button shut.
- [x] 3.4 Stand answers for both commands, and a calibrated picture makes the
      stand's cold start into a workspace with a map — otherwise the toast said
      the map had opened and the screen went back to the launcher.

## 4. Gates

- [x] 4.1 Walked on the stand: pick → `60°03'00"N 30°12'00"E` and
      `59.9500, 30.4000` → «Привязать и открыть» → the toast names
      `Сагра_скриншот.map` and the workspace opens.
- [x] 4.2 `just ci` green: 391 Rust, 604 frontend.
