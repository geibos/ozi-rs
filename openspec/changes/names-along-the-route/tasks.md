## 1. Glyphs

- [x] 1.1 Noto Sans Bold ranges under `static/glyphs/`, with the OFL text and where they came from
- [x] 1.2 `glyphs://` protocol: a shipped range answers its bytes, any other range an empty set, a failed read an empty set
- [x] 1.3 Test: every letter of both alphabets, the digits and the punctuation a callsign uses decode from the shipped ranges
- [x] 1.4 The map style points `glyphs` at the protocol

## 2. Names on the line

- [x] 2.1 Label features built from the track geometry: visible named tracks only, one per track, the selected one first and then by distance walked
- [x] 2.2 Symbol layer along the line, repeated, below zoom 10 nothing
- [x] 2.3 DOM label markers and their declutter removed

## 3. Gates

- [x] 3.1 `just ci` green (2026-10-02: 407 Rust, 678 vitest)
- [x] 3.2 Walked on the stand 2026-10-02: the name runs beside the line and repeats as the map zooms in; the first look had it on the line, where the line showed through between the letters and `_` read as `•`, so it is offset now. The stand project has one visible track, so the collision between names is covered by the unit tests, not the picture. `docs/progress/2026-10-02-names-along-the-route/`
- [ ] 3.3 `just smoke` green on a bundle built from this tree — 2026-10-02: on a bundle built from `fe06e6a`, `smoke_cj5_draw_track` and `smoke_report_capture_moment` green; `smoke_cj3_layer_management` failed twice on a machine in use (its tab click landed on an inactive window) and was not run a third time. Owed: one run with nobody at the keyboard.
