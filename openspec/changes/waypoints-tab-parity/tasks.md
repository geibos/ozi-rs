## 1. Backend

- [x] 1.1 `set_all_waypoints_visible(visible)` in the domain and application layers, as one style mutation, with tests
- [x] 1.2 `show_only_waypoint(layer_id, waypoint_id)` with tests including the missing-waypoint case
- [x] 1.3 Expose both as specta commands and regenerate `bindings.ts`

## 2. Frontend

- [x] 2.1 Extract the name-matching rule shared by both tabs, with tests
- [x] 2.2 Search field, clear button, counter and search-empty state in the Waypoints tab
- [x] 2.3 Show-all / hide-all controls and the "only this one" row action, localized
- [x] 2.4 Coordinates subline and a "show on map" row button, with a `waypoint` focus request consumed by `MapView`

## 3. Verification

- [x] 3.1 `just ci` green
- [x] 3.2 Screenshot in `docs/progress/`
