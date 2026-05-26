## Why

The current 224px `Sidebar.svelte` mixes project lifecycle (Open / Save / Undo / Redo), map context, mode toggles, and object lists across five sections. Tracks and waypoints surface only as toggle buttons that open `TracksPanel.svelte`, `WaypointsPanel.svelte`, `TrackPointsPanel.svelte`, and `SimplifyPanel.svelte` as floating cards, which cover the map and require multiple clicks before the user can see what is in the project. The UX review locked in a Gaia GPS-style Library: three persistent tabs (Maps / Tracks / Waypoints), each a row-based list with a visibility toggle, color swatch, name, and per-row action menu. This change fills the `library-rail` slot defined by `redesign-shell-layout` with that Library and retires the floating panels. Project lifecycle and mode toggles move to the top context-bar and the Cmd-K palette in change 3.

## What Changes

- The `library-rail` slot of the workspace shell SHALL host a single `LibraryRail.svelte` component containing three persistent tabs — Maps, Tracks, Waypoints — backed by the shadcn `Tabs` primitive. The tabs SHALL always be visible; no accordion / collapsible variant SHALL be introduced.
- The Maps tab SHALL list bundles and maps from the active project (sourced from `$currentProject`), highlight the active map, mark cached vs streamable maps with a small badge, and expose per-row `⋯` actions (Reveal in Finder, Switch to). Its tab header SHALL host a "Maps…" button that opens the bundle-loader Sheet defined by `redesign-shell-layout` (no new sheet code; this change consumes the existing primitive).
- The Tracks tab SHALL list every track in the active project. Each row SHALL render: visibility eye toggle, color swatch (opens a popover color picker), name (double-click to rename), a metadata subline (distance / duration / point-count, `font-mono tabular-nums`), and a `⋯` menu (Export GPX, Export PLT, Set line width, Simplify…, Delete). The tab header SHALL host the active-track-layer selector that today lives in `Sidebar.svelte` lines 252-272. Selecting a row SHALL set `$selectedTrack` (drives MapView highlighting and the Inspector pane planned for change 3).
- The Waypoints tab SHALL list every waypoint in the active project. Each row SHALL render: visibility eye toggle (per-layer or per-waypoint as appropriate), symbol button opening the existing `SymbolPicker` popover, name (double-click to rename), and a `⋯` menu (Export WPT, Delete). The tab header SHALL host the active-waypoint-layer selector. Selecting a row SHALL set `$selectedWaypointId`.
- The Simplify affordance SHALL be relocated from `SimplifyPanel.svelte` (floating card) to an inline popover triggered by the Track row's `⋯` menu → "Simplify…" item. The popover SHALL operate on the row's track ID; no separate panel SHALL remain.
- The legacy floating panels — `TracksPanel.svelte`, `WaypointsPanel.svelte`, `TrackPointsPanel.svelte`, `SimplifyPanel.svelte` — and the legacy `Sidebar.svelte` SHALL be removed. The data-loading helpers used by `TrackPointsPanel.svelte` SHALL be extracted into `src/lib/track-points.ts` so that the Inspector pane (change 3) can import them without re-deriving the logic.
- Project lifecycle actions (Open project, Save project, Undo, Redo) SHALL leave the Library entirely. They are not represented in this change's UI; their new homes (top context-bar, Cmd-K palette) are owned by change 3.
- Mode toggles (Create Track, Add Waypoint) SHALL leave the Library entirely. Their state stores (`drawingModeActive`, `addWaypointMode`, related companions) are unchanged; the chips that activate them belong to the top context-bar in change 3. This change SHALL NOT add new chips in the Library.
- The deprecated `tracksPanelOpen` / `waypointsPanelOpen` / `trackPointsPanelOpen` / `simplifyPanelOpen` writable stores SHALL be removed from `src/lib/stores.ts`. Library tabs are always open; their open-state has no analogue.

## Capabilities

### New Capabilities
- _none_

