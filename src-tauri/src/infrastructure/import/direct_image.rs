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
///
/// Not `Clone`: this is hundreds of megabytes, and the one place that used to
/// clone it did so on every tile request.
#[derive(Debug)]
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
        let (width, height) = (base.width(), base.height());
        // The decoded image is dropped here rather than kept for the resizes:
        // it is another four bytes a pixel alive for the whole build, and
        // `image::resize` would add a float buffer of half the height on top
        // — about sixteen bytes a pixel at the peak against the four the limit
        // was written for. Found by a reviewer, 2026-09-23.
        drop(image);

        let mut levels = vec![Level {
            width,
            height,
            rgba: base.into_raw(),
        }];

        loop {
            let last = levels.last().expect("level 0 exists");
            if last.width.min(last.height) <= MIN_LEVEL_SIDE {
                break;
            }
            levels.push(halve(last));
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

/// One level, halved.
///
/// A box filter over each 2×2 block, rather than nearest neighbour: a
/// topographic sheet is mostly thin contour lines, and dropping every other
/// pixel loses them. Done over the RGBA buffer directly, so the only memory
/// this costs is the level it produces — a quarter of the one before it.
fn halve(source: &Level) -> Level {
    let width = (source.width / 2).max(1);
    let height = (source.height / 2).max(1);
    let mut rgba = Vec::with_capacity((width as usize) * (height as usize) * 4);

    for y in 0..height {
        for x in 0..width {
            // The four source pixels this one stands for. An odd edge reuses
            // the last row or column rather than reading past it.
            let x0 = (x * 2).min(source.width - 1) as usize;
            let x1 = (x * 2 + 1).min(source.width - 1) as usize;
            let y0 = (y * 2).min(source.height - 1) as usize;
            let y1 = (y * 2 + 1).min(source.height - 1) as usize;
            let row0 = y0 * source.width as usize;
            let row1 = y1 * source.width as usize;

            for channel in 0..4 {
                let sum = u32::from(source.rgba[(row0 + x0) * 4 + channel])
                    + u32::from(source.rgba[(row0 + x1) * 4 + channel])
                    + u32::from(source.rgba[(row1 + x0) * 4 + channel])
                    + u32::from(source.rgba[(row1 + x1) * 4 + channel]);
                rgba.push((sum / 4) as u8);
            }
        }
    }

    Level {
        width,
        height,
        rgba,
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

    /// Halving must average, not sample: a contour line one pixel wide is
    /// most of what a topographic sheet carries, and dropping every other
    /// pixel loses half of them.
    #[test]
    fn halving_averages_rather_than_samples() {
        // A 2×2 block of one white pixel and three black ones becomes one
        // pixel of a quarter white — nearest neighbour would make it white or
        // black depending on which corner it read.
        let mut image = RgbaImage::new(2, 2);
        image.put_pixel(0, 0, image::Rgba([255, 255, 255, 255]));
        image.put_pixel(1, 0, image::Rgba([0, 0, 0, 255]));
        image.put_pixel(0, 1, image::Rgba([0, 0, 0, 255]));
        image.put_pixel(1, 1, image::Rgba([0, 0, 0, 255]));

        let level = super::halve(&super::Level {
            width: 2,
            height: 2,
            rgba: image.into_raw(),
        });

        assert_eq!((level.width, level.height), (1, 1));
        assert_eq!(level.rgba[0], 63, "one white corner of four is a quarter");
        assert_eq!(level.rgba[3], 255, "opaque stays opaque");
    }

    #[test]
    fn halving_an_odd_side_does_not_read_past_the_edge() {
        // 3×3 halves to 1×1, and the block for it runs off the picture.
        let level = super::halve(&super::Level {
            width: 3,
            height: 3,
            rgba: vec![128; 3 * 3 * 4],
        });
        assert_eq!((level.width, level.height), (1, 1));
        assert_eq!(level.rgba, vec![128, 128, 128, 128]);
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
