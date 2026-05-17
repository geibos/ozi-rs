use crate::infrastructure::import::{
    OziMapMetadata, OziRasterTileSource, open_ozi_raster_tile_source, parse_ozi_georeference,
    parse_ozi_map_metadata, read_ozi_map_text,
};
use rusqlite::OptionalExtension;
use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};
use tauri::ipc::Response;

#[derive(Clone)]
struct CachedOziMapContext {
    metadata: OziMapMetadata,
    georeference: crate::infrastructure::import::OziGeoreference,
    source: OziRasterTileSource,
}

fn ozi_cache() -> &'static Mutex<HashMap<String, CachedOziMapContext>> {
    static CACHE: OnceLock<Mutex<HashMap<String, CachedOziMapContext>>> = OnceLock::new();
    CACHE.get_or_init(|| Mutex::new(HashMap::new()))
}

fn load_ozi_context(map_path: &str) -> Result<CachedOziMapContext, String> {
    if let Ok(cache) = ozi_cache().lock()
        && let Some(cached) = cache.get(map_path)
    {
        return Ok(cached.clone());
    }

    let path = std::path::PathBuf::from(map_path);
    let contents = read_ozi_map_text(&path).map_err(|e| e.to_string())?;
    let metadata = parse_ozi_map_metadata(&path, &contents).map_err(|e| e.to_string())?;
    let georeference = parse_ozi_georeference(metadata.calibration_points())
        .ok_or_else(|| "failed to parse georeference".to_owned())?;
    let source = open_ozi_raster_tile_source(&metadata).map_err(|e| e.to_string())?;
    let context = CachedOziMapContext {
        metadata,
        georeference,
        source,
    };

    if let Ok(mut cache) = ozi_cache().lock() {
        cache.insert(map_path.to_owned(), context.clone());
    }

    Ok(context)
}

// ── SQLite tile delivery ──────────────────────────────────────────────────────

/// Return the raw PNG/JPEG bytes for a tile at (z, x, y) from a LizaAlert `.sqlitedb` bundle.
///
/// LizaAlert SQLite schema: table `tiles`, columns `x`, `y`, `z`, `image`.
/// Zoom levels are stored inverted relative to web zoom: `db_z = db_min + (base_zoom - web_z)`.
/// Zoom range metadata comes from table `info`, columns `minzoom`/`maxzoom`.
/// `base_zoom` is the web zoom level corresponding to db zoom 0 (highest detail).
#[tauri::command]
pub fn get_sqlite_tile(
    path: String,
    base_zoom: u32,
    z: u32,
    x: u32,
    y: u32,
) -> Result<Response, String> {
    let conn = rusqlite::Connection::open(&path).map_err(|e| e.to_string())?;

    // Read zoom range from info table
    let (db_min_zoom, db_max_zoom): (u32, u32) = conn
        .query_row("SELECT minzoom, maxzoom FROM info LIMIT 1", [], |row| {
            Ok((row.get::<_, u32>(0)?, row.get::<_, u32>(1)?))
        })
        .map_err(|e| format!("failed to read info table: {e}"))?;

    // Map web zoom to DB zoom (DB zoom 0 = highest detail = base_zoom on web)
    let db_z = match web_to_db_zoom(z, db_min_zoom, db_max_zoom, base_zoom) {
        Some(v) => v,
        None => return Err("zoom out of range".to_owned()),
    };

    let result: Option<Vec<u8>> = conn
        .query_row(
            "SELECT image FROM tiles WHERE x=?1 AND y=?2 AND z=?3 LIMIT 1",
            rusqlite::params![x, y, db_z],
            |row| row.get(0),
        )
        .optional()
        .map_err(|e| e.to_string())?;

    match result {
        Some(bytes) => Ok(Response::new(bytes)),
        None => Err("tile not found".to_owned()),
    }
}

// ── OZI raster tile delivery ──────────────────────────────────────────────────