### Modified Capabilities
- `ui-shell`: the legacy floating-panel requirement is replaced by a Library-tab requirement. The Library invariant (three tabs always present in the `library-rail` slot), the row pattern (visibility / color-or-symbol / name / `⋯`), the placement of layer selectors in tab headers, the relocation of Simplify into an inline popover, and the removal of the floating-panel surfaces are codified here.
- `layers`: a small clarification SHALL be added — clicking a Track row or a Waypoint row in the Library SHALL switch the active track / waypoint layer to the row's owning layer if it differs, so that subsequent edits and selection follow the row the user just engaged with. This is a non-destructive change consistent with the existing "Selecting an active layer is non-destructive" requirement.

## Impact

- **Frontend**: new `src/components/LibraryRail.svelte`, `src/components/library/MapsTab.svelte`, `src/components/library/TracksTab.svelte`, `src/components/library/WaypointsTab.svelte`, `src/components/library/LibraryRow.svelte`. New `src/lib/track-points.ts` parking the data-loading helpers from `TrackPointsPanel.svelte`. Removed: `src/components/Sidebar.svelte`, `src/components/TracksPanel.svelte`, `src/components/WaypointsPanel.svelte`, `src/components/TrackPointsPanel.svelte`, `src/components/SimplifyPanel.svelte`. `src/routes/project/+page.svelte` is updated to mount `<LibraryRail />` in the `library-rail` slot provided by change 1's `WorkspaceShell.svelte` instead of the old `<Sidebar />` mount site. `src/lib/stores.ts` loses the four panel-open booleans listed above; their usages disappear with the deleted panels.
- **Backend**: no change. No new IPC commands; the Library consumes the same `api.ts` surface (`set_track_color`, `set_track_visibility`, `simplify_track`, `export_track_*`, `set_active_track_layer`, `set_active_waypoint_layer`, etc.) the floating panels used.
- **Spec evidence**: smoke that all three tabs render with the seeded test project (one track, two waypoints, one map); that the visibility eye toggles the rendered overlay; that the color swatch popover writes through `set_track_color` and the map updates; that opening "Maps…" still produces the workspace Sheet (delegated to change 1's primitive); that the floating panels no longer exist as files. Manual verification follows `docs/agent-verification.md` (desktop, not Playwright).
- **Risk**: medium. Surface-area changes are mostly UI re-arrangement; risk concentrates on (a) state-store cleanup not breaking some forgotten consumer (mitigated by grep before delete, see task 7.1), (b) the active-layer-on-row-click behavior added to `layers` not surprising the user when rows belong to non-active layers (mitigated by the existing non-destructive invariant — overlays from other layers remain visible).

## Out of scope

- **Inspector pane content** — the right pane defined by change 1 is filled by change 3. This change exports the helpers the Inspector will need (`track-points.ts`) but does NOT render the Inspector's UI.
- **Cmd-K palette and top context-bar mode chips** — change 3 owns these. This change removes the old homes for project lifecycle and mode toggles, but does NOT add their new homes.
- **Elevation chart, route planning, MapView restyle** — explicitly deferred; future, separate changes.
- **Bundle-loader Sheet implementation** — owned by `redesign-shell-layout` (change 1). The Maps tab merely opens the Sheet via the already-defined `bundleLoaderOpen` store.
- **Per-map preview thumbnails** — see the same out-of-scope note in `bundle-loader-as-overlay`; not added by this change.
- **Catalog list at `/`** — the cold-start `/` route is the cold-start bundle loader; the Maps tab in the Library shows the active project's bundles, not the catalog. These are different surfaces.

## Dependencies

- **`redesign-shell-layout` MUST land first.** This change wires content into the `library-rail` slot that change 1 introduces in `WorkspaceShell.svelte`; without that slot there is no host for `LibraryRail.svelte`. The "Maps…" button in the Maps tab also delegates to the bundle-loader Sheet that change 1 brings (or that `bundle-loader-as-overlay` brought earlier and change 1 inherits) — if change 1 is not in, the Library has no way to open the loader without re-introducing a `goto('/')` regression.
- **`redesign-inspector-pane` (change 3) lands AFTER this change.** Change 3 mounts the Inspector pane and the Cmd-K palette, consumes `src/lib/track-points.ts` shipped here, and adds the mode chips to the top context-bar. If change 3 is delayed, project lifecycle actions and the track-points table are temporarily unreachable from the UI — this change MUST NOT ship to users in a release that does not also include change 3. The dependency is on shipping order, not on landing order; a feature flag or sequenced release is acceptable.
