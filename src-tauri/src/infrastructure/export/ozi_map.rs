//! Writing an OziExplorer `.map` for a picture that arrived without one.
//!
//! A headquarters is sometimes handed an image and nothing else: a screenshot
//! of a web map, a photograph of a sheet on a table, a raster somebody
//! exported from their own GIS. It is a map of the search area and it is
//! useless, because nothing says where on the Earth it sits.
//!
//! Calibration is the operator saying "this pixel is here": two or more points
//! tying a place in the picture to a latitude and longitude. From those an
//! affine fit gives every other pixel, and the result is written as a `.map`
//! beside the image — a real OziExplorer file, so the same pair opens in
//! OziExplorer on somebody else's laptop as well as here.
//!
//! The fit is affine over latitude and longitude, which is what
//! `ozi_georeference.rs` reads back and what OziExplorer itself does from two
//! points. Over a search area — twenty or thirty kilometres — the departure
//! from a true projection is metres. Over a sheet spanning degrees it is not,
//! and no number of calibration points fixes that: an affine cannot bend.

use std::fmt;
use std::path::Path;

/// One place in the picture whose position on the Earth the operator knows.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CalibrationPoint {
    pub pixel_x: f64,
    pub pixel_y: f64,
    pub lat: f64,
    pub lon: f64,
}

#[derive(Debug, PartialEq)]
pub enum CalibrationError {
    /// Fewer than two points: one point fixes a position and nothing else —
    /// not the scale, not which way is north.
    TooFewPoints { given: usize },
    /// Every point shares a latitude, or every point shares a longitude. Two
    /// places on the same parallel say nothing about north–south scale.
    Degenerate,
    /// A coordinate outside the Earth, which is a typo rather than a place.
    OutOfRange { lat: f64, lon: f64 },
}

impl fmt::Display for CalibrationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::TooFewPoints { given } => write!(
                f,
                "для привязки нужно минимум две точки, задана {given}: одна \
                 точка задаёт положение, но не масштаб и не поворот"
            ),
            Self::Degenerate => write!(
                f,
                "точки привязки лежат на одной параллели или на одном \
                 меридиане — по ним нельзя определить масштаб во второе \
                 направление; возьмите точки по диагонали"
            ),
            Self::OutOfRange { lat, lon } => {
                write!(f, "координата вне Земли: {lat}, {lon} — похоже на опечатку")
            }
        }
    }
}

impl std::error::Error for CalibrationError {}

/// Degrees and decimal minutes with a hemisphere, the way a `.map` holds them.
///
/// Four decimals of a minute is about 0.19 m of latitude — finer than any
/// point an operator picks off a picture, and the format's own convention.
fn degrees_and_minutes(value: f64, positive: char, negative: char) -> String {
    let hemisphere = if value < 0.0 { negative } else { positive };
    let magnitude = value.abs();
    let degrees = magnitude.trunc();
    let minutes = (magnitude - degrees) * 60.0;
    format!("{degrees:.0},{minutes:.4},{hemisphere}")
}

