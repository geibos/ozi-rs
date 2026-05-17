## Why

The `layers` capability requires that selecting an active layer is non-destructive — switching the active layer MUST NOT hide overlays from other layers. The track rendering pipeline honours this: `get_tracks_geojson` (`src-tauri/src/commands/mod.rs:205-252`) returns features from every track layer, and the frontend filters them only by per-track `visible` flags.

The waypoint rendering pipeline violates this invariant. `MapView.svelte` (`refreshWaypointMarkers`, lines 296–333) iterates only over waypoints of the active waypoint layer, so when a SAR volunteer switches the active waypoint layer (for example from "штаб" to "найдено"), all markers from the previously active layer disappear from the map even though they are still stored in the project and marked visible. This destroys situational awareness in field workflows where multiple waypoint layers ("штаб", "найдено", "опасности") need to be on screen simultaneously while the operator edits one of them.

The backend already exposes every waypoint layer in `AppStateDto`; the fix is frontend-only.

## What Changes

- `MapView.refreshWaypointMarkers` iterates over **all** waypoint layers, honouring both per-layer `visible` and per-waypoint `visible` flags, instead of reading only the active layer.
- Edit interactions (drag-to-move, click-to-add, rename, delete) continue to route only through the active waypoint layer; markers belonging to inactive layers render as read-only context.
- If the waypoint data source in `src/lib/stores.ts` currently exposes only the active layer's waypoints, it gains an "all visible waypoints" derived store so the renderer can consume the full set without leaking active-layer-only assumptions.
- No backend changes — `AppStateDto` already carries the necessary data.

## Impact

- Affected capability: `layers` (modifies the "Selecting an active layer is non-destructive" requirement to explicitly cover waypoints).
- Affected code: `src/components/MapView.svelte`, possibly `src/lib/stores.ts` (selector/derived store for visible waypoints across layers).
- No backend / Rust changes. No schema or `ProjectCommand` changes.
- Risk: low — purely additive on the render path; edit-routing logic is preserved.
