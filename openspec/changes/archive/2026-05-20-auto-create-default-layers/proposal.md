## Why

Two of the most important edits — "Add Waypoint" and "Create Track" — silently do nothing when the current project has no track layer or no waypoint layer. The Sidebar toggle flips, the cursor changes, the user clicks the map, and the click handler hits `if (layerId === null) return;` (`Sidebar.svelte:109`, `MapView.svelte:377`). No error, no toast, no recovery.

This is a direct consequence of the `layers` spec leaving layer existence to whatever happens to be in the project. There is no invariant that says "an open project always has at least one track layer and one waypoint layer". The frontend respects an unconstrained backend, the backend doesn't enforce defaults, and the user pays for the gap.

This change fixes the gap at its root: every project that the user sees in the workspace is guaranteed to have at least one default track layer and at least one default waypoint layer. The frontend can then drop its silent-null guards and treat the active layer IDs as always present.

## What Changes

- Newly created projects SHALL be initialized with one default track layer ("Tracks") and one default waypoint layer ("Waypoints"); these layers are first-class layers, not magic constants.
- Loading a legacy `.ozp` project that contains zero track layers or zero waypoint layers SHALL transparently create the missing default layers on load, leaving any existing layers untouched.
- The frontend stores `$activeTrackLayerId` and `$activeWaypointLayerId` SHALL be guaranteed non-null whenever a project is open. Click handlers in `Sidebar` and `MapView` SHALL no longer silently return on null layer IDs.
- Default layer names are not reserved; the user MAY rename or hide them like any other layer (full management UI remains out of scope).

## Capabilities

### New Capabilities
- _none_

### Modified Capabilities
- `layers`: add the invariant that every open project contains at least one track layer and at least one waypoint layer; specify default-layer creation on project initialization.
- `project-persistence`: add the normalization rule that load-from-disk creates the missing default layers when a legacy `.ozp` lacks them.

## Impact

- **Backend** (`src-tauri/src/domain/project.rs`, `src-tauri/src/application/` Project construction paths): `Project::default()` / `Project::new()` populates one track layer and one waypoint layer. Project load path normalizes missing layers.
- **Frontend** (`src/components/Sidebar.svelte`, `src/components/MapView.svelte`, `src/lib/stores.ts`): remove `if (layerId === null) return;` guards from `toggleTrackDrawingMode`, `handleMapClickForWaypoint`, and similar handlers; `syncActiveLayer` simplifies because layers are always non-empty.
- **Round-trip**: existing `.ozp` save/load is unaffected on the write side; the read side is additive (normalize before exposing). Files saved after this change still load correctly in pre-change builds because they contain explicit layers.
- **Spec evidence**: backend unit test for `Project::new()` producing default layers; load-path test for normalizing a `.ozp` with no track/waypoint layers; smoke that "Create Track" and "Add Waypoint" work on a freshly-loaded bundle without any user setup.
- **Risk**: low. Pre-change projects either already have layers (no change) or have none and were silently broken (now fixed).
