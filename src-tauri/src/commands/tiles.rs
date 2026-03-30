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
    let max_delta = db_max_zoom.saturating_sub(db_min_zoom);
    let min_web_zoom = base_zoom.saturating_sub(max_delta);
    if z < min_web_zoom || z > base_zoom {
        return Err("zoom out of range".to_owned());
    }
    let db_z = db_min_zoom + (base_zoom - z);

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
