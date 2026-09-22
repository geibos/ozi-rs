## 1. Frontend

- [x] 1.1 `boundsOf`, `geojsonPositions`, `isDegenerate` in `src/lib/map-bounds.ts`, with tests written first
- [x] 1.2 `MapView` frames tracks and waypoint markers together, centring a degenerate extent
- [x] 1.3 Both project-open paths ask for the frame

## 2. Verification

- [x] 2.1 On the stand: the camera moved from the 50 m scale to 500 m and both track segments and the waypoint markers are on screen
- [x] 2.2 `just ci` green
- [x] 2.3 `just smoke` green (2026-09-22, against a bundle built the same hour; the owner granted the Accessibility permission the Mac2 driver needs)