/// Build the text of a `.map` tying `raster_file_name` to the Earth.
///
/// `raster_file_name` is written as it is given: OziExplorer resolves a bare
/// name beside the `.map`, which is what keeps a calibrated pair movable
/// between machines.
pub fn build_calibration_map(
    title: &str,
    raster_file_name: &str,
    image_width: u32,
    image_height: u32,
    points: &[CalibrationPoint],
) -> Result<String, CalibrationError> {
    if points.len() < 2 {
        return Err(CalibrationError::TooFewPoints {
            given: points.len(),
        });
    }

    for point in points {
        if !(-90.0..=90.0).contains(&point.lat) || !(-180.0..=180.0).contains(&point.lon) {
            return Err(CalibrationError::OutOfRange {
                lat: point.lat,
                lon: point.lon,
            });
        }
    }

    let first = points[0];
    let spread_lat = points.iter().any(|p| (p.lat - first.lat).abs() > 1e-9);
    let spread_lon = points.iter().any(|p| (p.lon - first.lon).abs() > 1e-9);
    if !spread_lat || !spread_lon {
        return Err(CalibrationError::Degenerate);
    }

    // A comma in the title would become another field. The title is the
    // operator's own words, so it is repaired rather than refused.
    let safe_title = title.replace([',', '\r', '\n'], " ");

    let mut out = String::new();
    out.push_str("OziExplorer Map Data File Version 2.2\r\n");
    out.push_str(&safe_title);
    out.push_str("\r\n");
    out.push_str(raster_file_name);
    out.push_str("\r\n");
    out.push_str("1 ,Map Code,\r\n");
    out.push_str("WGS 84,,   0.0000,   0.0000,WGS 84\r\n");
    out.push_str("Reserved 1\r\n");
    out.push_str("Reserved 2\r\n");
    out.push_str("Magnetic Variation,,,E\r\n");
    out.push_str("Map Projection,Latitude/Longitude,PolyCal,No,AutoCalOnly,No,BSBUseWPX,No\r\n");

    // Thirty point slots, as OziExplorer writes them: the ones in use carry
    // their values, the rest are present and empty. A reader that counts
    // lines — and some do — finds what it expects.
    for index in 0..30 {
        let number = index + 1;
        match points.get(index) {
            Some(point) => out.push_str(&format!(
                "Point{number:02},xy,{:>5},{:>5},in, deg,{},{}, grid, , , ,N\r\n",
                point.pixel_x.round() as i64,
                point.pixel_y.round() as i64,
                degrees_and_minutes(point.lat, 'N', 'S'),
                degrees_and_minutes(point.lon, 'E', 'W'),
            )),
            None => out.push_str(&format!(
                "Point{number:02},xy,     ,     ,in, deg,    ,        ,N,    ,        ,E, grid, , , ,N\r\n"
            )),
        }
    }

    out.push_str("Projection Setup,,,,,,,,,,\r\n");
    out.push_str("Map Feature = MF ; Map Comment = MC     These follow if they exist\r\n");
    out.push_str("Track File = TF      These follow if they exist\r\n");
    out.push_str("Moving Map Parameters = MM?    These follow if they exist\r\n");
    out.push_str("MM0,Yes\r\n");
    out.push_str("MMPNUM,4\r\n");

    // The four corners of the picture, which is what OziExplorer uses to know
    // the extent without solving anything. They are the calibration read
    // backwards, so they come from the same fit rather than from the operator.
    let corners = [
        (0.0_f64, 0.0_f64),
        (f64::from(image_width), 0.0),
        (f64::from(image_width), f64::from(image_height)),
        (0.0, f64::from(image_height)),
    ];
    let fit = AffineFit::from_points(points).ok_or(CalibrationError::Degenerate)?;
    for (index, (px, py)) in corners.iter().enumerate() {
        out.push_str(&format!(
            "MMPXY,{},{},{}\r\n",
            index + 1,
            px.round() as i64,
            py.round() as i64
        ));
    }
    for (index, (px, py)) in corners.iter().enumerate() {
        let (lat, lon) = fit.pixel_to_lat_lon(*px, *py);
        out.push_str(&format!("MMPLL,{},{lon:.6},{lat:.6}\r\n", index + 1));
    }

    out.push_str(&format!("MM1B,{:.6}\r\n", fit.metres_per_pixel()));
    out.push_str("MOP,Map Open Position,0,0\r\n");
    out.push_str(&format!(
        "IWH,Map Image Width/Height,{image_width},{image_height}\r\n"
    ));

    Ok(out)
}

