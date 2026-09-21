## 1. Core

- [x] 1.1 `apply_trim_track_at_point`, positional, keeping the named point
- [x] 1.2 `apply_crop_with` takes an `FnMut`, since a positional rule cannot judge a point on its own
- [x] 1.3 Tests: both directions, the point survives, one undo restores all, and a trim at an end records nothing

## 2. Surface

- [x] 2.1 `trim_track_at_point` command, registered, bindings regenerated
- [x] 2.2 `trimTrackAtPoint` in `api.ts`
- [x] 2.3 Two controls per point row, labelled in both dictionaries, not stealing the row's own click
- [x] 2.4 The stand accepts the command

## 3. Gates

- [x] 3.1 `just ci` green
- [ ] 3.2 `just smoke` green (blocked: the Mac2 driver host crashes at session creation — see `docs/STATE.md`)
