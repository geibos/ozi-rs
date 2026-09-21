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
- [ ] 3.2 `just smoke` green (blocked: the Mac2 driver cannot initialise UI testing on this machine — see `docs/STATE.md`)
- [ ] 3.3 The map's two guards are not covered by a test — `MapView` needs a MapLibre instance. Stated here rather than implied.
