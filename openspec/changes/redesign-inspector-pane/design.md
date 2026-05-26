## Context

This is the third and final change in the workspace-redesign sequence. The first (`redesign-shell-layout`) created a 3-pane shell: left rail for navigation, centre for `MapView`, right `inspector-rail` ~360px wide that today is empty. The second (`redesign-library-sidebar`) filled the left rail with a Library (tabs Maps / Tracks / Waypoints, Gaia-style rows) and retired the floating `TracksPanel`, `WaypointsPanel`, and `TrackPointsPanel` surfaces — their logic is parked in branches / commented blocks waiting for a new host.

The user reviewed mockups during the redesign brainstorming session and locked the following:

- The Inspector is the right-rail surface that shows context-sensitive properties for the current Library selection.
- The Inspector starts collapsed. It slides in on selection.
- All editing inside the Inspector is inline — no dialogs, no separate forms.
- A global Cmd-K command palette is the cross-cutting "jump anywhere / run anything" affordance.
- The palette result row pattern mirrors the Library row pattern so the eye is calibrated against a single visual rhythm.

Constraints from the surrounding architecture:

- Edits must go through `ProjectCommand` (see `docs/commands-reference.md`). Inline editors in the Inspector dispatch the same commands the retired panels did.
- All IPC and stores already exist; no backend work in this change.
- Motion intensity 6 was locked in change 1 — slide-in 240ms, scale-up for the palette, internal spring transitions inside the rail. Geist / Zinc+Emerald / rounded-`[1.5rem]` cards / tinted shadows all inherit unchanged.

## Goals / Non-Goals

**Goals:**

- Fill the empty `inspector-rail` slot with a context-sensitive Inspector that reacts to Library selection.
- Port the parked `TrackPointsPanel` logic into a Track Inspector segments / points sub-view, preserving the exact stores (`$selectedPointId`, `$activeTrackLayerId`, `$editModeActive`) so the edit-mode behaviour is byte-equivalent to today.
- Provide inline editing for waypoints (name, symbol, visibility) without a dialog.
- Add a global Cmd-K command palette built on the shadcn-svelte `Command` primitive (added in change 1).
- Add a small versioned localStorage helper for the palette's Recent files group.
- Stay frontend-only — no new IPC, no new Tauri events.

**Non-Goals:**

- Implementing the elevation chart. The Track Inspector reserves a placeholder slot; the actual chart needs a charting library and per-point elevation series, both deferred.
- New backend IPC. No `get_map_preview`, no `get_track_elevations`, no new event subscriptions.
- Map calibration editor. The Map Inspector is read-only; a future change owns write semantics.
- Settings panel rewrite. The palette exposes settings shortcuts but does not replace `ThemePicker` or introduce a new settings surface.
- Route planning / new editing modes. The Track Inspector's actions row reuses only what exists today.
- Resizable Inspector width. The 360px width inherits from change 1 and stays fixed.
- Multi-selection in the Library / Inspector. Single-selection only, matching change 2.

## Decisions

### Decision 1: Inspector mount semantics — always mounted, body conditionally rendered

The `<InspectorRail />` component SHALL be mounted unconditionally in the workspace layout's `inspector-rail` slot. Its body (the active Track / Waypoint / Map Inspector subcomponent) SHALL be conditionally rendered based on the selection stores.

**Rationale**: mounting / unmounting the rail itself on selection changes would cause the slide-in animation to fire on every Library click. Keeping the rail mounted and letting it transition between three internal states (collapsed empty, slide-in entering, populated) plus the subcomponent swap gives a smoother, predictable motion. It also lets the rail keep its edge affordance ("pin open with most recent selection") visible even when collapsed.

**Alternatives considered**:

- _Unmount the rail when selection is null_: rejected — the slide-in / slide-out animation would re-fire on every selection change, and the edge affordance would disappear.
- _Always render the full subcomponent and let it self-handle "empty"_: rejected — the empty state is shared across all three subtypes (Track / Waypoint / Map all collapse to the same edge affordance), so hosting it once at the rail level is cleaner than triplicating it.

### Decision 2: Auto-expand on every selection change vs require explicit open

The Inspector SHALL auto-expand whenever the selection store transitions from null → non-null OR from one selection to another of any type. There SHALL be no "require user to click to open" gate.

