# Glossary

Terms used in code, docs, and SAR-domain conversation. Mixed Russian/English jargon makes this list practical for new contributors.

## Product / SAR domain

| Term | Meaning |
|------|---------|
| **LizaAlert** (Лиза Алерт) | Russian volunteer search-and-rescue organization. Primary user. Public site: lizaalert.org. |
| **maps.lizaalert.ru** | Source of map bundles and project metadata that ozi-rs downloads. |
| **OK-standard** (ОК-стандарт) | Operational-cartography naming/layout convention used by LizaAlert HQ. The only piece ozi-rs validates is track filenames matching `YYYYMMDD_Callsign`. |
| **Callsign** (Позывной) | Volunteer's radio handle, e.g. `Иванов`, used as track-name suffix. Cyrillic is allowed. |
| **HQ / штаб** | Field headquarters of a search operation. Where ozi-rs is typically run. |
| **SAR** | Search and rescue. |

## Top-level concepts

| Term | Meaning |
|------|---------|
| **Map bundle** | A directory of georeferenced raster maps for one geographic area, downloaded once and reused. May contain `.sqlitedb`, `.map`+`.ozf2`, and a `10-Tracks/` subfolder. ADR-0002. |
| **Bundles root** | Configurable parent directory for all map bundles. Default: `$HOME/Documents/LizaAlert Maps`. |
| **Project** | One SAR operation. Contains track and waypoint layers, references an active map. Saved as `.ozp` (JSON). ADR-0004. |
| **Active map** | The map currently rendered on the canvas. Identified by `ActiveMapSelection`. Persisted in the session file. |
| **Active layer** | The track or waypoint layer currently targeted by import / draw / add-waypoint workflows. UI-only; not persisted. ADR-0019. |
| **10-Tracks/** | Subfolder convention inside a bundle for exported track files. Used as the GPX/PLT export-dialog default when a bundle is active. |

## File formats

| Term | Meaning |
|------|---------|
| **`.ozp`** | ozi-rs project file. JSON, `serde_json::to_string_pretty`. ADR-0004. Custom extension; format is plain JSON. |
| **`.map`** | OziExplorer ASCII metadata file with calibration points and projection. Pairs with `.ozf2` raster. |
| **`.ozf2`** | OziExplorer pyramid raster, proprietary. Decoded by sibling crate `ozf2-rs`. ADR-0006. |
| **OZF4** | Newer OziExplorer raster format. **Not supported.** |
| **`.sqlitedb` / MBTiles** | SQLite-backed tile pyramid used by LizaAlert bundles. Custom zoom semantics; see `commands/tiles.rs`. |
| **`.gpx`** | GPS Exchange Format, XML. Imported via `gpx` crate; exported with Garmin color extension. |
| **`.plt`** | OziExplorer track format, ASCII, often Windows-1251 encoded. OLE date timestamps; COLORREF (BGR) colors. |
| **session.json** | Bounded app session file. Stores last project path + active map reference. Path: `$HOME/Library/Application Support/ozi-rs/session.json` on macOS. |

## Architectural / code terms

| Term | Meaning |
|------|---------|
| **Domain** | Pure entities and validation. No I/O, no GUI. ADR-0001. |
| **Application** | `AppState`, `ProjectCommand`, `CommandStack`, import workflow. Depends on Domain. |
| **Infrastructure** | Format adapters, persistence, LizaAlert API, tile decoding. |
| **Commands layer** | Tauri `#[command]` handlers in `src-tauri/src/commands/`. Thin wrappers around Application. |
| **`ProjectCommand`** | Enum variant representing one undoable edit. Each has an `apply()` and computed `reverse()`. ADR-0017. |
| **`CommandDelta`** | Forward + reverse `ProjectCommand` pair stored in the undo stack. |
| **`apply_or_merge`** | Coalesces consecutive same-target commands (e.g. drag moves) into one undo step. |
| **DTO** | Plain serializable struct used at the IPC boundary in `commands/mod.rs`. Mirrored in `src/lib/types.ts` manually. |
| **Newtype ID** | `LayerId(u64)`, `TrackId(u64)`, etc., with `#[serde(transparent)]`. ADR-0014. |
| **`AppState`** | Root mutable state. Lives behind `Arc<Mutex<_>>` (`SharedState`) on the Tauri side. |

## UI terms

| Term | Meaning |
|------|---------|
| **Drawing mode** | Click-to-add-points track creation. Captures the active track layer at start. ESC undoes the whole session. |
| **Edit mode** | Drag track points on the map. Drag deltas coalesce into one undo step. |
| **Add-waypoint mode** | Toggle that turns map clicks into `addWaypoint` calls. |
| **Simplification preview** | Live Douglas-Peucker overlay before committing. Confirm → `simplify_track`; cancel → drop preview. |
| **Bundle loader** | Secondary Tauri webview (`/?view=bundles`) for browsing/downloading LizaAlert maps. Pre-created hidden at startup. |
| **Console** | Bottom overlay showing `tracing` diagnostics. Toggle: backtick. |
| **Catppuccin** | Theme palette. Variants: Auto, Latte, Frappé, Macchiato, Mocha. CSS custom properties `--ctp-*`. |

## Coordinate systems and units

| Term | Meaning |
|------|---------|
| **Lat/Lon** | Geographic coordinates, WGS-84. Stored in domain as `f64` `latitude`/`longitude`. |
| **Web Mercator (EPSG:3857)** | MapLibre's tile coordinate system. OZF2 tiles are reprojected into it server-side (`get_ozi_tile_projected`). |
| **GeoJSON order** | `[longitude, latitude]`. Track geometry uses this. |
| **IPC `position` array** | `[latitude, longitude]`. Used by `move_track_point`, `move_waypoint`, etc. **Not GeoJSON order.** See `docs/conventions.md`. |
| **Base zoom** | The zoom level encoded in MBTiles. Tile URL embeds it because zoom inversion (`db_z = db_min + (base_zoom - web_z)`) is needed at fetch time. |
| **Native zoom** | An OZI map's natural zoom, derived from `.map` metadata. |

## Tooling

| Term | Meaning |
|------|---------|
| **`just`** | Task runner. See `justfile`. |
| **`ozi-rs-mcp`** | Project-local MCP server for native desktop QA (build/launch/screenshot/Appium). 13 tools. See `docs/native-qa-mcp.md`. |
| **Appium Mac2** | Optional UI-automation backend for ozi-rs-mcp. Dependency-gated; missing Appium is an acceptable degraded path. |
| **`RUST_LOG`** | Filter for `tracing-subscriber`. Default `info`. ADR-0009. |
| **`OZI_RS_PROJECT_ROOT`** | Override env var read by `ozi-rs-mcp::config::repo_root()`. |