/// Write the `.map` beside the picture it calibrates.
///
/// Answers the path written. The name is the picture's with the extension
/// replaced — `sheet.jpg` gives `sheet.map` — which is the pairing
/// OziExplorer looks for.
pub fn write_calibration_map(
    image_path: &Path,
    title: &str,
    image_width: u32,
    image_height: u32,
    points: &[CalibrationPoint],
) -> Result<std::path::PathBuf, CalibrationWriteError> {
    let raster_file_name = image_path
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| CalibrationWriteError::BadPath {
            path: image_path.to_path_buf(),
        })?;

    let text = build_calibration_map(title, raster_file_name, image_width, image_height, points)
        .map_err(CalibrationWriteError::Calibration)?;

    let map_path = image_path.with_extension("map");
    // cp1251, like every other OziExplorer file this writes: a title in
    // Cyrillic read on a Russian Windows must not come back as mojibake.
    let (bytes, _, _) = encoding_rs::WINDOWS_1251.encode(&text);
    std::fs::write(&map_path, bytes).map_err(|error| CalibrationWriteError::Write {
        path: map_path.clone(),
        message: error.to_string(),
    })?;
    Ok(map_path)
}

#[derive(Debug)]
pub enum CalibrationWriteError {
    Calibration(CalibrationError),
    BadPath {
        path: std::path::PathBuf,
    },
    Write {
        path: std::path::PathBuf,
        message: String,
    },
}

impl fmt::Display for CalibrationWriteError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Calibration(error) => write!(f, "{error}"),
            Self::BadPath { path } => {
                write!(f, "не удалось определить имя файла: {}", path.display())
            }
            Self::Write { path, message } => {
                write!(f, "не удалось записать {}: {message}", path.display())
            }
        }
    }
}

impl std::error::Error for CalibrationWriteError {}

/// The affine this writer needs to answer the four corners.
///
/// The reader in `ozi_georeference.rs` solves the same thing from parsed point
/// lines; this one solves it from the values, before there is any text to
/// parse. Two points give an axis-aligned fit — no rotation, which is what two
/// points can honestly say — and three or more give a least-squares plane.
struct AffineFit {
    lat_px: f64,
    lat_py: f64,
    lat_c: f64,
    lon_px: f64,
    lon_py: f64,
    lon_c: f64,
}

impl AffineFit {
    fn from_points(points: &[CalibrationPoint]) -> Option<Self> {
        if points.len() == 2 {
            let (a, b) = (points[0], points[1]);
            let dx = b.pixel_x - a.pixel_x;
            let dy = b.pixel_y - a.pixel_y;
            if dx.abs() < f64::EPSILON || dy.abs() < f64::EPSILON {
                return None;
            }
            let lon_px = (b.lon - a.lon) / dx;
            let lat_py = (b.lat - a.lat) / dy;
            return Some(Self {
                lat_px: 0.0,
                lat_py,
                lat_c: a.lat - lat_py * a.pixel_y,
                lon_px,
                lon_py: 0.0,
                lon_c: a.lon - lon_px * a.pixel_x,
            });
        }

        let (lat_px, lat_py, lat_c) = least_squares_plane(points, |p| p.lat)?;
        let (lon_px, lon_py, lon_c) = least_squares_plane(points, |p| p.lon)?;
        Some(Self {
            lat_px,
            lat_py,
            lat_c,
            lon_px,
            lon_py,
            lon_c,
        })
    }

    fn pixel_to_lat_lon(&self, px: f64, py: f64) -> (f64, f64) {
        (
            self.lat_px * px + self.lat_py * py + self.lat_c,
            self.lon_px * px + self.lon_py * py + self.lon_c,
        )
    }

    /// Ground metres per picture pixel, for the `MM1B` line OziExplorer reads.
    ///
    /// Both components of the step, not just the longitude one. A photograph
    /// of a sheet on a table is rotated, and for a map turned ninety degrees a
    /// step along X is pure latitude — so measuring only the longitude wrote
    /// `MM1B,0.000000` for a picture whose pixels are very much a size. Found
    /// by a reviewer, 2026-09-23.
    fn metres_per_pixel(&self) -> f64 {
        let (lat0, lon0) = self.pixel_to_lat_lon(0.0, 0.0);
        let (lat1, lon1) = self.pixel_to_lat_lon(1.0, 0.0);
        let metres_per_lat_degree = 111_320.0;
        let metres_per_lon_degree = 111_320.0 * lat0.to_radians().cos();
        let east = (lon1 - lon0) * metres_per_lon_degree;
        let north = (lat1 - lat0) * metres_per_lat_degree;
        east.hypot(north)
    }
}