**Rationale**: the user's workflow is "click a row → see its properties immediately". Adding an explicit open step would be friction; the Library row click is the implicit open intent. If the user pins the rail open via the edge affordance, that pin persists across selection changes — they will not need to re-open it on every selection.

**Edge case**: when the selection transitions to null (e.g. project cleared, or user clicks empty space in the Library), the Inspector SHALL collapse back, UNLESS the rail is currently pinned. When pinned, the rail stays open showing the most-recent-selection's Inspector frozen as a read-only snapshot of its header (the body collapses to "no selection" but the rail itself stays expanded for the next selection to slot in).

### Decision 3: Segments / points table lives inside Track Inspector, below the stats card

The ported `TrackPointsPanel` logic SHALL be placed inside `TrackInspector.svelte` immediately below the stats card and above the actions row. It SHALL be implemented as a child component `src/components/inspector/TrackSegmentsTable.svelte` (re-using the parked logic verbatim where possible), composed into `TrackInspector`. The edit-mode toggle SHALL stay on `$editModeActive`, not a new local store.

**Rationale**: the user's mental model is "this track → these properties → these points". The points list is intrinsically a property of the selected track. Hosting it under Track Inspector keeps the selection scope coherent (no separate panel that could be open while a different track is selected). Keeping it as a separate `TrackSegmentsTable` child component avoids ballooning `TrackInspector` past readable size and lets future work (elevation chart, segment colouring) slot in around it without rewriting the table.

**Alternatives considered**:

- _Inline the entire table directly in `TrackInspector.svelte`_: rejected — file would exceed reasonable size for the planned scope (stats + table + actions + future elevation slot).
- _Keep the table as a separate top-level rail variant ("Points" tab)_: rejected — tabs inside the rail were not in the locked design; the Library is the only tabbed surface.

### Decision 4: Palette command groups — fixed order, no user reorder

The Cmd-K palette SHALL render result groups in a fixed order: **Open map**, **Switch project**, **Find track / waypoint** (merged), **Project actions**, **Settings**, **Recent files**. Within each group, results are filtered by the search input and ranked by simple prefix-match → substring-match → fuzzy-fallback (Fuse.js or equivalent — implementation detail, library choice in tasks).

**Rationale**: the user explicitly listed these groups during the brainstorm. A fixed order means the user builds muscle memory ("the first result group is always Open map, so I never have to scan headers"). Custom reorder is a v2 concern.

**Empty groups**: a group with zero matching results SHALL be hidden entirely (no empty headers). If all groups are empty, the palette SHALL show a single empty state "No matches" without group headers.

**"Find track / waypoint" merge**: tracks and waypoints share the same group because they are both intra-project lookups and the user typically remembers the name, not the type. The result row's icon column distinguishes the two visually (existing track/waypoint icons from the Library).

### Decision 5: Palette row pattern parallels the Library row pattern

The palette result row SHALL use the same row component / shape as the Library: icon column on the left (24px), label column in the middle, optional meta line right-aligned. The visual differentiation is at the level of:

1. The palette has a search input on top (the Library's filter input also sits on top, but the palette's is the primary input with focus on open).
2. The palette has group headers (tiny uppercase labels with `text-muted-foreground/60`); the Library has tab headers instead.
3. The palette result row has a keyboard-shortcut hint slot on the far right (right-aligned subtle text like `⌘E` for the secondary action) — the Library has none.

**Rationale**: visual rhythm. The user already calibrates against the Library row pattern; reusing it inside the palette means the palette feels like a "search across all Library tabs plus actions" — which is exactly what it is.

**Implementation note**: a shared row component (`src/components/common/SelectableRow.svelte` or similar) SHOULD be extracted if not already extracted by change 2. If change 2 did not extract it, this change SHALL extract it as part of the palette work to avoid double-maintenance.

### Decision 6: Keyboard shortcut conventions

