# Frontend Architecture

## Stack

| Layer | Technology |
|-------|-----------|
| Desktop shell | Tauri 2 |
| UI framework | Svelte 5 |
| Map rendering | MapLibre GL 4 |
| Theming | Catppuccin via `@catppuccin/palette` |
| Build | Vite 6 |
| Testing | Vitest |

## Components

All components are in `src/components/`.

| Component | Purpose |
|-----------|---------|
| `MapView.svelte` | Main map canvas. Handles MapLibre init, track/waypoint rendering, drawing mode, drag editing, FPS counter, context menus |
| `Sidebar.svelte` | Left sidebar (200px). Project controls, active track/waypoint layer selectors, import/export buttons, undo/redo, mode toggles, theme picker |
| `TracksPanel.svelte` | Track list. Visibility toggle, rename, color, width, export, simplify action |
| `TrackPointsPanel.svelte` | Read-only point list with segment hierarchy. "Edit Mode" toggle for map drag |
| `WaypointsPanel.svelte` | Waypoint list. Delete, rename, symbol picker |
| `SimplifyPanel.svelte` | Douglas-Peucker tolerance slider (1–1000m), live preview stats, confirm/cancel |
| `SymbolPicker.svelte` | Popover with 10 emoji-based waypoint symbols |
| `ThemePicker.svelte` | Dropdown for Catppuccin theme selection |
| `Console.svelte` | Bottom overlay for diagnostics, toggled with backtick |

Views (`src/views/`):

| View | Purpose |
|------|---------|
| `BundleLoaderView.svelte` | Separate window (460×580). Two-column: project list + map list. Download progress, cached badges |

## Secondary Windows

`src/lib/windows.ts` owns frontend helpers for the map bundle loader Tauri webview.
The main window never calls Tauri window APIs directly from components; `Sidebar.svelte` opens
the loader through `openBundleLoader()` when the user clicks **Maps…**.

- `precreateBundleLoader()` creates a hidden `WebviewWindow` with label `bundles`, URL
  `/?view=bundles`, and `visible: false`. `src/main.ts` calls it after mounting the main
  app so the loader is already initialized and can open faster later.
- `openBundleLoader()` first reuses the cached/pre-created window or an existing Tauri window
  with label `bundles`, then calls `show()` and `setFocus()`. If pre-creation failed or no
  existing window is found, it creates the same `bundles` window as a fallback.
- `src/main.ts` reads the query string at startup: `/?view=bundles` mounts
  `BundleLoaderView.svelte`; all other URLs mount the main `App.svelte` and start hidden
  pre-creation for future bundle-loader opens.

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
| `consoleOpen` | `Writable<boolean>` | Console panel visibility |
| `tracksPanelOpen` | `Writable<boolean>` | Tracks panel visibility |
| `waypointsPanelOpen` | `Writable<boolean>` | Waypoints panel visibility |
| `trackPointsPanelOpen` | `Writable<boolean>` | Track points panel visibility |
| `editModeActive` | `Writable<boolean>` | Map point drag editing |
| `addWaypointMode` | `Writable<boolean>` | Click-to-add waypoint mode |
| `drawingModeActive` | `Writable<boolean>` | Track drawing mode |
| `activeTrackLayerId`, `activeWaypointLayerId` | `Writable<bigint \| null>` | Active-layer selection for current track and waypoint workflows; synchronized from backend layer summaries |
| `drawingTrackId`, `drawingSegmentId`, `drawingTrackLayerId`, `drawingPointCount` | Writable | Drawing session state; drawing captures the active track layer at creation time |
| `selectedTrack` | `Writable<{layerId, trackId}>` | Currently selected track |
| `selectedWaypointId` | `Writable<bigint \| null>` | Selected waypoint |
| `simplifyState` | `Writable<{active, layerId, trackId, tolerance, preview}>` | Simplification session |
| `selectedTheme` | `Writable<string>` | Catppuccin theme, persisted to localStorage |

## API Layer

`src/lib/api.ts` provides typed wrappers around `invoke()`. Components must never call `invoke()` directly.

Categories:
- **App state**: `getAppState()`, `loadProjects()`, `loadProject(slug)`
- **File I/O**: `saveProject(path)`, `loadProjectFile(path)`, `importGpx(path)`, `importPlt(path)`
- **Track mutations**: `renameTrack()`, `setTrackColor()`, `setTrackLineWidth()`, `moveTrackPoint()`, `deleteTrackPoint()`, `insertTrackPoint()`, `splitSegment()`, `joinSegments()`, `deleteTrack()`, `createEmptyTrack()`, `simplifyTrack()`
- **Waypoint mutations**: `addWaypoint()`, `moveWaypoint()`, `deleteWaypoint()`, `renameWaypoint()`, `setWaypointSymbol()`
- **Export**: `getTrackExportDefaultPath(trackName, extension)`, `exportGpx(layerId, path)`, `exportTrackPlt(layerId, trackId, path)`
- **History**: `undo()`, `redo()`
- **Maps**: `openSelectedMap()`, `openLocalBundle()`, `getOziMetadata()`

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

Uses `@catppuccin/palette` to set CSS custom properties (`--ctp-*`) on the document root.

Themes: Auto (follows OS), Latte, Frappé, Macchiato, Mocha.

Auto mode listens to `prefers-color-scheme: dark` media query and re-applies on system change. Selection is persisted to `localStorage`.

All components use `--ctp-*` variables for colors. Key tokens:
- Text: `--ctp-text`, `--ctp-subtext0`, `--ctp-subtext1`
- Backgrounds: `--ctp-base`, `--ctp-mantle`, `--ctp-crust`, `--ctp-surface0/1/2`
- Accents: `--ctp-blue`, `--ctp-green`, `--ctp-red`, `--ctp-peach`, `--ctp-yellow`

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

1. Click simplify button → `simplifyState.active = true`
2. Slider change (debounced 300ms) → `getSimplifiedPreview()` → stats + orange overlay
3. Confirm → `simplifyTrack()` | Cancel → clear preview

### Waypoint Placement

1. User chooses an active waypoint layer, then toggles "Add Waypoint" → `addWaypointMode = true`
2. Click map → `addWaypoint(activeLayerId, lat, lon, defaultName)`
3. Drag marker → `moveWaypoint(activeLayerId, waypointId, lat, lon)`
4. Symbol picker in WaypointsPanel → `setWaypointSymbol()`

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
| Ctrl+Z | Undo |
| Ctrl+Y | Redo |
| Enter | Finish drawing |
| Escape | Cancel drawing (undo all) |
| `` ` `` | Toggle developer console |
| F3 | Toggle FPS counter |