/// Least squares for `value = a*px + b*py + c`.
fn least_squares_plane(
    points: &[CalibrationPoint],
    value: impl Fn(&CalibrationPoint) -> f64,
) -> Option<(f64, f64, f64)> {
    let n = points.len() as f64;
    let (mut sx, mut sy, mut sv) = (0.0, 0.0, 0.0);
    for p in points {
        sx += p.pixel_x;
        sy += p.pixel_y;
        sv += value(p);
    }
    let (mx, my, mv) = (sx / n, sy / n, sv / n);

    let (mut sxx, mut sxy, mut syy, mut sxv, mut syv) = (0.0, 0.0, 0.0, 0.0, 0.0);
    for p in points {
        let (dx, dy, dv) = (p.pixel_x - mx, p.pixel_y - my, value(p) - mv);
        sxx += dx * dx;
        sxy += dx * dy;
        syy += dy * dy;
        sxv += dx * dv;
        syv += dy * dv;
    }

    let determinant = sxx * syy - sxy * sxy;
    if determinant.abs() < 1e-9 {
        return None;
    }
    let a = (sxv * syy - syv * sxy) / determinant;
    let b = (syv * sxx - sxv * sxy) / determinant;
    Some((a, b, mv - a * mx - b * my))
}

#[cfg(test)]
mod tests {
    use super::{CalibrationError, CalibrationPoint, build_calibration_map, write_calibration_map};
    use crate::infrastructure::import::parse_ozi_georeference;

    fn point(pixel_x: f64, pixel_y: f64, lat: f64, lon: f64) -> CalibrationPoint {
        CalibrationPoint {
            pixel_x,
            pixel_y,
            lat,
            lon,
        }
    }

    /// Two corners of a screenshot of a north-up web map — the commonest thing
    /// a headquarters has to calibrate.
    fn two_corners() -> Vec<CalibrationPoint> {
        vec![
            point(0.0, 0.0, 60.05, 30.20),
            point(1600.0, 1200.0, 59.95, 30.40),
        ]
    }

    #[test]
    fn one_point_is_refused_with_the_reason() {
        let error = build_calibration_map("Сагра", "sheet.jpg", 100, 100, &two_corners()[..1])
            .expect_err("one point cannot calibrate");
        assert_eq!(error, CalibrationError::TooFewPoints { given: 1 });
        assert!(error.to_string().contains("масштаб"));
    }

    #[test]
    fn points_on_one_parallel_are_refused() {
        let points = vec![point(0.0, 0.0, 60.0, 30.0), point(1000.0, 0.0, 60.0, 30.5)];
        assert_eq!(
            build_calibration_map("t", "s.png", 1000, 500, &points),
            Err(CalibrationError::Degenerate)
        );
    }

    #[test]
    fn a_coordinate_off_the_earth_is_a_typo() {
        let points = vec![
            point(0.0, 0.0, 160.0, 30.0),
            point(100.0, 100.0, 59.0, 31.0),
        ];
        assert!(matches!(
            build_calibration_map("t", "s.png", 100, 100, &points),
            Err(CalibrationError::OutOfRange { .. })
        ));
    }

    #[test]
    fn what_is_written_reads_back_as_the_same_places() {
        let points = two_corners();
        let text = build_calibration_map("Сагра, север", "sheet.jpg", 1600, 1200, &points)
            .expect("build map");

        // The title's comma would have become a field of its own.
        assert!(text.contains("Сагра  север"), "title: {text}");
        assert!(text.contains("sheet.jpg"));

        let lines: Vec<String> = text.lines().map(str::to_owned).collect();
        let point_lines: Vec<String> = lines
            .iter()
            .filter(|line| line.starts_with("Point"))
            .cloned()
            .collect();
        assert_eq!(point_lines.len(), 30, "OziExplorer writes thirty slots");

        let georeference =
            parse_ozi_georeference(&point_lines).expect("the reader must accept what we wrote");
        for expected in &points {
            let (lat, lon) = georeference.pixel_to_lat_lon(expected.pixel_x, expected.pixel_y);
            assert!(
                (lat - expected.lat).abs() < 1e-4,
                "lat {lat} vs {}",
                expected.lat
            );
            assert!(
                (lon - expected.lon).abs() < 1e-4,
                "lon {lon} vs {}",
                expected.lon
            );
        }
    }

