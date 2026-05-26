## 1. Confirm prerequisites

- [ ] 1.1 Verify `redesign-shell-layout` has landed: the workspace shell exposes an `inspector-rail` slot ~360px wide, the shadcn-svelte `command` primitive is present under `src/lib/components/ui/command/`, the motion-intensity-6 tokens are wired, and the rounded-`[1.5rem]` Inspector card class is available
- [ ] 1.2 Verify `redesign-library-sidebar` has landed: the left rail Library is the source of `$selectedTrack`, `$selectedWaypointId`, and the "Map info" affordance for the Maps tab; the legacy `TracksPanel`, `WaypointsPanel`, and `TrackPointsPanel` have been removed from the workspace mount tree (their logic preserved for porting into this change)
- [ ] 1.3 Verify a shared `SelectableRow` component exists from change 2; if not, plan extraction work as part of task 6 before the palette result list is wired

## 2. Build `InspectorRail.svelte` empty / collapsed state and mount in workspace shell

- [ ] 2.1 Create `src/components/InspectorRail.svelte`. The component SHALL render only an edge affordance (thin pinnable handle on the right edge of the viewport) when no Library selection is active
- [ ] 2.2 Add an internal store for the pinned state (local to the component, not exported); the edge affordance click toggles it
- [ ] 2.3 Mount `<InspectorRail />` inside the workspace shell's `inspector-rail` slot (the slot defined by `redesign-shell-layout`). Verify the 3-pane shell geometry is unchanged when no selection is present
- [ ] 2.4 Wire the slide-in animation: when the rail transitions from collapsed → expanded, animate 240ms slide from the right edge plus fade-in on the body, using motion-intensity-6 tokens

## 3. Build `MapInspector.svelte` (read-only — easiest wire-up first)

- [ ] 3.1 Create `src/components/inspector/MapInspector.svelte`. The header SHALL show the active map name + project; the body SHALL render a calibration card (CRS, bounds, resolution) sourced via `getOziMetadata` from `src/lib/api.ts`
- [ ] 3.2 Add a "Reveal in Finder" action that uses existing Tauri shell primitives (no new IPC) to open the active map file's parent directory in the host OS file manager
- [ ] 3.3 Wire `InspectorRail` to render `<MapInspector />` when the Library Maps tab dispatches a "Map info" selection
- [ ] 3.4 Manual: select a map from the Library Maps tab — confirm the rail slides in 240ms, calibration metadata renders read-only, Reveal in Finder opens the host file manager to the correct directory

## 4. Build `WaypointInspector.svelte` with inline editing

- [ ] 4.1 Create `src/components/inspector/WaypointInspector.svelte`. The header SHALL contain an editable name input + the existing `SymbolPicker` (imported from its current location) + a visibility toggle; the body SHALL contain a location card (lat / lng readout + "Move on map" action) and an actions row (Export WPT, Delete)
- [ ] 4.2 Wire all edits through the existing `ProjectCommand`-shaped endpoints in `src/lib/api.ts` (`updateWaypoint`, `deleteWaypoint`, the existing waypoint-export endpoint). NO direct store mutation. NO new wrapper layer
- [ ] 4.3 Wire `InspectorRail` to render `<WaypointInspector />` when `$selectedWaypointId` is non-null
- [ ] 4.4 Manual: select a waypoint from the Library Waypoints tab — confirm the rail body shows the inline editor (no dialog mounts at any point); rename the waypoint, blur the input — confirm the new name persists across a project reload; change the symbol via the inline `SymbolPicker` — confirm the new symbol is reflected on the map

## 5. Build `TrackInspector.svelte` header + stats + actions (no segments table yet)

- [ ] 5.1 Create `src/components/inspector/TrackInspector.svelte`. The header SHALL show the track name (read-only display) + a colour swatch matching `TrackStyle.color` + a visibility toggle; the body SHALL contain a stats card (distance, duration, point count, start time — mono-spaced numerals on a rounded-`[1.5rem]` card) sourced via `getTrackDetail`, and an actions row (Export GPX, Export PLT, Set line width, Simplify, Delete)
- [ ] 5.2 Wire all actions through the existing endpoints in `src/lib/api.ts` (`exportTrackGpx`, `exportTrackPlt`, `updateTrackStyle`, `simplifyTrack`, `deleteTrack`)
- [ ] 5.3 Wire `InspectorRail` to render `<TrackInspector />` when `$selectedTrack` is non-null
- [ ] 5.4 Manual: select a track from the Library Tracks tab — confirm the rail body shows the header + stats card + actions row; Export GPX still works; Simplify still works

