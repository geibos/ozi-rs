use eframe::egui::{self, Rect};
use lru::LruCache;
use rusqlite::{params, Connection};
use std::num::NonZeroUsize;
use std::path::Path;
use walkers::{
    sources::{Attribution, AttributionType},
    Tile, TileId, TilePiece, Tiles,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
struct TileKey {
    x: u32,
    y: u32,
    zoom: u8,
}

#[derive(Clone)]
enum CachedTile {
    Present(Tile),
    Missing,
}

pub struct SqliteTiles {
    attribution: String,
    cache: LruCache<TileKey, CachedTile>,
    connection: Connection,
    db_min_zoom: u8,
    db_max_zoom: u8,
    egui_ctx: egui::Context,
    web_max_zoom: u8,
}

impl SqliteTiles {
    pub fn open(
        path: &Path,
        egui_ctx: egui::Context,
        attribution: String,
        web_max_zoom: u8,
    ) -> Result<Self, String> {
        let connection = Connection::open(path).map_err(|err| err.to_string())?;
        let (db_min_zoom, db_max_zoom) = read_db_zoom_range(&connection)?;

        Ok(Self {
            attribution,
            cache: LruCache::new(NonZeroUsize::new(512).expect("cache size")),
            connection,
            db_min_zoom,
            db_max_zoom,
            egui_ctx,
            web_max_zoom,
        })
    }

    fn db_zoom_for(&self, requested_zoom: u8) -> Option<u8> {
        let max_delta = self.db_max_zoom.saturating_sub(self.db_min_zoom);
        let min_web_zoom = self.web_max_zoom.saturating_sub(max_delta);

        if requested_zoom < min_web_zoom || requested_zoom > self.web_max_zoom {
            return None;
        }

        Some(self.db_min_zoom + (self.web_max_zoom - requested_zoom))
    }

    fn query_tile(&self, tile_id: TileId) -> Option<Vec<u8>> {
        let db_zoom = self.db_zoom_for(tile_id.zoom)?;
        let mut statement = self
            .connection
            .prepare("SELECT image FROM tiles WHERE x = ?1 AND y = ?2 AND z = ?3 LIMIT 1")
            .ok()?;

        statement
            .query_row(params![tile_id.x, tile_id.y, db_zoom], |row| row.get(0))
            .ok()
    }
}

impl Tiles for SqliteTiles {
    fn at(&mut self, tile_id: TileId) -> Option<TilePiece> {
        let key = TileKey {
            x: tile_id.x,
            y: tile_id.y,
            zoom: tile_id.zoom,
        };

        if let Some(cached) = self.cache.get(&key) {
            return match cached {
                CachedTile::Present(tile) => Some(TilePiece::new(tile.clone(), full_uv())),
                CachedTile::Missing => None,
            };
        }

        let cached = self
            .query_tile(tile_id)
            .and_then(|bytes| {
                Tile::new(
                    &bytes,
                    &walkers::Style::default(),
                    tile_id.zoom,
                    &self.egui_ctx,
                )
                .ok()
            })
            .map_or(CachedTile::Missing, CachedTile::Present);

        self.cache.put(key, cached.clone());

        match cached {
            CachedTile::Present(tile) => Some(TilePiece::new(tile, full_uv())),
            CachedTile::Missing => None,
        }
    }

    fn attribution(&self) -> Attribution {
        Attribution {
            text: self.attribution.clone(),
            url: String::new(),
            logo_light: None,
            logo_dark: None,
            logo_link: None,
            attribution_type: AttributionType::Text,
        }
    }

    fn tile_size(&self) -> u32 {
        256
    }
}

fn full_uv() -> Rect {
    Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0))
}

fn read_db_zoom_range(connection: &Connection) -> Result<(u8, u8), String> {
    connection
        .query_row("SELECT minzoom, maxzoom FROM info LIMIT 1", [], |row| {
            Ok((row.get::<_, u8>(0)?, row.get::<_, u8>(1)?))
        })
        .map_err(|err| err.to_string())
}
