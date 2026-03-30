/// Affine georeference derived from OZI calibration points.
///
/// Converts between WGS-84 lat/lon (decimal degrees) and raster pixel coordinates
/// using two independent linear fits:
/// ```text
/// pixel_x = scale_x * lon + offset_x
/// pixel_y = scale_y * lat + offset_y
/// ```
#[derive(Debug, Clone)]
pub struct OziGeoreference {
    scale_x: f64,
    offset_x: f64,
    scale_y: f64,
    offset_y: f64,
}

impl OziGeoreference {
    /// Returns `(pixel_x, pixel_y)` for the given geographic coordinate.
    pub fn lat_lon_to_pixel(&self, lat: f64, lon: f64) -> (f64, f64) {
        let px = lon * self.scale_x + self.offset_x;
        let py = lat * self.scale_y + self.offset_y;
        (px, py)
    }
}

/// Parse OZI calibration point strings into an `OziGeoreference`.
///
/// Returns `None` if fewer than two valid calibration points are found or the
/// calibration points are degenerate (all at the same latitude or longitude).
///
/// Each calibration point string is expected in the OziExplorer `.map` file format:
/// ```text
/// Point01,xy,  100,  200,in, deg,54,30.000,N,48,24.000,E, grid, , , ,N
/// ```
pub fn parse_ozi_georeference(calibration_points: &[String]) -> Option<OziGeoreference> {
    let points: Vec<(f64, f64, f64, f64)> = calibration_points
        .iter()
        .filter_map(|line| parse_calibration_point(line))
        .collect();

    if points.len() < 2 {
        return None;
    }

    // pixel_x = scale_x * lon + offset_x
    let (scale_x, offset_x) = linear_fit(points.iter().map(|p| (p.3, p.0)))?;
    // pixel_y = scale_y * lat + offset_y
    let (scale_y, offset_y) = linear_fit(points.iter().map(|p| (p.2, p.1)))?;

    Some(OziGeoreference {
        scale_x,
        offset_x,
        scale_y,
        offset_y,
    })
}

/// Parse a single calibration point line.
///
/// Returns `(pixel_x, pixel_y, lat, lon)` on success.
///
/// Field layout (comma-separated, trimmed):
/// - [0]  Point name
/// - [1]  "xy"
/// - [2]  pixel_x
/// - [3]  pixel_y
/// - [4]  unit ("in")
/// - [5]  "deg"
/// - [6]  lat degrees
/// - [7]  lat minutes
/// - [8]  N/S hemisphere
/// - [9]  lon degrees
/// - [10] lon minutes
/// - [11] E/W hemisphere
fn parse_calibration_point(line: &str) -> Option<(f64, f64, f64, f64)> {
    let fields: Vec<&str> = line.split(',').map(str::trim).collect();
    if fields.len() < 12 {
        return None;
    }

    let pixel_x: f64 = fields[2].parse().ok()?;
    let pixel_y: f64 = fields[3].parse().ok()?;
    let lat_deg: f64 = fields[6].parse().ok()?;
    let lat_min: f64 = fields[7].parse().ok()?;
    let lat_sign = if fields[8].eq_ignore_ascii_case("S") {
        -1.0_f64
    } else {
        1.0_f64
    };
    let lon_deg: f64 = fields[9].parse().ok()?;
    let lon_min: f64 = fields[10].parse().ok()?;
    let lon_sign = if fields[11].eq_ignore_ascii_case("W") {
        -1.0_f64
    } else {
        1.0_f64
    };

    let lat = lat_sign * (lat_deg + lat_min / 60.0);
    let lon = lon_sign * (lon_deg + lon_min / 60.0);

    Some((pixel_x, pixel_y, lat, lon))
}