## 6. Port `TrackPointsPanel` logic into `TrackSegmentsTable.svelte`

- [ ] 6.1 Create `src/components/inspector/TrackSegmentsTable.svelte`. Port the segments / points table logic from the parked `TrackPointsPanel` verbatim where possible — reuse the same column structure, the same row-render path, and the same edit-mode UI
- [ ] 6.2 Subscribe the table to the exact same stores the legacy panel did: `$selectedPointId`, `$activeTrackLayerId`, `$editModeActive`. Do NOT introduce new selection / edit-mode stores
- [ ] 6.3 Verify bidirectional highlight: clicking a point on the map highlights the corresponding row in the table; clicking a row in the table highlights the corresponding point on the map and centres / scrolls it
- [ ] 6.4 Compose `<TrackSegmentsTable />` into `TrackInspector` immediately below the stats card and above the actions row
- [ ] 6.5 Manual: open Track Inspector — confirm the table shows all segments / points; toggle edit mode — confirm per-row edit affordances appear and the `$editModeActive` store flips; click a point on the map — confirm the corresponding row highlights; click a row in the table — confirm the corresponding point on the map highlights

## 7. Add the elevation-chart placeholder slot

- [ ] 7.1 Add a labeled empty container inside `TrackInspector`, positioned below the segments table and above the actions row, displaying the static text "Elevation chart — coming in a follow-up change"
- [ ] 7.2 Verify the placeholder makes NO IPC calls AND loads NO charting library; it is a styled empty container only
- [ ] 7.3 Manual: switch between tracks in the Library — confirm the placeholder does not reflow neighbouring sections and that no console warnings are emitted

## 8. Implement the recent-files localStorage helper

- [ ] 8.1 Create `src/lib/recentFiles.ts`. Export at minimum: `getRecentFiles(): RecentFile[]`, `appendRecentFile(record: RecentFile): void`, and a module constant `MAX_RECENT_FILES = 8`. The localStorage key SHALL be `ozi:recent-files:v1`
- [ ] 8.2 Implement deduplication by `mapPath`: an existing record with the same path SHALL be moved to the front rather than appended a second time
- [ ] 8.3 Implement failure handling: on quota-exceeded, drop the oldest record and retry once; on unparseable stored JSON, treat as empty list and overwrite on next write; emit `console.warn` on either path; no user-facing error
- [ ] 8.4 Add a single call site at the existing `openSelectedMap` success branch (one line: `appendRecentFile({ projectSlug, mapPath, mapName, openedAt: Date.now() })`)
- [ ] 8.5 Manual: open a map — confirm `ozi:recent-files:v1` updates in localStorage; open a 9th distinct map — confirm the entry caps at 8 records and the oldest is dropped; corrupt the localStorage value to a non-JSON string and reload — confirm the palette renders an empty Recent files group, a `console.warn` fires, and the next map open overwrites the entry cleanly

## 9. Build `CommandPalette.svelte` on the shadcn `Command` primitive

- [ ] 9.1 Create `src/components/CommandPalette.svelte`. Mount one instance in `src/routes/+layout.svelte`, sibling to `MapView`. Back its open state with a new writable store `commandPaletteOpen` in `src/lib/stores.ts`
- [ ] 9.2 Register the `⌘K` / `Ctrl+K` global key handler at the layout level via `window.addEventListener('keydown', ...)`. The handler SHALL call `event.preventDefault()` on match AND SHALL fire from any focus state in the app except inside the palette itself
- [ ] 9.3 Render a single search input at the top (auto-focused on open) and a grouped results list below. Group order: Open map → Switch project → Find track / waypoint → Project actions → Settings → Recent files. Each group header SHALL be a small uppercase label with `text-muted-foreground/60`
- [ ] 9.4 Implement search filtering: each group filters its results against the current input via prefix-match → substring-match → fuzzy-fallback (choose Fuse.js or a similar small library; document the choice in the PR). Empty groups SHALL be hidden entirely
- [ ] 9.5 Build the row pattern. If change 2 extracted a shared `SelectableRow` component, import and reuse it. Otherwise extract it from the Library row markup as part of this task and replace both call sites. The palette row SHALL additionally render a keyboard-shortcut hint slot on the far right (e.g. `⌘E`) when the result has a secondary action
- [ ] 9.6 Wire group data sources:
  - Open map: maps in the active project (existing project / map stores)
  - Switch project: LizaAlert catalog cache (`cache-project-catalog-locally`)
  - Find track / waypoint: tracks + waypoints in the active project, merged
  - Project actions: Open, Save, Undo, Redo (existing project-action endpoints)
  - Settings: theme (opens `ThemePicker`), units (placeholder no-op + toast "Coming soon"), GPS (placeholder no-op + toast "Coming soon")
  - Recent files: `getRecentFiles()` from `src/lib/recentFiles.ts`
