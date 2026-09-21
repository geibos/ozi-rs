## 1. Backend

- [x] 1.1 `track_colours_in_document_order` in `import/gpx.rs`, with tests written first
- [x] 1.2 One `GARMIN_COLORS` table, with `garmin_color_to_rgba` beside `rgba_to_garmin_color`
- [x] 1.3 Both import paths — a file and an entry inside an archive — apply it
- [x] 1.4 The test that pinned the loss is gone, replaced by the one that pins the recovery

## 2. Verification

- [x] 2.1 `just ci` green (329 Rust, 493 frontend)
- [x] 2.2 `cargo audit --file Cargo.lock`: unchanged, 12 pre-existing allowed warnings, no new crate
- [ ] 2.3 `just smoke` green (blocked: the Mac2 driver cannot enable automation mode — see `docs/STATE.md`)
