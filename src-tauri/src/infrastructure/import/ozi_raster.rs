#![allow(dead_code)]

use crate::infrastructure::import::direct_image::{DirectImageRaster, TILE_SIZE};
use crate::infrastructure::import::{OziMapMetadata, OziRasterKind};
use ozf2::{DecodedTile, OzfError, OziRaster, PaletteEntry};
use std::fmt;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DecodedOziRasterImage {
    source_path: PathBuf,
    width: u32,
    height: u32,
    rgba_pixels: Vec<u8>,
}

impl DecodedOziRasterImage {
    fn new(source_path: PathBuf, width: u32, height: u32, rgba_pixels: Vec<u8>) -> Self {
        Self {
            source_path,
            width,
            height,
            rgba_pixels,
        }
    }

    pub fn source_path(&self) -> &Path {
        &self.source_path
    }

    pub const fn width(&self) -> u32 {
        self.width
    }

    pub const fn height(&self) -> u32 {
        self.height
    }

    pub fn rgba_pixels(&self) -> &[u8] {
        &self.rgba_pixels
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OziRasterLevelMetadata {
    level_index: usize,
    width: u32,
    height: u32,
    tile_width: u32,
    tile_height: u32,
    tile_columns: u32,
    tile_rows: u32,
}

impl OziRasterLevelMetadata {
    pub(crate) fn new(
        level_index: usize,
        width: u32,
        height: u32,
        tile_width: u32,
        tile_height: u32,
        tile_columns: u32,
        tile_rows: u32,
    ) -> Self {
        Self {
            level_index,
            width,
            height,
            tile_width,
            tile_height,
            tile_columns,
            tile_rows,
        }
    }

    pub const fn level_index(&self) -> usize {
        self.level_index
    }

    pub const fn width(&self) -> u32 {
        self.width
    }

    pub const fn height(&self) -> u32 {
        self.height
    }

    pub const fn tile_width(&self) -> u32 {
        self.tile_width
    }

    pub const fn tile_height(&self) -> u32 {
        self.tile_height
    }

    pub const fn tile_columns(&self) -> u32 {
        self.tile_columns
    }

    pub const fn tile_rows(&self) -> u32 {
        self.tile_rows
    }

    pub fn tile_pixel_size(&self, tile_x: u32, tile_y: u32) -> Option<(u32, u32)> {
        if tile_x >= self.tile_columns || tile_y >= self.tile_rows {
            return None;
        }

        let w = visible_tile_extent(self.width, self.tile_width, tile_x);
        let h = visible_tile_extent(self.height, self.tile_height, tile_y);

        if w == 0 || h == 0 {
            return None;
        }

        Some((w, h))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DecodedOziRasterTile {
    source_path: PathBuf,
    level_index: usize,
    tile_x: u32,
    tile_y: u32,
    width: u32,
    height: u32,
    rgba_pixels: Vec<u8>,
}

impl DecodedOziRasterTile {
    fn new(
        source_path: PathBuf,
        level_index: usize,
        tile_x: u32,
        tile_y: u32,
        width: u32,
        height: u32,
        rgba_pixels: Vec<u8>,
    ) -> Self {
        Self {
            source_path,
            level_index,
            tile_x,
            tile_y,
            width,
            height,
            rgba_pixels,
        }
    }

    pub fn source_path(&self) -> &Path {
        &self.source_path
    }

    pub const fn level_index(&self) -> usize {
        self.level_index
    }

    pub const fn tile_x(&self) -> u32 {
        self.tile_x
    }

    pub const fn tile_y(&self) -> u32 {
        self.tile_y
    }

    pub const fn width(&self) -> u32 {
        self.width
    }

    pub const fn height(&self) -> u32 {
        self.height
    }

    pub fn rgba_pixels(&self) -> &[u8] {
        &self.rgba_pixels
    }
}

/// Where a level's pixels come from.
///
/// OZF2 is the optimised form, already tiled and already pyramided, and it is
/// what a bundle from maps.lizaalert.ru contains. A plain picture beside a
/// `.map` is what a headquarters is handed — a scan, a screenshot, a
/// photograph — and OziExplorer opens both without distinction, so nothing
/// above this level should have to know which it got.
#[derive(Debug)]
enum RasterBackend {
    Ozf2(OziRaster),
    Picture(Box<DirectImageRaster>),
}

/// Not `Clone`, deliberately: a picture raster holds its whole decoded
/// pyramid, and cloning it per tile request cost up to 640 MB a tile before a
/// reviewer found it on 2026-09-23. Callers share it behind an `Arc`.
#[derive(Debug)]
pub struct OziRasterTileSource {
    source_path: PathBuf,
    levels: Vec<OziRasterLevelMetadata>,
    backend: RasterBackend,
}

impl OziRasterTileSource {
    fn new(
        source_path: PathBuf,
        levels: Vec<OziRasterLevelMetadata>,
        backend: RasterBackend,
    ) -> Self {
        Self {
            source_path,
            levels,
            backend,
        }
    }

    pub fn source_path(&self) -> &Path {
        &self.source_path
    }

    pub fn levels(&self) -> &[OziRasterLevelMetadata] {
        &self.levels
    }

    pub fn level(&self, level_index: usize) -> Option<&OziRasterLevelMetadata> {
        self.levels.get(level_index)
    }

    pub fn decode_rgba_tile(
        &self,
        level_index: usize,
        tile_x: u32,
        tile_y: u32,
    ) -> Result<DecodedOziRasterTile, OziRasterDecodeError> {
        let level = self
            .level(level_index)
            .ok_or(OzfError::LevelOutOfBounds { level_index })?;

        let raster = match &self.backend {
            RasterBackend::Ozf2(raster) => raster,
            RasterBackend::Picture(picture) => {
                let (width, height, rgba) = picture.tile_rgba(level_index, tile_x, tile_y).ok_or(
                    OzfError::TileOutOfBounds {
                        level_index,
                        tile_x: u16::try_from(tile_x).unwrap_or(u16::MAX),
                        tile_y: u16::try_from(tile_y).unwrap_or(u16::MAX),
                    },
                )?;
                return Ok(DecodedOziRasterTile::new(
                    self.source_path.clone(),
                    level_index,
                    tile_x,
                    tile_y,
                    width,
                    height,
                    rgba,
                ));
            }
        };
        let decoded_tile = raster.decode_tile(
            level_index,
            u16::try_from(tile_x).map_err(|_| OzfError::TileOutOfBounds {
                level_index,
                tile_x: u16::MAX,
                tile_y: u16::MAX,
            })?,
            u16::try_from(tile_y).map_err(|_| OzfError::TileOutOfBounds {
                level_index,
                tile_x: u16::MAX,
                tile_y: u16::MAX,
            })?,
        )?;
        let (visible_width, visible_height) =
            level
                .tile_pixel_size(tile_x, tile_y)
                .ok_or(OzfError::TileOutOfBounds {
                    level_index,
                    tile_x: u16::try_from(tile_x).unwrap_or(u16::MAX),
                    tile_y: u16::try_from(tile_y).unwrap_or(u16::MAX),
                })?;

        Ok(DecodedOziRasterTile::new(
            self.source_path.clone(),
            level_index,
            tile_x,
            tile_y,
            visible_width,
            visible_height,
            crop_rgba_tile(&decoded_tile, visible_width, visible_height),
        ))
    }
}

#[derive(Debug)]
pub enum OziRasterDecodeError {
    UnsupportedRasterKind(OziRasterKind),
    Decode(OzfError),
    Picture(crate::infrastructure::import::direct_image::DirectImageError),
}

impl fmt::Display for OziRasterDecodeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnsupportedRasterKind(kind) => {
                write!(f, "unsupported OZI raster kind for decoding: {kind:?}")
            }
            Self::Decode(error) => write!(f, "failed to decode OZF raster: {error}"),
            Self::Picture(error) => write!(f, "{error}"),
        }
    }
}

impl std::error::Error for OziRasterDecodeError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::UnsupportedRasterKind(_) => None,
            Self::Decode(error) => Some(error),
            Self::Picture(error) => Some(error),
        }
    }
}

