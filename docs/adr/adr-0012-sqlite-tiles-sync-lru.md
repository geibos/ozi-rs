# ADR-0012: SQLite Tile Queries Synchronous on UI Thread with 512-Entry LRU Cache

- Status: accepted
- Date: 2026-03-23

## Context

LizaAlert map bundles include SQLite tile databases (`.sqlitedb`). The `walkers` crate
calls `Tiles::at()` on every frame for every tile in the viewport. This requires a
fast, bounded tile-serving strategy.

Two questions needed answering:
1. Should tile queries run on the UI thread or a background thread?
2. How should decoded tiles be cached in memory?

## Decision

**Synchronous queries on the UI thread**, wrapped in a `512-entry LRU cache`.

`SqliteTiles` opens a `rusqlite::Connection` and caches decoded tiles in
`LruCache<TileKey, CachedTile>` where `CachedTile` is either `Present(egui::TextureHandle)`
or `Missing` (a sentinel for tiles absent from the database).

- **512 tiles** is the cache capacity. At typical tile sizes (~256×256 px) this is
  sufficient to cover several screen-widths of map without exceeding a few hundred MB.
- **`Missing` sentinel** prevents repeated SQL queries for zoom levels or coordinates
  absent from the database (common at the edge of a map bundle's coverage).
- **Synchronous** because SQLite reads from a memory-mapped file are fast enough
  (sub-millisecond) for the tile sizes in LizaAlert bundles.

This mirrors the decision for OZI raster tiles (ADR-0010) but is a separate
implementation for a different backend.

## Consequences

### Positive

- Simple implementation; no tile request queue or background thread
- `Missing` sentinel eliminates repeated failed queries at coverage boundaries
- LRU bound prevents unbounded GPU memory growth

### Negative

- If SQLite queries ever take longer than ~5ms, frames will drop noticeably
- 512 is a hardcoded heuristic — no automatic adaptation to screen resolution or
  tile size
- Unlike OZI raster (ADR-0010), there is no multi-level zoom selection; SQLite bundles
  store a single target zoom level

## Rejected Alternatives

### Background tile-loader thread

Rejected for the same reason as in ADR-0010: adds partial-frame rendering complexity
(tiles appear mid-frame) without measurable benefit for current tile sizes.

### Unbounded tile cache

Rejected because GPU memory is finite; LRU ensures the oldest unseen tiles are dropped
when new tiles scroll into view.

## Relationship to ADR-0010

Both SQLite tiles and OZI raster tiles use a synchronous LRU cache strategy on the
UI thread. The cache sizes differ (512 vs 256) because SQLite tiles are typically
smaller and more numerous than OZF2 decoded tiles.