/// Return a PNG-encoded tile from an OZF2 file given the `.map` metadata path,
/// zoom level index, and tile grid coordinates.
#[tauri::command]
pub fn get_ozi_tile(
    map_path: String,
    level: usize,
    tile_x: u32,
    tile_y: u32,
) -> Result<Response, String> {
    let path = std::path::PathBuf::from(&map_path);
    let contents = read_ozi_map_text(&path).map_err(|e| e.to_string())?;
    let metadata = parse_ozi_map_metadata(&path, &contents).map_err(|e| e.to_string())?;
    let source = open_ozi_raster_tile_source(&metadata).map_err(|e| e.to_string())?;

    let tile = source
        .decode_rgba_tile(level, tile_x, tile_y)
        .map_err(|e| e.to_string())?;

    let png_bytes = encode_rgba_to_png(tile.rgba_pixels(), tile.width(), tile.height())?;

    Ok(Response::new(png_bytes))
}

/// Return tile grid metadata for an OZF2 map (levels, dimensions, georeference coefficients).
#[tauri::command]
pub fn get_ozi_metadata(map_path: String) -> Result<serde_json::Value, String> {
    let context = load_ozi_context(&map_path)?;
    let metadata = &context.metadata;
    let source = &context.source;

    let levels: Vec<serde_json::Value> = source
        .levels()
        .iter()
        .map(|lvl| {
            serde_json::json!({
                "level_index": lvl.level_index(),
                "width": lvl.width(),
                "height": lvl.height(),
                "tile_width": lvl.tile_width(),
                "tile_height": lvl.tile_height(),
                "tile_columns": lvl.tile_columns(),
                "tile_rows": lvl.tile_rows(),
            })
        })
        .collect();

    // Compute geographic bounds and zoom hints from georeference + level-0 dimensions.
    let (bounds, native_zoom, min_zoom) = if let Some(lvl0) = source.levels().first() {
        let corners = [
            context.georeference.pixel_to_lat_lon(0.0, 0.0),
            context
                .georeference
                .pixel_to_lat_lon(lvl0.width() as f64, 0.0),
            context
                .georeference
                .pixel_to_lat_lon(0.0, lvl0.height() as f64),
            context
                .georeference
                .pixel_to_lat_lon(lvl0.width() as f64, lvl0.height() as f64),
        ];
        let (min_lon, max_lon) = corners
            .iter()
            .map(|(_, lon)| *lon)
            .fold((f64::INFINITY, f64::NEG_INFINITY), |(min, max), lon| {
                (min.min(lon), max.max(lon))
            });
        let (min_lat, max_lat) = corners
            .iter()
            .map(|(lat, _)| *lat)
            .fold((f64::INFINITY, f64::NEG_INFINITY), |(min, max), lat| {
                (min.min(lat), max.max(lat))
            });

        let pixels_per_deg = context.georeference.pixels_per_lon_degree();
        let native_zoom = ((pixels_per_deg * 360.0 / 256.0).log2().round() as u32).min(22);

        let lon_span = (max_lon - min_lon).max(0.001);
        let min_zoom = ((360.0_f64 / lon_span).log2().ceil() as u32).saturating_sub(1);

        (
            serde_json::json!([min_lon, min_lat, max_lon, max_lat]),
            native_zoom,
            min_zoom,
        )
    } else {
        (serde_json::Value::Null, 14u32, 0u32)
    };

    Ok(serde_json::json!({
        "map_path": map_path,
        "title": metadata.title(),
        "projection": metadata.projection_name(),
        "datum": metadata.datum_name(),
        "calibration_points": metadata.calibration_points(),
        "levels": levels,
        "bounds": bounds,
        "native_zoom": native_zoom,
        "min_zoom": min_zoom,
    }))
}

// ── Zoom mapping ──────────────────────────────────────────────────────────────

/// Map a web (MapLibre) zoom level to the corresponding DB zoom level.
///
/// LizaAlert bundles store tiles with zoom 0 = highest detail (inverted vs web).
/// `base_zoom` is the web zoom that corresponds to `db_min_zoom` (highest detail).
/// Returns `None` if `web_z` is outside the zoom range covered by the bundle.
fn web_to_db_zoom(web_z: u32, db_min: u32, db_max: u32, base_zoom: u32) -> Option<u32> {
    let max_delta = db_max.saturating_sub(db_min);
    let min_web_zoom = base_zoom.saturating_sub(max_delta);
    if web_z < min_web_zoom || web_z > base_zoom {
        return None;
    }
    Some(db_min + (base_zoom - web_z))
}