impl From<crate::infrastructure::import::direct_image::DirectImageError> for OziRasterDecodeError {
    fn from(value: crate::infrastructure::import::direct_image::DirectImageError) -> Self {
        Self::Picture(value)
    }
}

impl From<OzfError> for OziRasterDecodeError {
    fn from(value: OzfError) -> Self {
        Self::Decode(value)
    }
}

pub fn open_ozi_raster_tile_source(
    metadata: &OziMapMetadata,
) -> Result<OziRasterTileSource, OziRasterDecodeError> {
    match metadata.raster_kind() {
        OziRasterKind::Ozf2 => open_ozf2_tile_source(metadata),
        OziRasterKind::DirectImage(_) => open_picture_tile_source(metadata),
        other => Err(OziRasterDecodeError::UnsupportedRasterKind(other.clone())),
    }
}

pub fn decode_ozi_raster_image(
    metadata: &OziMapMetadata,
) -> Result<DecodedOziRasterImage, OziRasterDecodeError> {
    let source = open_ozi_raster_tile_source(metadata)?;
    let base_level = source
        .level(0)
        .ok_or(OzfError::LevelOutOfBounds { level_index: 0 })?;
    let pixels = match &source.backend {
        RasterBackend::Ozf2(raster) => raster.decode_rgba_image(0)?.pixels().to_vec(),
        RasterBackend::Picture(picture) => picture
            .level_rgba(0)
            .ok_or(OzfError::LevelOutOfBounds { level_index: 0 })?
            .to_vec(),
    };

    Ok(DecodedOziRasterImage::new(
        source.source_path.clone(),
        base_level.width(),
        base_level.height(),
        pixels,
    ))
}

