## 1. Frontend

- [x] 1.1 `geo.ts`: haversine matching the Rust one, path length, and a readable format — with tests
- [x] 1.2 `measuringActive` / `measuredPoints`, and one function that turns the tool on or off and discards the points
- [x] 1.3 The palette toggles it, labelled for the state it is in
- [x] 1.4 A click adds a point and is not passed on to selection or waypoint placement
- [x] 1.5 The running total over the canvas; Esc finishes

## 2. Evidence

- [x] 2.1 Walked on the stand: the palette turns it on, the readout reads "0 м · Клик — мерить · Esc — закончить", two clicks on the canvas make it "101 м", and Esc takes it away

## 3. The tape

- [x] 3.1 A point drawn where each click landed, and a dashed line through them, in a colour that is neither a track's nor a waypoint's
- [x] 3.2 Backspace removes the last point, so a misclick does not cost the measurement
- [x] 3.3 `isEditableTarget` shared rather than copied, so the layout's chords and the tool's Backspace cannot disagree about one keypress — with its own tests
- [x] 3.4 Tests on the GeoJSON the tape hands MapLibre, including `lon, lat` order and that one point draws no line

## 4. The radius ring

- [x] 4.1 `destinationPoint` and `ringAround` in `geo.ts`, with tests that every ring point holds the radius at 60° north, at 500 m and at 25 km
- [x] 4.2 Longitudes wrapped into −180..180, so a ring near the antimeridian is not a band around the world
- [x] 4.3 Click a centre, click a radius, click again to move it; Esc finishes
- [x] 4.4 One tool at a time, each discarding what it held — with tests
- [x] 4.5 Walked on the stand: "Клик — центр", then "Клик — радиус", then "52 м"

## 5. Projection

- [x] 5.1 Click an origin, type a bearing and a distance, place the waypoint there
- [x] 5.2 The result previewed on the map before it is committed
- [x] 5.3 "Place" disabled until there is a distance to place at
- [x] 5.4 All three tools mutually exclusive, with tests
- [x] 5.5 The `Waypoint N` default name translated — it was English in a Russian window
- [x] 5.6 Walked on the stand: hint, then the fields, then 240° and 1200 m placing "Точка 4" at the computed coordinates

## 6. Not in this slice
- [x] 6.1 The tools' rendered pixels are verified (2026-09-22). The earlier attempt read the canvas back, which MapLibre leaves empty without `preserveDrawingBuffer` — the zero meant nothing. A page screenshot is a compositor capture, not a canvas readback, so it shows what is actually on screen. All three, on the stand, in `docs/progress/2026-09-22-verification/`: `measure-distance.png` — an orange dashed line through three clicked points with vertex dots, reading «398 м»; `radius-ring.png` — a dashed circle about its centre, reading «165 м»; `bearing-projection.png` — a waypoint placed 250 m at 45° from the origin, with the preview line before it. Placing the waypoint also needed `add_waypoint` added to the stand, which had no answer for it, so that tool had never been walked past its form

## 7. Gates

- [x] 7.1 `just ci` green
- [x] 7.2 `just smoke` green (2026-09-22, against a bundle built the same hour; the owner granted the Accessibility permission the Mac2 driver needs)
