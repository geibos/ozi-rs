## 1. The reporter

- [x] 1.1 `reportEditFailure` with tests: the interface's language, the backend's message, an `Error` passed through as its message
- [x] 1.2 Ten keys in both dictionaries

## 2. The sites

- [x] 2.1 Move, delete and insert a track point
- [x] 2.2 Add a waypoint, add a drawing point, cancel a draw, fit all tracks
- [x] 2.3 Both inspectors' load failures
- [x] 2.4 A failed track-point drag reloads the points, so the marker stops lying

## 3. The guard

- [x] 3.1 A test over every component for a `catch` that only logs
- [x] 3.2 Two allowlisted, each with its reason
- [x] 3.3 Verified red by putting one `console.error` back

## 4. Gates

- [x] 4.1 `just ci` green
- [ ] 4.2 `just smoke` green (blocked: the Mac2 driver host crashes at session creation — see `docs/STATE.md`)
- [ ] 4.3 The ten call sites are not covered by a test — `MapView` and the inspectors need a MapLibre instance. Stated rather than implied.