fn open_ozf2_tile_source(
    metadata: &OziMapMetadata,
) -> Result<OziRasterTileSource, OziRasterDecodeError> {
    let raster = OziRaster::open(metadata.resolved_raster_path())?;
    let tile_width = u32::from(raster.info().tile_width);
    let tile_height = u32::from(raster.info().tile_height);
    let levels = raster
        .levels()
        .iter()
        .map(|level| {
            OziRasterLevelMetadata::new(
                level.level_index,
                level.width,
                level.height,
                tile_width,
                tile_height,
                u32::from(level.tile_columns),
                u32::from(level.tile_rows),
            )
        })
        .collect();

    Ok(OziRasterTileSource::new(
        metadata.resolved_raster_path().to_path_buf(),
        levels,
        RasterBackend::Ozf2(raster),
    ))
}

/// A `.map` beside an ordinary picture: decode it and cut it into tiles.
fn open_picture_tile_source(
    metadata: &OziMapMetadata,
) -> Result<OziRasterTileSource, OziRasterDecodeError> {
    let path = metadata.resolved_raster_path().to_path_buf();
    let picture = DirectImageRaster::open(&path)?;

    let levels = (0..picture.level_count())
        .filter_map(|level_index| {
            let (width, height) = picture.level_size(level_index)?;
            Some(OziRasterLevelMetadata::new(
                level_index,
                width,
                height,
                TILE_SIZE,
                TILE_SIZE,
                width.div_ceil(TILE_SIZE),
                height.div_ceil(TILE_SIZE),
            ))
        })
        .collect();

    Ok(OziRasterTileSource::new(
        path,
        levels,
        RasterBackend::Picture(Box::new(picture)),
    ))
}

fn crop_rgba_tile(tile: &DecodedTile, visible_width: u32, visible_height: u32) -> Vec<u8> {
    let full_width = usize::from(tile.width());
    let visible_width = visible_width as usize;
    let visible_height = visible_height as usize;
    let full_rgba = indexed_tile_to_rgba(tile);
    let mut cropped = Vec::with_capacity(visible_width * visible_height * 4);

    for row in 0..visible_height {
        let start = row * full_width * 4;
        let end = start + visible_width * 4;
        cropped.extend_from_slice(&full_rgba[start..end]);
    }

    cropped
}

fn indexed_tile_to_rgba(tile: &DecodedTile) -> Vec<u8> {
    let mut rgba_pixels = Vec::with_capacity(tile.pixels().len() * 4);

    for palette_index in tile.pixels() {
        let color = tile
            .palette()
            .get(*palette_index as usize)
            .copied()
            .unwrap_or(PaletteEntry {
                red: 0,
                green: 0,
                blue: 0,
                alpha: 0,
            });
        rgba_pixels.extend_from_slice(&[color.red, color.green, color.blue, color.alpha]);
    }

    rgba_pixels
}

fn visible_tile_extent(total_extent: u32, tile_extent: u32, tile_index: u32) -> u32 {
    let start = tile_index.saturating_mul(tile_extent);
    total_extent.saturating_sub(start).min(tile_extent)
}

