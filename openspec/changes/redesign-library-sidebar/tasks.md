## 1. Confirm prerequisites

- [ ] 1.1 Confirm `redesign-shell-layout` has landed (or is the immediate parent of this branch) and the `WorkspaceShell.svelte` it introduces exposes a `library-rail` slot at ~280px on the left. If the shell change has not landed, STOP — this change has no host to mount into.
- [ ] 1.2 Confirm the `bundleLoaderOpen` writable in `src/lib/stores.ts` still exists and is consumed by a `Sheet.Root` host (delivered by `bundle-loader-as-overlay` and inherited by `redesign-shell-layout`). The Maps tab will only set this flag; it will not own the Sheet.
- [ ] 1.3 Add the shadcn `DropdownMenu` primitive if it is not yet present: `npx shadcn-svelte@latest add dropdown-menu`. Verify the generated component imports `cn` from `$lib/utils` and uses semantic-token Tailwind classes.

## 2. Build `LibraryRow.svelte`

- [ ] 2.1 Create `src/components/library/LibraryRow.svelte` implementing the four-cell row anatomy from design.md Decision 2: visibility-toggle, color-or-symbol, name (+ optional subline), actions menu.
- [ ] 2.2 Parameterise the swatch / symbol cell via a slot (`leadingControl`) so Tracks pass a color swatch and Waypoints pass a symbol button.
- [ ] 2.3 Parameterise the actions menu via a slot (`actions`) so each tab can supply its own `DropdownMenu.Item`s.
- [ ] 2.4 Implement inline-rename mode on the name cell: local `editing` writable, `<input>` rendered when `editing === true`, commit on Enter / blur, cancel on Esc.
- [ ] 2.5 Wire visibility toggle to a prop-supplied `onToggleVisibility` callback; do NOT call any API directly from `LibraryRow` — the caller (tab component) is responsible for the API call.
- [ ] 2.6 Style row surfaces with Tailwind utility classes consuming semantic tokens; the color swatch's `background-color` SHALL be an inline `style=` value sourced from the row's domain RGBA.

## 3. Build `MapsTab.svelte`

- [ ] 3.1 Create `src/components/library/MapsTab.svelte`. The tab header contains a single shadcn `Button` labelled "Maps…" that calls `bundleLoaderOpen.set(true)`.
- [ ] 3.2 Render the list of bundles + maps sourced from `$currentProject`. The active map (`$activeMap`) gets a highlight (`bg-accent` on the row).
- [ ] 3.3 Map rows render WITHOUT a visibility toggle. The leading-control slot is occupied by a "cached" badge if the map's tiles are fully cached locally; otherwise the slot is empty.
- [ ] 3.4 Map row actions menu items: "Reveal in Finder" (calls the existing reveal API), "Switch to" (calls the existing switch-active-map API). Port the implementations from `Sidebar.svelte:207-235`.
- [ ] 3.5 Add a manual smoke check: open a project with one active map and one non-active cached map, confirm both rows render with correct highlight and badge, confirm clicking "Maps…" opens the bundle-loader Sheet.

## 4. Build `TracksTab.svelte`

- [ ] 4.1 Create `src/components/library/TracksTab.svelte`. The tab header contains a shadcn `Select` bound to `$activeTrackLayerId` — port markup and binding logic from `Sidebar.svelte:252-272`.
- [ ] 4.2 Below the selector, render a scrollable list of `LibraryRow` instances — one per track in the active project (across all track layers). Sort: by layer, then by track index within layer.
- [ ] 4.3 For each row, wire the visibility toggle to call `set_track_visibility` with the new state.
- [ ] 4.4 For each row, wire the color swatch slot to render a 16x16 inline-styled circle that, on click, opens a shadcn `Popover` containing a native `<input type="color">` bound through `set_track_color`. Port the picker logic from `TracksPanel.svelte`.
- [ ] 4.5 For each row, render the name + subline. The subline format is `<distance> · <duration> · <pointcount> pts` with `font-mono tabular-nums`; reuse the existing distance / duration formatting helpers from `TracksPanel.svelte` (or extract them into `src/lib/track-format.ts` if they are not already there).
- [ ] 4.6 For each row, wire the actions menu items: Export GPX, Export PLT, Set line width (opens a slider popover), Simplify… (Decision 6 — see task 4.8), Delete. Each item calls the same API the legacy panel called.
- [ ] 4.7 On row click: call `set_active_track_layer` if the row's owning layer is not the current active track layer, then set `$selectedTrack` to the row's track ID.
- [ ] 4.8 Implement the Simplify popover: when the "Simplify…" menu item is activated, open a shadcn `Popover` anchored to the row containing the algorithm selector, tolerance slider, preview / commit / cancel controls. Port the content from `SimplifyPanel.svelte`. Operate on the row's track ID — no global "active simplify target" store needed.

