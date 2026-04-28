# Conventions

Cross-cutting conventions that span Rust and TypeScript code. These are easy to get wrong because they are not enforced by types.

## Coordinate order

The codebase mixes two orderings on purpose. Pick the right one for the boundary you are at.

| Boundary | Order | Example |
|----------|-------|---------|
| Domain types (`TrackPoint`, `Waypoint`) | separate `latitude` / `longitude` fields | `point.latitude()`, `point.longitude()` |
| Tauri IPC mutation payloads (`move_track_point`, `move_waypoint`, `insert_track_point`) | `position: [lat, lon]` | `[55.75, 37.61]` |
| GeoJSON output (`get_tracks_geojson`) | `[lon, lat]` per the spec | `[37.61, 55.75]` |
| MapLibre `LngLat` | `[lon, lat]` | `new LngLat(lon, lat)` |
| `add_waypoint` IPC | named `lat`, `lon` (not an array) | — |

Rule of thumb: anything that crosses into MapLibre or GeoJSON is `lon, lat`. Anything that is a custom IPC array is `lat, lon`. Always use the named `lat` / `lon` keys when available; arrays exist only where the IPC contract already settled on them.

## Tile URL formats

Both MapLibre custom protocols use `<protocol>://` URLs that the Rust handler parses verbatim.

| Protocol | URL format | Parsed by |
|----------|-----------|-----------|
| `sqlite://` | `sqlite://<abs-path-to-.sqlitedb>/<base_zoom>/{z}/{x}/{y}` | `src/lib/maplibre/tile-url.ts::parseSqliteTileUrl` |
| `ozi://` | `ozi://<abs-path-to-.map>/{z}/{x}/{y}` | inline regex in `ozi-protocol.ts` |

`base_zoom` is embedded in the URL because MBTiles store zoom inverted from MapLibre's `{z}` and the inversion needs the bundle's max zoom: `db_z = db_min + (base_zoom - web_z)`.

The Rust-side OZI handler (`get_ozi_tile_projected`) accepts `(map_path, tx, ty, tz)` in MapLibre tile coordinates and does the OZF2-to-Web-Mercator reprojection itself. No client-side coordinate math.

Empty responses (`new ArrayBuffer(0)`) are the canonical "tile is missing/out-of-bounds" reply — both protocols use them so MapLibre stops re-requesting at coverage edges.

## Color encodings

| Surface | Encoding |
|---------|----------|
| Domain `TrackStyle::color` | `[u8; 4]` RGBA bytes |
| `set_track_color` IPC | `[u8; 4]` RGBA |
| GeoJSON `color` property | `rgba(r,g,b,a)` CSS string built in `get_tracks_geojson` (alpha = `style_alpha/255 * style_opacity`) |
| GPX export | Garmin `<extensions>` color (named or RGB) |
| PLT export | COLORREF, **BGR** order (`(r<<16) | (g<<8) | b` is wrong — the format swaps it; see `infrastructure/export/plt.rs`) |

## Identifiers

- All entity IDs are `u64` newtypes with `#[serde(transparent)]` (ADR-0014). They serialize as bare integers.
- TypeScript uses `bigint` for IDs because numeric `u64` overflows JS `number`. All IPC wrappers and store types use `bigint`.
- Convert with `BigInt(id)` (TS) and `LayerId::new(value)` / `.value()` (Rust).
- IDs are *not* globally unique. They are unique within their parent collection. The next-ID strategy is `max + 1`.

## Naming

- Rust modules and files: `snake_case`.
- Rust types: `PascalCase`. ID newtypes get the suffix `Id` (`LayerId`, not `LayerID`).
- TypeScript types matching Rust DTOs: identical name and field casing (`AppStateDto`, `LayerSummaryDto`). Field casing stays `snake_case` to match serde output.
- Svelte stores: `camelCase`, no `$` prefix in declaration (Svelte 5).
- Tauri command names: `snake_case` in Rust + IPC; matching `camelCase` wrapper in `src/lib/api.ts`. Tauri auto-converts argument names from `camelCase` (frontend) to `snake_case` (Rust handler) — keep this in mind when invoking new commands.

## Track-name validation (OK-standard)

Pattern: `^\d{8}_.*\S.*$` (Unicode flag).
- Eight digits, underscore, non-whitespace remainder.
- The eight digits are a *date hint* but **not validated as a calendar date**. `99999999_X` passes.
- Cyrillic and other Unicode in the callsign are allowed.
- Validation is **warning-only** in the UI. Backend never rejects names. Rename, save, export are unblocked. ADR-0019.

Defined in `src/lib/track-names.ts`; re-validate there if you need parity.

## Logging

- Use `tracing::info!` / `tracing::warn!` / `tracing::error!`. Never `println!` / `eprintln!` outside of build scripts. ADR-0009.
- `AppState::push_diagnostic` is the only function that both records to the in-app console ring buffer and emits via `tracing`. UI-visible messages should go through it.
- Filter at runtime via `RUST_LOG=debug` or per-target (`RUST_LOG=ozi_rs=trace`).

## Concurrency

- No async runtime. Background work uses `std::thread::spawn` and clones what it needs. ADR-0011.
- Long-running ops emit Tauri events (`download-progress`, `bundle-progress`, `projects-chunk`, `state-changed`) and write back via `apply_*` methods on `AppState` after re-locking the mutex.
- Frontend does not poll. It listens to `state-changed` and re-fetches via `appState.refresh()`.
- The shared lock is a `std::sync::Mutex` wrapped in `Arc`; do not hold it across `.await` (there is no await) or across blocking I/O. Use the `lock_app_state` helper.

## Tauri permissions

`src-tauri/capabilities/default.json` lists the IPC plugins allowed for the `main` and `bundles` webviews. Adding a new Tauri plugin or permission requires editing this file plus updating the capability `windows` list if you introduce another webview label.

## Frontend file boundaries

- Components must call backend only through `src/lib/api.ts` wrappers. No `invoke()` in `.svelte` files.
- TypeScript DTOs in `src/lib/types.ts` must stay in sync with Rust DTOs in `src-tauri/src/commands/mod.rs` manually. There is no codegen. Update both halves in the same change.
- UI-only state lives in `src/lib/stores.ts` and is not persisted by the backend. See `frontend-architecture.md` for the synced/UI-only split.

## Docs hygiene

When backend capability and UI exposure diverge, the public docs must say so explicitly. Use the language from ADR-0019: "backend supports", "UI surfaces", "planned", "not implemented". Update `docs/feature-status.md` whenever this mapping changes.
