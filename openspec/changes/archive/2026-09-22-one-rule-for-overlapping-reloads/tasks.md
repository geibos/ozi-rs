## 1. The rule

- [x] 1.1 `createLatestRun` with tests: the newest token is current, an overtaken one never becomes current again, counters are independent
- [x] 1.2 Both library tabs use it in place of their own counters

## 2. The map

- [x] 2.1 The marker refresh takes a token and drops an overtaken run before it draws
- [x] 2.2 The track geometry fetch does the same before it updates the layer
- [x] 2.3 Waypoint layers read in parallel
- [x] 2.4 The source assertions guarding "all visible layers, not only the active one" no longer depend on the loop's shape

## 3. Gates

- [x] 3.1 `just ci` green
- [x] 3.2 `just smoke` green (2026-09-22, against a bundle built the same hour; the owner granted the Accessibility permission the Mac2 driver needs)
- [x] 3.3 The map's two guards are now covered (2026-09-22). `MapView` still needs a MapLibre instance, so vitest cannot mount it; what a test can pin is the wiring, and `src/test/map-refresh-liveness.test.ts` does: for `waypointMarkerRuns` and `trackGeometryRuns` alike, the token is taken, the asynchronous boundary follows, and `isCurrent` is checked before anything is written — read by index, so reordering them fails. The rule's own behaviour stays in `latest-run.test.ts`.
