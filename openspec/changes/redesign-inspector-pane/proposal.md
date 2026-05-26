## Why

The new 3-pane workspace shell (`redesign-shell-layout`) reserves a ~360px `inspector-rail` slot on the right edge but leaves it empty. The legacy floating panels — `TracksPanel`, `WaypointsPanel`, `TrackPointsPanel` — were retired by the Library sidebar work (`redesign-library-sidebar`), which means today there is no surface for editing per-track / per-waypoint / per-map properties from the new shell at all. Users can select a Library row but the row click has nothing to drive. Inline editing without a dialog was a locked design decision; the inspector rail is where it lands. The same redesign session also locked a global Cmd-K command palette as the cross-cutting "where do I jump to / what action do I run" surface: today's only way to switch maps, find a track, or run a project action is to mouse through the sidebar, which is what the palette replaces for keyboard-first usage.

## What Changes

- A new component `src/components/InspectorRail.svelte` SHALL occupy the `inspector-rail` slot of the workspace shell. It SHALL start collapsed (no body, only an edge affordance) when no Library row is selected and SHALL slide in from the right at 240ms when a selection arrives.
- Three context-sensitive subcomponents SHALL be created under `src/components/inspector/`: `TrackInspector.svelte`, `WaypointInspector.svelte`, `MapInspector.svelte`. The rail SHALL dispatch to one of them based on the active selection (`$selectedTrack`, `$selectedWaypointId`, or "map info" affordance from the Library Maps tab).
- The segments / points table currently parked from `TrackPointsPanel` (its logic was preserved by change 2 but its host panel was removed) SHALL be ported as a sub-view inside `TrackInspector`, sitting below the stats card and toggleable into edit mode. `$selectedPointId`, `$activeTrackLayerId`, and `$editModeActive` SHALL be the stores driving its state — no new stores are introduced.
- The Waypoint Inspector SHALL provide inline editing for name, symbol (via existing `SymbolPicker`), lat / lng readout, and visibility — no dialog. Edits SHALL be dispatched through the existing `ProjectCommand`-based CRUD endpoints in `src/lib/api.ts`.
- The Map Inspector SHALL display calibration metadata read-only (CRS, bounds, resolution from `getOziMetadata`) plus a "Reveal in Finder" action.
- A new global component `src/components/CommandPalette.svelte` SHALL be added, built on the shadcn-svelte `Command` primitive added by `redesign-shell-layout`. It SHALL be triggered globally by `⌘K` (macOS) / `Ctrl+K` (Windows/Linux) and SHALL also be reachable via a button in the top context-bar.
- The palette SHALL render a single search input plus grouped results: Open map, Switch project, Find track / waypoint, Project actions (Open / Save / Undo / Redo), Settings shortcuts (theme, units placeholder, GPS placeholder), Recent files.
- Each palette result row SHALL share the row pattern used by the Library sidebar — same icon column, label, optional meta line. The Enter key SHALL run the primary action. A secondary action SHALL be available via a keyboard chord (e.g. `⌘E` for export, `⌘R` for reveal) where the result type defines one.
- The "Recent files" group SHALL be sourced from a new lightweight localStorage helper keyed `ozi:recent-files:v1` (versioned key so future migrations are explicit). The helper SHALL track the last N (default 8) opened maps as `{ projectSlug, mapPath, openedAt }` records. It SHALL be appended to whenever `openSelectedMap` resolves successfully.
- The palette SHALL close on `Esc`, on click-outside, and after any primary action resolves. The current selection in the rail SHALL NOT be reset by a palette open/close — the rail's lifecycle is independent of the palette's.

## Capabilities

### New Capabilities

- _none_

### Modified Capabilities

- `ui-shell`: the empty `inspector-rail` slot gains an invariant (collapsed-by-default, slide-in on selection), the segments / points sub-view formally lands under Track Inspector, and a new global Cmd-K palette requirement is added with grouping + primary-secondary action semantics. The shadcn-svelte primitive list gains nothing new — `command` was added by change 1.

## Impact

- **Frontend**: new components `src/components/InspectorRail.svelte`, `src/components/inspector/TrackInspector.svelte`, `src/components/inspector/WaypointInspector.svelte`, `src/components/inspector/MapInspector.svelte`, `src/components/CommandPalette.svelte`. The old `TrackPointsPanel.svelte` logic moves into `TrackInspector` (or is wrapped as a sub-component re-exported from the inspector folder — chosen in design.md). A new helper `src/lib/recentFiles.ts` (small localStorage wrapper, keyed `ozi:recent-files:v1`). The workspace shell from change 1 fills its `inspector-rail` slot with `<InspectorRail />` and mounts `<CommandPalette />` once at the layout level for global `⌘K`.
- **Backend**: no change. No new IPC commands, no new Tauri events. The inspector reads through existing stores and `src/lib/api.ts` endpoints; the palette reads from the same.
- **Spec evidence**: smoke that selecting a track in the Library opens the Track Inspector with stats + segments table; that editing a waypoint name inline persists through `ProjectCommand`; that `⌘K` opens the palette from any focus state; that picking a map result from the palette opens the map and appends to `ozi:recent-files:v1`; that closing a project and reopening the app shows the recent file in the palette.
- **Risk**: low — the change is additive on the frontend only. Chief risk is double-binding of point-selection state if the segments table accidentally subscribes to a different store than the legacy `TrackPointsPanel` did; mitigated by reusing the exact same stores (`$selectedPointId`, `$activeTrackLayerId`) and porting the logic byte-equivalent.

## Out of scope

- **Elevation chart implementation**: the Track Inspector reserves a slot for an elevation chart but SHALL render only a placeholder ("Elevation chart — coming in a follow-up change"). The actual chart needs a charting library (decision deferred) and per-point elevation series exposure that may or may not already be in `TrackDetail` — a separate proposal will cover it.
- **New backend IPC commands**: the Inspector and the palette SHALL consume only existing stores and `api.ts` endpoints. No `get_map_preview`, no `get_track_elevations`, no new Tauri events as part of this change.
- **Route planning / new editing modes**: the Track Inspector's actions row contains only what already exists today (export GPX, export PLT, set line width, simplify, delete). No new editing primitives.
- **Map calibration editor UI**: the Map Inspector is read-only. A future calibration-editor change SHALL own write semantics for CRS / bounds / corner points.
- **Settings panel rewrite**: the palette exposes settings shortcuts (theme, units placeholder, GPS placeholder) but does NOT introduce a new settings surface; the existing `ThemePicker` flow remains the host for theme changes when the user picks the "Theme" palette entry.

## Dependencies

- **MUST land after `redesign-shell-layout`**: the `inspector-rail` slot, the shadcn `command` primitive, the motion-intensity tokens, and the rounded-`[1.5rem]` Inspector card class all come from that change. Without it, this change has nothing to mount into and no primitive to build the palette on.
- **MUST land after `redesign-library-sidebar`**: the selection model (the Library is what writes `$selectedTrack` / `$selectedWaypointId` and exposes the "Map info" affordance for the Map Inspector), the row pattern this change mirrors for palette results, and the parking of `TrackPointsPanel` logic all originate there. Building this change before the Library sidebar lands would leave the rail with no selection events to react to.
- **This change MUST be the last of the three redesign changes to land.** The full ordering is: `redesign-shell-layout` → `redesign-library-sidebar` → `redesign-inspector-pane`.