    #[test]
    fn the_corners_are_the_calibration_read_backwards() {
        let text = build_calibration_map("t", "s.png", 1600, 1200, &two_corners()).expect("build");
        // Corner 1 is pixel (0,0), which is the first calibration point.
        assert!(
            text.contains("MMPLL,1,30.200000,60.050000"),
            "corners: {text}"
        );
        // Corner 3 is pixel (1600,1200), which is the second.
        assert!(text.contains("MMPLL,3,30.400000,59.950000"));
        assert!(text.contains("IWH,Map Image Width/Height,1600,1200"));
    }

    /// A photograph of a sheet on a table is rotated; for a map turned ninety
    /// degrees a step along X is pure latitude, and measuring only the
    /// longitude wrote a pixel size of zero.
    #[test]
    fn metres_per_pixel_survives_a_rotated_calibration() {
        // X runs north, Y runs east: a sheet photographed sideways.
        let points = vec![
            point(0.0, 0.0, 60.0, 30.0),
            point(1000.0, 0.0, 60.01, 30.0),
            point(0.0, 1000.0, 60.0, 30.02),
        ];
        let text = build_calibration_map("боком", "photo.jpg", 1000, 1000, &points).expect("build");
        let line = text
            .lines()
            .find(|line| line.starts_with("MM1B,"))
            .expect("MM1B");
        let metres: f64 = line["MM1B,".len()..].trim().parse().expect("a number");
        assert!(
            metres > 0.5 && metres < 5.0,
            "a thousand pixels over 0.01° of latitude is about 1.1 m a pixel, got {metres}"
        );
    }

    #[test]
    fn three_points_fit_a_rotated_picture() {
        // A photograph of a sheet on a table is never square to the camera.
        let points = vec![
            point(100.0, 100.0, 60.00, 30.00),
            point(1100.0, 200.0, 60.00, 30.10),
            point(0.0, 1100.0, 59.90, 30.00),
        ];
        let text = build_calibration_map("наклон", "photo.jpg", 1200, 1200, &points)
            .expect("three points fit");
        let point_lines: Vec<String> = text
            .lines()
            .filter(|line| line.starts_with("Point"))
            .map(str::to_owned)
            .collect();
        let georeference = parse_ozi_georeference(&point_lines).expect("reads back");
        for expected in &points {
            let (lat, lon) = georeference.pixel_to_lat_lon(expected.pixel_x, expected.pixel_y);
            assert!((lat - expected.lat).abs() < 1e-3, "lat {lat}");
            assert!((lon - expected.lon).abs() < 1e-3, "lon {lon}");
        }
    }

    #[test]
    fn the_file_lands_beside_the_picture_in_cp1251() {
        let dir = std::env::temp_dir().join(format!("ozi-rs-calib-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).expect("temp dir");
        let image_path = dir.join("Сагра.jpg");

        let map_path =
            write_calibration_map(&image_path, "Сагра", 1600, 1200, &two_corners()).expect("write");

        assert_eq!(map_path, dir.join("Сагра.map"));
        let bytes = std::fs::read(&map_path).expect("read back");
        // "Сагра" in cp1251 starts with 0xD1 0xE0; in UTF-8 it would be 0xD0 0xA1.
        assert!(
            bytes.windows(2).any(|w| w == [0xD1, 0xE0]),
            "the title must be cp1251"
        );
        let _ = std::fs::remove_dir_all(&dir);
    }
}
