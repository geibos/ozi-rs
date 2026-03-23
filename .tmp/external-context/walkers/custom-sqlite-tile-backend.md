---
source: docs.rs and source code
library: walkers
package: walkers
topic: custom tile source and custom local sqlite tile backend for eframe
fetched: 2026-03-23T17:30:26Z
official_docs: https://docs.rs/walkers/latest/walkers/
---

# walkers 0.52.0: custom tile source vs custom local backend

## Relevant extension points/types/traits

- `walkers::Tiles`
  - Core extension point for anything that can render map tiles.
  - Required API:
    - `fn at(&mut self, tile_id: TileId) -> Option<TilePiece>;`
    - `fn attribution(&self) -> Attribution;`
    - `fn tile_size(&self) -> u32;`
- `walkers::TileId`
  - Requested tile key from the map widget.
  - Public fields:
    - `x: u32`
    - `y: u32`
    - `zoom: u8`
- `walkers::TilePiece`
  - Return value from `Tiles::at`.
  - Constructor:
    - `pub fn new(tile: Tile, uv: Rect) -> Self`
- `walkers::Tile`
  - Raster tiles are stored as `Tile::Raster(TextureHandle)`.
  - Main constructor for image bytes:
    - `pub fn new(image: &[u8], style: &Style, zoom: u8, ctx: &Context) -> Result<Self, TileError>`
- `walkers::Map`
  - Consumes `Option<&mut dyn Tiles>`.
  - Constructor:
    - `pub fn new(tiles: Option<&mut dyn Tiles>, memory: &mut MapMemory, my_position: Position) -> Self`
- `walkers::MapMemory`
  - Must persist between frames.

## What is and is not extensible

- `walkers::sources::TileSource` is only for HTTP-backed `HttpTiles`.
  - Required API:
    - `fn tile_url(&self, tile_id: TileId) -> String;`
    - `fn attribution(&self) -> Attribution;`
  - Optional overrides:
    - `fn tile_size(&self) -> u32 { 256 }`
    - `fn max_zoom(&self) -> u8 { 19 }`
- For an offline SQLite MBTiles-like store, the correct seam is `Tiles`, not `TileSource`.
- `LocalTiles` is the closest built-in example of a non-HTTP backend, but it is directory-based and deprecated.

## Code-shape examples from walkers source

### Minimal map integration

```rust
ui.add(walkers::Map::new(
    Some(&mut my_tiles_backend),
    &mut self.map_memory,
    walkers::lon_lat(lon, lat),
));
```

### How built-in local backend implements `Tiles`

```rust
impl Tiles for LocalTiles {
    fn at(&mut self, tile_id: TileId) -> Option<TilePiece> {
        (0..=tile_id.zoom).rev().find_map(|zoom_candidate| {
            let (donor_tile_id, uv) = interpolate_from_lower_zoom(tile_id, zoom_candidate);
            match self.load_and_cache(donor_tile_id) {
                CachedTexture::Valid(texture) => Some(TilePiece::new(texture.clone(), uv)),
                CachedTexture::Invalid => None,
            }
        })
    }

    fn attribution(&self) -> Attribution { ... }
    fn tile_size(&self) -> u32 { 256 }
}
```

### Converting raw bytes to a walkers tile

```rust
let tile = walkers::Tile::new(&bytes, &walkers::Style::default(), tile_id.zoom, &egui_ctx)?;
let piece = walkers::TilePiece::new(tile, egui::Rect::from_min_max(
    egui::pos2(0.0, 0.0),
    egui::pos2(1.0, 1.0),
));
```

## Minimal implementation strategy for local SQLite tiles

1. Add a custom type like `SqliteTiles` that owns:
   - `rusqlite::Connection` or a small DB wrapper;
   - cloned `egui::Context`;
   - an in-memory cache like `LruCache<TileId, CachedTile>`.
2. Implement `walkers::Tiles` for `SqliteTiles`.
3. In `at(tile_id)`:
   - query `tiles` by `x`, `y`, `z`;
   - ignore `s` initially unless your DB actually needs it;
   - pass the `image` blob to `walkers::Tile::new(..., tile_id.zoom, &egui_ctx)`;
   - return `TilePiece::new(tile, full_uv)`.
