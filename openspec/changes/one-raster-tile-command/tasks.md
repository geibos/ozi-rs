## 1. Removal

- [x] 1.1 `get_ozi_tile` out of `commands/tiles.rs` and the raw handler in `lib.rs`
- [x] 1.2 `getOziTile` out of `api.ts`; the stand's answer and type out of `tauri-core.ts`; `ipc.ts` names two commands
- [x] 1.3 `docs/commands-reference.md`, `docs/feature-status.md`, `docs/STATE.md`, `docs/backlog.md`

## 2. Gates

- [x] 2.1 `just ci` green (2026-10-02: 423 Rust, 718 vitest)
- [ ] 2.2 `just smoke` green on a bundle built from this tree, and an OZF2 map drawn in it — 2026-10-02: on a bundle built from `fe06e6a`, `smoke_cj5_draw_track` and `smoke_report_capture_moment` green; `smoke_cj3_layer_management` failed twice on a machine in use (its tab click landed on an inactive window) and was not run a third time. Owed: one run with nobody at the keyboard. The OZF2 half is done: the packaged app drew `2026-09-21_Kruglinskiy_Satell_z17_ozf.map` through `get_ozi_tile_projected`.
