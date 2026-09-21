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

## 5. Not in this slice

- [ ] 5.1 Projection, the last of the three on-map tools ADR-0020 declares
- [ ] 5.2 The tape's and the ring's rendered pixels are unverified. MapLibre does not set `preserveDrawingBuffer`, so reading the canvas back gives an empty buffer — the zero I got means nothing either way, and is recorded rather than reported as a pass

## 6. Gates

- [x] 6.1 `just ci` green
- [ ] 6.2 `just smoke` green (blocked: the Mac2 driver host crashes at session creation — see `docs/STATE.md`)
