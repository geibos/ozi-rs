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
