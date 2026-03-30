# ADR-0010: OZI Raster Rendering via Multi-Level Tiles with LRU Texture Cache

- Status: accepted
- Date: 2026-03-29

## Context

OZF2 raster maps store image data as a pyramid of zoom levels, each divided into
fixed-size tiles. Uploading the entire raster as a single texture is not feasible:
- full-resolution maps can be hundreds of megabytes
- egui/glow texture upload is synchronous and blocks the render thread
- most tiles are outside the viewport and waste GPU memory

An earlier version uploaded oversized textures and crashed the GPU driver on large maps.

## Decision

Render OZI rasters by:

1. **Level selection** — choose the native OZF2 zoom level whose pixel density is
   closest to the current viewport zoom, avoiding both extreme upscaling and
   unnecessary downscaling.

2. **Tile culling** — compute the visible tile range from the current pan/zoom
   viewport state and render only tiles that intersect the screen.

3. **LRU texture cache** — cache decoded tiles as `egui::TextureHandle` in an LRU
   cache keyed by `(level_index, tile_x, tile_y)`. Cache size: 256 tiles.
   Tiles are uploaded to the GPU on first use; evicted tiles are dropped and
   re-decoded on demand.

4. **Oversized tile guard** — tiles wider or taller than 4096 px are skipped to
   prevent GPU driver crashes (observed on some integrated GPUs).

The viewport state (`zoom`, `top_left_base_pixels`) is maintained in
`OziRasterRenderer` and updated from egui input events each frame.

## Consequences

### Positive

- GPU memory is bounded by the LRU cache size regardless of map size
- Only visible tiles are decoded and uploaded each frame
- Zoom level selection avoids blurry upscaling and unnecessary CPU work
- Pan and zoom remain responsive because tile decoding is per-tile, not per-map

### Negative

- First display of a new map area has a one-frame delay while tiles are decoded
- LRU eviction during fast pan can cause visible tile pop-in
- 256-tile cache is a fixed heuristic; optimal size depends on map resolution and
  screen size
- The rendering loop runs on the UI thread; slow OZF2 decoding can drop frames

## Rejected Alternatives

### Single full-resolution texture upload

Rejected after causing GPU crashes on maps larger than ~4096×4096 px.

### Background tile decoding thread

Considered but deferred; would require a tile request queue and introduce partial-frame
rendering complexity. The synchronous per-tile approach is sufficient for current map
sizes.
