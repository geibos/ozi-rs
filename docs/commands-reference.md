# Commands Reference

## Principle

All non-trivial edits flow through `ProjectCommand` variants defined in
`src-tauri/src/application/commands.rs`. Each command validates before applying, and
undoable commands store a forward/reverse delta pair for undo/redo.

Some style mutations bypass the command stack (non-undoable) because they are
non-destructive and immediately visible.

## ProjectCommand Variants

| Command | Description | Undoable |
|---------|-------------|----------|
| `AddMapLayer` | Add a new map layer | yes |
| `AddMapLayerWithSource` | Add map layer with file path | yes |
| `AddTrackLayer` | Add a new track layer | yes |
| `AddWaypointLayer` | Add a new waypoint layer | yes |
| `RemoveMapLayer` | Remove a map layer (stores full layer for undo) | yes |
| `RemoveTrackLayer` | Remove a track layer (stores full layer for undo) | yes |
| `RemoveWaypointLayer` | Remove a waypoint layer (stores full layer for undo) | yes |
| `AddTrack` | Add a track to a layer | yes |
| `RemoveTrack` | Remove a track (stores full track for undo) | yes |
| `DeleteTrack` | Delete track by ID (resolves to RemoveTrack) | yes |
| `RenameTrack` | Rename a track (stores old and new names) | yes |
| `CreateEmptyTrack` | Create a new empty track with a name | yes |
| `MoveTrackPoint` | Move a point to new coordinates (stores old position) | yes |
| `DeleteTrackPoint` | Delete a point (stores removed index and point) | yes |
| `InsertTrackPoint` | Insert a point at index in a segment | yes |
| `SplitSegment` | Split segment at a point into two segments | yes |
| `JoinSegments` | Join two adjacent segments into one | yes |
| `SimplifyTrack` | Douglas-Peucker simplification (stores removed points) | yes |
| `RestoreTrackPoints` | Reverse of SimplifyTrack — re-inserts removed points | yes |
| `AddWaypoint` | Add a waypoint to a layer | yes |
| `RemoveWaypoint` | Remove a waypoint (stores full waypoint for undo) | yes |
| `DeleteWaypoint` | Delete waypoint by ID (resolves to RemoveWaypoint) | yes |
| `MoveWaypoint` | Move waypoint to new coordinates | yes |
| `RenameWaypoint` | Rename a waypoint (stores old and new names) | yes |
| `SetWaypointSymbol` | Set or clear a waypoint symbol (stores old and new symbols) | yes |

## Tauri IPC Commands

Defined in `src-tauri/src/commands/mod.rs` and `commands/tiles.rs`.

### Query Commands (read-only)

| Command | Description |
|---------|-------------|
| `get_app_state` | Full app state snapshot for the frontend |
| `get_tracks_geojson` | All tracks as GeoJSON FeatureCollection |
| `get_track_detail` | Full track geometry (segments, points) |
| `get_waypoints` | All waypoints in a layer |
| `get_simplified_preview` | Preview of Douglas-Peucker without committing |
| `get_track_export_default_path` | Suggested active-bundle `10-Tracks/<track>.<ext>` export path, or no suggestion if unavailable |

### Project / Bundle Management

| Command | Description |
|---------|-------------|
| `load_projects` | Fetch LizaAlert project list (streaming) |
| `load_project` | Open a LizaAlert project by slug |
| `open_selected_map` | Activate a map package (downloads if needed) |
| `open_local_bundle` | Open map bundle from a local directory |
| `set_bundles_root` | Set root directory for map bundles |
| `save_project` | Save project to `.ozp` file |
| `load_project_file` | Load project from `.ozp` file |
| `reveal_bundle` | Open active bundle directory in file explorer |

### Import / Export

| Command | Description |
|---------|-------------|
| `import_gpx` | Import GPX file or ZIP archive |
| `import_plt` | Import PLT file |
| `export_gpx` | Export track layer to GPX |
| `export_track_plt` | Export single track to PLT |
| `export_wpt_waypoints` | Export waypoint layer to OziExplorer WPT v1.1 (cp1251, CRLF) |
| `get_wpt_export_default_path` | Suggest WPT export path (`<bundle>/<layer>.wpt`) |

### Track Mutations (via CommandStack, undoable)

| Command | Description |
|---------|-------------|
| `rename_track` | Rename a track |
| `move_track_point` | Move a point by drag |
| `delete_track_point` | Delete a point |
| `insert_track_point` | Insert a point at index |
| `split_segment` | Split segment at a point |
| `join_segments` | Join two adjacent segments |
| `delete_track` | Delete a track |
| `create_empty_track` | Create empty track (for drawing mode) |
| `simplify_track` | Apply Douglas-Peucker simplification |

