//! A raster that is just a picture: `.map` beside `.jpg`, `.png` or `.tif`.
//!
//! OziExplorer's own pairing is a calibration file and an ordinary image; OZF2
//! is the optimised form somebody produced later with Img2Ozf. A headquarters
//! is handed the first far more often than the second — a scan of a sheet, a
//! screenshot of a web map, a photograph of a paper map on a table — and this
//! application refused all of them, answering "unsupported raster kind" for a
//! file OziExplorer opens without comment.
//!
//! There is no random access into a JPEG: the decoder has to walk the whole
//! image whatever part of it is wanted. So the picture is decoded once, halved
//! repeatedly into a pyramid, and tiles are cut out of those buffers. That
//! costs memory — four bytes a pixel, a third again for the pyramid — which is
//! why [`MAX_PIXELS`] exists: a field laptop that runs out of memory gives no
//! reason for it, and a refusal that names the size does.

use image::imageops::FilterType;
use image::{DynamicImage, ImageReader};
use std::fmt;
use std::path::{Path, PathBuf};

/// The side of a tile this source cuts, matching what the map asks for.
pub const TILE_SIZE: u32 = 256;

/// How small the smallest level may get before the pyramid stops.
const MIN_LEVEL_SIDE: u32 = 256;

/// The largest picture this will hold in memory, in pixels.
///
/// 120 megapixels is 480 MB of RGBA at full size and about 640 MB with the
/// pyramid — a 1:25000 sheet scanned at 600 dpi, and more than anything a
/// crew hands over in practice. Beyond it the honest answer is the size, not
/// an allocation failure halfway through a launch.
pub const MAX_PIXELS: u64 = 120_000_000;

#[derive(Debug)]
pub enum DirectImageError {
    Read {
        path: PathBuf,
        message: String,
    },
    Decode {
        path: PathBuf,
        message: String,
    },
    TooLarge {
        path: PathBuf,
        width: u32,
        height: u32,
    },
}

impl fmt::Display for DirectImageError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Read { path, message } => {
                write!(f, "не удалось открыть {}: {message}", path.display())
            }
            Self::Decode { path, message } => {
                write!(f, "не удалось прочитать {}: {message}", path.display())
            }
            Self::TooLarge {
                path,
                width,
                height,
            } => write!(
                f,
                "{} — {width}×{height} точек, это больше {} мегапикселей, \
                 которые можно держать в памяти; уменьшите картинку или \
                 переведите её в OZF2",
                path.display(),
                MAX_PIXELS / 1_000_000
            ),
        }
    }
}

impl std::error::Error for DirectImageError {}

/// One level of the pyramid: RGBA pixels and how wide a row is.
#[derive(Debug, Clone)]
struct Level {
    width: u32,
    height: u32,
    rgba: Vec<u8>,
}

/// A decoded picture with a pyramid over it, cut into tiles on demand.
///
/// It does not keep the path it came from: the tile source above it owns that,
/// and two copies of a path are two chances to answer with the wrong one.
#[derive(Debug, Clone)]
pub struct DirectImageRaster {
    levels: Vec<Level>,
}

impl DirectImageRaster {
    /// Decode the picture and build the pyramid.
    ///
    /// Level 0 is the picture itself; each level after it is the one before
    /// halved, down to [`MIN_LEVEL_SIDE`]. The same shape the OZF2 reader
    /// reports, so everything above this does not have to know which it got.
    pub fn open(path: impl AsRef<Path>) -> Result<Self, DirectImageError> {
        let path = path.as_ref();

        let read_error = |error: std::io::Error| DirectImageError::Read {
            path: path.to_path_buf(),
            message: error.to_string(),
        };
        let decode_error = |error: image::ImageError| DirectImageError::Decode {
            path: path.to_path_buf(),
            message: error.to_string(),
        };

        // The header first. A picture too big to hold is refused before its
        // pixels are read, so the refusal costs a file open rather than a
        // minute and half a gigabyte.
        let (width, height) = ImageReader::open(path)
            .map_err(read_error)?
            .with_guessed_format()
            .map_err(read_error)?
            .into_dimensions()
            .map_err(decode_error)?;
        if u64::from(width) * u64::from(height) > MAX_PIXELS {
            return Err(DirectImageError::TooLarge {
                path: path.to_path_buf(),
                width,
                height,
            });
        }

        let image = ImageReader::open(path)
            .map_err(read_error)?
            .with_guessed_format()
            .map_err(read_error)?
            .decode()
            .map_err(decode_error)?;

        Ok(Self::from_image(image))
    }

    fn from_image(image: DynamicImage) -> Self {
        let base = image.to_rgba8();
        let mut levels = vec![Level {
            width: base.width(),
            height: base.height(),
            rgba: base.into_raw(),
        }];

        let mut current = image;
        loop {
            let (width, height) = (current.width(), current.height());
            if width.min(height) <= MIN_LEVEL_SIDE {
                break;
            }
            // Triangle filtering: a map halved by nearest neighbour loses thin
            // contour lines entirely, which is most of what a topographic sheet
            // is made of.
            current = current.resize_exact(
                (width / 2).max(1),
                (height / 2).max(1),
                FilterType::Triangle,
            );
            let rgba = current.to_rgba8();
            levels.push(Level {
                width: rgba.width(),
                height: rgba.height(),
                rgba: rgba.into_raw(),
            });
        }

        Self { levels }
    }

