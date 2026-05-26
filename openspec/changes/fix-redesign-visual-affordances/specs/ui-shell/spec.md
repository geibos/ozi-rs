## MODIFIED Requirements

### Requirement: The workspace `library-rail` slot is filled by a three-tab Library always-visible component

The `library-rail` slot of the workspace shell SHALL host a single component `src/components/LibraryRail.svelte` that renders three persistent tabs — Maps, Tracks, Waypoints — backed by the shadcn `Tabs` primitive. All three tab triggers SHALL be visible at all times whenever the workspace surface is active; the rail SHALL NOT collapse, accordion, or hide any of the three tabs. Exactly one tab's content panel SHALL be visible at a time.

The active tab SHALL be backed by a session-scoped Svelte store (`libraryActiveTab`) holding one of `'maps' | 'tracks' | 'waypoints'`. The store SHALL NOT be persisted to localStorage or to the Rust session file. Default value SHALL be `'maps'` on a fresh app launch.

The active tab trigger SHALL render with a visible active state distinct from the inactive triggers in both light and dark themes: a swap to the accent background token (`hsl(var(--accent))` background, `hsl(var(--accent-foreground))` text) AND a 2px under-border drawn in `hsl(var(--ring))` (the Teal accent) via inset box-shadow so the indicator does not affect layout. Inactive triggers SHALL render with `text-muted-foreground` and a transparent background. The active state SHALL be driven by the rendered `data-state="active"` attribute on the trigger element; no JavaScript polling SHALL be required to apply it.

#### Scenario: Library is mounted in the library-rail slot

- **WHEN** the user is at `/project` with an active map and inspects the workspace shell
- **THEN** the `library-rail` slot contains exactly one `<LibraryRail />` component, and `<LibraryRail />` renders three `Tabs.Trigger` elements labelled "Maps", "Tracks", "Waypoints"

#### Scenario: All three tab triggers are always visible

- **WHEN** the workspace is the active surface AND a project is open
- **THEN** the Maps, Tracks, and Waypoints tab triggers are all rendered and clickable, regardless of which tab is currently active, regardless of whether the project contains zero tracks or zero waypoints

#### Scenario: Active tab is session-scoped and not persisted across restarts

- **WHEN** the user opens the Tracks tab in one session AND restarts the app
- **THEN** the next session starts with the Maps tab active (default), not with the Tracks tab the previous session ended on; no localStorage or session-file entry records the choice

#### Scenario: Active tab is visually distinct from inactive tabs

- **WHEN** the workspace renders with any of the three tabs active
- **THEN** the active trigger has a non-transparent background sourced from `hsl(var(--accent))`, a foreground colour sourced from `hsl(var(--accent-foreground))`, and a 2px Teal under-border drawn via `hsl(var(--ring))`; the other two triggers have a transparent background and `text-muted-foreground` text colour AND no under-border

#### Scenario: Active-tab styling survives a theme switch

- **WHEN** the user toggles between the auto / light / dark Zinc themes
- **THEN** the active Library tab remains visibly distinct from the inactive tabs in every theme; the contrast is achieved through tokens (`--accent`, `--accent-foreground`, `--ring`) not through hard-coded colours

### Requirement: Workspace renders through a 3-pane shell with named slots

The project workspace at `/project` SHALL render through a single shell component (`src/components/WorkspaceShell.svelte`) that lays out a 3-pane structure with three named regions:

1. **`library-rail`** — a left rail with a fixed width of 280px, hosting the Library three-tab component (see the library-rail requirement above).
2. **`canvas`** — a center region of flexible width that hosts `MapView`. `MapView` SHALL continue to be mounted from the root layout (`src/routes/+layout.svelte`); the canvas slot SHALL be the visual region the layout's `MapView` becomes visible inside while the workspace route is active.
3. **`inspector-rail`** — a right rail mounted unconditionally for the lifetime of the workspace surface. The rail SHALL render at a fixed width of 360px when expanded (selection present OR pinned) and SHALL collapse to an 8px-wide edge-handle strip otherwise. The canvas SHALL claim the freed horizontal space when the rail is collapsed without shifting the library rail.

