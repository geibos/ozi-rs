use crate::infrastructure::import::{OziMapMetadata, OziRasterKind};
use ozf2_rs::{OzfError, OziRaster};
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

pub fn decode_ozi_raster_image(
    metadata: &OziMapMetadata,
) -> Result<DecodedOziRasterImage, OziRasterDecodeError> {
    match metadata.raster_kind() {
        OziRasterKind::Ozf2 => decode_ozf2_image(metadata),
        other => Err(OziRasterDecodeError::UnsupportedRasterKind(other.clone())),
    }
}

fn decode_ozf2_image(
    metadata: &OziMapMetadata,
) -> Result<DecodedOziRasterImage, OziRasterDecodeError> {
    let raster = OziRaster::open(metadata.resolved_raster_path())?;
    let image = raster.decode_rgba_image(0)?;

    Ok(DecodedOziRasterImage::new(
        metadata.resolved_raster_path().to_path_buf(),
        image.width(),
        image.height(),
        image.pixels().to_vec(),
    ))
}
