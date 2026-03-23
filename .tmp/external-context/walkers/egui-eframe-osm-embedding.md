---
source: Official docs, docs.rs, and source README
library: walkers
package: walkers
topic: egui eframe osm embedding with pan support
fetched: 2026-03-22T20:17:19Z
official_docs: https://docs.rs/walkers/latest/walkers/
---

Version context
- `walkers` 0.52.0 on docs.rs.
- `eframe` 0.33.3 / `egui` 0.33.3 on docs.rs.

Why it fits
- `walkers` describes itself as a slippy maps widget for `egui`, similar to Leaflet, written in Rust.
- It supports OpenStreetMap and other compatible tile servers, and compiles to native applications as well as WASM.
- The quick start uses `eframe::App`, `HttpTiles`, `MapMemory`, and `Map::new`, which is a direct fit for an `eframe` desktop app.

Minimal API shape
```rust
use walkers::{HttpTiles, Map, MapMemory, sources::OpenStreetMap, lon_lat};
use egui::{Context, CentralPanel};
use eframe::{App, Frame};

struct MyApp {
    tiles: HttpTiles,
    map_memory: MapMemory,
}

impl MyApp {
    fn new(egui_ctx: Context) -> Self {
        Self {
            tiles: HttpTiles::new(OpenStreetMap, egui_ctx),
            map_memory: MapMemory::default(),
        }
    }
}

impl App for MyApp {
    fn update(&mut self, ctx: &Context, _frame: &mut Frame) {
        CentralPanel::default().show(ctx, |ui| {
            ui.add(Map::new(
                Some(&mut self.tiles),
                &mut self.map_memory,
                lon_lat(17.03664, 51.09916),
            ));
        });
    }
}
```

Pan-related details
- `MapMemory` is the persistent state that must live across frames.
- `Map` is recreated each frame; pan/zoom state lives in `MapMemory` and tile state in `HttpTiles`.
- `Map::drag_pan_buttons(...)` configures drag panning buttons.
- `Map::panning(bool)` controls mouse-wheel panning behavior.
- `Map::zoom_with_ctrl(bool)` controls whether wheel zoom requires Ctrl; when enabled, wheel without Ctrl is used for panning.

Programmatic control
- `MapMemory::center_at(position)` centers the map.
- `MapMemory::follow_my_position()` reattaches to the supplied `my_position`.
- `MapMemory::detached()` reports whether the user has dragged away from the followed position.

Noted limitations / cautions
- Docs.rs reports only 59.02% documented for `walkers` 0.52.0.
- The crate depends on HTTP tile downloads and caching for OSM tiles; production use should follow the OpenStreetMap tile usage policy.
- The widget is designed around slippy-map tiles rather than GIS-grade desktop mapping features.
