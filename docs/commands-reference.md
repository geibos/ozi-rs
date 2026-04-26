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

## Backend Events (backend → frontend)

| Event | Payload | Purpose |
|-------|---------|---------|
| `state-changed` | `{}` | Frontend re-fetches app state |
| `download-progress` | `{ package_name, downloaded_bytes, total_bytes? }` | Map download progress |
| `projects-chunk` | `Vec<LizaProjectSummaryDto>` | Streaming project list |
| `bundle-progress` | `{ message, phase, completed?, total? }` | Bundle extraction progress |
