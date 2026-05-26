## Context

The project workspace today renders a 224-pixel `Sidebar.svelte` (`src/components/Sidebar.svelte:133-375`) carrying five distinct concerns:

1. **Project lifecycle** — Open / Save / Undo / Redo buttons.
2. **Map context** — "Maps…" entry + active map description + Reveal-in-Finder.
3. **Tracks** — active track-layer selector (a shadcn `Select` over the project's track layers), Import GPX/PLT, Create Track toggle, and a Show/Hide button that opens `TracksPanel.svelte` as a floating card.
4. **Waypoints** — active waypoint-layer selector, Show/Hide for the active layer's markers, Add Waypoint toggle.
5. **Footer** — status text + Console toggle.

The actual lists of tracks and waypoints live in floating panels (`TracksPanel.svelte`, `WaypointsPanel.svelte`) that overlay the map. Simplify and the points-table live in their own floating panels (`SimplifyPanel.svelte`, `TrackPointsPanel.svelte`). The UX review described this as "I cannot see what is in the project without playing whack-a-mole with floating cards". The locked direction is to mirror Gaia GPS: a persistent left rail with three tabs (Maps / Tracks / Waypoints), each a vertical row-based list.

Change 1 (`redesign-shell-layout`) introduces a three-pane workspace shell with a fixed ~280px `library-rail` slot on the left. This change fills that slot. Change 3 introduces an Inspector pane (right) and a Cmd-K palette; that change is where project lifecycle and mode toggles land — they leave the Library entirely.

The state and IPC surface the Library consumes is unchanged: `$appState`, `$currentProject`, `$activeMap`, `$activeTrackLayerId`, `$activeWaypointLayerId`, `$visibleWaypointLayers`, `$selectedTrack`, `$selectedWaypointId`, and the existing `api.ts` functions for track / waypoint mutation.

## Goals / Non-Goals

**Goals:**

- Replace the floating-panel UX with a persistent, scrollable, tabbed Library that fills the `library-rail` slot.
- Mirror Gaia GPS's row pattern (visibility · color / symbol · name · `⋯`) so users coming from that app recognise the interaction model.
- Preserve every existing track / waypoint operation; nothing is regressed in capability, only relocated.
- Remove the legacy floating panels and the deprecated panel-open booleans cleanly — no zombie code.
- Keep the active-layer selectors visible, but move them out of the navigation chrome and into the tab they belong to.
- Park the `TrackPointsPanel` data-loading helpers in a stable, importable location for change 3 to consume — no logic loss when the panel is deleted.

**Non-Goals:**

- Inspector pane content (right pane). Out of scope; change 3.
- Cmd-K palette UI. Out of scope; change 3.
- Top context-bar mode chips (Draw / Add Waypoint). Out of scope; change 3.
- Map preview thumbnails on Map rows. Out of scope (same rationale as `bundle-loader-as-overlay` Decision 4).
- Drag-to-reorder rows. Not requested for v1.
- Multi-select in the Library. Not requested for v1.
- Persisting per-tab scroll position across sessions. Not requested for v1.
- Renaming layers from the Library. The active-layer selector chooses between existing layers; CRUD on layers stays where it is today (out-of-band utility, not in scope).

## Decisions

### Decision 1: Tabs, not accordions

The Library SHALL use the shadcn `Tabs` primitive with three `Tabs.Trigger`s (Maps, Tracks, Waypoints) and three `Tabs.Content` panels. Only one panel is visible at a time. The active tab SHALL be persisted to a `writable<'maps' | 'tracks' | 'waypoints'>` store (`libraryActiveTab`) so it survives navigation between routes within a session; it SHALL NOT be persisted to localStorage or the session file.

**Rationale**: Gaia GPS uses tabs, the user explicitly cited Gaia as the model, and tabs give each tab a full vertical column for the scrollable list. Accordions would force users to expand each section and would split the vertical space three ways, defeating the "see what is in the project at a glance" goal.

**Alternatives considered**:

- _Accordion / collapsible sections_: rejected — the explicit user reference is Gaia, which is tabbed. Accordions also give worse density for long lists.
- _Single combined list with type chips_: rejected — Maps / Tracks / Waypoints have different row shapes (Maps have no color, Waypoints have a symbol rather than a color swatch); a unified row would need conditional cells everywhere.
- _Persisting active tab to localStorage_: rejected — the active tab is a transient navigation state, not a user preference; reopening the app in the same tab is not a strong UX expectation. If a user later asks for this, it is one line on the store.

### Decision 2: Row anatomy

Every row in the Tracks and Waypoints tabs SHALL follow the same horizontal anatomy:

`[visibility-toggle] [color-or-symbol] [name + optional subline] [actions-menu]`

- **visibility-toggle**: a shadcn `Button` (icon variant) with the Lucide `Eye` / `EyeOff` icon. Click toggles visibility through the existing `set_track_visibility` / waypoint visibility API. No double-click or hover behavior. Aria-label SHALL describe the current state ("Hide track Foo" / "Show track Foo").
- **color-or-symbol**:
  - Tracks tab: a 16x16 swatch rendered as a circle with `background-color: <inline RGBA from TrackStyle.color>`. Click opens a shadcn `Popover` containing a native `<input type="color">` (same control today's `TracksPanel.svelte` uses; reuse, do not replace). Decision 4 of the theme spec ("Per-track and per-waypoint colours are isolated from the theme system") MUST be respected — the swatch's `style=` attribute is an inline domain value, not a Tailwind class.
  - Waypoints tab: a button rendering the waypoint's current symbol glyph at 16x16. Click opens the existing `SymbolPicker` popover (same component used today; reuse, do not replace).
- **name**: the object's display name, truncated with `text-overflow: ellipsis`. The full name SHALL appear in a shadcn `Tooltip` on hover. Double-click on the name SHALL switch the row into an inline-rename mode (a `<input>` bound to a local writable; commit on Enter or blur, cancel on Esc).
- **subline** (Tracks only): a single line below the name in `text-xs text-muted-foreground font-mono tabular-nums` reading distance / duration / point-count separated by middle-dots, e.g. `12.4 km · 02:31 · 412 pts`. Source: derive from the `Track` summary the way `TracksPanel.svelte` already does today.
- **actions-menu**: a shadcn `DropdownMenu` triggered by a `⋯` icon button. Items per tab listed in the proposal. Each item SHALL call the same API function the current floating panel calls.

Rows for the Maps tab SHALL diverge slightly: no visibility toggle (maps are not toggleable per item; they are either the active map or not), and a "cached" badge SHALL replace the color swatch position. The actions menu items (Reveal in Finder, Switch to) are reduced.

**Rationale**: the row pattern is the single most-repeated unit in the Library; consolidating it in a `LibraryRow.svelte` component avoids drift between Tracks and Waypoints implementations and lets per-tab changes (e.g. adding distance to the subline) happen once. The `LibraryRow` slot pattern — slots for the leading control, the swatch / symbol, and the actions menu — matches the existing shadcn primitive style.

### Decision 3: Layer selectors live in the tab header, not in a separate section

The active-track-layer `Select` SHALL render at the top of the Tracks tab's `Tabs.Content` panel, immediately above the row list. The active-waypoint-layer `Select` SHALL render at the top of the Waypoints tab's `Tabs.Content` panel. Each SHALL show the active layer's name as the trigger label and SHALL list every layer of its kind in the open dropdown.

The Maps tab SHALL NOT have an equivalent — bundles and maps are not layered the way tracks and waypoints are. Its header SHALL contain only the "Maps…" button (Decision 5).

**Rationale**: today's sidebar puts the selector at the section header (Tracks section / Waypoints section); preserving that placement in the new Library tabs keeps the muscle memory and avoids burying the selector inside the row list. The selector also visually anchors the tab — it tells the user "rows below are from this layer" (though rows from other layers may also appear if the user has opted to see them — Decision 6 below).

### Decision 4: Row click activates the row's layer

When the user clicks a row in the Tracks tab whose owning layer is not the active track layer, the system SHALL set the active track layer to that row's owning layer (calling the existing `set_active_track_layer` API) before applying the selection. The same SHALL hold for Waypoints.

**Rationale**: in the current UX, the active-layer selector is the single source of truth for "which layer am I editing"; clicking a row from another layer leaves that flag pointing at the previous layer, which is surprising — subsequent edits land in the wrong place. Aligning the active layer to the row's layer on click eliminates the foot-gun. The change is non-destructive (consistent with the `layers` capability's existing requirement) — overlays from other layers remain visible.

**Alternatives considered**:

- _Show only rows from the active layer_: rejected — the user explicitly asked for "every track in the active project" in the Tracks tab, not "every track in the active layer".
- _Disable / dim rows from non-active layers_: rejected — same reason; also visually noisy.
- _Confirm-modal on switching layer_: rejected — too friction-heavy for a navigation action.

### Decision 5: "Maps…" button delegates to the existing Sheet

The Maps tab header SHALL render a single "Maps…" button (shadcn `Button`, secondary variant) that calls `bundleLoaderOpen.set(true)`. The Sheet primitive, the `BundleLoader` component, and the close-on-success behavior already exist (delivered by `bundle-loader-as-overlay`, inherited by `redesign-shell-layout`). This change SHALL NOT touch the Sheet implementation; it only re-points the button.

The button SHALL replace the "Maps…" entry that lives in `Sidebar.svelte` lines 188-196 today. Removing the old button and adding the new one in the Library is a net-zero behavioral change; the button's behavior (open the Sheet without leaving `/project`) is preserved end-to-end.

**Rationale**: the Sheet is the locked UX for in-workspace map switching. Re-introducing a route navigation or a different surface would regress prior work.

### Decision 6: Simplify becomes an inline popover, not a panel

The Simplify operation SHALL be invoked from the Track row's `⋯` menu → "Simplify…" item, which SHALL open a shadcn `Popover` anchored to the row. The popover SHALL contain the same controls `SimplifyPanel.svelte` exposes today (algorithm selector, tolerance slider, preview / commit / cancel buttons) and SHALL operate on the row's track ID.

`SimplifyPanel.svelte` SHALL be deleted after the popover content is ported.

**Rationale**: the user's complaint about floating cards covering the map applies to `SimplifyPanel.svelte` too. An inline popover scoped to a specific row is the natural place for "do this operation on this object" — it removes the question "which track is Simplify simplifying?" because the popover is anchored to the row. The popover is dismissable by the same Esc / click-outside contract as the rest of the shadcn primitives.

**Alternatives considered**:

- _Simplify as a route_: rejected — too heavy for a per-track utility.
- _Simplify in the Inspector_: deferred to change 3 if the Inspector grows a Simplify section there; for v1 of the Library the inline popover is enough.
- _Keep the floating panel and just hide it by default_: rejected — does not remove the floating-card pattern the user disliked.

### Decision 7: Project lifecycle leaves the Library entirely

The Open project, Save project, Undo, and Redo buttons SHALL NOT appear anywhere in `LibraryRail.svelte` or its sub-components. They are not represented in this change's UI.

Their new homes are the top context-bar (Open / Save) and the Cmd-K palette (Undo / Redo, plus everything in the palette), both owned by change 3. Until change 3 ships, those actions are accessible only via keyboard shortcuts already wired (Cmd-Z / Cmd-Shift-Z for undo/redo, Cmd-S for save) and via the system menu bar if the user has one configured.

**Rationale**: the Library is for **objects** (maps, tracks, waypoints). Mixing project-lifecycle verbs into an object-list pane is the cardinal sin of the current Sidebar — the user complained that "I can't find my tracks because the sidebar is full of buttons". Pulling lifecycle out is the single biggest signal-to-noise improvement available in this change.

**Trade-off acknowledged**: between this change landing and change 3 landing, lifecycle actions are reachable only via keyboard / system menu, not via mouse. This is acceptable for an internal-tooling app; the affected users are the same maintainers landing the changes. If this becomes a problem for an external release before change 3 lands, the proposal will be revised to add a stop-gap top-bar.

### Decision 8: Mode toggles leave the Library entirely

The "Create Track" toggle (today the section header button in the Tracks section) and the "Add Waypoint" toggle (today the section header button in the Waypoints section) SHALL NOT be reproduced inside the Library. They become chips on the top context-bar in change 3.

The underlying state stores — `drawingModeActive`, `addWaypointMode`, and their companion stores — SHALL be left untouched. Removing the buttons from the Sidebar SHALL NOT change the stores' semantics. Keyboard shortcuts (if any) wired to toggle these modes continue to work end-to-end.

**Rationale**: same as Decision 7 — Library is for objects, not modes. Modes are application-state verbs that change how the map interprets clicks; they belong with the top context-bar's mode-chip cluster, where the user can see "I am currently in Draw mode" without scanning a list of tracks.

**Trade-off acknowledged**: between this change and change 3, mode toggles are reachable only via keyboard shortcuts (where present). Same mitigation as Decision 7.

### Decision 9: Floating panels are replaced wholesale, not incrementally

`TracksPanel.svelte`, `WaypointsPanel.svelte`, `TrackPointsPanel.svelte`, and `SimplifyPanel.svelte` SHALL be deleted in the same change that introduces `LibraryRail.svelte`. There SHALL NOT be an intermediate state where both surfaces coexist behind a feature flag.

**Rationale**: keeping both surfaces alive would require dual code paths for color setting, visibility toggling, rename, export — every API touchpoint the panels and the Library share. The maintenance cost is higher than the migration risk. The migration is a one-shot file move; the diff is reviewable in one PR. The four deprecated stores (`tracksPanelOpen`, `waypointsPanelOpen`, `trackPointsPanelOpen`, `simplifyPanelOpen`) also disappear in this change — no zombie state.

**Trade-off acknowledged**: a one-shot PR is larger than incremental PRs. Mitigated by the per-tab task breakdown in `tasks.md` — the work is split inside the PR for easy review, just landed together.

### Decision 10: TrackPointsPanel logic is preserved for change 3

The data-loading helpers in `TrackPointsPanel.svelte` (the functions that pull track point arrays, format coordinates, format timestamps, compute speeds, paginate the table) SHALL be extracted into a new module `src/lib/track-points.ts` before the `.svelte` file is deleted. The module SHALL be a pure-TypeScript utility: no Svelte runes, no store subscriptions, no DOM. The Inspector pane in change 3 SHALL import from this module to render its track-points table.

**Rationale**: re-deriving the helpers in change 3 from a deleted file is error-prone. Parking them in a stable module is cheap and unblocks change 3's implementation. The module SHALL NOT export Svelte components — only data functions — so it is not visible UI in this change.

**Alternatives considered**:

- _Leave the helpers in `TrackPointsPanel.svelte` and have change 3 import from it_: rejected — keeps a `.svelte` file alive purely as a logic-host, which is confusing and re-introduces the floating panel's DOM tree on the import graph.
- _Inline the helpers into the Inspector in change 3_: rejected — duplicates the existing logic and forces change 3 to re-author work this change has direct context for.

## Risks / Trade-offs

- **Risk**: removing `tracksPanelOpen` and friends from `src/lib/stores.ts` breaks a forgotten consumer (e.g. a keyboard shortcut or a hidden command path). **Mitigation**: task 7.1 mandates `rg "tracksPanelOpen|waypointsPanelOpen|trackPointsPanelOpen|simplifyPanelOpen"` before deletion; any hit outside the four panel files MUST be triaged before the store is removed.
- **Risk**: the "click row → switch active layer" behavior (Decision 4) surprises the user when they intended only to peek at a track from another layer without committing the active-layer flip. **Mitigation**: this matches Gaia GPS behavior (the reference design); the existing non-destructive invariant in `layers` ensures no overlays disappear, so the worst case is "next created track lands in a different layer than expected". A future refinement could add a peek-vs-commit distinction; not in scope for v1.
- **Risk**: inline rename via double-click on the name conflicts with double-click semantics on the map (zoom). **Mitigation**: the row's name is its own pointer-events surface inside the rail; double-click is captured before it reaches the map. The Library rail and the map are in different DOM trees of the shell (`library-rail` vs `map-canvas`), so the events do not interleave.
- **Risk**: deleting the floating panels in the same PR as introducing the Library produces a large diff. **Mitigation**: the diff is mostly deletions and net-new files — both sides are easy to review. The task list in `tasks.md` groups the work so reviewers can verify Library mount → port → delete in order.
- **Risk**: deferred lifecycle actions (Open / Save / Undo / Redo) are temporarily harder to reach between this change and change 3. **Mitigation**: same as Decision 7 — keyboard shortcuts cover the common cases; sequenced release pairs this change with change 3.
- **Trade-off**: the row component is fairly opinionated (visibility / color-or-symbol / name / actions). Future row types (e.g. a Photos tab) would need either a slot-heavier `LibraryRow` or their own row component. Acceptable for v1; the four-cell pattern is what the user asked for.
- **Trade-off**: `src/lib/track-points.ts` lands in this change but is consumed in change 3. Until change 3, the module has zero importers, which `dead_code` lints could flag. Acceptable — the module is small, and change 3 follows within the same release cycle.

## Migration Plan

Implementation order, kept reviewable at each step:

1. Create `src/components/library/LibraryRow.svelte` with the four-cell pattern, parameterised via slots / props for visibility, swatch-or-symbol, name (+ optional subline), and actions menu. Add storybook-free unit smoke (manual: render once with a stub track and once with a stub waypoint).
2. Create `src/components/library/MapsTab.svelte`. Port the maps-listing logic from `Sidebar.svelte` lines 207-235 (active map description, Reveal in Finder). The "Maps…" button calls `bundleLoaderOpen.set(true)`. No row visibility toggle.
3. Create `src/components/library/TracksTab.svelte`. Port the track-list rendering and per-track operations from `TracksPanel.svelte`. The active-layer selector moves from `Sidebar.svelte` lines 252-272 into this tab's header. Wire row click → `set_active_track_layer` (Decision 4) and row selection → `$selectedTrack`. The Simplify popover (Decision 6) ports content from `SimplifyPanel.svelte`.
4. Create `src/components/library/WaypointsTab.svelte`. Port from `WaypointsPanel.svelte` analogously. Active-waypoint-layer selector moves into the tab header. Wire row click → `set_active_waypoint_layer` and row selection → `$selectedWaypointId`.
5. Create `src/components/LibraryRail.svelte` composing the three tabs via shadcn `Tabs`. Persist the active tab in `libraryActiveTab` writable (session-scoped, no localStorage).
6. Mount `<LibraryRail />` in the `library-rail` slot of `WorkspaceShell.svelte` (provided by change 1). Remove the old `<Sidebar />` mount site. The `Sidebar.svelte` file remains on disk until step 8.
7. Extract `TrackPointsPanel.svelte`'s data helpers into `src/lib/track-points.ts`. No callers yet — change 3 will import.
8. Delete `src/components/Sidebar.svelte`, `src/components/TracksPanel.svelte`, `src/components/WaypointsPanel.svelte`, `src/components/TrackPointsPanel.svelte`, `src/components/SimplifyPanel.svelte`. Remove the four panel-open stores from `src/lib/stores.ts`. Run the grep from task 7.1; resolve any residual references.
9. Manual verification on macOS (see `tasks.md` task 9). `just ci` passes. `openspec validate redesign-library-sidebar --strict` passes.

No backend changes. No new IPC. No data migration. No persisted-state migration (the deleted stores were not persisted).
