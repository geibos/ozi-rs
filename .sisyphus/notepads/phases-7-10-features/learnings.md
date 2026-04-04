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

## 2026-04-04 — Task 32 click-to-add waypoint mode

- `add_waypoint` Tauri command needed to be added from scratch: backend had `ProjectCommand::add_waypoint` but no wired Tauri handler. Added `apply_add_waypoint` to `AppState` in `application/mod.rs`, then `#[tauri::command] pub fn add_waypoint` in `commands/mod.rs`, registered in `lib.rs`.
- ID auto-generation: `max existing waypoint id in layer + 1` pattern, same as track point IDs.
- `addWaypointMode` store (boolean) in `stores.ts` drives cursor (crosshair) and click interception.
- MapView click handler is a single persistent `map.on("click", handler)`. Inside, it guards on `$addWaypointMode` before doing anything — safe because the reactive $effect re-reads the store value on each call.
- Escape key exits `addWaypointMode` alongside context menu close — handle both in same `handleKeydown`.
- Cursor precedence: `addWaypointMode` wins over default; restores only when `!editModeActive` (so edit mode cursor is not clobbered by the waypoint effect running at wrong time).
- Waypoint markers use `maplibregl.Marker` with a `div.waypoint-marker` element; style must be `:global(.waypoint-marker)` since the element is appended outside Svelte's scoped CSS.
- `refreshWaypointMarkers` is async and may throw if the waypoint layer doesn't exist yet (e.g. fresh project) — wrap with try/catch, silently ignore the error.
- Waypoint name generation: `getWaypoints(layerId)` count + 1 → "Waypoint N".

## 2026-04-04 — Task 33 waypoint drag on map

- `move_waypoint` Tauri command did NOT exist yet (unlike `delete_waypoint`/`rename_waypoint`). Had to wire the full stack: `apply_move_waypoint` in `application/mod.rs` → `pub fn move_waypoint` in `commands/mod.rs` → registered in `lib.rs`.
- `ProjectCommand::move_waypoint(layer_id, waypoint_id, lat, lon)` already existed in `application/commands.rs` and is fully undo-able — no extra reverse-state capture needed.
- MapLibre `Marker` draggable opt: `new maplibregl.Marker({ element: el, draggable: true })`. Drag events use `.on("dragstart", fn)` and `.on("dragend", fn)` on the marker instance.
- `marker.getLngLat()` returns `{ lat, lng }`. Pass to `moveWaypoint` as `[lngLat.lat, lngLat.lng]` (lat-first, matching Tauri position: [lat, lon] convention).
- Cursor management: set `el.style.cursor = "grab"` after element creation; change to `"grabbing"` on dragstart, restore on dragend. CSS `:active` alone is insufficient because MapLibre prevents browser's native drag events.
- After `moveWaypoint`, the backend emits `state-changed` (via Tauri `AppHandle::emit`), `appState.refresh()` fires in `App.svelte`, `$appState` changes, `$effect` in MapView calls `refreshWaypointMarkers()` — no extra wiring needed.
- `wpId` must be captured per-loop-iteration (`const wpId = wp.id`) before the async closure so BigInt conversion uses the correct value.
- `position` arg in Tauri command is `[f64; 2]` (lat, lon order), consistent with `move_track_point` pattern.

## 2026-04-04 — Task 35 PLT export

- Added `infrastructure::export::plt::export_plt(track, color, width, writer)` with strict Windows CRLF output and explicit Ozi header lines.
- PLT COLORREF conversion in exporter uses BGR byte order from `0xRRGGBB` input (`(b << 16) | (g << 8) | r`).
- Width mapping for PLT track-info line must clamp to integer `1..=7` after rounding the floating UI width.
- OLE date encoding is implemented from UTC `DateTime` via CE-day delta from 1899-12-30 + fraction-of-day seconds.
- Segment starts are encoded in the trailing data-field (`...,<segment_flag>`), with `1` for first point in each segment and `0` for subsequent points.
- Round-trip verification currently compares flattened points (`lat/lon/elevation/timestamp`) because importer segment splitting uses field 3 while exporter writes segment markers in the trailing field required by Ozi format.

