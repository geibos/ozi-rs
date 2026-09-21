## 1. Core

- [x] 1.1 `Waypoint.color`, optional, `#[serde(default)]` for older projects
- [x] 1.2 Layer- and project-level setters returning the previous colour
- [x] 1.3 `SetWaypointColor` command with its inverse, and the application method
- [x] 1.4 A test over set, set again, undo, redo, and clearing back to the default

## 2. Surface

- [x] 2.1 `set_waypoint_color` command, registered, bindings regenerated
- [x] 2.2 `WaypointDto.color` and the `api.ts` wrapper
- [x] 2.3 The inspector's swatch and its "default colour" control
- [x] 2.4 The map marker's background
- [x] 2.6 The row in the Waypoints tab and the inspector's picker draw the same disc, from one shared conversion — a crew reads the list to find what they are looking at on the map, so the two have to agree
- [x] 2.5 The fixture gives one waypoint a colour and leaves another without, so the stand shows both cases

## 3. Evidence

- [x] 3.1 Walked on the stand: the flagged waypoint draws 🏁 on `rgb(37, 99, 235)`, the uncoloured one 📍 on the default

## 4. Gates

- [x] 4.1 `just ci` green
- [ ] 4.2 `just smoke` green (blocked: the Mac2 driver host crashes at session creation — see `docs/STATE.md`)
