# Learnings

## Architecture Conventions
- All Tauri commands return `Result<T, String>` — use `map_err(|e| e.to_string())?` pattern
- Commands registered via `generate_handler!` macro in `lib.rs`
- `AppState` wrapper methods in `application/mod.rs`, raw Tauri handlers in `commands/mod.rs`
- Error type: `ProjectLayerError` with manual `Display` impl (no thiserror/anyhow)
- Tests go inline in `#[cfg(cfg(test))] mod tests` blocks, never separate files
- DTOs only at Tauri IPC boundary — internal domain→application comms use native types

## File Structure
- Domain: `src-tauri/src/domain/` (project.rs, track.rs, waypoint.rs)
- Application: `src-tauri/src/application/` (mod.rs commands.rs, import.rs)
- Infrastructure: `src-tauri/src/infrastructure/` (import/, export/, tiles, persistence)
- Tauri commands: `src-tauri/src/commands/` (mod.rs, tiles.rs)
- Frontend: `src/` (components/, lib/, views/)

## Key Patterns
- Layer traversal: `project.track_layers().iter()` then find by id
- Safe lock: `state.lock().map_err(|e| format!("State lock poisoned: {}", e))?`
- State change: emit `state-changed` event via `app_handle.emit()`
- Style mutations (non-undoable): direct `track.style_mut().set_X()` — no CommandStack

## Track Lookup Updates
- `Project` now exposes `track_layer_mut(layer_id: u64)`, `track_mut(layer_id, track_id)`, `track_segment_mut(layer_id, track_id, segment_id)`, and `track_point_mut(layer_id, track_id, segment_id, point_id)` as `Result`-returning helpers.
- `ProjectLayerError` uses manual `Display` strings for missing track/segment/point lookups, matching the existing layer/waypoint error style.
- `TrackId`, `TrackSegmentId`, `TrackPointId`, and `WaypointId` now expose `value()` accessors for readable error formatting and lookup helpers.

## Lint Cleanup Notes
- `clippy::enum_variant_names` was resolved by renaming `ProjectLayerError` and `OziMapParseError` variants to avoid shared prefixes/postfixes.
- `double_ended_iterator_last` in track duration logic now uses `next_back()` on the timestamp iterator.
- GPX export now uses `writeln!` for newline-terminated XML lines.
- Archive extraction no longer performs a redundant `PathBuf` conversion for `enclosed_name()`.
- PLT parsing and threaded archive extraction use let-chains to collapse nested `if` statements.
- Bounds-sensitive raster work should use `.get(...)` for palette lookups and check destination slices before copying; this kept OZF2 tile conversion/reprojection safe without changing the rendering flow.
- Tauri command handlers that lock shared state need to return `Result<_, String>` so `?` can propagate poisoned-lock errors cleanly.

## [2026-04-04] Task: T5
- Delta-based undo/redo can stay command-only (no project snapshot cloning) by storing per-entry `{ forward, reverse }` and generating `reverse` from pre-apply state.
- Drag coalescing behavior is safe when `apply_or_merge` only updates the last entry's `forward` command and keeps the original `reverse`, so undo returns to the pre-drag coordinates.
- Add/remove command pairs for layers/entities allow reversible add operations while preserving the existing public command payloads for caller-facing variants.
- Enforcing `MAX_STACK_DEPTH` by dropping the oldest undo delta keeps memory bounded and naturally leaves earliest operations non-undoable once the cap is exceeded.

## [2026-04-04] Task: T6
- `TrackSegment` point mutations (`move_point`, `remove_point`, `insert_point_at`, `split_at_point`, `points_mut`) reuse `ProjectLayerError::MissingTrackPoint` for all error returns; at segment level, `layer_id`/`track_id` are set to 0 as placeholders — the project-level wrappers re-map errors with proper full context IDs.
- `split_at_point` takes a `new_segment_id: TrackSegmentId` parameter; the project-level `split_segment_in_layer` generates the new ID by scanning `track.segments()` for the max existing segment ID and adding 1.
- Sibling module reference (`track.rs` → `crate::domain::ProjectLayerError` from `project.rs`) compiles cleanly in Rust — no circular dependency despite both files living in the same `domain` module.
- `insert_point_at` uses the same `MissingTrackPoint` error variant for index-out-of-bounds since there is no separate bounds-check variant; `point_id` field carries the out-of-bounds index value for debugging.
- Project-level wrapper methods call `track_segment_mut()` helper for traversal then delegate to segment method, remapping any error to the full-context variant.
- 8 new tests in `track.rs #[cfg(test)] mod tests`: 4 success + 4 error cases; helper `make_segment_with_points()` used to reduce boilerplate.

## [2026-04-04] Task: T7
- `remove_segment` returns `(index, TrackSegment)` so undo can call `insert_segment_at(index, segment)` (also added on `Track`).
- `join_segments(a, b)` requires `index_b == index_a + 1`; adjacency check uses `position()` on both IDs, then compares indices — non-adjacent returns `MissingTrackSegment` with `segment_id: seg_id_b` since that's the only applicable variant (no separate adjacency error exists).
- `join_segments` removes B before draining its points into A to avoid borrow issues with two `&mut` references into the same vec; drains via `seg_b.points_mut().drain(..).collect()` then `extend`.
- Project-level wrappers (`remove_segment_from_layer`, `join_segments_in_layer`) call `track_mut()` then delegate to `Track` method, remapping to full-context `MissingTrackSegment`.
- `insert_segment_at` uses `.min(len)` clamping to avoid panics on out-of-bounds undo scenarios.
- 4 new tests in `track.rs`: `remove_segment` success + error, `join_segments` success + non-adjacent error; helper `make_track_with_two_segments()` used for reuse.
- Final test count: 101 (was 97 before T7).

## [2026-04-04] Task: T8
- `TrackLayer::remove_track` mirrors `Track::remove_segment`: return `(index, Track)` so undo can reinsert at the same position.
- `Project::remove_track_from_layer` should delegate to the layer helper instead of duplicating the search/removal logic.
- Track-layer removal failures use `ProjectLayerError::MissingTrack { layer_id, track_id }`, keeping the project-level error shape consistent with existing nested lookup helpers.
- Inline tests now cover both success and missing-track cases for direct layer removal and project-level removal.

## [2026-04-04] Task: T9
- Waypoint undo-friendly mutations should mirror track patterns: `Waypoint::set_name` and `Waypoint::set_symbol` return previous values, and `WaypointLayer::remove_waypoint` returns `(index, Waypoint)` for reinsert-based undo.
- GPX waypoint symbol support maps cleanly through `gpx::Waypoint.symbol`; export should emit `<sym>` only when present, and import should preserve `None` vs `Some(...)` without validation.
- Project-level waypoint helpers can stay thin wrappers over layer helpers; keeping the existing `WaypointNotFound` error variant was enough for full-context remapping.