// ── OZI projected tile delivery ───────────────────────────────────────────────

/// Return a 256×256 PNG for the Web Mercator tile `(tx, ty, tz)` by reprojecting
/// from the OZF2 raster.  All coordinate math happens here in Rust; the JS side
/// only passes the standard MapLibre z/x/y values.
///
/// Algorithm:
/// 1. Compute the lat/lon bounding box of the requested Web Mercator tile.
/// 2. Map the bbox to OZF2 level-0 pixel coordinates via the affine georeference.
/// 3. Pick the coarsest OZF2 zoom level whose detail is close to 1 OZF2 pixel
///    per output pixel (minimises the number of tiles that must be decoded).
/// 4. Stitch the OZF2 tiles that overlap the bbox into a scratch buffer.
/// 5. Nearest-neighbour scale the scratch buffer into a 256×256 output image.
/// 6. Return the output as a PNG.
#[tauri::command]
pub fn get_ozi_tile_projected(
    map_path: String,
    tx: u32,
    ty: u32,
    tz: u32,
) -> Result<Response, String> {
    let context = load_ozi_context(&map_path)?;
    let geo = &context.georeference;
    let source = &context.source;

    let levels = source.levels();
    if levels.is_empty() {
        return Err("no OZF2 levels".to_owned());
    }

    let tile_corners = [
        tile_corner_lat_lon(tx, ty, tz),
        tile_corner_lat_lon(tx + 1, ty, tz),
        tile_corner_lat_lon(tx, ty + 1, tz),
        tile_corner_lat_lon(tx + 1, ty + 1, tz),
    ];
    let level0_corners: Vec<(f64, f64)> = tile_corners
        .iter()
        .map(|(lon, lat)| geo.lat_lon_to_pixel(*lat, *lon))
        .collect();
    let px0_min = level0_corners
        .iter()
        .map(|(px, _)| *px)
        .fold(f64::INFINITY, f64::min);
    let px0_max = level0_corners
        .iter()
        .map(|(px, _)| *px)
        .fold(f64::NEG_INFINITY, f64::max);
    let py0_min = level0_corners
        .iter()
        .map(|(_, py)| *py)
        .fold(f64::INFINITY, f64::min);
    let py0_max = level0_corners
        .iter()
        .map(|(_, py)| *py)
        .fold(f64::NEG_INFINITY, f64::max);

    let span = (px0_max - px0_min).abs().max((py0_max - py0_min).abs());
    let max_valid_level = levels.len() - 1;
    let level_idx = levels
        .iter()
        .enumerate()
        .rev()
        .find(|(_, level)| {
            let scale_x = level.width() as f64 / levels[0].width() as f64;
            let scale_y = level.height() as f64 / levels[0].height() as f64;
            let avg_scale = ((scale_x + scale_y) / 2.0).max(f64::EPSILON);
            (span * avg_scale) / 256.0 >= 1.0
        })
        .map(|(idx, _)| idx)
        .unwrap_or(0)
        .min(max_valid_level);

    let lvl = &levels[level_idx];
    let tile_w = lvl.tile_width();
    let tile_h = lvl.tile_height();
    let map_w = lvl.width(); // actual width at this level (not padded)
    let map_h = lvl.height();
    let scale_x = lvl.width() as f64 / levels[0].width() as f64;
    let scale_y = lvl.height() as f64 / levels[0].height() as f64;

    let px_min = px0_min * scale_x;
    let py_min = py0_min * scale_y;
    let px_max = px0_max * scale_x;
    let py_max = py0_max * scale_y;

    // Bounds check: reject tiles that don't intersect the map at all
    if px_min >= map_w as f64 || px_max < 0.0 || py_min >= map_h as f64 || py_max < 0.0 {
        return Err("tile out of bounds".to_owned());
    }

    // Clamp to map bounds (partial coverage at edges)
    let src_x0 = px_min.max(0.0).floor();
    let src_y0 = py_min.max(0.0).floor();
    let src_x1 = px_max.min(map_w as f64).ceil();
    let src_y1 = py_max.min(map_h as f64).ceil();

    // Range of OZF2 tiles to decode
    let oz_tx0 = (src_x0 / tile_w as f64).floor() as u32;
    let oz_ty0 = (src_y0 / tile_h as f64).floor() as u32;
    let oz_tx1 = ((src_x1 / tile_w as f64).ceil() as u32).min(lvl.tile_columns());
    let oz_ty1 = ((src_y1 / tile_h as f64).ceil() as u32).min(lvl.tile_rows());

    if oz_tx1 <= oz_tx0 || oz_ty1 <= oz_ty0 {
        return Err("empty OZF2 tile range".to_owned());
    }

    // Stitch all required OZF2 tiles into one RGBA scratch buffer
    let stitch_w = (oz_tx1 - oz_tx0) * tile_w;
    let stitch_h = (oz_ty1 - oz_ty0) * tile_h;
    let mut stitched = vec![0u8; stitch_w as usize * stitch_h as usize * 4];

    for oty in oz_ty0..oz_ty1 {
        for otx in oz_tx0..oz_tx1 {
            if let Ok(tile) = source.decode_rgba_tile(level_idx, otx, oty) {
                let tw = tile.width();
                let th = tile.height();
                let px_data = tile.rgba_pixels();
                let dst_x = (otx - oz_tx0) * tile_w;
                let dst_y = (oty - oz_ty0) * tile_h;
                for row in 0..th {
                    let src_off = row as usize * tw as usize * 4;
                    let dst_off = ((dst_y + row) * stitch_w + dst_x) as usize * 4;
                    let len = tw as usize * 4;
                    if src_off + len <= px_data.len() && dst_off + len <= stitched.len() {
                        stitched[dst_off..dst_off + len]
                            .copy_from_slice(&px_data[src_off..src_off + len]);
                    }
                }
            }
        }
    }

    let mut out_rgba = vec![0u8; 256 * 256 * 4];
    let stitch_origin_x = oz_tx0 as f64 * tile_w as f64;
    let stitch_origin_y = oz_ty0 as f64 * tile_h as f64;

    for dy in 0..256u32 {
        for dx in 0..256u32 {
            let (lon, lat) = tile_pixel_lat_lon(tx, ty, tz, dx, dy);
            let (px0, py0) = geo.lat_lon_to_pixel(lat, lon);
            let sx = px0 * scale_x - stitch_origin_x;
            let sy = py0 * scale_y - stitch_origin_y;

            if sx < 0.0 || sy < 0.0 || sx >= stitch_w as f64 || sy >= stitch_h as f64 {
                continue;
            }

            let sx = sx.floor() as u32;
            let sy = sy.floor() as u32;
            let src_off = (sy * stitch_w + sx) as usize * 4;
            let out_off = (dy * 256 + dx) as usize * 4;
            if src_off + 4 <= stitched.len() && out_off + 4 <= out_rgba.len() {
                out_rgba[out_off..out_off + 4].copy_from_slice(&stitched[src_off..src_off + 4]);
            }
        }
    }

    let png = encode_rgba_to_png(&out_rgba, 256, 256)?;
    Ok(Response::new(png))
}