Above the canvas the shell SHALL render a thin top context-bar containing four mode-chip placeholders labelled View / Draw / Edit / Measure and a single Cmd-K trigger button. Mode chips SHALL be rendered as a visually-distinct cluster from the Cmd-K trigger, separated by a 1px vertical divider in `var(--inner-border)`. The mode chips SHALL render with `aria-disabled="true"`, `tabindex="-1"`, `cursor: not-allowed`, and reduced opacity (≈0.55), so they read as inert scaffolding rather than as peer affordances of the Library tabs. The mode chips SHALL NOT visually resemble the Library `Tabs.Trigger` elements: the chips have no active-state styling, no under-border, and a disabled appearance.

The Cmd-K trigger SHALL render as a button-shaped pill that cannot be visually mistaken for an `<input>`: no `--border`-token border in the default state, no fixed `min-width` mimicking an input field, focus styling via a Teal ring (`hsl(var(--ring) / 0.45)` 2px outline) rather than an input-style outline, hover background sourced from `hsl(var(--secondary))`, and the `⌘K` kbd glyph rendered as a pill on the right side. The trigger SHALL be a `<button>` element with `aria-label="Open command palette"`, SHALL NOT carry `role="search"`, and SHALL NOT use `<input>` markup.

Below the canvas the shell SHALL render a status bar of stable height (`--status-bar-height`).

The previous workspace layout (the 2-column `Sidebar.svelte` + 3 floating panels above `MapView` at `src/routes/project/+page.svelte`) SHALL be replaced by the shell.

#### Scenario: Workspace renders the 3-pane shell with mounted Inspector rail

- **WHEN** the user navigates to `/project` with an active map AND no Library row is selected AND the Inspector is not pinned
- **THEN** the workspace renders a 280px left rail with the Library, the centre canvas with `MapView`, the top context-bar with four mode chips and the Cmd-K trigger, the bottom status bar, AND an 8px-wide inspector edge handle on the right edge of the workspace; the inspector aside is present in the DOM even when its body is empty

#### Scenario: Inspector expands to 360px when a selection arrives

- **WHEN** the user clicks a Library row of any of the three supported types (Track, Waypoint, Map)
- **THEN** the inspector rail expands from 8px to 360px AND the canvas reflows to leave room for it without shifting the library rail

#### Scenario: Mode chips are visually distinct from Library tabs

- **WHEN** the user inspects the top context-bar
- **THEN** the four mode chips render with reduced opacity (≈0.55), `cursor: not-allowed`, no hover-state change, and `aria-disabled="true"` AND they are grouped together AND a 1px vertical divider (`var(--inner-border)`) separates the chip cluster from the Cmd-K trigger; no chip carries any active-state styling resembling the Library tabs' active state

#### Scenario: Mode chips do not respond to clicks

- **WHEN** the user clicks any mode chip
- **THEN** no application state changes AND focus does not move to the chip AND no console error is emitted (the chips have `tabindex="-1"` and `aria-disabled="true"`)

#### Scenario: Cmd-K trigger reads as a button, not an input

- **WHEN** the user inspects the Cmd-K trigger in the workspace top context-bar
- **THEN** the trigger is rendered as a `<button>` element AND does NOT carry `role="search"` AND does NOT use any `<input>` markup AND its default state has no border resembling the shadcn `Input` component (`border-transparent`, not `border-input`) AND its focus state renders as a 2px Teal ring rather than an input-style outline AND it does not enforce a min-width that mimics an input field's intrinsic width

#### Scenario: Cmd-K trigger opens the palette on click

- **WHEN** the user clicks the Cmd-K trigger button
- **THEN** the command palette opens (the `commandPaletteOpen` store flips to `true`); the trigger does not accept text input AND no caret appears inside it

#### Scenario: MapView remains mounted once across the redesign

- **WHEN** the user navigates between `/` and `/project` after this change is implemented
- **THEN** the same `MapView` instance is reused (the mount-once invariant from the prior `ui-shell` requirement is preserved); the shell component does NOT remount `MapView` on slot changes or on inspector expand / collapse

### Requirement: The workspace `inspector-rail` slot hosts a context-sensitive Inspector that is collapsed by default and slides in on Library selection

The right-side `inspector-rail` slot of the workspace shell SHALL be filled by a single component `src/components/InspectorRail.svelte`. The rail SHALL be mounted unconditionally for the lifetime of the workspace surface; only its body content SHALL appear and disappear in response to selection.

