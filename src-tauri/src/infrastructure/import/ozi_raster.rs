#![allow(dead_code)]

use crate::infrastructure::import::{OziMapMetadata, OziRasterKind};
use ozf2_rs::{DecodedTile, OzfError, OziRaster, PaletteEntry};
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

#[derive(Debug, Clone)]
pub struct OziRasterTileSource {
    source_path: PathBuf,
    levels: Vec<OziRasterLevelMetadata>,
    raster: OziRaster,
}

impl OziRasterTileSource {
    fn new(source_path: PathBuf, levels: Vec<OziRasterLevelMetadata>, raster: OziRaster) -> Self {
        Self {
            source_path,
            levels,
            raster,
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
        let decoded_tile = self.raster.decode_tile(
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
}

impl fmt::Display for OziRasterDecodeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnsupportedRasterKind(kind) => {
                write!(f, "unsupported OZI raster kind for decoding: {kind:?}")
            }
            Self::Decode(error) => write!(f, "failed to decode OZF raster: {error}"),
        }
    }
}

impl std::error::Error for OziRasterDecodeError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::UnsupportedRasterKind(_) => None,
            Self::Decode(error) => Some(error),
        }
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
    let image = source.raster.decode_rgba_image(0)?;

    Ok(DecodedOziRasterImage::new(
        source.source_path.clone(),
        base_level.width(),
        base_level.height(),
        image.pixels().to_vec(),
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
        raster,
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
    use ozf2_rs::{DecodedTile, PaletteEntry};

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