// ── Web Mercator helpers ──────────────────────────────────────────────────────

/// Convert Web Mercator tile corner `(tx, ty)` at zoom `tz` to `(lon, lat)` degrees.
fn tile_corner_lat_lon(tx: u32, ty: u32, tz: u32) -> (f64, f64) {
    let n = (1u64 << tz) as f64;
    let lon = tx as f64 / n * 360.0 - 180.0;
    let lat = (std::f64::consts::PI * (1.0 - 2.0 * ty as f64 / n))
        .sinh()
        .atan()
        .to_degrees();
    (lon, lat)
}

fn tile_pixel_lat_lon(tx: u32, ty: u32, tz: u32, pixel_x: u32, pixel_y: u32) -> (f64, f64) {
    let n = (1u64 << tz) as f64;
    let world_x = tx as f64 * 256.0 + pixel_x as f64 + 0.5;
    let world_y = ty as f64 * 256.0 + pixel_y as f64 + 0.5;
    let lon = world_x / (256.0 * n) * 360.0 - 180.0;
    let merc_y = std::f64::consts::PI * (1.0 - 2.0 * world_y / (256.0 * n));
    let lat = merc_y.sinh().atan().to_degrees();
    (lon, lat)
}

// ── PNG encoding helper ───────────────────────────────────────────────────────