### Track Style (non-undoable, bypass CommandStack)

| Command | Description |
|---------|-------------|
| `set_track_color` | Set track RGBA color |
| `set_track_line_width` | Set track line width |
| `toggle_track_visible` | Toggle track visibility |

### Waypoint Mutations (via CommandStack, undoable)

| Command | Description |
|---------|-------------|
| `add_waypoint` | Add waypoint at coordinates |
| `move_waypoint` | Move waypoint to new coordinates |
| `delete_waypoint` | Delete a waypoint |
| `rename_waypoint` | Rename a waypoint |
| `set_waypoint_symbol` | Set or clear waypoint symbol |

### History

| Command | Description |
|---------|-------------|
| `undo` | Undo last command |
| `redo` | Redo last undone command |

### Tile Serving (in `commands/tiles.rs`)

| Command | Description |
|---------|-------------|
| `get_sqlite_tile` | Return tile bytes from MBTiles SQLite |
| `get_ozi_tile` | Return raw OZF2 tile (ungeoreferenced) |
| `get_ozi_tile_projected` | Return OZF2 tile reprojected to Web Mercator |
| `get_ozi_metadata` | Return OZI map metadata (levels, bounds, calibration) |

## Adding a New `ProjectCommand`

Concrete checklist when introducing a new undoable edit. Each step has tests inline; do not skip them.

1. **Domain mutation method** (if missing). Add the smallest possible mutation on the relevant domain entity in `src-tauri/src/domain/`. Return values needed for the inverse (e.g. previous coordinates, removed index + entity for undo). Cover with `#[cfg(test)]` tests.

2. **`ProjectCommand` variant.** Add the variant to `application/commands.rs` with all data needed for both directions. Inverses must be computable; for "remove" commands, store the removed entity in the variant itself so undo can re-insert it. Add a constructor (`ProjectCommand::your_command(...)`).

3. **`apply()` and `reverse()`.** Wire the new variant into `apply_to_project` (forward) and `reverse_for_project` (computes the reverse `ProjectCommand` from the pre-mutation state). For drag-style commands, also implement `targets_same_entity()` so coalescing works.

4. **Tests in `commands.rs`.** At minimum: apply mutates, undo restores exactly, redo reapplies, error variants surface as `CommandError::ProjectLayer(_)`. For coalescing commands, test that two consecutive operations on the same target collapse into one undo step.

5. **`AppState` adapter** in `application/mod.rs`. Add an `apply_<your_command>` method that fetches any pre-state needed for the inverse and calls `self.history.apply(...)` (or `apply_or_merge` for drag commands). Map errors to `ProjectLayerError`.

6. **Tauri handler** in `src-tauri/src/commands/mod.rs`. Add `#[tauri::command] pub fn your_command(...)` that locks state, calls the AppState adapter, emits `state-changed`, and converts errors to `String`.

7. **Register** in `src-tauri/src/lib.rs::generate_handler!`. The build fails fast if you forget — but the failure is far from the change, so save yourself the round-trip.

8. **Typed wrapper** in `src/lib/api.ts`. Mirror the parameter list using `bigint` for IDs, named arguments matching Rust `snake_case` → TypeScript `camelCase` (Tauri auto-converts).

9. **DTO sync** if the command returns or accepts a struct. Update `src/lib/types.ts` and the Rust DTO in `commands/mod.rs` together. There is no codegen.

10. **UI hookup.** Call the wrapper from the relevant component or store. Never call `invoke()` directly. Update Svelte stores if state shape changes.

11. **Update this file.** Add the variant to the table above and the IPC command to the relevant subsection. Update `docs/feature-status.md` if user-visible behavior changes.

If the command is **non-undoable style only** (track color, line width, visibility), skip steps 2-4 and route directly through an `AppState` setter that mutates without going through `CommandStack`. Document the skip in the Style table above and reference ADR-0017.

## Backend Events (backend → frontend)

| Event | Payload | Purpose |
|-------|---------|---------|
| `state-changed` | `{}` | Frontend re-fetches app state |
| `download-progress` | `{ package_name, downloaded_bytes, total_bytes? }` | Map download progress |
| `projects-chunk` | `Vec<LizaProjectSummaryDto>` | Streaming project list |
| `bundle-progress` | `{ message, phase, completed?, total? }` | Bundle extraction progress |
