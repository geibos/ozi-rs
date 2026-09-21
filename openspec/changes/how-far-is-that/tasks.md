## 1. Frontend

- [x] 1.1 `geo.ts`: haversine matching the Rust one, path length, and a readable format — with tests
- [x] 1.2 `measuringActive` / `measuredPoints`, and one function that turns the tool on or off and discards the points
- [x] 1.3 The palette toggles it, labelled for the state it is in
- [x] 1.4 A click adds a point and is not passed on to selection or waypoint placement
- [x] 1.5 The running total over the canvas; Esc finishes

## 2. Evidence

- [x] 2.1 Walked on the stand: the palette turns it on, the readout reads "0 м · Клик — мерить · Esc — закончить", two clicks on the canvas make it "101 м", and Esc takes it away

## 3. Not in this slice

- [ ] 3.1 Radius circle and projection, the other two on-map tools ADR-0020 declares
- [ ] 3.2 A drawn line between the measured points — the number is the tool; the line is decoration this slice did not need to argue about

## 4. Gates

- [x] 4.1 `just ci` green
- [ ] 4.2 `just smoke` green (blocked: the Mac2 driver host crashes at session creation — see `docs/STATE.md`)
