/// Affine georeference derived from OZI calibration points.
///
/// Converts between WGS-84 lat/lon (decimal degrees) and raster pixel coordinates.
#[derive(Debug, Clone)]
pub struct OziGeoreference {
    lon_x: f64,
    lat_x: f64,
    offset_x: f64,
    lon_y: f64,
    lat_y: f64,
    offset_y: f64,
    px_lon: f64,
    px_lat: f64,
    px_offset: f64,
    py_lon: f64,
    py_lat: f64,
    py_offset: f64,
}

impl OziGeoreference {
    /// Returns `(pixel_x, pixel_y)` for the given geographic coordinate.
    pub fn lat_lon_to_pixel(&self, lat: f64, lon: f64) -> (f64, f64) {
        let px = lon * self.lon_x + lat * self.lat_x + self.offset_x;
        let py = lon * self.lon_y + lat * self.lat_y + self.offset_y;
        (px, py)
    }

    /// Inverse of `lat_lon_to_pixel`: returns `(lat, lon)` for the given pixel coordinate.
    pub fn pixel_to_lat_lon(&self, px: f64, py: f64) -> (f64, f64) {
        let lon = px * self.px_lon + py * self.py_lon + self.px_offset;
        let lat = px * self.px_lat + py * self.py_lat + self.py_offset;
        (lat, lon)
    }

    /// Absolute number of level-0 pixels per degree of longitude.
    pub fn pixels_per_lon_degree(&self) -> f64 {
        self.lon_x.hypot(self.lon_y)
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

    if points.len() == 2 {
        return fit_axis_aligned(&points);
    }

    fit_affine(&points).or_else(|| fit_axis_aligned(&points))
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

fn fit_axis_aligned(points: &[(f64, f64, f64, f64)]) -> Option<OziGeoreference> {
    let (lon_x, offset_x) = linear_fit(points.iter().map(|p| (p.3, p.0)))?;
    let (lat_y, offset_y) = linear_fit(points.iter().map(|p| (p.2, p.1)))?;
    build_georeference(lon_x, 0.0, offset_x, 0.0, lat_y, offset_y)
}

fn fit_affine(points: &[(f64, f64, f64, f64)]) -> Option<OziGeoreference> {
    let x_coeffs = affine_fit(
        points
            .iter()
            .map(|(px, _, lat, lon)| (*lon, *lat, *px))
            .collect(),
    )?;
    let y_coeffs = affine_fit(
        points
            .iter()
            .map(|(_, py, lat, lon)| (*lon, *lat, *py))
            .collect(),
    )?;

    build_georeference(
        x_coeffs[0],
        x_coeffs[1],
        x_coeffs[2],
        y_coeffs[0],
        y_coeffs[1],
        y_coeffs[2],
    )
}

fn build_georeference(
    lon_x: f64,
    lat_x: f64,
    offset_x: f64,
    lon_y: f64,
    lat_y: f64,
    offset_y: f64,
) -> Option<OziGeoreference> {
    let det = lon_x * lat_y - lat_x * lon_y;
    if det.abs() < 1e-12 {
        return None;
    }

    let px_lon = lat_y / det;
    let py_lon = -lat_x / det;
    let px_lat = -lon_y / det;
    let py_lat = lon_x / det;

    Some(OziGeoreference {
        lon_x,
        lat_x,
        offset_x,
        lon_y,
        lat_y,
        offset_y,
        px_lon,
        px_lat,
        px_offset: -(offset_x * px_lon + offset_y * py_lon),
        py_lon,
        py_lat,
        py_offset: -(offset_x * px_lat + offset_y * py_lat),
    })
}

fn affine_fit(points: Vec<(f64, f64, f64)>) -> Option<[f64; 3]> {
    if points.len() < 3 {
        return None;
    }

    let mut ata = [[0.0; 3]; 3];
    let mut atb = [0.0; 3];

    for (x0, x1, y) in points {
        let row = [x0, x1, 1.0];
        for r in 0..3 {
            atb[r] += row[r] * y;
            for c in 0..3 {
                ata[r][c] += row[r] * row[c];
            }
        }
    }

    solve_3x3(ata, atb)
}

fn solve_3x3(mut a: [[f64; 3]; 3], mut b: [f64; 3]) -> Option<[f64; 3]> {
    for pivot in 0..3 {
        let mut best = pivot;
        for row in (pivot + 1)..3 {
            if a[row][pivot].abs() > a[best][pivot].abs() {
                best = row;
            }
        }
        if a[best][pivot].abs() < 1e-12 {
            return None;
        }
        if best != pivot {
            a.swap(best, pivot);
            b.swap(best, pivot);
        }

        let div = a[pivot][pivot];
        for item in &mut a[pivot][pivot..3] {
            *item /= div;
        }
        b[pivot] /= div;

        for row in 0..3 {
            if row == pivot {
                continue;
            }
            let factor = a[row][pivot];
            #[allow(clippy::needless_range_loop)]
            for col in pivot..3 {
                a[row][col] -= factor * a[pivot][col];
            }
            b[row] -= factor * b[pivot];
        }
    }

    Some(b)
}

#[cfg(test)]
mod tests {
    use super::{
        OziGeoreference, affine_fit, linear_fit, parse_calibration_point, parse_ozi_georeference,
    };

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
            lon_x: 1000.0,
            lat_x: 0.0,
            offset_x: -10000.0,
            lon_y: 0.0,
            lat_y: -1200.0,
            offset_y: 70000.0,
            px_lon: 1.0 / 1000.0,
            px_lat: 0.0,
            px_offset: 10.0,
            py_lon: 0.0,
            py_lat: -1.0 / 1200.0,
            py_offset: 58.333333333333336,
        };

        let (px, py) = geo.lat_lon_to_pixel(55.0, 37.0);
        assert_eq!(px, 37.0 * 1000.0 - 10000.0);
        assert_eq!(py, 55.0 * -1200.0 + 70000.0);
    }

    #[test]
    fn affine_fit_handles_cross_axis_terms() {
        let coeffs = affine_fit(vec![
            (1.0, 2.0, 17.0),
            (2.0, 3.0, 24.0),
            (3.0, 5.0, 36.0),
            (4.0, 7.0, 48.0),
        ])
        .expect("affine fit");

        assert!((coeffs[0] - 2.0).abs() < 1e-9);
        assert!((coeffs[1] - 5.0).abs() < 1e-9);
        assert!((coeffs[2] - 5.0).abs() < 1e-9);
    }

    #[test]
    fn parse_ozi_georeference_with_three_points_round_trips_rotated_fit() {
        let pts = vec![
            "Point01,xy,100,200,in, deg,54,30.000,N,48,24.000,E, grid, , , ,N".to_owned(),
            "Point02,xy,220,260,in, deg,54,31.000,N,48,25.000,E, grid, , , ,N".to_owned(),
            "Point03,xy,170,340,in, deg,54,32.000,N,48,24.300,E, grid, , , ,N".to_owned(),
        ];

        let geo = parse_ozi_georeference(&pts).expect("georeference");
        let (px, py) = geo.lat_lon_to_pixel(54.5, 48.4);
        assert!((px - 100.0).abs() < 1e-6, "pixel_x={px}");
        assert!((py - 200.0).abs() < 1e-6, "pixel_y={py}");

        let (lat, lon) = geo.pixel_to_lat_lon(170.0, 340.0);
        assert!((lat - 54.53333333333333).abs() < 1e-6, "lat={lat}");
        assert!((lon - 48.405).abs() < 1e-6, "lon={lon}");
    }
}
