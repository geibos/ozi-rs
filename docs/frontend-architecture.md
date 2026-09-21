# Frontend Architecture

> Last verified against code: 2026-07-15

## Stack

| Layer | Technology |
|-------|-----------|
| Desktop shell | Tauri 2 |
| UI framework | Svelte 5 + SvelteKit (`adapter-static`) |
| UI kit | shadcn-svelte primitives (`src/lib/components/ui/`) over bits-ui |
| Map rendering | MapLibre GL 4 |
| Theming | Catppuccin palette + semantic-token layer; Tailwind 4 utilities |
| Toasts / tooltips | `svelte-sonner`, bits-ui Tooltip — both hosted globally in `routes/+layout.svelte` |
| Build | Vite 6 |
| Testing | Vitest |

## Components

Feature components live under `src/components/` and consume primitives from
`src/lib/components/ui/` (see `migrate-panels-to-shadcn`). Component-local
`<style>` blocks are removed; static chrome moves to Tailwind utility
classes that read semantic tokens (`bg-popover`, `text-popover-foreground`,
`border-border`, …). Dynamic values sourced from the domain (track colour,
line width, MapLibre marker DOM) stay on inline `style=` or `:global()`
rules.

The floating-panels UI (`Sidebar.svelte`, `TracksPanel.svelte`, `TrackPointsPanel.svelte`,
`WaypointsPanel.svelte`, `SimplifyPanel.svelte`) was **removed on 2026-05-26** and replaced
by a rail-based workspace shell. The current tree:

| Component | Purpose |
|-----------|---------|
| `WorkspaceShell.svelte` | Workspace layout grid: Library rail (left), map canvas (center), Inspector rail (right), status bar (bottom). Writes canvas inset CSS variables so `MapView` (mounted in the layout) aligns with the free canvas area. Hosts the Cmd-K trigger |
| `LibraryRail.svelte` | Left rail. Three persistent shadcn `Tabs`: Maps / Tracks / Waypoints; active tab held in the `libraryActiveTab` store |
| `library/MapsTab.svelte` | Bundle map list: switch active map (`openSelectedMap`), reveal in Finder (`revealBundle`), per-map download progress; opens the bundle loader via `bundleLoaderOpen` |
| `library/TracksTab.svelte` | Track rows (via `LibraryRow`): visibility toggle, inline rename with OK-standard warning, colour swatch `Popover` (binds to `TrackStyle.color` RGBA via `setTrackColor` — never to a theme token), ⋯ menu with Export GPX, Export PLT, Set line width, Simplify… (debounced live preview), Delete; Import GPX / Import PLT buttons; “Create track” drawing toggle |
| `library/WaypointsTab.svelte` | Waypoint rows: visibility (Show/Hide), double-click rename, symbol via `SymbolPicker`, ⋯ menu with Export WPT and Delete; “Add waypoint” toggle |
| `library/LibraryRow.svelte` | Shared selectable row primitive for the three tabs |
| `InspectorRail.svelte` | Right rail; renders one inspector based on selection: `$selectedTrack` → `TrackInspector`, `$selectedWaypointId` → `WaypointInspector`, `$selectedMapInfo` → `MapInspector` |
| `inspector/TrackInspector.svelte` | Track statistics (distance/duration/points), line-width input, visibility, Export GPX/PLT, Simplify, Delete; embeds `TrackSegmentsTable` |
| `inspector/TrackSegmentsTable.svelte` | Paged per-segment point rows in a `Table`; click selects a point (`selectedPointId`); Edit Mode toggle |
| `inspector/WaypointInspector.svelte` | Waypoint name input, lat/lon, symbol picker, Move on map, visibility, Export WPT, Delete |
| `inspector/MapInspector.svelte` | Active map metadata (projection, datum, tile source) |
| `CommandPalette.svelte` | Cmd/Ctrl+K palette (cmdk): open/save project, switch project/map, undo/redo, find track/waypoint, exports, recent files, settings stubs. With the palette open, Cmd/Ctrl+E exports the highlighted track (GPX) or waypoint layer (WPT); Cmd/Ctrl+R reveals the highlighted map |
| `BundleLoader.svelte` | Project list + map list, download progress, cached badges, bundles-root picker. Rendered by the `/` route on cold start and inside a `Sheet` overlay on `/project` (driven by `bundleLoaderOpen`) |
| `MapView.svelte` | Main map canvas. MapLibre init, track/waypoint rendering, drawing mode, drag editing, FPS counter (F3), context menus. `:global()` rules retained for `.track-point-marker` / `.waypoint-marker` because MapLibre creates those DOM elements outside this template |
| `SymbolPicker.svelte` | `Popover` grid of domain emoji symbols — the `SYMBOLS` array stays as-is |
| `ThemePicker.svelte` | Theme selector (Native — Auto, Catppuccin Auto / Latte / Frappé / Macchiato / Mocha). **Currently not mounted anywhere** — the workspace redesign removed its host; theme switching is not reachable from the UI |
| `Console.svelte` | Backtick-toggled diagnostics overlay |

