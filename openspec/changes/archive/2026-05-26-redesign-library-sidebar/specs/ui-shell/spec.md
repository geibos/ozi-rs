## MODIFIED Requirements

### Requirement: All in-app panels render through shadcn-svelte primitives and Tailwind utility classes that consume semantic tokens

The system SHALL render every in-app panel (`Console`, `ThemePicker`, `SymbolPicker`, `LibraryRail`, `MapView` wrapper) through shadcn-svelte primitives — `Card`, `ScrollArea`, `Button`, `Select`, `Popover`, `Tooltip`, `Slider`, `Switch`, `Label`, `Separator`, `Dialog`, `Table`, `Tabs`, `Sheet`, `DropdownMenu` — and SHALL style their static surfaces through Tailwind utility classes that read the semantic-token CSS variables (`bg-background`, `text-foreground`, `bg-popover`, `border-border`, `bg-card`, `bg-muted`, …).

Component-local `<style>` blocks SHALL be removed, except where a rule expresses a dynamic value sourced from the domain (e.g. a `TrackStyle.color` swatch or a waypoint symbol glyph). Such dynamic values SHALL be expressed as inline `style=` attributes, not as Tailwind classes.

`MapView.svelte` SHALL be migrated at the wrapper level only; MapLibre initialization, source/layer setup, drag handlers, click handlers, and the tile-protocol code SHALL NOT be modified by this requirement.

The `LibraryRail` Track-row color swatch and Waypoint-row symbol control SHALL retain their domain-driven inputs — a native `<input type="color">` for tracks (operating on the RGBA bytes in `TrackStyle.color`) and the existing `SymbolPicker` popover for waypoints — and SHALL NOT bind to `--ctp-*` palette variables or to semantic-token CSS variables.

Toast notifications SHALL be routed through `svelte-sonner` mounted in the root layout, rather than ad-hoc `alert()` or inline error surfaces.

