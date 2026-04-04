# Learnings & Conventions

## 2026-04-04 — Wave 0 + Wave 1 (Tasks 1-9)

### Key patterns from domain module
- `haversine_distance(lat1, lon1, lat2, lon2) -> f64` already exists in `src-tauri/src/domain/track.rs` — returns meters
- `TrackPoint` struct has `lat: f64`, `lon: f64`, `id: u64` fields
- All helpers on `Project` follow naming: `{verb}_{entity}_in_layer(layer_id, ...)` pattern
- Error type is `ProjectLayerError` with variants: `MissingLayer`, `MissingTrack`, `MissingTrackSegment`, `MissingTrackPoint`
- No `thiserror`/`anyhow` — manual `Display` impls only
- No `#[allow(clippy::...)]` suppressions (except pre-existing)
- Tests inline in `#[cfg(test)] mod tests` only — no separate test files

### CommandStack
- Delta-based: stores `{forward, reverse}` pairs
- `apply_or_merge()` for drag coalescing — merges if same (layer_id, track_id, segment_id, point_id)
- Max 100 entries

### Cargo commands
- All cargo commands run from `src-tauri/` directory
- Pre-commit: `cargo test && cargo clippy --all-targets --all-features -- -D warnings`

### No new lock().unwrap()
- All new handlers use `lock_app_state()` helper

## 2026-04-04 — Wave 2 (Tasks 11-19)

- In `ProjectCommand`, undo reverse is computed before apply (`apply_or_merge`), so destructive commands should reconstruct reverse payload from current `Project` state rather than mutating command state.
- `Project::split_segment_in_layer` auto-generates new segment IDs; to preserve explicit undo/redo IDs, `SplitSegment::apply` must call `TrackSegment::split_at_point` directly and insert the returned segment into the track.
- `SimplifyTrack` can be implemented with a paired internal `RestoreTrackPoints` command: reverse computes removed points list, simplify removes by point id in reverse order, restore reinserts by `(segment_id, index)` ascending order.

## 2026-04-04 — Wave 3 (Tasks 20-26)

- Tauri commands with `State<SharedState>` + `AppHandle` already use 2 mandatory args, so domain args beyond 5 trigger clippy's too-many-arguments lint. Work around by grouping: `position: [f64; 2]` instead of `lat: f64, lon: f64`.
- `set_track_line_width` bypasses the command stack (no undo), same pattern as `set_track_color`. Uses `project.track_mut()` directly, clamps to `0.5..=20.0`.
- Read-only endpoints (`get_track_detail`, `get_waypoints`, `get_simplified_preview`) take immutable state lock — no `mut`, no `AppHandle`.
- `project_waypoint_layers()` accessor needed on `AppState` to expose `project.waypoint_layers()` for the `get_waypoints` endpoint.
- New segment/point IDs generated via `max_existing_id + 1` pattern in AppState before dispatch to command.
- All 7 commits were made via incremental rebuild (stash + re-apply per task) because code was originally written in one pass across 3 files.

- Created TrackPointsPanel component matching the Svelte 5 / Catppuccin theme pattern of TracksPanel. Used standard svelte-ignore a11y comments to allow onClick handlers on div rows.

## WaypointsPanel Implementation
- Used Svelte 5 runes `$state` and `$effect` for reactive components.
- Mirrored the TracksPanel design, applying Catppuccin CSS tokens (e.g. `--ctp-mantle`, `--ctp-surface0`) for visual consistency.
- Used `BigInt` correctly when translating from `number` IDs in the `WaypointData` DTO to `bigint` required by Tauri API endpoints.
- Comment tags inside HTML blocks in Svelte need to use standard HTML format (`<!-- -->`), rather than JSX `{/* */}`, which causes JS parse errors.

### Waypoint Symbol Picker (Svelte 5)
- Used a custom Popover with grid for selecting standard emojis instead of a native `<select>`, to provide better visual experience.
- State mutation commands (like `set_waypoint_symbol`) don't need to be part of the `CommandStack` if they represent lightweight visual preferences instead of crucial geographic data (following the `set_track_color` pattern).
- Svelte 5 click-outside handler using `$effect` needs to use the capture phase (`true` third parameter) for robust click closing.

## 2026-04-04 — Task 31 map drag edit mode

- MapLibre point editing can be done without custom layers by creating one `maplibregl.Marker` per track point and rebuilding marker set after each mutation (`move/delete/insert`) to keep point IDs/indexes in sync.
- Tauri `move_track_point` expects `position` as `[lat, lon]` even though `Marker#getLngLat()` returns `{ lng, lat }`, so frontend must convert to `[lngLat.lat, lngLat.lng]`.
- Edit mode UX is cleanest when map drag-pan is disabled and canvas cursor is switched (crosshair), then restored immediately on exit.
- Track-point context menu should be positioned relative to map container (`clientX/Y - container rect`) so menu stays anchored correctly even with floating panels.
