## Why

SAR operators need to temporarily hide individual waypoints without deleting them — for example, to clear "completed" search points from the map while keeping active ones visible. The `waypoints` spec already requires "Multiple waypoints render simultaneously with per-waypoint visibility", but the backend has no `set_waypoint_visible` / `toggle_waypoint_visible` API on `ProjectCommand` or the Tauri command surface (compare to the working `set_track_visible` / `toggle_track_visible` for tracks). The frontend has no UI affordance for it either. This change formalizes the missing behavior and aligns it with the existing non-undoable style-mutation model used for tracks.

## What Changes

- Add a `visible: bool` field to the `Waypoint` domain model (default `true`).
- Add a non-undoable Tauri command `toggle_waypoint_visible(layer_id, waypoint_id)` that flips the flag immediately, mirroring `toggle_track_visible`. The mutation SHALL NOT pass through the `ProjectCommand` undo stack — it follows the same model as `set_track_color`, `set_track_line_width`, and `toggle_track_visible`.
- Extend the Waypoints panel (`WaypointsPanel.svelte`) with a per-row visibility checkbox bound to the new command.
- MapView SHALL render only waypoints whose `visible` flag is `true`; hidden waypoints remain present in the panel and in the saved project file.

## Impact

- Affected capabilities:
  - `waypoints` — MODIFIED: existing "Multiple waypoints render simultaneously with per-waypoint visibility" requirement gains explicit toggle-and-restore and not-undoable scenarios.
  - `undo-redo` — MODIFIED: "Track style mutations bypass the command stack" requirement renamed to "Style and visibility mutations bypass the command stack" and extended to include `toggle_waypoint_visible`.
- Affected code (implementation, follow-up change):
  - `src-tauri/src/domain/waypoint.rs` — new field
  - `src-tauri/src/application/commands.rs` — no new `ProjectCommand` variant (mutation is non-undoable)
  - `src-tauri/src/commands/mod.rs` — new Tauri handler
  - `src/lib/types.ts` — sync `Waypoint` type
  - `src/components/WaypointsPanel.svelte` — visibility checkbox
  - `src/components/MapView.svelte` — filter hidden waypoints from render
- Project file (`.ozp`) format: backward-compatible additive field; missing `visible` deserializes as `true`.