The `<Toaster />` host and a single `Tooltip.Provider` live in
`src/routes/+layout.svelte`; panels surface user-visible failures via
`toast.error(...)` from `svelte-sonner` rather than ad-hoc `alert()`.

Routes (`src/routes/`):

| Route | File | Purpose |
|-------|------|---------|
| `/` | `+page.svelte` | Bundle loader (hosts `BundleLoader.svelte`). Lands here when no active map; a module-level cold-start guard redirects to `/project` via client-side `onMount(goto)` only on the first mount when an active map is restored from session — later visits to `/` are intentional and do not bounce back |
| `/project` | `project/+page.svelte` | Workspace: `WorkspaceShell` with `LibraryRail` + `InspectorRail`, plus a `Sheet`-hosted `BundleLoader` (opened via the `bundleLoaderOpen` store). Redirects back to `/` if the active map is cleared. `MapView` is mounted once in `+layout.svelte` and shown only on this route |

## Routing & layout

`src/routes/+layout.svelte` is the single host for global UI surfaces:

- Per-route `+page.svelte` content renders through `{@render children?.()}`.
- `MapView` is mounted **once** at layout level so navigating between `/` and `/project`
  does not destroy or re-create the MapLibre map; visibility is toggled by route
  (visible on `/project`, `display: none` on `/`).
- `<Console />` (backtick-toggled developer overlay), `<CommandPalette />` (opened by a
  global Cmd/Ctrl+K keydown handler in the layout), `<Toaster />` (`svelte-sonner`), and a
  single `<Tooltip.Provider delayDuration={300}>` wrap the whole tree — feature components
  never mount their own provider.
- `applyStoredTheme()` runs synchronously at layout module top, and `installAutoThemeListener()`
  is registered on mount so the `prefers-color-scheme` media query keeps Auto mode in sync.

## State Management

All stores are in `src/lib/stores.ts`.

### Backend-Synced State

| Store | Type | Source |
|-------|------|--------|
| `appState` | `Writable<AppStateDto>` | `getAppState()` IPC call |
| `busy`, `status`, `diagnostics` | Derived | From `appState` |
| `currentProject`, `activeMap` | Derived | From `appState` |
| `projectsStore` | `Writable<LizaProjectSummaryDto[]>` | Streaming `projects-chunk` events |
| `downloadProgress` | `Writable<Map>` | `download-progress` events |

`AppStateDto` also exposes `track_layers` and `waypoint_layers` summaries so the UI can
select the active layer for existing workflows without implementing full layer management.

### UI-Only State (not persisted to backend)