The legacy floating-panel components — `Sidebar.svelte`, `TracksPanel.svelte`, `WaypointsPanel.svelte`, `TrackPointsPanel.svelte`, `SimplifyPanel.svelte` — SHALL NOT exist in the codebase. Their functionality has been relocated to the `LibraryRail` (Tracks and Waypoints tabs, with Simplify as an inline popover on the Track row's actions menu) and to the Inspector pane / Cmd-K palette delivered by a separate change.

#### Scenario: Panel surfaces follow the active flavour

- **WHEN** the user switches the active Catppuccin flavour while any panel is open
- **THEN** the panel's static surfaces (background, foreground text, borders, dividers, hover states) re-render through the new flavour's semantic-token values without any `<style>`-block override

#### Scenario: Track colour swatch is independent of the theme switch

- **WHEN** the user assigns a track a specific colour (e.g. `#ff8800`) via the native `<input type="color">` inside the `LibraryRail` Tracks tab, then switches the Catppuccin flavour
- **THEN** the track's colour swatch and its MapLibre rendering remain exactly `#ff8800` across all four flavours; no `--ctp-*` or semantic-token variable rebinds the swatch

#### Scenario: All four flavours pass the visual smoke pass

- **WHEN** the maintainer captures screenshots of `LibraryRail` (with the Tracks tab active and at least one track loaded) plus `MapView` across all four Catppuccin flavours via `ozi-rs-mcp`
- **THEN** each flavour renders without missing styles, without leftover hard-coded colours that bypass semantic tokens, and without regressions to MapLibre map content

#### Scenario: MapView wrapper migrates without touching MapLibre internals

- **WHEN** the maintainer reviews the `MapView.svelte` migration commit
- **THEN** the diff modifies only the outer wrapper container's classes (Tailwind utilities) and any Tailwind-replaced wrapper styles; MapLibre `new maplibregl.Map(...)` construction, source/layer setup, drag handlers, click handlers, and the tile-protocol code SHALL be byte-identical to before the commit

#### Scenario: Legacy floating-panel files are absent

- **WHEN** the maintainer inspects `src/components/` after this change lands
- **THEN** none of `Sidebar.svelte`, `TracksPanel.svelte`, `WaypointsPanel.svelte`, `TrackPointsPanel.svelte`, or `SimplifyPanel.svelte` are present AND no `import` statement anywhere in the codebase references any of them

## ADDED Requirements

### Requirement: The workspace `library-rail` slot is filled by a three-tab Library always-visible component

The `library-rail` slot of the workspace shell SHALL host a single component `src/components/LibraryRail.svelte` that renders three persistent tabs — Maps, Tracks, Waypoints — backed by the shadcn `Tabs` primitive. All three tab triggers SHALL be visible at all times whenever the workspace surface is active; the rail SHALL NOT collapse, accordion, or hide any of the three tabs. Exactly one tab's content panel SHALL be visible at a time.

The active tab SHALL be backed by a session-scoped Svelte store (`libraryActiveTab`) holding one of `'maps' | 'tracks' | 'waypoints'`. The store SHALL NOT be persisted to localStorage or to the Rust session file. Default value SHALL be `'maps'` on a fresh app launch.

#### Scenario: Library is mounted in the library-rail slot

- **WHEN** the user is at `/project` with an active map and inspects the workspace shell
- **THEN** the `library-rail` slot contains exactly one `<LibraryRail />` component, and `<LibraryRail />` renders three `Tabs.Trigger` elements labelled "Maps", "Tracks", "Waypoints"

#### Scenario: All three tab triggers are always visible

- **WHEN** the workspace is the active surface AND a project is open
- **THEN** the Maps, Tracks, and Waypoints tab triggers are all rendered and clickable, regardless of which tab is currently active, regardless of whether the project contains zero tracks or zero waypoints

#### Scenario: Active tab is session-scoped and not persisted across restarts

- **WHEN** the user opens the Tracks tab in one session AND restarts the app
- **THEN** the next session starts with the Maps tab active (default), not with the Tracks tab the previous session ended on; no localStorage or session-file entry records the choice

### Requirement: Library rows follow a unified visibility / color-or-symbol / name / actions pattern

Every row in the Tracks and Waypoints tabs SHALL be rendered through a shared component `src/components/library/LibraryRow.svelte` exposing four ordered cells:

1. **Visibility toggle** — a shadcn icon `Button` with the Lucide `Eye` or `EyeOff` icon reflecting the row's current visibility. Click SHALL toggle visibility through the existing API (`set_track_visibility` for tracks, the equivalent waypoint API for waypoints). The button SHALL carry an `aria-label` that describes the current state ("Hide track Foo" / "Show track Foo").
2. **Color swatch (tracks)** OR **Symbol button (waypoints)** — for tracks: a 16x16 circular swatch styled via inline `style="background-color: …"` reading the row's RGBA domain colour. Click SHALL open a shadcn `Popover` containing a native `<input type="color">`. For waypoints: a 16x16 button rendering the waypoint's current symbol glyph. Click SHALL open the existing `SymbolPicker` popover.
3. **Name (plus optional subline)** — the row's display name, truncated with `text-overflow: ellipsis`. The full name SHALL appear in a shadcn `Tooltip` on hover. Double-click SHALL switch the row into an inline-rename mode (`<input>` bound to a local writable; commit on Enter or blur, cancel on Esc). For Tracks rows only, a second line SHALL render in `text-xs text-muted-foreground font-mono tabular-nums` showing distance / duration / point-count separated by middle-dots (e.g. `12.4 km · 02:31 · 412 pts`).
4. **Actions menu** — a shadcn `DropdownMenu` triggered by a `⋯` icon button. The menu items SHALL be:
   - Tracks: Export GPX, Export PLT, Set line width, Simplify…, Delete.
   - Waypoints: Export WPT, Delete.

Each menu item SHALL invoke the same API function the legacy floating panel called for the equivalent operation; no operation SHALL be removed, only relocated.

Maps tab rows SHALL diverge from this pattern: no visibility toggle, no color swatch — a "cached" badge SHALL occupy the swatch position when the map's tiles are fully cached locally. The actions menu items SHALL be: Reveal in Finder, Switch to. The active map SHALL be visually highlighted (e.g. a `bg-accent` background on the row).

#### Scenario: Track row toggles visibility on the map

- **WHEN** the user clicks the visibility-toggle icon on a Track row that is currently visible
- **THEN** the row's icon switches from `Eye` to `EyeOff`, `set_track_visibility` is called with `visible=false`, and the track disappears from the MapView overlay without remounting the map

#### Scenario: Track row opens a color-picker popover

- **WHEN** the user clicks the color swatch on a Track row
- **THEN** a popover opens anchored to the swatch, containing a native `<input type="color">` initialised to the row's current colour; changing the value calls `set_track_color` with the new RGBA and the MapView re-renders the track in the new colour

#### Scenario: Track row renames inline via double-click

- **WHEN** the user double-clicks the name cell of a Track row AND types a new name AND presses Enter
- **THEN** the row enters rename mode, accepts text input, calls the rename API on Enter, and exits rename mode showing the new name; pressing Esc instead SHALL discard the edit and exit rename mode showing the original name

#### Scenario: Waypoint row symbol button opens the SymbolPicker

- **WHEN** the user clicks the symbol cell on a Waypoint row
- **THEN** the existing `SymbolPicker` popover opens anchored to the symbol cell; selecting a symbol writes through to the waypoint's domain symbol field and the MapView marker updates

#### Scenario: Map row marks the active map and offers Reveal-in-Finder

- **WHEN** the user inspects the Maps tab AND the project has an active map plus two non-active cached maps
- **THEN** the active map row has a visible highlight (e.g. `bg-accent`), the two non-active rows show a "cached" badge, and each row's `⋯` menu contains a "Reveal in Finder" item that opens the OS file browser at the map's path

### Requirement: Library tabs host the active-layer selectors in their headers

The Tracks tab's content panel SHALL render a shadcn `Select` at the top, above the row list, exposing the project's track layers and bound to the active-track-layer ID (`$activeTrackLayerId`). The Waypoints tab's content panel SHALL render the analogous shadcn `Select` bound to `$activeWaypointLayerId`. The Maps tab SHALL NOT contain a layer selector — its header SHALL contain only the "Maps…" button (see the workspace-bundle-loader-Sheet requirement).

The selectors SHALL change the active layer via the existing `set_active_track_layer` / `set_active_waypoint_layer` API calls. The selectors SHALL NOT modify, hide, or unload any layer; they SHALL only retarget where new edits land, consistent with the `layers` capability's non-destructive-selection invariant.

Clicking a row in the Tracks tab whose owning layer differs from the current active track layer SHALL trigger the same `set_active_track_layer` call before applying the row selection. The same SHALL hold for Waypoint rows and `set_active_waypoint_layer`. This row-activation behavior keeps the active-layer flag aligned with the row the user has just engaged with.

#### Scenario: Tracks tab header shows the active track layer selector

- **WHEN** the user opens the Tracks tab AND the project has two track layers "A" and "B" with "A" active
- **THEN** the top of the Tracks tab content shows a `Select` reading "A" as its current value, and opening the dropdown lists both "A" and "B"; choosing "B" calls `set_active_track_layer` with B's ID

#### Scenario: Clicking a row from a different layer switches the active layer

- **WHEN** layer "A" is the active track layer AND the user clicks a row in the Tracks tab whose owning layer is "B"
- **THEN** the system calls `set_active_track_layer` with B's ID before applying the row selection AND the Tracks tab's active-layer Select updates its trigger label to "B"

#### Scenario: Switching active layer keeps overlays visible

- **WHEN** the user changes the active track layer via the tab-header `Select` from "A" to "B"
- **THEN** both layers' track overlays remain rendered on the map; the change is purely a routing change for subsequent edits, consistent with the `layers` non-destructive-selection invariant

### Requirement: Project lifecycle and mode toggles do not live in the Library

The `LibraryRail` and its three tabs SHALL NOT render any of the following:

- Open project, Save project, Undo, Redo (project lifecycle).
- Create Track / Add Waypoint mode toggles, or any other mode toggle that mutates map-click interpretation.
- Console toggle, Theme picker, or other application chrome.

These affordances belong on the top context-bar and / or in the Cmd-K palette, which are owned by a separate change. The Library is for **objects** (maps, tracks, waypoints), not for project-lifecycle verbs, modes, or chrome.

Underlying state stores driving the modes (`drawingModeActive`, `addWaypointMode`, and their companions) SHALL NOT be modified by this change. Removing the buttons from the legacy `Sidebar` SHALL NOT change the stores' semantics; keyboard shortcuts wired to toggle these modes (where any exist) continue to work.

The four deprecated panel-open writable stores — `tracksPanelOpen`, `waypointsPanelOpen`, `trackPointsPanelOpen`, `simplifyPanelOpen` — SHALL be removed from `src/lib/stores.ts`. The Library tabs are always visible; these stores have no analogue in the new Library.

#### Scenario: Library tabs contain no project-lifecycle buttons

- **WHEN** the maintainer inspects every `Tabs.Content` panel in `LibraryRail.svelte` and its sub-components
- **THEN** none of them render a button labelled or icon-tagged Open, Save, Undo, or Redo; none of them render a Console toggle or a Theme picker

#### Scenario: Library tabs contain no mode-toggle buttons

- **WHEN** the maintainer inspects every `Tabs.Content` panel in `LibraryRail.svelte` and its sub-components
- **THEN** none of them render a "Create Track" toggle or an "Add Waypoint" toggle; these affordances exist only on the top context-bar (added by a separate change)

#### Scenario: Deprecated panel-open stores are removed

- **WHEN** the maintainer greps for `tracksPanelOpen`, `waypointsPanelOpen`, `trackPointsPanelOpen`, and `simplifyPanelOpen` across `src/`
- **THEN** zero matches are found; the stores have been removed from `src/lib/stores.ts` and no consumer remains

### Requirement: Simplify is invoked as an inline popover from the Track row, not as a floating panel

The Simplify operation SHALL be invoked from a Track row's `⋯` actions menu via a "Simplify…" item. Selecting the item SHALL open a shadcn `Popover` anchored to the row. The popover SHALL contain the algorithm selector, tolerance slider, preview button, commit button, and cancel button that the legacy `SimplifyPanel.svelte` exposed. The popover SHALL operate on the row's track ID — there SHALL be no ambiguity about "which track is being simplified".

The popover SHALL be dismissable via the Esc key and via click-outside, consistent with the shadcn `Popover` defaults.

`SimplifyPanel.svelte` SHALL NOT exist in the codebase after this change.

#### Scenario: Simplify opens anchored to a Track row

- **WHEN** the user opens the `⋯` menu on a Track row AND clicks "Simplify…"
- **THEN** a popover opens anchored to that row containing the algorithm selector, tolerance slider, preview / commit / cancel controls; the popover operates on the same track ID as the row

#### Scenario: Simplify popover dismisses via Esc

- **WHEN** the Simplify popover is open AND the user presses Esc
- **THEN** the popover closes without applying any simplification; the track's points are unchanged

#### Scenario: SimplifyPanel.svelte is absent

- **WHEN** the maintainer inspects `src/components/`
- **THEN** `SimplifyPanel.svelte` is not present and no `import` statement references it

### Requirement: TrackPointsPanel data helpers are preserved in a standalone module

The data-loading helpers that `TrackPointsPanel.svelte` exposed — functions that fetch track point arrays, format coordinates, format timestamps, compute per-segment speeds, and paginate the points list — SHALL be extracted to a new module `src/lib/track-points.ts` before `TrackPointsPanel.svelte` is deleted. The module SHALL be a pure-TypeScript utility: no Svelte runes, no store subscriptions, no DOM access.

`TrackPointsPanel.svelte` itself SHALL NOT exist in the codebase after this change. The track-points UI lives in the Inspector pane delivered by a separate change, which SHALL import from `src/lib/track-points.ts`.

#### Scenario: track-points module exists and exports the helpers

- **WHEN** the maintainer inspects `src/lib/track-points.ts`
- **THEN** the module exports the data-loading and formatting helpers previously co-located in `TrackPointsPanel.svelte`, with no Svelte-specific or DOM-specific code in the module

#### Scenario: TrackPointsPanel.svelte is absent

- **WHEN** the maintainer inspects `src/components/`
- **THEN** `TrackPointsPanel.svelte` is not present and no `import` statement references it