- [ ] 9.7 Implement context adaptation: at the cold-start surface (no active project), hide the Open map, Find track / waypoint, and Project actions groups. Render only Switch project, Settings, and Recent files
- [ ] 9.8 Add the top-context-bar button that opens the palette (the same button visual the shell from change 1 reserved a slot for, if any; otherwise add a small `⌘K` icon button on the right of the context bar)

## 10. Wire palette primary and secondary actions

- [ ] 10.1 Primary action on `Enter`: the highlighted result runs its primary action and the palette closes
  - Open map → opens the map (same call site as the bundle loader)
  - Switch project → opens the bundle-loader Sheet (from `bundle-loader-as-overlay`) pre-scrolled to the chosen project
  - Find track / waypoint → focuses the matching row in the Library AND populates the corresponding selection store so the Inspector opens
  - Project actions → dispatches the corresponding action through `ProjectCommand`
  - Settings → opens the theme picker (or placeholder toast for units / GPS)
  - Recent files → opens the chosen map (same path as Open map)
- [ ] 10.2 Secondary action `⌘E`: when a track or waypoint result is highlighted, run the corresponding export action through the existing `api.ts` endpoint and close the palette
- [ ] 10.3 Secondary action `⌘R`: when a map result is highlighted, run "Reveal in Finder" via existing Tauri shell primitives and close the palette
- [ ] 10.4 Wire close behaviours: `Esc` (focus returns to the previously-focused element), click-outside, and on any successful primary or secondary action
- [ ] 10.5 Wire keyboard navigation: arrow keys move the highlight; `Tab` jumps to the first result of the next group; the highlight defaults to the first visible result whenever the input changes

## 11. Verification

- [ ] 11.1 Manual: open a project, select a track in the Library — confirm the rail slides in over 240ms, the Track Inspector renders header + stats + segments table + elevation placeholder + actions row in that order
- [ ] 11.2 Manual: select a waypoint in the Library — confirm the rail swaps to Waypoint Inspector via spring transition without re-running the slide-in animation; rename the waypoint inline, blur the input — confirm persistence via `ProjectCommand` (open another project and back, name is still updated)
- [ ] 11.3 Manual: select a map (Map info affordance on the Library Maps tab) — confirm the Map Inspector renders calibration data read-only; click Reveal in Finder — confirm the host file manager opens to the correct directory
- [ ] 11.4 Manual: clear the Library selection (click empty space) — confirm the rail collapses to the edge affordance only; click the pin affordance, then clear selection — confirm the rail stays expanded with an empty body
- [ ] 11.5 Manual: from anywhere in the workspace, press `⌘K` (or `Ctrl+K`) — confirm the palette opens with the search input focused; type a partial map name — confirm the Open map group filters to matching maps and empty groups disappear
- [ ] 11.6 Manual: highlight a track result in the palette, press `⌘E` — confirm the track is exported and the palette closes; highlight a map result, press `⌘R` — confirm Reveal in Finder runs and the palette closes
- [ ] 11.7 Manual: press `⌘K`, then `Esc` — confirm the palette closes and focus returns to the previously-focused element
- [ ] 11.8 Manual: open a map, then press `⌘K` and inspect the Recent files group — confirm the just-opened map appears at the top; open it 8 more times via different distinct maps — confirm cap at 8 records is enforced in localStorage
- [ ] 11.9 Manual: at cold start (no active project), press `⌘K` — confirm only Switch project, Settings, and Recent files groups render
- [ ] 11.10 Manual: verify all four Catppuccin flavours render the rail + the palette without missing styles, without hard-coded colours bypassing semantic tokens, and without console errors
- [ ] 11.11 `just ci` passes (clippy, check, lint, test)
- [ ] 11.12 `openspec validate redesign-inspector-pane --strict` passes
