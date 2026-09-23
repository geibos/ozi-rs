## 1. The loss that had to go first

- [x] 1.1 Failing test: a removed track layer holding tracks comes back whole on undo
- [x] 1.2 `RestoreTrackLayer` / `RestoreWaypointLayer`; a removal reverses to them
- [x] 1.3 The same for a waypoint layer and its marks

## 2. Renaming

- [x] 2.1 `RenameTrackLayer` / `RenameWaypointLayer` with reverses, tested
- [x] 2.2 Undo and redo of a rename, tested

## 3. Reaching the frontend

- [x] 3.1 `create_track_layer` / `create_waypoint_layer` answering with the new id
- [x] 3.2 `rename_track_layer` / `rename_waypoint_layer`
- [x] 3.3 `delete_track_layer` / `delete_waypoint_layer`, active layer moved on
- [x] 3.4 Bindings regenerated; `api.ts` wrappers

## 4. The controls

- [x] 4.1 Tracks tab: new layer, rename, delete
- [x] 4.2 Waypoints tab: the same
- [x] 4.3 Both localized, both dictionaries

## 5. Gates

- [x] 5.1 `just ci` green
- [x] 5.2 Walked on the stand 2026-09-23: created «День 3 · ЛИСА», renamed it to «День 3», deleted it — the active layer moved to «Треки» and the toast read «Слой удалён — Cmd+Z вернёт». `docs/progress/2026-09-23-layers/layer-menu.png`
- [x] 5.3 `just smoke` green (2026-09-23, both journeys against a bundle built the same hour; the layer journey added to the gate that day walks create → active → delete over real IPC)
