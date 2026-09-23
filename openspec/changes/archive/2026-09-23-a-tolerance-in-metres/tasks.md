# Tasks

- [x] 1.1 A failing test first: a 1 km leg with a 50 m bulge, kept at a 10 m
      tolerance and removed at 100 m
      (`domain::track::tests::simplify_in_metres_keeps_a_fifty_metre_bulge_at_ten_metres`).
- [x] 1.2 `simplify_track_points_m` in `domain/track.rs`, exported from
      `domain/mod.rs`.
- [x] 1.3 `apply_simplify_track` takes metres and converts once.
- [x] 1.4 Both Tauri commands rename their parameter to `tolerance_m`;
      bindings regenerated, `api.ts` wrappers renamed.
- [x] 1.5 `simplifyState.tolerance` → `toleranceM` in the store and both
      components that open the control.
- [x] 1.6 `src/test/simplify-tolerance-is-metres.test.ts`: the unit survives
      the next regeneration, nothing converts twice, the label still says «м».
- [x] 1.7 The stand's `get_simplified_preview` answers a count that falls as
      the tolerance rises.
- [x] 1.8 `just ci` green: 376 Rust, 587 frontend.