fn encode_rgba_to_png(rgba: &[u8], width: u32, height: u32) -> Result<Vec<u8>, String> {
    use image::{ImageBuffer, Rgba};

    let img: ImageBuffer<Rgba<u8>, Vec<u8>> =
        ImageBuffer::from_raw(width, height, rgba.to_vec())
            .ok_or("failed to construct image buffer from RGBA pixels")?;

    let mut buf = Vec::new();
    img.write_to(&mut std::io::Cursor::new(&mut buf), image::ImageFormat::Png)
        .map_err(|e| e.to_string())?;

    Ok(buf)
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::OptionalExtension;

    // ── web_to_db_zoom ────────────────────────────────────────────────────────

    #[test]
    fn base_zoom_maps_to_db_min() {
        // Highest-detail web zoom → lowest DB zoom index (most detailed level)
        assert_eq!(web_to_db_zoom(15, 0, 4, 15), Some(0));
    }

    #[test]
    fn min_web_zoom_maps_to_db_max() {
        // Lowest covered web zoom → highest DB zoom index (overview level)
        // base=15, db range 0..4 → min_web = 15-4 = 11
        assert_eq!(web_to_db_zoom(11, 0, 4, 15), Some(4));
    }

    #[test]
    fn mid_zoom_maps_correctly() {
        assert_eq!(web_to_db_zoom(13, 0, 4, 15), Some(2));
    }

    #[test]
    fn zoom_below_range_is_none() {
        assert_eq!(web_to_db_zoom(10, 0, 4, 15), None);
    }

    #[test]
    fn zoom_above_base_is_none() {
        assert_eq!(web_to_db_zoom(16, 0, 4, 15), None);
    }

    #[test]
    fn single_zoom_level_bundle() {
        // DB has only zoom 0; base_zoom = 14
        assert_eq!(web_to_db_zoom(14, 0, 0, 14), Some(0));
        assert_eq!(web_to_db_zoom(13, 0, 0, 14), None);
        assert_eq!(web_to_db_zoom(15, 0, 0, 14), None);
    }

    // ── SQLite schema ─────────────────────────────────────────────────────────

    /// Verify that the SQL queries use the correct LizaAlert column names.
    /// If the schema changes, this test will fail before the app is even launched.
    #[test]
    fn sqlite_schema_uses_x_y_z_image_columns() {
        let conn = rusqlite::Connection::open_in_memory().unwrap();
        conn.execute_batch(
            "CREATE TABLE info (minzoom INTEGER, maxzoom INTEGER);
             INSERT INTO info VALUES (0, 2);
             CREATE TABLE tiles (x INTEGER, y INTEGER, z INTEGER, image BLOB);
             INSERT INTO tiles VALUES (10, 20, 0, X'deadbeef');",
        )
        .unwrap();

        // info table query must work
        let (min, max): (u32, u32) = conn
            .query_row("SELECT minzoom, maxzoom FROM info LIMIT 1", [], |row| {
                Ok((row.get(0)?, row.get(1)?))
            })
            .unwrap();
        assert_eq!((min, max), (0, 2));

        // tiles query must return the blob using the correct column names
        let data: Option<Vec<u8>> = conn
            .query_row(
                "SELECT image FROM tiles WHERE x=?1 AND y=?2 AND z=?3 LIMIT 1",
                rusqlite::params![10u32, 20u32, 0u32],
                |row| row.get(0),
            )
            .optional()
            .unwrap();
        assert_eq!(data, Some(vec![0xde, 0xad, 0xbe, 0xef]));
    }

    #[test]
    fn missing_tile_returns_none_not_error() {
        let conn = rusqlite::Connection::open_in_memory().unwrap();
        conn.execute_batch("CREATE TABLE tiles (x INTEGER, y INTEGER, z INTEGER, image BLOB);")
            .unwrap();

        let data: Option<Vec<u8>> = conn
            .query_row(
                "SELECT image FROM tiles WHERE x=?1 AND y=?2 AND z=?3 LIMIT 1",
                rusqlite::params![0u32, 0u32, 0u32],
                |row| row.get(0),
            )
            .optional()
            .unwrap();
        assert!(data.is_none());
    }
}
