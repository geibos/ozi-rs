## 1. Tests

- [x] 1.1 A whole-format legacy file: track, segment, points, waypoint, only the always-present fields
- [x] 1.2 It asserts what absence means, not only that the load returns Ok
- [x] 1.3 Verified red by removing `#[serde(default)]` from `Track.style`

## 2. Gates

- [x] 2.1 `just ci` green
- [ ] 2.2 `just smoke` green (blocked: the Mac2 driver host crashes at session creation — see `docs/STATE.md`)