#[cfg(test)]
mod tests {
    use super::{
        OziRasterLevelMetadata, crop_rgba_tile, indexed_tile_to_rgba, visible_tile_extent,
    };
    use ozf2::{DecodedTile, PaletteEntry};

    #[test]
    fn level_metadata_reports_partial_edge_tile_size() {
        let level = OziRasterLevelMetadata::new(0, 130, 70, 64, 64, 3, 2);

        assert_eq!(level.tile_pixel_size(2, 1), Some((2, 6)));
    }

    #[test]
    fn level_metadata_rejects_out_of_bounds_tile_size_queries() {
        let level = OziRasterLevelMetadata::new(0, 128, 128, 64, 64, 2, 2);

        assert_eq!(level.tile_pixel_size(2, 0), None);
    }

    #[test]
    fn visible_tile_extent_clamps_to_remaining_pixels() {
        assert_eq!(visible_tile_extent(130, 64, 0), 64);
        assert_eq!(visible_tile_extent(130, 64, 1), 64);
        assert_eq!(visible_tile_extent(130, 64, 2), 2);
    }

    #[test]
    fn indexed_tile_to_rgba_expands_palette_indexes() {
        let tile = sample_tile();

        let rgba = indexed_tile_to_rgba(&tile);

        assert_eq!(&rgba[0..8], &[10, 20, 30, 255, 40, 50, 60, 255]);
    }

    #[test]
    fn crop_rgba_tile_trims_to_visible_edge_size() {
        let tile = sample_tile();

        let rgba = crop_rgba_tile(&tile, 2, 2);

        assert_eq!(rgba.len(), 2 * 2 * 4);
        assert_eq!(
            rgba,
            vec![
                10, 20, 30, 255, 40, 50, 60, 255, 10, 20, 30, 255, 40, 50, 60, 255,
            ]
        );
    }

    fn sample_tile() -> DecodedTile {
        let mut pixels = vec![0_u8; 64 * 64];
        pixels[1] = 1;
        pixels[64] = 0;
        pixels[65] = 1;

        DecodedTile::new(
            64,
            64,
            pixels,
            vec![
                PaletteEntry {
                    red: 10,
                    green: 20,
                    blue: 30,
                    alpha: 255,
                },
                PaletteEntry {
                    red: 40,
                    green: 50,
                    blue: 60,
                    alpha: 255,
                },
            ],
        )
    }
}

/// A `.map` beside an ordinary picture, opened the way the application opens
/// one: parse the calibration file, resolve the raster it names, cut tiles.
///
/// The unit tests below the picture reader prove the pyramid and the tile
/// cutting; this proves the wiring, which is where it was broken — the reader
/// existed in the `.map` parser's vocabulary (`DirectImage(Jpeg)`,
/// `DirectImage(Png)`, …) for months while `open_ozi_raster_tile_source`
/// answered `UnsupportedRasterKind` for every one of them.
#[cfg(test)]
mod picture_map_tests {
    use super::{OziRasterDecodeError, open_ozi_raster_tile_source};
    use crate::infrastructure::export::{CalibrationPoint, write_calibration_map};
    use crate::infrastructure::import::{
        OziRasterKind, parse_ozi_georeference, parse_ozi_map_metadata,
    };
    use std::path::PathBuf;