- `⌘K` / `Ctrl+K`: open palette (global, captures from any focus state including form inputs — the only place it does NOT capture is inside the palette itself).
- `Esc`: close palette OR collapse Inspector (palette takes priority if open).
- `Enter` inside the palette: run the highlighted result's primary action.
- `⌘E` inside the palette: run the highlighted result's secondary action (Export, for track/waypoint results) — no-op if the result has no secondary.
- `⌘R` inside the palette: run the highlighted result's secondary action when it is "Reveal in Finder" (for map results) — no-op otherwise.
- Arrow keys inside the palette: move the highlight.
- `Tab` inside the palette: cycles through groups (jumps to the first result of the next group).

These chord conventions follow Linear / Raycast precedent. The `⌘E` / `⌘R` split is intentional: a single "secondary" chord would be ambiguous when a track row and a map row both expose different secondaries; the user knows from the row's type which chord applies.

**Out of scope for v1**: customisable shortcuts. The conventions ship as defaults; a future settings change can expose them.

### Decision 7: Recent files persistence — localStorage with versioned key

A new helper `src/lib/recentFiles.ts` SHALL maintain a localStorage entry keyed `ozi:recent-files:v1`. The value SHALL be a JSON array of records `{ projectSlug: string, mapPath: string, mapName: string, openedAt: number }`, capped at the most recent N (default 8, configurable via a module constant). On every successful `openSelectedMap` resolution, the helper appends a new record (deduplicating by `mapPath`, moving an existing entry to the front rather than duplicating).

**Why a versioned key (`:v1`)**: future schema migrations are explicit. If the record shape changes (e.g. adds a thumbnail field), the new helper writes to `:v2` and the old key is either migrated or ignored — no silent data loss, no "is this record from before or after the change" ambiguity. This mirrors the `:v1` pattern documented for the catalog cache in `cache-project-catalog-locally`.

**Failure modes**:

- localStorage quota exceeded: catch the exception, log via `console.warn`, drop the oldest record, retry once. If still failing, give up silently — the palette just shows fewer recents.
- Stored JSON unparseable: catch on read, treat as empty list, overwrite on next successful open.
- `mapPath` no longer exists on disk: the palette result for that file SHALL still render. Clicking it SHALL trigger the existing "map not found" error path (the same one that fires when a session-restored map vanishes). No proactive disk check on palette open — that would be slow and out of scope.

**Why localStorage, not the Rust session file**: recent-files is a UX convenience, not domain data. It is per-machine, not per-project. The session file is for domain state (active project, active map, open tracks); polluting it with UI affordances breaks the separation that `project-persistence` established.

### Decision 8: Palette mount site — root layout, single instance

`<CommandPalette />` SHALL be mounted once in `src/routes/+layout.svelte`, sibling to `MapView`. Its open state SHALL be a single writable store (`commandPaletteOpen` in `src/lib/stores.ts`).

**Rationale**: the palette is global. Mounting it inside the project route would mean cold-start (no active map, user at `/`) has no palette — that breaks the "global trigger" requirement, since the user might want to open the palette from the bundle loader to switch projects.

**Caveat**: when the user is at `/` (cold-start, no active project), groups that depend on active-project state (Find track / waypoint, Project actions, Open map within active project) SHALL be hidden. Only Switch project, Settings, and Recent files groups SHALL render. The palette adapts to context without crashing.

### Decision 9: Inspector edit dispatch — `ProjectCommand` always

All Inspector edits (rename waypoint, change symbol, toggle visibility, set track line width, simplify track, delete) SHALL dispatch through `ProjectCommand` via the existing endpoints in `src/lib/api.ts`. No direct store mutation. No bypassing of the command bus.

**Rationale**: this is a project-wide invariant from `CLAUDE.md` ("All edits must go through `ProjectCommand`"). The retired panels followed it; the Inspector cannot drop it. Undo/redo, persistence, and event broadcast all rely on it.

**Implementation note**: the existing `api.ts` functions (`updateWaypoint`, `deleteWaypoint`, `updateTrackStyle`, `simplifyTrack`, `deleteTrack`, …) are already `ProjectCommand`-shaped wrappers. The Inspector calls them directly. No new wrapper layer.

## Risks / Trade-offs

