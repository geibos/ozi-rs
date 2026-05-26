## ADDED Requirements

### Requirement: The workspace `inspector-rail` slot hosts a context-sensitive Inspector that is collapsed by default and slides in on Library selection

The right-side `inspector-rail` slot of the workspace shell (introduced by `redesign-shell-layout`) SHALL be filled by a single component `src/components/InspectorRail.svelte`. The rail SHALL be mounted unconditionally for the lifetime of the workspace surface; only its body content SHALL appear and disappear in response to selection.

In its default state (no Library selection), the rail SHALL render only an edge affordance (a thin pinnable handle on the right edge of the viewport) and SHALL occupy zero horizontal width inside the centre pane (i.e. the `MapView` extends to the right edge of the workspace minus the edge affordance's narrow strip).

When the Library selection store transitions from null to a non-null value of any of the three supported types (Track, Waypoint, Map), the rail SHALL slide in from the right edge over 240ms, fade its body content in, and render the appropriate Inspector subcomponent for the selected type. The slide-in animation SHALL respect the motion-intensity-6 tokens established by `redesign-shell-layout`.

When the selection transitions between two non-null values (regardless of whether the types match), the rail SHALL remain open and SHALL swap its body subcomponent via the spring transition (stiffness 100, damping 20) established by `redesign-shell-layout`; the rail itself SHALL NOT re-run its slide-in animation on such transitions.

When the selection transitions back to null AND the rail is not pinned, the rail SHALL slide its body out and return to the collapsed default. When the rail is pinned (the user has clicked the pin affordance on the edge handle), the rail SHALL remain open with an empty body in this case.

The rail's fixed expanded width SHALL be ~360px, matching the slot dimension reserved by `redesign-shell-layout`. It SHALL NOT be user-resizable in v1.

#### Scenario: Cold workspace — Inspector starts collapsed

- **WHEN** the user opens a project workspace AND no Library row is selected
- **THEN** the `inspector-rail` slot renders only the edge affordance, the `MapView` extends to the workspace's right edge minus the edge strip, and no Inspector body is mounted

#### Scenario: Selecting a Track row opens Track Inspector

- **WHEN** the user clicks a Track row in the Library AND `$selectedTrack` transitions from null to a track
- **THEN** the rail slides in over 240ms from the right edge, the Track Inspector subcomponent mounts inside the rail, and the rail's body fades in

#### Scenario: Switching from a Track to a Waypoint swaps Inspector body without re-animating the rail

- **WHEN** the rail is open showing a Track Inspector AND the user clicks a Waypoint row in the Library
- **THEN** the rail itself remains expanded with no slide animation re-fire; the rail's body subcomponent swaps from Track Inspector to Waypoint Inspector via the spring transition

#### Scenario: Clearing selection collapses the rail

- **WHEN** the rail is open AND the Library selection transitions back to null (no row highlighted) AND the rail is not pinned
- **THEN** the rail slides its body out and returns to the collapsed default state with only the edge affordance visible

#### Scenario: Pinning the rail keeps it open across selection changes

- **WHEN** the user clicks the pin affordance on the rail's edge handle AND subsequently clears the Library selection
- **THEN** the rail remains expanded at full width with its body rendering an empty state, ready for the next selection to populate it without re-running the slide-in animation

### Requirement: Track Inspector contains a stats card, an inline segments / points table, an elevation-chart placeholder, and an actions row

When the Inspector rail renders the Track Inspector subcomponent, the subcomponent SHALL lay out the following sections in order:

1. **Header** — track name (read-only display, with a colour swatch matching the track's `TrackStyle.color`) and a visibility toggle.
2. **Stats card** — distance, duration, point count, start time, all rendered with mono-spaced numerals on a rounded-`[1.5rem]` card surface.
3. **Segments / points table** — the table SHALL be implemented as a child component `src/components/inspector/TrackSegmentsTable.svelte` that ports the logic of the retired `TrackPointsPanel` (parked by `redesign-library-sidebar`). The table SHALL subscribe to the exact same stores the legacy panel did: `$selectedPointId`, `$activeTrackLayerId`, and `$editModeActive`. Bidirectional highlight between the map and the table SHALL be preserved. The edit-mode toggle SHALL stay on the `$editModeActive` store and SHALL NOT introduce a new store.
4. **Elevation chart placeholder slot** — a labeled empty container reserving space for a future elevation chart. It SHALL display a static message ("Elevation chart — coming in a follow-up change") and SHALL NOT call any new IPC or charting library in this change.
5. **Actions row** — Export GPX, Export PLT, Set line width, Simplify, Delete. Each action SHALL dispatch through the existing `ProjectCommand`-shaped endpoints in `src/lib/api.ts` (`exportTrackGpx`, `exportTrackPlt`, `updateTrackStyle`, `simplifyTrack`, `deleteTrack`).

#### Scenario: Selecting a track populates the stats card

- **WHEN** the user clicks a Track row in the Library
- **THEN** the Track Inspector mounts AND the stats card renders distance, duration, point count, and start time from `getTrackDetail` for the selected track

#### Scenario: Map point click highlights the corresponding table row

- **WHEN** Track Inspector is open with the segments table visible AND the user clicks a point on the map that belongs to the selected track
- **THEN** the corresponding row in the segments table is highlighted (matching the behaviour of the retired `TrackPointsPanel`)

#### Scenario: Table row click highlights the corresponding map point

- **WHEN** Track Inspector is open with the segments table visible AND the user clicks a row in the segments table
- **THEN** the corresponding point on the map is highlighted and the map view scrolls / centres the point as the retired `TrackPointsPanel` did

#### Scenario: Edit mode toggle uses the existing `$editModeActive` store

- **WHEN** the user toggles the edit-mode switch in the Track Inspector's segments table header
- **THEN** the `$editModeActive` store flips AND the table's per-row edit affordances appear / disappear (same UI semantics as the retired panel)

#### Scenario: Elevation chart placeholder renders without loading data

- **WHEN** Track Inspector mounts AND the segments table has loaded
- **THEN** an elevation-chart placeholder is visible below the table AND no IPC call to fetch elevation data is made AND no charting library is loaded

#### Scenario: Track actions dispatch through `ProjectCommand` endpoints

- **WHEN** the user clicks any action in the Track Inspector's actions row (Export GPX, Export PLT, Set line width, Simplify, Delete)
- **THEN** the corresponding `src/lib/api.ts` endpoint is invoked AND the call resolves through the existing `ProjectCommand` pipeline (no direct store mutation, no bypass of the command bus)

### Requirement: Waypoint Inspector edits inline (name, symbol, lat/lng readout, visibility) without a dialog

When the Inspector rail renders the Waypoint Inspector subcomponent, the subcomponent SHALL provide inline editing for the selected waypoint. There SHALL be no separate modal dialog or popout form.

The subcomponent SHALL lay out:

1. **Header** — waypoint name as an editable text input AND a symbol picker (re-using the existing `SymbolPicker` component) AND a visibility toggle.
2. **Location card** — lat / lng readout (read-only display) AND a "Move on map" action that delegates to the existing map-driven move flow.
3. **Actions row** — Export WPT, Delete.

All edits SHALL dispatch through the existing `ProjectCommand`-shaped endpoints in `src/lib/api.ts` (e.g. `updateWaypoint`, `deleteWaypoint`). No direct store mutation. No bypass of the command bus.

#### Scenario: Renaming a waypoint inline persists through ProjectCommand

- **WHEN** the user types a new name in the Waypoint Inspector header AND blurs / commits the input
- **THEN** `updateWaypoint` is invoked through `src/lib/api.ts` AND the waypoint's new name is persisted via the `ProjectCommand` pipeline (no separate dialog mounts at any point)

#### Scenario: Changing the symbol inline

- **WHEN** the user picks a symbol from the inline `SymbolPicker` in the Waypoint Inspector
- **THEN** `updateWaypoint` is invoked with the new symbol AND the change is reflected on the map without a dialog opening or closing

### Requirement: Map Inspector is read-only and shows calibration metadata plus a Reveal in Finder action

When the Inspector rail renders the Map Inspector subcomponent, the subcomponent SHALL display the active map's calibration metadata read-only (CRS, bounds, resolution — sourced via the existing `getOziMetadata` endpoint in `src/lib/api.ts`) and SHALL expose a "Reveal in Finder" action that opens the map file's directory in the host OS file manager via existing Tauri shell primitives.

The Map Inspector SHALL NOT expose any write affordances for calibration data in this change.

#### Scenario: Selecting a map shows its calibration

- **WHEN** the user opens Map info for the active map from the Library Maps tab
- **THEN** the Map Inspector renders inside the rail showing CRS, bounds, and resolution from `getOziMetadata`, all as read-only displays

#### Scenario: Reveal in Finder opens the host file manager

- **WHEN** the user clicks the "Reveal in Finder" action in the Map Inspector
- **THEN** the host OS file manager opens to the active map file's parent directory (no new Tauri command is added; existing shell primitives are used)

### Requirement: A global Cmd-K command palette is available everywhere with grouped results, primary actions on Enter, and optional secondary actions via keyboard chords

A single instance of `src/components/CommandPalette.svelte` SHALL be mounted in `src/routes/+layout.svelte`, sibling to `MapView`. Its open / closed state SHALL be backed by a writable store `commandPaletteOpen` in `src/lib/stores.ts`.

The palette SHALL be opened by the keyboard chord `⌘K` (macOS) or `Ctrl+K` (Windows / Linux), registered as a `window`-level `keydown` listener at the layout level. The chord SHALL fire from any focus state in the application — including inside form inputs — except when focus is already inside the palette itself.

The palette SHALL also be openable via a button in the top context-bar of the workspace shell.

The palette SHALL close on:

1. pressing `Esc` while focus is anywhere inside the palette,
2. clicking outside the palette content area,
3. successful resolution of any primary or secondary action.

The palette SHALL render a single search input at the top (auto-focused on open) and a results list below. The results list SHALL be grouped, in the following fixed order:

1. **Open map** — maps in the active project (sourced from existing project / map stores).
2. **Switch project** — projects in the LizaAlert catalog (sourced from the catalog cache established by `cache-project-catalog-locally`).
3. **Find track / waypoint** — tracks and waypoints in the active project, merged into one group; result row icon distinguishes the type.
4. **Project actions** — Open, Save, Undo, Redo.
5. **Settings** — theme, units (placeholder), GPS (placeholder).
6. **Recent files** — sourced from the localStorage entry `ozi:recent-files:v1`.

Each result row SHALL share the row pattern used by the Library (icon column + label + optional meta line). Groups with zero matching results after the current search input SHALL be hidden entirely (no empty headers). If all groups are empty, the palette SHALL show a single "No matches" state.

Each result SHALL have a primary action that runs on `Enter` while the result is highlighted. A result MAY additionally have a secondary action exposed via a keyboard chord:

- `⌘E` — secondary action for track and waypoint results (Export).
- `⌘R` — secondary action for map results (Reveal in Finder).

Arrow keys SHALL move the highlight up and down through the visible results. `Tab` SHALL jump to the first result of the next group.

The palette SHALL adapt to context: when the user is at the cold-start surface (no active project), the groups that depend on active-project state (Open map, Find track / waypoint, Project actions) SHALL be hidden. Only Switch project, Settings, and Recent files SHALL render in that context.

#### Scenario: Cmd-K opens the palette from any focus state

- **WHEN** the user presses `⌘K` (macOS) or `Ctrl+K` (Windows / Linux) AND focus is anywhere in the application except inside the palette itself
- **THEN** the palette opens with its search input focused, and the previously-focused element does not receive the keystroke as text input

#### Scenario: Search filters across all groups

- **WHEN** the palette is open AND the user types a query in the search input
- **THEN** each group filters its results against the query AND empty groups disappear entirely AND the highlight defaults to the first visible result

#### Scenario: Enter runs primary action

- **WHEN** the palette is open AND a result is highlighted AND the user presses `Enter`
- **THEN** the result's primary action runs (open map / switch project / focus track / run project action / open theme picker / open recent file) AND the palette closes after the action resolves

#### Scenario: Cmd-E exports the highlighted track or waypoint

- **WHEN** the palette is open AND a track or waypoint result is highlighted AND the user presses `⌘E` (`Ctrl+E` on Windows / Linux)
- **THEN** the secondary "Export" action runs for that result AND the palette closes after the action resolves

#### Scenario: Cmd-R reveals the highlighted map file

- **WHEN** the palette is open AND a map result is highlighted AND the user presses `⌘R` (`Ctrl+R` on Windows / Linux)
- **THEN** the secondary "Reveal in Finder" action runs for that result AND the palette closes after the action resolves

#### Scenario: Esc closes the palette

- **WHEN** the palette is open AND the user presses `Esc`
- **THEN** the palette closes AND focus returns to the element that was focused before the palette opened

#### Scenario: Click-outside closes the palette

- **WHEN** the palette is open AND the user clicks outside the palette content area
- **THEN** the palette closes AND no action runs

#### Scenario: Cold-start surface hides project-dependent groups

- **WHEN** the user is at the cold-start surface (no active project) AND presses `⌘K`
- **THEN** the palette opens AND shows only Switch project, Settings, and Recent files groups; the Open map, Find track / waypoint, and Project actions groups are not rendered

### Requirement: Recent files are persisted in localStorage under a versioned key

The system SHALL provide a helper `src/lib/recentFiles.ts` that maintains a localStorage entry keyed `ozi:recent-files:v1`. The entry's value SHALL be a JSON array of records `{ projectSlug: string, mapPath: string, mapName: string, openedAt: number }`, capped at the most recent 8 records.

On every successful resolution of `openSelectedMap`, the helper SHALL append a new record, deduplicating by `mapPath` (an existing entry with the same path SHALL move to the front of the list rather than appearing twice). When the cap is exceeded, the oldest record SHALL be dropped.

The helper SHALL handle the following failure modes silently (logged via `console.warn`, no user-facing error):

- localStorage quota exceeded — drop the oldest record, retry once; if still failing, leave the list unchanged.
- Stored JSON unparseable — treat as empty list, overwrite on next successful write.

The Cmd-K palette's Recent files group SHALL read from this helper. The helper's key SHALL be versioned (`:v1`) so that any future schema change writes to a new key (`:v2`) rather than silently invalidating existing data.

The system SHALL NOT store recent-files data in the Rust session file. Recent files are a UX convenience, per-machine, and SHALL NOT be conflated with domain state.

#### Scenario: Opening a map appends to recent files

- **WHEN** the user opens a map via any path (bundle-loader sheet, Library, or palette) AND `openSelectedMap` resolves successfully
- **THEN** a record `{ projectSlug, mapPath, mapName, openedAt }` is appended to `ozi:recent-files:v1` AND if a record with the same `mapPath` already existed, it is moved to the front rather than duplicated

#### Scenario: Recent files survive restart

- **WHEN** the user opens at least one map in a session AND restarts the application
- **THEN** the palette's Recent files group renders the previously-opened maps in most-recent-first order from `ozi:recent-files:v1`

#### Scenario: Cap is enforced at 8 records

- **WHEN** the user opens a 9th distinct map
- **THEN** `ozi:recent-files:v1` contains exactly 8 records AND the oldest is no longer present

#### Scenario: Unparseable storage falls back to empty

- **WHEN** the localStorage entry `ozi:recent-files:v1` contains a non-JSON or schema-invalid value
- **THEN** the helper treats the recent files list as empty AND overwrites the entry on the next successful map open AND a `console.warn` is emitted

#### Scenario: Recent files do not enter the session file

- **WHEN** the Rust session file (per `project-persistence`) is inspected after any sequence of map opens
- **THEN** the file contains no recent-files data; that information lives exclusively in localStorage