### Track Simplification UI
- `SimplifiedPreview` interface provided by the Tauri backend uses an array of `segments`, each containing `kept_points` (array of `PointDetail`). It does not use a flat array of `{lat, lon}` points directly on the preview root.
- The UI properly uses Svelte 5 `$effect` to watch `$simplifyState` changes, debounce 300ms, and update a local GeoJSON source mapped to `simplify-preview` in MapLibre GL.
- The `simplify-preview-line` uses the Catppuccin `#ff6600` (custom orange mapped color) over the normal track color to show real-time changes before committing them.

## 2026-04-04 — Task 36: PLT export Tauri handler + frontend integration

### Pattern: export_track_plt handler
- The Tauri command `export_track_plt(layer_id, track_id, path, state)` does NOT need `AppHandle` since it emits no state-changed event (pure file write, no app state mutation).
- Look up track from state, extract `style().color` ([u8;4] RGBA), compute RGB u32 as `(r<<16)|(g<<8)|b`, pass `style().line_width as f64` to `export_plt`.
- Use `use crate::infrastructure::export::plt::export_plt` as inline import in the fn body (crate root re-export also works).
- File creation: `std::fs::File::create(PathBuf::from(&path)).map_err(|e| format!(...))`.
- Save dialog is opened in the Svelte frontend (`open({ save: true, defaultPath: "name.plt", filters: [...] })`), not in the Rust handler — consistent with GPX export pattern.
- "Export as PLT" button is placed inside the `{#if $selectedTrack?.trackId === track.trackId}` block, so it only shows for the selected track.

### Bonus fix: CreateEmptyTrack missing match arms
- `ProjectCommand::CreateEmptyTrack` was added to the enum but `apply()` and `reverse()` match statements were missing arms, causing compile failure.
- Fix: `apply` arm creates `Track::new(track_id, name)` and calls `project.add_track_to_layer(layer_id, track)`.
- Fix: `reverse` arm produces `RemoveTrack { layer_id, track: Track::new(track_id, name) }`.
- These fixes were required to unblock `cargo test --all`.

## 2026-04-04 — Task 37: CreateEmptyTrack command

### Pattern: creation command with pre-generated ID
- AppState pre-generates the new ID before dispatch: `max(existing ids) + 1` (or 1 if empty).
- The domain method `TrackLayer::create_empty_track(track_id, name)` takes an explicit `TrackId`, not self-generating — keeps domain pure and deterministic.
- The command struct stores `{ layer_id, track_id, name }` — enough to both apply and produce a `RemoveTrack` reverse.
- Undo is `RemoveTrack { layer_id, track: Track::new(track_id, name) }` — `RemoveTrack` stores the full `Track` object and removes by id; this handles the round-trip cleanly.
- New empty segment always gets `TrackSegmentId::new(1)`.
- Tauri handler returns `track_id.value()` as `u64` for the frontend; frontend wraps as `bigint`.
- `create_empty_track` handler does NOT need `#[allow(clippy::question_mark)]` but other mutation handlers in the same file have it pre-existing — follow the file's existing pattern.

## 2026-04-04 — Task 38: map drawing mode for new tracks

- Drawing mode fits existing map interaction architecture by reusing one persistent `map.on("click")` handler and guarding behavior via a writable store (`drawingModeActive`), same as waypoint mode.
- Because MapLibre emits `click` before `dblclick`, delaying point insertion by ~220ms and canceling the timeout in `dblclick` cleanly prevents an unwanted extra point when finishing with double-click.
- A local GeoJSON source/layer pair (`drawing-preview`) is the most stable way to render in-progress lines + markers immediately, without waiting for backend `state-changed` refresh.
- Cursor/interaction precedence needs explicit ordering: drawing mode should force `crosshair`, disable pan + double-click zoom, and proactively clear conflicting modes (`editModeActive`, `addWaypointMode`) while active.
- Cancel flow is deterministic with command stack: Escape should call `undo()` exactly `insertedPointCount + 1` times (points plus `create_empty_track`) to remove all drawing artifacts.