| Store | Type | Purpose |
|-------|------|---------|
| `consoleOpen` | `Writable<boolean>` | Console overlay visibility |
| `libraryActiveTab` | Writable | Active Library rail tab (maps / tracks / waypoints) |
| `bundleLoaderOpen` | `Writable<boolean>` | Bundle-loader `Sheet` visibility on `/project` |
| `inspectorOpen` | `Writable<boolean>` | Inspector rail visibility |
| `commandPaletteOpen` | `Writable<boolean>` | Cmd-K palette visibility |
| `editModeActive` | `Writable<boolean>` | Map point drag editing |
| `addWaypointMode` | `Writable<boolean>` | Click-to-add waypoint mode |
| `drawingModeActive`, `drawingFinishRequested` | `Writable<boolean>` | Track drawing mode / finish signal |
| `activeTrackLayerId`, `activeWaypointLayerId` | `Writable<bigint \| null>` | Active-layer selection for current track and waypoint workflows; synchronized from backend layer summaries |
| `drawingTrackId`, `drawingSegmentId`, `drawingTrackLayerId`, `drawingPointCount` | Writable | Drawing session state; drawing captures the active track layer at creation time |
| `selectedTrack` | `Writable<{layerId, trackId}>` | Currently selected track (drives `TrackInspector`) |
| `selectedWaypointId` | `Writable<bigint \| null>` | Selected waypoint (drives `WaypointInspector`) |
| `selectedPointId` | Writable | Selected track point in `TrackSegmentsTable` |
| `selectedMapInfo` | Writable | Selected map (drives `MapInspector`) |
| `simplifyState` | `Writable<{active, layerId, trackId, tolerance, preview}>` | Simplification session |
| `selectedTheme` | `Writable<string>` | Theme name, persisted to localStorage (default `native-auto`) |

## API Layer

`src/lib/api.ts` provides typed wrappers around `invoke()`. Components must never call `invoke()` directly.

Categories:
- **App state**: `getAppState()`, `loadProjects()`, `loadProject(slug)`
- **File I/O**: `saveProject(path)`, `loadProjectFile(path)`, `importGpx(path)`, `importPlt(path)`
- **Track mutations**: `renameTrack()`, `setTrackColor()`, `setTrackLineWidth()`, `moveTrackPoint()`, `deleteTrackPoint()`, `insertTrackPoint()`, `splitSegment()`, `joinSegments()`, `deleteTrack()`, `createEmptyTrack()`, `simplifyTrack()`
- **Waypoint mutations**: `addWaypoint()`, `moveWaypoint()`, `deleteWaypoint()`, `renameWaypoint()`, `setWaypointSymbol()`
- **Export**: `getTrackExportDefaultPath(trackName, extension)`, `exportGpx(layerId, path)`, `exportTrackPlt(layerId, trackId, path)`, `exportWptWaypoints(layerId, path)`, `getWptExportDefaultPath(layerId)`
- **History**: `undo()`, `redo()`
- **Maps**: `openSelectedMap()`, `openLocalBundle()`, `setBundlesRoot()`, `revealBundle()`, `cancelDownload()`, `getOziMetadata()`

## Tile Protocols

MapLibre uses `addProtocol()` to register custom tile sources:

### `sqlite://` (MBTiles)

File: `src/lib/maplibre/sqlite-protocol.ts`

URL format: `sqlite://<abs-path>/<base_zoom>/{z}/{x}/{y}`

The handler calls `getSqliteTile()` backend command which queries `SELECT image FROM tiles WHERE x=?, y=?, z=?`. Zoom levels are inverted: `db_z = db_min + (base_zoom - web_z)`.

### `ozi://` (OZF2 Raster)

File: `src/lib/maplibre/ozi-protocol.ts`

URL format: `ozi://<abs-path-to-.map>/{z}/{x}/{y}`

The handler calls `getOziTileProjected()` which reprojects OZF2 raster tiles to Web Mercator in the Rust backend, returning 256×256 PNG. No client-side coordinate math.

## Track Rendering

File: `src/lib/maplibre/tracks-layer.ts`

Tracks are rendered as a MapLibre GeoJSON source with two layers:
- `tracks-lines` — LineString features, color and width from feature properties
- `tracks-labels` — Symbol layer showing track names along lines

Both layers are filtered by a `visible` property. Data is fetched via `getTracksGeojson()` and updated when `state-changed` fires.

## Theme System

File: `src/lib/theme.ts`

Two coexisting CSS custom-property layers are written to the root element on every flavour change:

1. **Palette layer** — hex variables `--ctp-<colour>` for every named colour in `@catppuccin/palette` (e.g. `--ctp-base`, `--ctp-red`, `--ctp-mauve`).
2. **Semantic layer** — HSL-triplet variables (no `hsl()` wrapper) consumed by Tailwind utilities and shadcn-svelte primitives: `--background`, `--foreground`, `--card`, `--card-foreground`, `--popover`, `--popover-foreground`, `--primary`, `--primary-foreground`, `--secondary`, `--secondary-foreground`, `--muted`, `--muted-foreground`, `--accent`, `--accent-foreground`, `--destructive`, `--destructive-foreground`, `--border`, `--input`, `--ring`.