In its default state (no Library selection AND not pinned), the rail SHALL render an edge affordance — a thin vertical handle on the right edge of the viewport — and SHALL occupy 8px of horizontal width inside the workspace shell. The handle SHALL be at minimum 8px wide in its idle state and SHALL widen to ≈12px on hover, with `cursor: ew-resize` (or equivalent affordance cursor) to signal that it is the manual open control. The handle SHALL host a pin-toggle button (the same pin affordance the spec already contracts for) reachable in both the collapsed and expanded states.

When the Library selection store transitions from null to a non-null value of any of the three supported types (Track, Waypoint, Map), the rail SHALL slide in from the right edge over 240ms, fade its body content in, and render the appropriate Inspector subcomponent for the selected type. The slide-in animation SHALL respect the motion-intensity-6 tokens established by `redesign-shell-layout`.

When the selection transitions between two non-null values (regardless of whether the types match), the rail SHALL remain open and SHALL swap its body subcomponent via the spring transition (stiffness 100, damping 20) established by `redesign-shell-layout`; the rail itself SHALL NOT re-run its slide-in animation on such transitions.

When the selection transitions back to null AND the rail is not pinned, the rail SHALL slide its body out and return to the collapsed default state (8px edge handle, no body). When the rail is pinned (the user has clicked the pin affordance on the edge handle), the rail SHALL remain open with an empty body in this case.

The rail's fixed expanded width SHALL be ~360px, matching the slot dimension reserved by `redesign-shell-layout`. It SHALL NOT be user-resizable in v1.

The `inspectorOpen` writable store SHALL reflect the expanded / collapsed state (`true` = expanded, `false` = collapsed) so that the `WorkspaceShell` canvas-inset writer (`writeCanvasInsets`) can set `--canvas-right` to `360px` (expanded) or `8px` (collapsed) without shifting the library rail.

#### Scenario: Cold workspace — Inspector renders the edge handle in the collapsed state

- **WHEN** the user opens a project workspace AND no Library row is selected AND the rail is not pinned
- **THEN** the `inspector-rail` slot is present in the DOM AND renders an 8px-wide edge handle on the right edge of the viewport AND the `MapView` extends to the workspace's right edge minus that 8px strip AND no Inspector body subcomponent is mounted

#### Scenario: User pins the rail open from the edge handle

- **WHEN** the user clicks the pin button on the edge handle in the collapsed state
- **THEN** the rail expands to 360px AND `$inspectorOpen` becomes `true` AND the rail body renders the empty-state placeholder (no selection yet) AND the rail remains expanded after subsequent selection clears

#### Scenario: Selecting a Track row opens Track Inspector

- **WHEN** the user clicks a Track row in the Library AND `$selectedTrack` transitions from null to a track
- **THEN** the rail slides in over 240ms from the right edge, the Track Inspector subcomponent mounts inside the rail, and the rail's body fades in

#### Scenario: Switching from a Track to a Waypoint swaps Inspector body without re-animating the rail

- **WHEN** the rail is open showing a Track Inspector AND the user clicks a Waypoint row in the Library
- **THEN** the rail itself remains expanded with no slide animation re-fire; the rail's body subcomponent swaps from Track Inspector to Waypoint Inspector via the spring transition

#### Scenario: Clearing selection collapses the rail back to the edge handle

- **WHEN** the rail is open AND the Library selection transitions back to null (no row highlighted) AND the rail is not pinned
- **THEN** the rail slides its body out and returns to the collapsed default state with only the 8px edge handle visible; the canvas reclaims the freed horizontal space without shifting the library rail

#### Scenario: Pinning the rail keeps it open across selection changes

- **WHEN** the user clicks the pin affordance on the rail's edge handle AND subsequently clears the Library selection
- **THEN** the rail remains expanded at full width with its body rendering an empty state, ready for the next selection to populate it without re-running the slide-in animation

### Requirement: Library rows follow a unified visibility / color-or-symbol / name / actions pattern

Every row in the Tracks and Waypoints tabs SHALL be rendered through a shared component `src/components/library/LibraryRow.svelte` exposing four ordered cells:

