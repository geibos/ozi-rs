## 1. Where a name goes

- [x] 1.1 `labelAnchor`: half way by length, tested against the rest-stop case
- [x] 1.2 `declutter`: selected first, then the longer route, tested
- [x] 1.3 `positionsOf`: a multi-segment track in the order it was walked

## 2. On the map

- [x] 2.1 DOM markers reconciled like the waypoints, `textContent` never `innerHTML`
- [x] 2.2 Recomputed when the camera settles, not only when the data changes
- [x] 2.3 White halo, colour from the track, no pointer events

## 3. Gates

- [x] 3.1 `just ci` green
- [x] 3.2 Seen on the map 2026-09-23 (`docs/progress/2026-09-23-track-names/name-on-the-track.png`): «20260708_Veter2» in its own red, half way along the route. Finding it took three attempts — the geometry reaches the map through *three* paths and the names were added to two of them; then the map opens at zoom 5, where the threshold suppresses every name, and the flight to the data had already finished before the `moveend` handler was attached, so nothing asked again. `idle` as well as `moveend` now.
