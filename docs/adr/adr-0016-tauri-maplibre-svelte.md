# ADR-0016: Migrate UI to Tauri 2 + MapLibre GL + Svelte 5

- Status: accepted
- Date: 2026-03-30
- Supersedes: ADR-0003 (eframe + egui + walkers)

## Context

ADR-0003 selected eframe + egui + walkers as the UI stack and explicitly marked the
decision as "under review" due to known egui limitations. By Phase 6 the following
friction points were confirmed in practice:

1. **First-drag jitter** — egui immediate-mode redraw causes visible lag on the first
   pointer movement after a pause; not fully fixable without hacking the frame loop.
2. **Drag-based point editing (Phase 7)** — moving individual track points on a raster
   map requires manual hit-testing, coordinate projection, and drag-state management
   that egui does not provide. Equivalent functionality in MapLibre GL is ~10 lines.
3. **Native multi-window layout** — egui Viewport API exists but is immature; the
   Tracks window request revealed real limitations.
4. **catppuccin-egui** — did not support egui 0.34; theme was implemented manually as
   a workaround (see ADR-0003).
5. **Immediate mode CPU cost** — for a mostly-static map editor this is waste.
6. **No tile map widget ecosystem** — walkers is the only egui tile map widget; it
   has limited map interaction support.

MapLibre GL JS solves the map-specific problems completely: raster tile sources,
GeoJSON track layers, drag markers, custom protocols for local tile data — all
first-class features.

## Decision

Replace `src/ui/` and `src/main.rs` with **Tauri 2** (desktop shell) +
**Svelte 5** (UI components) + **MapLibre GL 4** (map rendering).

The domain, application, and infrastructure layers (`src/domain/`,
`src/application/`, `src/infrastructure/`) are **not changed**.

### Project Structure

```
ozi-rs/
  src-tauri/               ← Rust backend (Tauri)
    src/
      domain/              ← unchanged (moved from src/)
      application/         ← unchanged (moved from src/)
      infrastructure/      ← unchanged (moved from src/)
      commands/            ← new: Tauri IPC command handlers
      lib.rs               ← registers Tauri commands and state
      main.rs              ← Tauri entry point
    Cargo.toml
    build.rs
    tauri.conf.json
    capabilities/
  src/                     ← Svelte frontend
    lib/
      api.ts               ← typed invoke() wrappers
      types.ts             ← TypeScript DTOs mirroring Rust structs
      stores.ts            ← Svelte stores for app state
      maplibre/
        sqlite-protocol.ts ← custom addProtocol for SQLite tiles
        ozi-protocol.ts    ← custom addProtocol for OZF2 tiles
        tracks-layer.ts    ← GeoJSON helpers for track rendering
    components/
      MapView.svelte
      Sidebar.svelte
      TracksPanel.svelte
      Console.svelte
      ThemePicker.svelte
    App.svelte
    main.ts
    app.css
  index.html
  package.json
  vite.config.ts
  svelte.config.js
  tsconfig.json
```

### IPC Model

**Commands (frontend → backend):**

| Command | Purpose |
|---------|---------|
| `load_projects` | Fetch LizaAlert project list |
| `load_project` | Open a project by slug |
| `open_selected_map` | Activate a map package |
| `open_local_bundle` | Open bundle from local path |
| `set_bundles_root` | Change bundle storage directory |
| `get_app_state` | Snapshot of serializable frontend state |
| `get_tracks_geojson` | Track layers as GeoJSON FeatureCollection |
| `import_gpx` | Import GPX file |
| `import_plt` | Import PLT file |
| `export_gpx` | Export track layer to GPX |
| `apply_command` | Apply a ProjectCommand (edit) |
| `undo` / `redo` | Undo/redo |
| `save_project` | Save project to file |
| `load_project_file` | Load project from file |
| `get_sqlite_tile` | Return PNG tile bytes from SQLite bundle |
| `get_ozi_tile` | Return PNG tile bytes from OZF2 map |
| `get_ozi_metadata` | Return georeference + tile grid info |

**Events (backend → frontend):**

| Event | Payload |
|-------|---------|
| `download-progress` | `{ downloaded_bytes, total_bytes? }` |
| `diagnostic` | `{ level, message }` |
| `state-changed` | `{}` (frontend re-fetches state) |

### Tile Delivery

Local tiles (SQLite and OZF2) are delivered as binary blobs via Tauri IPC:

```rust
#[tauri::command]
fn get_sqlite_tile(path: String, z: u32, x: u32, y: u32)
    -> tauri::ipc::Response { ... }
```

MapLibre uses a custom protocol handler (`addProtocol`) that calls the Tauri command
and returns an `ArrayBuffer`. This avoids running a local HTTP server.

### Catppuccin Theme

Implemented via CSS custom properties using `@catppuccin/palette`. Auto mode reads
`window.matchMedia('(prefers-color-scheme: dark)')` and updates on system change.
Theme selection persisted in `localStorage`.

## Consequences

### Positive

- Drag-based track point editing (Phase 7) becomes straightforward via MapLibre events
- No more first-drag jitter
- Raster tile rendering, GeoJSON overlays, zoom/pan — all handled by MapLibre
- True native OS windows for any floating panel (Tauri `WebviewWindow`)
- catppuccin-palette npm package works perfectly; no manual color maintenance
- Async-first: Tauri commands can be `async`, background work emits events naturally
- Domain/application/infrastructure layers untouched

### Negative

- TypeScript frontend code alongside Rust; contributors need both
- Tauri IPC adds one serialization boundary that egui did not have
- OZF2 tile projection must be mapped to MapLibre's coordinate system
- Bundle size increases (~10MB Tauri shell + ~2MB MapLibre)
- `walkers` and `eframe` are completely removed; no fallback

## Rejected Alternatives

### iced

Rejected because it has no mature tile map widget. A full tile renderer would need
to be written from scratch on wgpu, representing weeks of work to reach feature parity.

### Keep egui and fix individual pain points

Rejected because the drag-based editing requirement in Phase 7 is a fundamental
mismatch with immediate-mode architecture, not a fixable edge case.

### gtk4-rs + Shumate

Rejected because of macOS runtime requirements and a complex API surface.