- **Risk**: double-subscription of point-selection state if the segments table accidentally subscribes to a different store than the legacy `TrackPointsPanel` did. **Mitigation**: port the table verbatim, reusing the exact stores (`$selectedPointId`, `$activeTrackLayerId`, `$editModeActive`). Smoke verification step explicitly compares behavior (clicking a point on the map highlights the row in the table, and vice versa).
- **Risk**: palette `⌘K` handler conflicts with browser / OS chords inside the WebView. **Mitigation**: register the listener at the layout level via `window.addEventListener('keydown', …)` with `event.preventDefault()` on match; document the chord; manual verification on macOS, Windows, Linux. The Tauri WebView passes raw keydown events to JS reliably for `⌘K` and `Ctrl+K`.
- **Risk**: Recent files leaks across machines if user copies the localStorage manually (unlikely, but the `mapPath` is absolute and machine-specific). **Mitigation**: the helper SHALL silently drop records whose path does not satisfy the host's `path.isAbsolute` check via a small Tauri-side helper OR (cheaper) leave them in and let the "map not found" error path handle them. Decision deferred to implementation; either is acceptable.
- **Risk**: the Inspector subscribes to too many stores and re-renders on every project mutation. **Mitigation**: use derived stores where the subcomponent only depends on a slice; document the subscriptions in the component header comment so future maintainers see the dependency set.
- **Trade-off**: extracting `SelectableRow` (Decision 5) is a small refactor of the Library if change 2 did not already do it. Acceptable: the duplication cost across two surfaces (Library + palette) is higher than the extraction cost; doing it now prevents two slightly-diverging copies later.
- **Trade-off**: the elevation-chart placeholder is visible to the user — it occupies real space in the Track Inspector with a "coming soon" label. Acceptable for v1: the user knows the chart is planned, and the placeholder anchors the layout so the future chart drop-in does not reflow neighbouring sections.
- **Trade-off**: fixed group order in the palette (Decision 4) means power users cannot pin their favourites first. Acceptable for v1: muscle memory wins over personalisation at first; a future change can expose reorder if users ask.

## Migration Plan

Implementation order, keeping the working tree shippable at each step:

1. Create `src/components/InspectorRail.svelte` with the empty / collapsed state only, mount it in the workspace shell's `inspector-rail` slot. The rail renders the edge affordance but no body. Verify the shell layout still passes change 1's smoke (3-pane geometry intact, MapView centered, slide-in animation registered but never triggered).
2. Add `MapInspector.svelte` (read-only — easiest to wire). Hook it to the Map info affordance on the Library Maps tab from change 2. Verify selecting a map from the Library opens the rail with calibration metadata.
3. Add `WaypointInspector.svelte` with inline name + symbol picker + visibility toggle. Wire to `$selectedWaypointId`. Verify editing a waypoint name persists across reload.
4. Add `TrackInspector.svelte` header + stats card + actions row (no segments table yet). Wire to `$selectedTrack`. Verify selecting a track shows stats and that Export GPX still works.
5. Port `TrackPointsPanel` logic into `TrackSegmentsTable.svelte`, compose into `TrackInspector` below the stats card. Verify point-click ↔ row-click bidirectional highlight, edit mode toggle, all operations preserved.
6. Add the elevation-chart placeholder slot inside `TrackInspector` (just the labeled empty container — no chart). Verify it does not reflow on track switch.
7. Create `src/lib/recentFiles.ts` with the localStorage helper. Add a call site at the existing `openSelectedMap` success branch (one line addition). Verify localStorage updates on map open.
8. Create `CommandPalette.svelte` on the shadcn `Command` primitive. Wire the `⌘K` / `Ctrl+K` global handler in `+layout.svelte`. Implement groups one at a time: Open map → Switch project → Find track / waypoint → Project actions → Settings → Recent files. Verify each group activates the correct primary action.
9. Implement palette secondary actions (`⌘E`, `⌘R`) on track / waypoint / map result rows. Verify chord triggers the correct command without closing on the primary path.
10. If `SelectableRow` was not extracted by change 2, extract it now from the Library and reuse it in the palette. Verify Library still renders identically.
11. Manual smoke pass against the verification scenarios in `tasks.md` section 7. `just ci` pass. `openspec validate redesign-inspector-pane --strict` pass.

No user data, session file, or persisted domain state is affected. No backend changes. No new IPC. The only new persisted artifact is the localStorage entry `ozi:recent-files:v1`, which the user can clear at any time without consequence to project data.