/// Fit `y = slope * x + intercept` via ordinary least squares.
///
/// Returns `None` when the denominator is near zero (degenerate, all x values equal).
fn linear_fit(points: impl Iterator<Item = (f64, f64)>) -> Option<(f64, f64)> {
    let pts: Vec<(f64, f64)> = points.collect();
    let n = pts.len() as f64;

    let sum_x: f64 = pts.iter().map(|p| p.0).sum();
    let sum_y: f64 = pts.iter().map(|p| p.1).sum();
    let sum_xx: f64 = pts.iter().map(|p| p.0 * p.0).sum();
    let sum_xy: f64 = pts.iter().map(|p| p.0 * p.1).sum();

    let denom = n * sum_xx - sum_x * sum_x;
    if denom.abs() < f64::EPSILON {
        return None;
    }

    let slope = (n * sum_xy - sum_x * sum_y) / denom;
    let intercept = (sum_y - slope * sum_x) / n;

    Some((slope, intercept))
}

#[cfg(test)]
mod tests {
    use super::{OziGeoreference, linear_fit, parse_calibration_point, parse_ozi_georeference};

    #[test]
    fn parse_calibration_point_extracts_pixel_and_latlon() {
        let (px, py, lat, lon) = parse_calibration_point(
            "Point01,xy,100,200,in, deg,54,30.000,N,48,24.000,E, grid, , , ,N",
        )
        .expect("valid point");

        assert_eq!(px, 100.0);
        assert_eq!(py, 200.0);
        assert!((lat - 54.5).abs() < 1e-9);
        assert!((lon - 48.4).abs() < 1e-9);
    }

    #[test]
    fn parse_calibration_point_handles_southern_western_hemisphere() {
        let (_, _, lat, lon) = parse_calibration_point(
            "Point01,xy,100,200,in, deg,10,30.000,S,70,15.000,W, grid, , , ,N",
        )
        .expect("valid point");

        assert!((lat - (-10.5)).abs() < 1e-9);
        assert!((lon - (-70.25)).abs() < 1e-9);
    }

    #[test]
    fn parse_calibration_point_returns_none_for_short_line() {
        assert!(parse_calibration_point("Point01,xy,100,200").is_none());
    }

    #[test]
    fn linear_fit_recovers_exact_slope_and_intercept() {
        // y = 3x + 7
        let points = vec![(1.0_f64, 10.0_f64), (2.0, 13.0), (3.0, 16.0)];
        let (slope, intercept) = linear_fit(points.into_iter()).expect("fit");

        assert!((slope - 3.0).abs() < 1e-9);
        assert!((intercept - 7.0).abs() < 1e-9);
    }

    #[test]
    fn linear_fit_returns_none_for_degenerate_same_x() {
        let points = vec![(5.0_f64, 1.0_f64), (5.0, 2.0)];

        assert!(linear_fit(points.into_iter()).is_none());
    }

    #[test]
    fn parse_ozi_georeference_produces_correct_pixel_coordinates() {
        let pts = vec![
            "Point01,xy,100,200,in, deg,54,30.000,N,48,24.000,E, grid, , , ,N".to_owned(),
            "Point02,xy,300,400,in, deg,54,31.000,N,48,25.000,E, grid, , , ,N".to_owned(),
        ];

        let geo = parse_ozi_georeference(&pts).expect("georeference");

        // Round-trip: pixel_x for Point01 should be ~100, pixel_y should be ~200.
        let (px, py) = geo.lat_lon_to_pixel(54.5, 48.4);
        assert!((px - 100.0).abs() < 1.0, "pixel_x={px}");
        assert!((py - 200.0).abs() < 1.0, "pixel_y={py}");
    }

    #[test]
    fn parse_ozi_georeference_returns_none_for_single_point() {
        let pts =
            vec!["Point01,xy,100,200,in, deg,54,30.000,N,48,24.000,E, grid, , , ,N".to_owned()];

        assert!(parse_ozi_georeference(&pts).is_none());
    }

    #[test]
    fn ozi_georeference_lat_lon_to_pixel_is_affine() {
        let geo = OziGeoreference {
            scale_x: 1000.0,
            offset_x: -10000.0,
            scale_y: -1200.0,
            offset_y: 70000.0,
        };

        let (px, py) = geo.lat_lon_to_pixel(55.0, 37.0);
        assert_eq!(px, 37.0 * 1000.0 - 10000.0);
        assert_eq!(py, 55.0 * -1200.0 + 70000.0);
    }
}