    fn temp_dir(name: &str) -> PathBuf {
        let dir =
            std::env::temp_dir().join(format!("ozi-rs-picture-map-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).expect("temp dir");
        dir
    }

    fn write_picture(dir: &std::path::Path, name: &str, width: u32, height: u32) {
        let mut image = image::RgbaImage::new(width, height);
        for (x, y, pixel) in image.enumerate_pixels_mut() {
            *pixel = image::Rgba([(x % 256) as u8, (y % 256) as u8, 64, 255]);
        }
        image.save(dir.join(name)).expect("write picture");
    }

    fn write_map(dir: &std::path::Path, raster_name: &str) -> PathBuf {
        let path = dir.join("sheet.map");
        let contents = format!(
            "OziExplorer Map Data File Version 2.2\nSheet\n{raster_name}\n1 ,Map Code,\nWGS 84,,   0.0000,   0.0000,WGS 84\nReserved 1\nReserved 2\nMagnetic Variation,,,E\nMap Projection,Mercator,PolyCal,No,AutoCalOnly,No,BSBUseWPX,No\nPoint01,xy,100,200,in, deg,54,30.000,N,48,24.000,E, grid, , , ,N\nPoint02,xy,300,400,in, deg,54,31.000,N,48,25.000,E, grid, , , ,N\n"
        );
        std::fs::write(&path, contents).expect("write map");
        path
    }

    #[test]
    fn a_map_beside_a_png_opens_and_hands_out_tiles() {
        let dir = temp_dir("png");
        write_picture(&dir, "sheet.png", 600, 400);
        let map_path = write_map(&dir, "sheet.png");

        let contents = std::fs::read_to_string(&map_path).expect("read map");
        let metadata = parse_ozi_map_metadata(&map_path, &contents).expect("parse map");
        assert!(matches!(
            metadata.raster_kind(),
            OziRasterKind::DirectImage(_)
        ));

        let source = open_ozi_raster_tile_source(&metadata).expect("open picture source");
        let base = source.level(0).expect("level 0");
        assert_eq!((base.width(), base.height()), (600, 400));
        assert_eq!((base.tile_columns(), base.tile_rows()), (3, 2));

        let tile = source.decode_rgba_tile(0, 0, 0).expect("tile 0,0");
        assert_eq!((tile.width(), tile.height()), (256, 256));
        assert_eq!(tile.rgba_pixels().len(), 256 * 256 * 4);

        // The last column is 600 - 512 = 88 pixels wide, and asking past it is
        // not an error the caller has to guess at.
        let edge = source.decode_rgba_tile(0, 2, 0).expect("tile 2,0");
        assert_eq!(edge.width(), 88);
        assert!(source.decode_rgba_tile(0, 3, 0).is_err());

        let _ = std::fs::remove_dir_all(&dir);
    }

    /// The pyramid, through the path a tile request actually takes.
    ///
    /// `halve` has its own unit tests and the wiring test above reads one tile
    /// of level 0. Neither says that level 2 of a real picture, opened through
    /// a `.map`, is the right size and full of the right pixels — and the
    /// halving was rewritten on 2026-09-23 to stop the build peaking at four
    /// times its memory limit, so that is new code on the path every tile of a
    /// calibrated picture goes through.
    #[test]
    fn every_level_of_a_real_picture_hands_out_the_right_tiles() {
        let dir = temp_dir("pyramid");
        // 2000×1200 halves until the short side is no bigger than one tile:
        // 1200 → 600 → 300 → 150, so four levels.
        write_picture(&dir, "sheet.png", 2000, 1200);
        let map_path = write_map(&dir, "sheet.png");
        let contents = std::fs::read_to_string(&map_path).expect("read map");
        let metadata = parse_ozi_map_metadata(&map_path, &contents).expect("parse map");
        let source = open_ozi_raster_tile_source(&metadata).expect("open");

        let sizes: Vec<(u32, u32)> = source
            .levels()
            .iter()
            .map(|level| (level.width(), level.height()))
            .collect();
        assert_eq!(
            sizes,
            vec![(2000, 1200), (1000, 600), (500, 300), (250, 150)]
        );

        for level in source.levels() {
            let index = level.level_index();
            let columns = level.width().div_ceil(256);
            let rows = level.height().div_ceil(256);

            // The contract, uniformly: a tile is 256 square except at the
            // right and bottom edges, where it is exactly what is left. Stated
            // this way rather than "a middle tile is whole", which is only
            // true of levels more than two tiles across — the coarse ones are
            // the whole map in one tile and have no middle.
            for y in 0..rows {
                for x in 0..columns {
                    let tile = source
                        .decode_rgba_tile(index, x, y)
                        .unwrap_or_else(|e| panic!("level {index} tile {x},{y}: {e:?}"));
                    let expected_width = 256.min(level.width() - x * 256);
                    let expected_height = 256.min(level.height() - y * 256);
                    assert_eq!(
                        (tile.width(), tile.height()),
                        (expected_width, expected_height),
                        "level {index} tile {x},{y}"
                    );
                    assert_eq!(
                        tile.rgba_pixels().len() as u32,
                        expected_width * expected_height * 4,
                        "level {index} tile {x},{y} pixel count"
                    );
                }
            }

            // And past the grid there is nothing.
            assert!(
                source.decode_rgba_tile(index, columns, 0).is_err(),
                "level {index} answered a tile past its right edge"
            );
            assert!(
                source.decode_rgba_tile(index, 0, rows).is_err(),
                "level {index} answered a tile past its bottom edge"
            );
        }

        // Halving averages: the picture's red channel is `x % 256`, so a
        // pixel of level 1 is the mean of two neighbouring columns. At the
        // very left that is (0 + 1) / 2 = 0.
        let level1 = source.decode_rgba_tile(1, 0, 0).expect("level 1 tile 0,0");
        assert_eq!(level1.rgba_pixels()[0], 0);
        // Ten pixels in: columns 20 and 21, mean 20.
        assert_eq!(level1.rgba_pixels()[10 * 4], 20);

        let _ = std::fs::remove_dir_all(&dir);
    }

    /// Calibration, all the way round, through the pieces a tile request uses.
    ///
    /// The writer has its own tests and so does the reader, and they agree —
    /// but they agree about a `.map` written by the test rather than by the
    /// application, and neither of them opens a raster. This is the whole
    /// journey a headquarters makes with a picture that arrived with nothing:
    /// calibrate it, and then have the map served.
    #[test]
    fn a_calibrated_picture_serves_tiles_at_the_coordinates_it_was_given() {
        let dir = temp_dir("calibrated");
        write_picture(&dir, "Сагра.png", 1600, 1200);

        // The two corners a coordinator reads off a screenshot of a web map.
        let top_left = CalibrationPoint {
            pixel_x: 0.0,
            pixel_y: 0.0,
            lat: 60.05,
            lon: 30.20,
        };
        let bottom_right = CalibrationPoint {
            pixel_x: 1600.0,
            pixel_y: 1200.0,
            lat: 59.95,
            lon: 30.40,
        };
        let map_path = write_calibration_map(
            &dir.join("Сагра.png"),
            "Сагра",
            1600,
            1200,
            &[top_left, bottom_right],
        )
        .expect("write the calibration");
        assert_eq!(map_path, dir.join("Сагра.map"));

        // Now open it the way the application does.
        let contents = crate::infrastructure::import::read_ozi_map_text(&map_path).expect("read");
        let metadata = parse_ozi_map_metadata(&map_path, &contents).expect("parse");
        assert!(matches!(
            metadata.raster_kind(),
            OziRasterKind::DirectImage(_)
        ));

        // The georeference answers where the operator said the corners are.
        let georeference =
            parse_ozi_georeference(metadata.calibration_points()).expect("georeference");
        for corner in [top_left, bottom_right] {
            let (lat, lon) = georeference.pixel_to_lat_lon(corner.pixel_x, corner.pixel_y);
            assert!(
                (lat - corner.lat).abs() < 1e-4 && (lon - corner.lon).abs() < 1e-4,
                "corner at ({}, {}) came back as {lat}, {lon}",
                corner.pixel_x,
                corner.pixel_y
            );
        }
        // And the middle of the picture is the middle of the ground.
        let (lat, lon) = georeference.pixel_to_lat_lon(800.0, 600.0);
        assert!((lat - 60.0).abs() < 1e-4, "middle latitude {lat}");
        assert!((lon - 30.3).abs() < 1e-4, "middle longitude {lon}");

        // And the raster the `.map` names is served.
        let source = open_ozi_raster_tile_source(&metadata).expect("open the raster");
        assert_eq!(
            source.level(0).map(|l| (l.width(), l.height())),
            Some((1600, 1200))
        );
        let tile = source.decode_rgba_tile(0, 0, 0).expect("first tile");
        assert_eq!((tile.width(), tile.height()), (256, 256));

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn a_map_naming_a_picture_that_is_not_there_says_so() {
        let dir = temp_dir("missing");
        let map_path = write_map(&dir, "sheet.jpg");
        let contents = std::fs::read_to_string(&map_path).expect("read map");
        let metadata = parse_ozi_map_metadata(&map_path, &contents).expect("parse map");

        let error = open_ozi_raster_tile_source(&metadata).expect_err("must fail");
        assert!(matches!(error, OziRasterDecodeError::Picture(_)));
        assert!(
            error.to_string().contains("sheet.jpg"),
            "the message must name the file: {error}"
        );

        let _ = std::fs::remove_dir_all(&dir);
    }
}