Semantic values come from two mapping tables — `SEMANTIC_MAP_LIGHT` for Latte and `SEMANTIC_MAP_DARK` for Frappé / Macchiato / Mocha — so surface semantics stay correct in both light and dark modes (e.g. `--popover` resolves to `base` in light and `surface0` in dark). The root element also carries the `dark` class whenever the resolved flavour is not Latte so Tailwind's `dark:` variant utilities apply.

Themes: Native — Auto (fresh-install default) plus the Catppuccin pack — Auto (follows OS), Latte, Frappé, Macchiato, Mocha. Auto modes listen to `prefers-color-scheme: dark` and re-apply both layers on every change. Selection is persisted to `localStorage["theme"]`. Note: `ThemePicker.svelte` is currently not mounted anywhere, so the theme cannot be changed from the UI (see `docs/feature-status.md`).

Migrated panels almost exclusively read the **semantic layer** through Tailwind utility classes (`bg-popover`, `text-card-foreground`, `border-border`, …). Direct `--ctp-*` reads survive only where load-bearing for MapLibre marker DOM (see the `:global()` rules in `MapView.svelte`) and where palette colour is needed without a semantic analogue (the FPS counter overlay uses `text-emerald-400` against `bg-black/55`, which is Quake-style convention and intentionally outside theme).

## Key Interaction Modes

### Drawing Mode

1. User chooses an active track layer, then clicks "Create Track" → `createEmptyTrack()` → sets `drawingModeActive` and captures `drawingTrackLayerId`
2. Map pan and double-click zoom disabled
3. Each click → `insertTrackPoint()` → blue preview line updates
4. Double-click or Enter → finish drawing
5. Escape → undo all drawing operations (createEmptyTrack + all insertions)

### Edit Mode

1. User selects track → `getTrackDetail()` → point markers rendered
2. Toggle "Edit Mode" → crosshair cursor, map pan disabled
3. Drag points → `moveTrackPoint()` (coalesced undo via `apply_or_merge`)
4. Right-click → context menu: Delete Point, Insert Point After

### Simplification Preview

1. `TracksTab` ⋯ menu → Simplify… (or `TrackInspector`) → `simplifyState.active = true`
2. Slider change (debounced 300ms) → `getSimplifiedPreview()` → stats + orange overlay
3. Confirm → `simplifyTrack()` | Cancel → clear preview

### Waypoint Placement

1. User chooses an active waypoint layer, then toggles "Add Waypoint" → `addWaypointMode = true`
2. Click map → `addWaypoint(activeLayerId, lat, lon, defaultName)`
3. Drag marker → `moveWaypoint(activeLayerId, waypointId, lat, lon)`
4. Symbol picker in `WaypointsTab` / `WaypointInspector` → `setWaypointSymbol()`

## Event-Driven Updates

The frontend uses Tauri events for real-time backend communication:

| Event | Handler |
|-------|---------|
| `state-changed` | `appState.refresh()` → derived stores update |
| `download-progress` | Update `downloadProgress` map |
| `projects-chunk` | Append to `projectsStore` (deduplication) |
| `bundle-progress` | Update bundle loading progress bar |

## Keyboard Shortcuts

| Key | Action |
|-----|--------|
| Cmd/Ctrl+K | Open command palette |
| Cmd/Ctrl+Z | Undo (`+Shift` to redo) — `handleGlobalKeydown`, `src/routes/+layout.svelte` |
| Cmd/Ctrl+S | Save the project |
| Cmd/Ctrl+E | (palette open) Export highlighted track (GPX) or waypoint layer (WPT) |
| Cmd/Ctrl+R | (palette open) Reveal highlighted map in Finder |
| Enter | Finish drawing |
| Escape | Cancel drawing (undo all) / close map context menu |
| `` ` `` | Toggle developer console |
| F3 | Toggle FPS counter |

Undo and redo are also on the command palette (Undo / Redo entries).
