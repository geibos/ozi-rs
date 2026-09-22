## 1. Tests

- [x] 1.1 A whole-format legacy file: track, segment, points, waypoint, only the always-present fields
- [x] 1.2 It asserts what absence means, not only that the load returns Ok
- [x] 1.3 Verified red by removing `#[serde(default)]` from `Track.style`, which is not an `Option` and so depends on it
- [x] 1.4 The same guard for the session file, whose failure costs the restored project as well
- [x] 1.5 A corrupt session is distinguished from an absent one — absent is a first run
- [x] 1.6 Corrected two comments and an earlier write-up that credited `#[serde(default)]` on `Option` fields with tolerance they have anyway

## 2. Gates

- [x] 2.1 `just ci` green
- [x] 2.2 `just smoke` green (2026-09-22, against a bundle built the same hour; the owner granted the Accessibility permission the Mac2 driver needs)