1. **Visibility toggle** — a shadcn icon `Button` with the Lucide `Eye` or `EyeOff` icon reflecting the row's current visibility. Click SHALL toggle visibility through the existing API (`set_track_visibility` for tracks, the equivalent waypoint API for waypoints). The button SHALL carry an `aria-label` that describes the current state ("Hide track Foo" / "Show track Foo").
2. **Color swatch (tracks)** OR **Symbol button (waypoints)** — for tracks: a 16x16 circular swatch styled via inline `style="background-color: …"` reading the row's RGBA domain colour. Click SHALL open a shadcn `Popover` containing a native `<input type="color">`. For waypoints: a 16x16 button rendering the waypoint's current symbol glyph. Click SHALL open the existing `SymbolPicker` popover.
3. **Name (plus optional subline)** — the row's display name. The name cell SHALL truncate gracefully when the available horizontal space is smaller than the name's intrinsic width: the name span SHALL apply `white-space: nowrap`, `overflow: hidden`, and `text-overflow: ellipsis`, AND the parent flex item containing the span SHALL set `min-width: 0` so flex shrinkage actually takes effect. The full untruncated name SHALL be set on the `title` attribute of the name span so the browser renders a native tooltip on hover. A shadcn `Tooltip` MAY additionally wrap the name span for typographic consistency; the `title` attribute is the always-present fallback. Double-click SHALL switch the row into an inline-rename mode (`<input>` bound to a local writable; commit on Enter or blur, cancel on Esc). For Tracks rows only, a second line SHALL render in `text-xs text-muted-foreground font-mono tabular-nums` showing distance / duration / point-count separated by middle-dots (e.g. `12.4 km · 02:31 · 412 pts`). The second line SHALL also truncate via the same `text-overflow: ellipsis` rule.
4. **Actions menu** — a shadcn `DropdownMenu` triggered by a `⋯` icon button. The menu items SHALL be:
   - Tracks: Export GPX, Export PLT, Set line width, Simplify…, Delete.
   - Waypoints: Export WPT, Delete.

Each menu item SHALL invoke the same API function the legacy floating panel called for the equivalent operation; no operation SHALL be removed, only relocated.

Maps tab rows SHALL diverge from this pattern: no visibility toggle, no color swatch — a "cached" badge SHALL occupy the swatch position when the map's tiles are fully cached locally. The actions menu items SHALL be: Reveal in Finder, Switch to. The active map SHALL be visually highlighted (e.g. a `bg-accent` background on the row).

#### Scenario: Long name truncates with ellipsis and surfaces full text on hover

- **WHEN** a Track row with a long display name ("Спасатель — Поисково-спасательный отряд Лиза Алерт. Очень длинное название") is rendered inside the 280px Library rail
- **THEN** the visible name text is truncated with a trailing ellipsis (`…`), the row's vertical height remains the single-line row height (no wrap), AND hovering the name span causes the browser to display a native tooltip with the full untruncated text

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

The Tracks tab's content panel SHALL render a shadcn `Select` at the top, above the row list, exposing the project's track layers and bound to the active-track-layer ID (`$activeTrackLayerId`). The Waypoints tab's content panel SHALL render the analogous shadcn `Select` bound to `$activeWaypointLayerId`.

The Maps tab SHALL NOT contain a layer selector. The Maps tab SHALL ALSO NOT contain a header button that opens the bundle-loader Sheet. The Maps tab body alone is the in-project map-switching affordance; the bundle-loader Sheet (for loading new bundles or switching projects) is reached through the Cmd-K command palette (the Switch project group writes to `bundleLoaderOpen`) and through the `/` cold-start route. The Maps-tab-header SHALL be empty (or absent) in this change.

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

#### Scenario: Maps-tab header has no bundle-loader Sheet trigger

- **WHEN** the user opens the Maps tab in the Library
- **THEN** no header button labelled "Maps…" or an equivalent bundle-loader Sheet trigger renders above the maps list AND the tab body renders only the active project's maps with the active-map highlight and the `cached` badge AND no static scan of `MapsTab.svelte` matches an import of `bundleLoaderOpen` from `$lib/stores`

#### Scenario: Bundle loader remains reachable from Cmd-K

- **WHEN** the user opens the command palette via `⌘K` AND highlights any entry in the Switch project group AND presses Enter
- **THEN** the bundle-loader Sheet opens (the `bundleLoaderOpen` store flips to `true`); the absence of the Maps-tab-header button does not remove the bundle-loader entry overall
