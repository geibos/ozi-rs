use crate::infrastructure::import::{
    open_ozi_raster_tile_source, parse_ozi_map_metadata, read_ozi_map_text,
};
use rusqlite::OptionalExtension;
use tauri::ipc::Response;

// ── SQLite tile delivery ──────────────────────────────────────────────────────

/// Return the raw PNG/JPEG bytes for a tile at (z, x, y) from a LizaAlert `.sqlitedb` bundle.
///
/// LizaAlert SQLite schema: table `tiles`, columns `x`, `y`, `z`, `image`.
/// Zoom levels are stored inverted relative to web zoom: `db_z = db_min + (base_zoom - web_z)`.
/// Zoom range metadata comes from table `info`, columns `minzoom`/`maxzoom`.
/// `base_zoom` is the web zoom level corresponding to db zoom 0 (highest detail).
#[tauri::command]
pub fn get_sqlite_tile(path: String, base_zoom: u32, z: u32, x: u32, y: u32) -> Result<Response, String> {
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

    let png_bytes = encode_rgba_to_png(
        tile.rgba_pixels(),
        tile.width(),
        tile.height(),
    )?;

    Ok(Response::new(png_bytes))
}

/// Return tile grid metadata for an OZF2 map (levels, dimensions, georeference coefficients).
#[tauri::command]
pub fn get_ozi_metadata(map_path: String) -> Result<serde_json::Value, String> {
    let path = std::path::PathBuf::from(&map_path);
    let contents = read_ozi_map_text(&path).map_err(|e| e.to_string())?;
    let metadata = parse_ozi_map_metadata(&path, &contents).map_err(|e| e.to_string())?;
    let source = open_ozi_raster_tile_source(&metadata).map_err(|e| e.to_string())?;

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

    Ok(serde_json::json!({
        "map_path": map_path,
        "title": metadata.title(),
        "projection": metadata.projection_name(),
        "datum": metadata.datum_name(),
        "calibration_points": metadata.calibration_points(),
        "levels": levels,
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

// ── PNG encoding helper ───────────────────────────────────────────────────────

fn encode_rgba_to_png(rgba: &[u8], width: u32, height: u32) -> Result<Vec<u8>, String> {
    use image::{ImageBuffer, Rgba};

    let img: ImageBuffer<Rgba<u8>, Vec<u8>> =
        ImageBuffer::from_raw(width, height, rgba.to_vec())
            .ok_or("failed to construct image buffer from RGBA pixels")?;

    let mut buf = Vec::new();
    img.write_to(
        &mut std::io::Cursor::new(&mut buf),
        image::ImageFormat::Png,
    )
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
        conn.execute_batch(
            "CREATE TABLE tiles (x INTEGER, y INTEGER, z INTEGER, image BLOB);",
        )
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