4. Match built-in `LocalTiles` behavior for fallback:
   - try exact `z/x/y` first;
   - if missing, walk down lower zooms and use `interpolate_from_lower_zoom(tile_id, donor_zoom)` to crop a parent tile.
5. Read `info(minzoom,maxzoom)` once at startup and expose:
   - `tile_size() -> 256` unless your data says otherwise;
   - internal clamping/fallback behavior based on `minzoom` and `maxzoom`.
6. Keep `SqliteTiles` alive across frames in your `eframe::App`, alongside `MapMemory`.

## Suggested backend shape

```rust
struct SqliteTiles {
    conn: rusqlite::Connection,
    egui_ctx: egui::Context,
    cache: lru::LruCache<walkers::TileId, CachedTile>,
    min_zoom: u8,
    max_zoom: u8,
}

enum CachedTile {
    Valid(walkers::Tile),
    Missing,
}

impl walkers::Tiles for SqliteTiles {
    fn at(&mut self, tile_id: walkers::TileId) -> Option<walkers::TilePiece> {
        // same overall control flow as LocalTiles:
        // exact tile -> lower zoom fallback -> crop via interpolate_from_lower_zoom
        todo!()
    }

    fn attribution(&self) -> walkers::sources::Attribution {
        walkers::sources::Attribution {
            text: "Local SQLite tiles",
            url: "",
            logo_light: None,
            logo_dark: None,
        }
    }

    fn tile_size(&self) -> u32 { 256 }
}
```

## Important notes for your schema

- Your schema `tiles(x,y,z,s,image blob)` maps naturally to `TileId { x, y, zoom: z }`.
- `walkers` requests XYZ/slippy coordinates. If the DB uses TMS-style flipped Y, you must convert before querying.
- `Tile::new` accepts PNG/JPEG/etc. bytes directly, so you do not need to decode the image yourself first.
- `LocalTiles` uses full-tile UV `Rect` for exact matches and cropped UVs only for lower-zoom fallback.

## Risks / unknowns

- `LocalTiles` is deprecated and walkers recommends local `.pmtiles` for built-in offline support, so custom `Tiles` is the stable-looking path for SQLite.
- `walkers` docs coverage is incomplete; several useful details are only obvious from source.
- It is not documented whether all offline raster databases encountered in practice use XYZ vs TMS Y ordering; verify your DB.
- `rusqlite::Connection` is generally single-thread-affine; simplest safe shape is synchronous reads on the UI thread plus LRU caching. If that stutters, move DB reads behind your own worker/cache layer but keep `Tiles::at` fast.
- There is no built-in SQLite tile backend in current walkers docs/source, so fallback/interpolation semantics need to be copied from `LocalTiles` manually.

## Exact docs/source links used

- https://docs.rs/walkers/latest/walkers/
- https://docs.rs/walkers/latest/walkers/trait.Tiles.html
- https://docs.rs/walkers/latest/walkers/enum.Tile.html
- https://docs.rs/walkers/latest/walkers/struct.TileId.html
- https://docs.rs/walkers/latest/walkers/struct.TilePiece.html
- https://docs.rs/walkers/latest/walkers/struct.Map.html
- https://docs.rs/walkers/latest/walkers/struct.MapMemory.html
- https://docs.rs/walkers/latest/walkers/struct.LocalTiles.html
- https://docs.rs/walkers/latest/walkers/struct.HttpTiles.html
- https://docs.rs/walkers/latest/walkers/sources/trait.TileSource.html
- https://docs.rs/crate/walkers/latest/source/src/tiles.rs
- https://docs.rs/crate/walkers/latest/source/src/local_tiles.rs
- https://docs.rs/crate/walkers/latest/source/src/http_tiles.rs
- https://docs.rs/crate/walkers/latest/source/src/map.rs
- https://docs.rs/crate/walkers/latest/source/src/sources/mod.rs
