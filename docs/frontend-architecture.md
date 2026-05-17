# Frontend Architecture

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

| Component | Primitives | Native control retained | Purpose |
|-----------|-----------|------------------------|---------|
| `MapView.svelte` | wrapper-only — Tailwind utility wrapper around MapLibre canvas; `:global()` rules retained for `.track-point-marker` / `.waypoint-marker` because MapLibre creates those DOM elements outside this template | none | Main map canvas. MapLibre init, track/waypoint rendering, drawing mode, drag editing, FPS counter, context menus. Internals kept under `@ts-nocheck` and explicitly out of scope of `migrate-panels-to-shadcn` |
| `Sidebar.svelte` | `Button`, `Select`, `Separator`, `ScrollArea`, `Tooltip` | none | Left sidebar (w-56). Project controls, active layer selectors, import/export, undo/redo, mode toggles. Theme picker mounted in header. NOTE: `Tabs.Root` was deliberately skipped — the user requires Tracks / Waypoints / Track Points to remain independent floating panels that can be visible simultaneously, incompatible with Tabs' one-active-tab semantics |
| `TracksPanel.svelte` | `Button`, `Separator`, `Tooltip`; independent floating window | `<input type="color">`, `<input type="range">` | Track list. Visibility, rename, native colour swatch (binds to `TrackStyle.color` RGBA via `setTrackColor` — never to a theme token), line-width range, GPX/PLT export, Simplify launcher |
| `TrackPointsPanel.svelte` | `Table`, `ScrollArea`, `Button`, `Tooltip` | none | Per-segment point rows in a `Table`; `data-state="selected"` rings the active row; Edit Mode toggle in header |
| `WaypointsPanel.svelte` | `Button`, `Separator`, `Tooltip`, `Dialog` (delete confirm) | none today (waypoint colour not yet exposed; same D6 rule will apply if added) | Waypoint list. Visibility checkbox, rename, symbol via SymbolPicker, delete via Dialog confirm, WPT export in header |
| `SimplifyPanel.svelte` | `Slider`, `Switch`, `Label`, `Button` | none | Douglas–Peucker tolerance `Slider` (1–1000m), `Switch` for live preview, stats chip, `Button` (default/outline) confirm/cancel; debounced preview effect unchanged |
| `SymbolPicker.svelte` | `Popover`, `Button` (via `buttonVariants`), `Tooltip` | none | 5-col grid of domain emoji symbols — the `SYMBOLS` array stays as-is |
| `ThemePicker.svelte` | `Select` | none | Catppuccin flavour selector (Auto / Latte / Frappé / Macchiato / Mocha) |
| `Console.svelte` | `Card`, `ScrollArea`, `Button` (close) | none | Top-fixed `Card` overlay for diagnostics; `ScrollArea` viewport powers autoscroll-on-new-message |

The `<Toaster />` host and a single `Tooltip.Provider` live in
`src/routes/+layout.svelte`; panels surface user-visible failures via
`toast.error(...)` from `svelte-sonner` rather than ad-hoc `alert()`.

Routes (`src/routes/`):

| Route | File | Purpose |
|-------|------|---------|
| `/` | `+page.svelte` | Bundle loader — project list + map list, download progress, cached badges. Lands here when no active map; redirects to `/project` via client-side `onMount(goto)` when an active map is restored from session |
| `/project` | `project/+page.svelte` | Workspace: `Sidebar` + open floating panels. `MapView` is mounted once in `+layout.svelte` and shown only on this route |

## Routing & layout

`src/routes/+layout.svelte` is the single host for global UI surfaces:

- The `Sidebar` + per-route `+page.svelte` content render through `{@render children?.()}`.
- `MapView` is mounted **once** at layout level so navigating between `/` and `/project`
  does not destroy or re-create the MapLibre map; visibility is toggled by route
  (visible on `/project`, `display: none` on `/`).
- `<Console />` (backtick-toggled developer overlay), `<Toaster />` (`svelte-sonner`), and a
  single `<Tooltip.Provider delayDuration={300}>` wrap the whole tree — feature panels never
  mount their own provider.
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

Two coexisting CSS custom-property layers are written to the root element on every flavour change:

1. **Palette layer** — hex variables `--ctp-<colour>` for every named colour in `@catppuccin/palette` (e.g. `--ctp-base`, `--ctp-red`, `--ctp-mauve`).
2. **Semantic layer** — HSL-triplet variables (no `hsl()` wrapper) consumed by Tailwind utilities and shadcn-svelte primitives: `--background`, `--foreground`, `--card`, `--card-foreground`, `--popover`, `--popover-foreground`, `--primary`, `--primary-foreground`, `--secondary`, `--secondary-foreground`, `--muted`, `--muted-foreground`, `--accent`, `--accent-foreground`, `--destructive`, `--destructive-foreground`, `--border`, `--input`, `--ring`.

Semantic values come from two mapping tables — `SEMANTIC_MAP_LIGHT` for Latte and `SEMANTIC_MAP_DARK` for Frappé / Macchiato / Mocha — so surface semantics stay correct in both light and dark modes (e.g. `--popover` resolves to `base` in light and `surface0` in dark). The root element also carries the `dark` class whenever the resolved flavour is not Latte so Tailwind's `dark:` variant utilities apply.

Themes: Auto (follows OS), Latte, Frappé, Macchiato, Mocha. Auto mode listens to `prefers-color-scheme: dark` and re-applies both layers on every change. Selection is persisted to `localStorage`.

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
