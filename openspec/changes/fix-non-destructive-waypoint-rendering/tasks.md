## 1. Validate delta

- [x] 1.1 Read existing `layers` spec and confirm the "non-destructive" requirement currently has only a track scenario.
- [x] 1.2 Run `openspec validate fix-non-destructive-waypoint-rendering --strict` and confirm it passes before any code work begins.

## 2. Refactor refreshWaypointMarkers to iterate all layers

- [x] 2.1 Inspect `src/lib/stores.ts` and confirm whether the renderer can already access all waypoint layers; if not, add a derived store exposing visible waypoints across every layer (respecting per-layer `visible`).
- [x] 2.2 Update `MapView.svelte::refreshWaypointMarkers` (lines ~296–333) to iterate every waypoint layer, honouring per-layer `visible` and per-waypoint `visible`. Remove the implicit "active layer only" filter.
- [x] 2.3 Ensure marker identity / keying includes the owning layer id so toggling layer visibility removes the correct markers without touching others.

## 3. Ensure edit interactions still route to active layer

- [x] 3.1 Click-to-add (waypoint placement mode) appends only to the active waypoint layer.
- [x] 3.2 Drag-to-move emits `ProjectCommand` targeting the waypoint's owning layer (so dragging a non-active-layer marker is either disabled or routes correctly — pick whichever matches the existing read-only-context UX decision and document it). **Decision:** drag is disabled (`draggable: false`) on markers from inactive layers — they render as read-only context with reduced opacity. The dragend handler is wired only for the active layer.
- [x] 3.3 Rename / delete in the Waypoints panel continue to operate on the active layer only.

## 4. Frontend tests

- [x] 4.1 Add or update a unit/component test asserting that with two waypoint layers (A active, B inactive) all waypoints from both layers are rendered. (`src/test/non-destructive-waypoint-rendering.test.ts`)
- [x] 4.2 Add a test asserting that placing a new waypoint while A is active appends to A and leaves B unchanged. (covered by the click-to-add routing assertion in the same suite)

## 5. QA

- [x] 5.1 Run `just clippy` (no Rust changes expected, but verify clean). Verified clean (Rust untouched).
- [x] 5.2 Run `just test`. Worktree vitest: 8 files, 39 tests passed. Rust tests: 173 passed.
- [ ] 5.3 Manual desktop verification per `docs/agent-verification.md`: load a project with two waypoint layers, switch active layer, confirm markers from both layers remain visible. *(Deferred to interactive QA — requires a running Tauri build with multi-layer fixture.)*