## 5. Build `WaypointsTab.svelte`

- [ ] 5.1 Create `src/components/library/WaypointsTab.svelte`. The tab header contains a shadcn `Select` bound to `$activeWaypointLayerId` — port markup and binding logic from `Sidebar.svelte` (the waypoints section's selector).
- [ ] 5.2 Render a scrollable list of `LibraryRow` instances — one per waypoint in the active project (across all waypoint layers). Sort: by layer, then by waypoint index within layer.
- [ ] 5.3 For each row, wire the visibility toggle to call the waypoint visibility API. Confirm whether visibility is per-waypoint or per-layer in the current codebase and document the choice in the row's `aria-label`.
- [ ] 5.4 For each row, wire the symbol-button slot to render the waypoint's current symbol glyph at 16x16 and, on click, open the existing `SymbolPicker` popover. Reuse `SymbolPicker.svelte` as-is.
- [ ] 5.5 For each row, render the waypoint name; no subline for waypoints in v1.
- [ ] 5.6 For each row, wire the actions menu items: Export WPT, Delete. Each calls the same API the legacy panel called.
- [ ] 5.7 On row click: call `set_active_waypoint_layer` if the row's owning layer is not the current active waypoint layer, then set `$selectedWaypointId` to the row's waypoint ID.

## 6. Build `LibraryRail.svelte`

- [ ] 6.1 Create `src/components/LibraryRail.svelte`. Use the shadcn `Tabs` primitive with `Tabs.List` carrying three `Tabs.Trigger`s ("Maps", "Tracks", "Waypoints") and three `Tabs.Content` panels mounting `MapsTab`, `TracksTab`, `WaypointsTab` respectively.
- [ ] 6.2 Add a `libraryActiveTab` writable to `src/lib/stores.ts` (`writable<'maps' | 'tracks' | 'waypoints'>('maps')`). Bind `Tabs.Root`'s value to this store. DO NOT persist it to localStorage or the session file.
- [ ] 6.3 Style the rail to fill the `library-rail` slot — `flex flex-col h-full` outer container, `Tabs.List` at the top, `Tabs.Content` flex-grow with `overflow-y-auto` so each tab's row list scrolls independently of the tab list.
- [ ] 6.4 Mount `<LibraryRail />` inside the `library-rail` slot of `WorkspaceShell.svelte` (provided by change 1). Remove the old `<Sidebar />` mount site in `src/routes/project/+page.svelte` or wherever change 1 placed the slot. If change 1 has not yet provided the slot, STOP and surface the dependency to the parent agent.

## 7. Extract `TrackPointsPanel` helpers

- [ ] 7.1 Inventory the data-loading and formatting functions currently inside `TrackPointsPanel.svelte` (functions that fetch track points, format coords, format timestamps, compute speeds, paginate).
- [ ] 7.2 Create `src/lib/track-points.ts` and move each function into it as a plain TypeScript export. The module SHALL have no Svelte runes, no store subscriptions, no DOM access — pure functions over plain data.
- [ ] 7.3 Confirm via `rg "from .*TrackPointsPanel"` that no other component currently imports helpers from `TrackPointsPanel.svelte`. If any does, update the import to `src/lib/track-points.ts`. (Most likely zero hits — `TrackPointsPanel` is the only consumer of its own helpers today.)
- [ ] 7.4 Add a brief comment at the top of `src/lib/track-points.ts` noting that the Inspector pane (change `redesign-inspector-pane`) is the intended consumer.

## 8. Delete legacy panels and deprecated stores

- [ ] 8.1 Run `rg "tracksPanelOpen|waypointsPanelOpen|trackPointsPanelOpen|simplifyPanelOpen"` across `src/`. Triage every hit:
   - Hits inside `TracksPanel.svelte`, `WaypointsPanel.svelte`, `TrackPointsPanel.svelte`, `SimplifyPanel.svelte`, `Sidebar.svelte` are expected — they disappear with the file deletion.
   - Hits anywhere else are unexpected. STOP and resolve them before deleting the stores.
- [ ] 8.2 Run `rg "from .*Sidebar.svelte|from .*TracksPanel.svelte|from .*WaypointsPanel.svelte|from .*TrackPointsPanel.svelte|from .*SimplifyPanel.svelte"` and confirm the only remaining importers are the route page (`src/routes/project/+page.svelte`) which is being updated in task 6.4, and any layout file that imported `Sidebar` directly. Update all to import `LibraryRail` instead, or remove the import if no longer needed.
- [ ] 8.3 Delete `src/components/Sidebar.svelte` (using `rip`).
- [ ] 8.4 Delete `src/components/TracksPanel.svelte` (using `rip`).
- [ ] 8.5 Delete `src/components/WaypointsPanel.svelte` (using `rip`).
- [ ] 8.6 Delete `src/components/TrackPointsPanel.svelte` (using `rip`).
- [ ] 8.7 Delete `src/components/SimplifyPanel.svelte` (using `rip`).
- [ ] 8.8 Remove the writables `tracksPanelOpen`, `waypointsPanelOpen`, `trackPointsPanelOpen`, `simplifyPanelOpen` from `src/lib/stores.ts`.
- [ ] 8.9 Re-run the grep from 8.1; expect zero hits.

## 9. Verification

- [ ] 9.1 Launch the app via `just dev` (or the project's equivalent). Confirm the workspace renders the new `library-rail` with three tab triggers ("Maps", "Tracks", "Waypoints") all visible.
- [ ] 9.2 Open a test project containing at least one bundle, one map, two track layers (each with at least one track), and two waypoint layers (each with at least one waypoint).
- [ ] 9.3 Maps tab: confirm the active map is highlighted, non-active cached maps show the badge, "Maps…" button opens the bundle-loader Sheet without leaving `/project`, "Reveal in Finder" opens the OS file browser.
- [ ] 9.4 Tracks tab: confirm the active-layer Select shows the active layer's name, every track in every layer appears as a row, visibility toggle hides and shows the track on the map, color swatch popover changes the color domain-wide, double-click rename works, distance / duration / point-count subline is correct.
- [ ] 9.5 Tracks tab: click a track row whose owning layer is not the current active layer; confirm the Select updates to the row's layer AND `$selectedTrack` reflects the clicked row AND no overlays disappear.
- [ ] 9.6 Tracks tab: open a Track row's `⋯` menu → "Simplify…"; confirm a popover opens anchored to the row, the controls match the old `SimplifyPanel`, commit changes the track's point count and the map re-renders.
- [ ] 9.7 Waypoints tab: confirm visibility toggle, symbol picker, rename, Export WPT, Delete all work; clicking a row in a non-active waypoint layer flips the active waypoint layer.
- [ ] 9.8 Cycle through all four Catppuccin flavours and confirm Library surfaces re-render through semantic tokens without per-component overrides; track color swatches stay at their domain RGBA across flavour changes.
- [ ] 9.9 Confirm none of `Sidebar.svelte`, `TracksPanel.svelte`, `WaypointsPanel.svelte`, `TrackPointsPanel.svelte`, `SimplifyPanel.svelte` exist on disk after the change. Confirm `src/lib/track-points.ts` exists and exports the formatting helpers.
- [ ] 9.10 Follow `docs/agent-verification.md` for desktop evidence collection. Playwright is NOT acceptable; verification uses `ozi-rs-mcp` against the actual Tauri build.
- [ ] 9.11 Run `just ci` and confirm it passes (clippy, check, lint, test).
- [ ] 9.12 Run `openspec validate redesign-library-sidebar --strict` and confirm it passes.