    pub fn level_count(&self) -> usize {
        self.levels.len()
    }

    /// `(width, height)` of a level, or `None` past the end of the pyramid.
    pub fn level_size(&self, level_index: usize) -> Option<(u32, u32)> {
        self.levels.get(level_index).map(|l| (l.width, l.height))
    }

    /// Every pixel of a level, RGBA, row by row.
    pub fn level_rgba(&self, level_index: usize) -> Option<&[u8]> {
        self.levels.get(level_index).map(|l| l.rgba.as_slice())
    }

    /// Cut one tile out of a level.
    ///
    /// Answers the tile's own size, which on the right and bottom edges is
    /// smaller than [`TILE_SIZE`] — the same convention the OZF2 reader uses,
    /// so the caller crops nothing.
    pub fn tile_rgba(
        &self,
        level_index: usize,
        tile_x: u32,
        tile_y: u32,
    ) -> Option<(u32, u32, Vec<u8>)> {
        let level = self.levels.get(level_index)?;
        let left = tile_x.checked_mul(TILE_SIZE)?;
        let top = tile_y.checked_mul(TILE_SIZE)?;
        if left >= level.width || top >= level.height {
            return None;
        }

        let width = TILE_SIZE.min(level.width - left);
        let height = TILE_SIZE.min(level.height - top);
        let mut out = Vec::with_capacity((width * height * 4) as usize);
        for row in 0..height {
            let start = (((top + row) * level.width + left) * 4) as usize;
            let end = start + (width * 4) as usize;
            out.extend_from_slice(&level.rgba[start..end]);
        }
        Some((width, height, out))
    }
}

#[cfg(test)]
mod tests {
    use super::{DirectImageError, DirectImageRaster, TILE_SIZE};
    use image::{DynamicImage, RgbaImage};

    fn picture(width: u32, height: u32) -> DynamicImage {
        let mut image = RgbaImage::new(width, height);
        for (x, y, pixel) in image.enumerate_pixels_mut() {
            *pixel = image::Rgba([(x % 256) as u8, (y % 256) as u8, 128, 255]);
        }
        DynamicImage::ImageRgba8(image)
    }

    #[test]
    fn a_small_picture_is_one_level() {
        let raster = DirectImageRaster::from_image(picture(300, 200));
        assert_eq!(raster.level_count(), 1);
        assert_eq!(raster.level_size(0), Some((300, 200)));
    }

    #[test]
    fn the_pyramid_halves_until_a_side_fits_one_tile() {
        // 2048×1024 → 1024×512 → stops: the short side is at the floor.
        let raster = DirectImageRaster::from_image(picture(2048, 1024));
        assert_eq!(raster.level_size(0), Some((2048, 1024)));
        assert_eq!(raster.level_size(1), Some((1024, 512)));
        assert_eq!(raster.level_size(2), Some((512, 256)));
        assert_eq!(raster.level_size(3), None);
    }

    #[test]
    fn a_tile_in_the_middle_is_a_full_tile() {
        let raster = DirectImageRaster::from_image(picture(1000, 1000));
        let (width, height, pixels) = raster.tile_rgba(0, 1, 1).expect("tile 1,1");
        assert_eq!((width, height), (TILE_SIZE, TILE_SIZE));
        assert_eq!(pixels.len() as u32, TILE_SIZE * TILE_SIZE * 4);
        // The tile starts at (256, 256) of a picture whose red channel is x%256.
        assert_eq!(pixels[0], 0);
        assert_eq!(pixels[1], 0);
    }

    #[test]
    fn an_edge_tile_is_only_as_big_as_what_is_there() {
        // 1000 = 3 whole tiles and 232 pixels.
        let raster = DirectImageRaster::from_image(picture(1000, 1000));
        let (width, height, pixels) = raster.tile_rgba(0, 3, 3).expect("tile 3,3");
        assert_eq!((width, height), (232, 232));
        assert_eq!(pixels.len(), 232 * 232 * 4);
    }

    #[test]
    fn a_tile_past_the_edge_is_not_there() {
        let raster = DirectImageRaster::from_image(picture(500, 500));
        assert!(raster.tile_rgba(0, 2, 0).is_none());
        assert!(raster.tile_rgba(9, 0, 0).is_none());
    }

    #[test]
    fn a_missing_file_says_which_one() {
        let error = DirectImageRaster::open("/nowhere/sheet.jpg").expect_err("must fail");
        assert!(matches!(error, DirectImageError::Read { .. }));
        assert!(error.to_string().contains("/nowhere/sheet.jpg"));
    }
}
