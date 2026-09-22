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
- [x] 4.2 `just smoke` green (2026-09-22, against a bundle built the same hour; the owner granted the Accessibility permission the Mac2 driver needs)
- [x] 4.3 Covered as far as this repository can cover it, and seen on screen (2026-09-22). `MapView` and the inspectors need a MapLibre instance, so no vitest mounts them; what exists instead is `edit-failure.test.ts` on the reporter, `no-silent-failures.test.ts` as a class guard that fails on any `catch` in a component that tells nobody, and a live check on the stand: placing a waypoint by bearing against a backend that refused it raised «Не удалось поставить точку» carrying the backend's own message (`docs/progress/2026-09-22-verification/refused-edit-toast.png`). The residual gap — no automated test drives `MapView` itself — is in `docs/backlog.md`.
