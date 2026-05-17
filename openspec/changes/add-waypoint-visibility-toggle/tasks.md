## 1. Validate deltas

- [ ] 1.1 Run `openspec validate add-waypoint-visibility-toggle --strict` and confirm both `waypoints` and `undo-redo` MODIFIED requirements parse with at least one scenario each.
- [ ] 1.2 Confirm full-text copy of modified requirements matches the current `openspec/specs/<capability>/spec.md` body before the additions.

## 2. Domain field

- [ ] 2.1 Add `pub visible: bool` to `Waypoint` in `src-tauri/src/domain/waypoint.rs` with `#[serde(default = "default_true")]` so existing `.ozp` files deserialize with `visible = true`.
- [ ] 2.2 Update every `Waypoint` constructor / test fixture to initialize `visible: true`.

## 3. Application / command layer

- [ ] 3.1 Add a method on the project state (mirroring `toggle_track_visible`) that flips `visible` on the addressed waypoint and returns the new value.
- [ ] 3.2 Do NOT add a new `ProjectCommand` variant — confirm the mutation never enters `apply_or_merge()` or the undo stack.

## 4. Tauri handler

- [ ] 4.1 Add `#[tauri::command] async fn toggle_waypoint_visible(layer_id: LayerId, waypoint_id: WaypointId, ...)` in `src-tauri/src/commands/mod.rs`.
- [ ] 4.2 Register the handler in the Tauri `invoke_handler` builder.
- [ ] 4.3 Emit the same state-changed event used by `toggle_track_visible` so the frontend can re-render.

## 5. Frontend UI + API wrapper

- [ ] 5.1 Add a TypeScript wrapper `toggleWaypointVisible(layerId, waypointId)` alongside the existing `toggleTrackVisible` wrapper.
- [ ] 5.2 Sync the `Waypoint` type in `src/lib/types.ts` (add `visible: boolean`).
- [ ] 5.3 Add a per-row visibility checkbox to `WaypointsPanel.svelte` bound to the new wrapper; hidden rows SHALL remain visible in the panel and indicate hidden state.
- [ ] 5.4 In `MapView`, filter out waypoints whose `visible === false` when building the markers source.

## 6. Tests

- [ ] 6.1 Rust unit test: toggle round-trip flips `visible` and persists across `serde_json` round-trip.
- [ ] 6.2 Rust test: toggling visibility does NOT push onto the undo stack (assert stack depth unchanged).
- [ ] 6.3 Rust test: legacy `.ozp` JSON without the `visible` field deserializes with `visible = true`.
- [ ] 6.4 Frontend test (vitest): toggling the panel checkbox invokes the wrapper with the correct `(layer_id, waypoint_id)`.

## 7. QA via `just test` / `just clippy`

- [ ] 7.1 `just clippy` is clean.
- [ ] 7.2 `just test` passes (Rust + frontend).
- [ ] 7.3 Manual smoke per `docs/agent-verification.md`: place two waypoints, toggle one hidden, confirm only one renders; toggle back; reload project; both render.
