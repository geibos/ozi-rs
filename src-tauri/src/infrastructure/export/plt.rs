use crate::domain::Track;
use chrono::{Datelike, NaiveDate, Timelike};
use encoding_rs::WINDOWS_1251;
use std::io::Write;

const OLE_BASE_DATE: NaiveDate =
    NaiveDate::from_ymd_opt(1899, 12, 30).expect("valid OLE base date");

/// Tail of the track properties line after the name: skip value, track type,
/// fill style, fill colour, "closed" flag, reserved — copied verbatim from an
/// OziExplorer-produced file (see
/// `example_data/2021-07-30_Murino/.../2021-07-30_Murino_500m.plt`).
const TRACK_PROPERTIES_TAIL: &str = "0,0,2,8421376,-1,0";

#[derive(Debug)]
pub enum ExportError {
    Io(std::io::Error),
}

impl std::fmt::Display for ExportError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(err) => write!(f, "export write failed: {err}"),
        }
    }
}

impl std::error::Error for ExportError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io(err) => Some(err),
        }
    }
}

impl From<std::io::Error> for ExportError {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value)
    }
}

pub fn export_plt(
    track: &Track,
    color: u32,
    width: f64,
    writer: &mut impl Write,
) -> Result<(), ExportError> {
    let colorref = rgb_to_colorref_bgr(color);
    let width_int = map_line_width(width);
    let name = sanitise_name(track.name());
    let point_count: usize = track
        .segments()
        .iter()
        .map(|segment| segment.points().len())
        .sum();

    write_line(writer, "OziExplorer Track Point File Version 2.1")?;
    write_line(writer, "WGS 84")?;
    write_line(writer, "Altitude is in Feet")?;
    write_line(
        writer,
        "Field 1 = Lat, Field 2 = Lon, Field 3 = Code, Field 4 = Alt, Field 5 = Date, Field 6 = Stop, Field 7 = Bearing",
    )?;
    // Track properties in the layout OziExplorer (and our own importer,
    // `import/plt.rs::parse_track_style`) expects:
    // visible(0=shown), line_width, COLORREF, name, skip, type, fill_style,
    // fill_color, closed, reserved.
    write_line(
        writer,
        &format!("0,{width_int},{colorref},{name},{TRACK_PROPERTIES_TAIL}"),
    )?;
    write_line(writer, &point_count.to_string())?;

    for segment in track.segments() {
        for (index, point) in segment.points().iter().enumerate() {
            let is_segment_start = index == 0;
            let segment_flag = if is_segment_start { 1 } else { 0 };
            let altitude_ft = point
                .elevation()
                .map_or(-777_i32, |meters| (meters * 3.28084).round() as i32);
            let ole_date = point.timestamp().map_or(0.0, datetime_to_ole_date);
            let (date_field, time_field) = match point.timestamp() {
                Some(ts) => (
                    ts.format("%d-%m-%Y").to_string(),
                    ts.format("%H:%M:%S").to_string(),
                ),
                None => (String::new(), String::new()),
            };

            write_line(
                writer,
                &format!(
                    "{:.6},{:.6},{segment_flag},{altitude_ft},{ole_date:.7},{date_field},{time_field}",
                    point.latitude(),
                    point.longitude(),
                ),
            )?;
        }
    }

    Ok(())
}

/// Encode `line + "\r\n"` as Windows-1251 and append to writer.
///
/// Mirrors `infrastructure::export::wpt::write_line`: OziExplorer on Russian
/// Windows reads cp1251, and `encoding_rs` replaces unmappable characters
/// with '?', which matches legacy OziExplorer behaviour.
fn write_line(writer: &mut impl Write, line: &str) -> Result<(), ExportError> {
    let mut buf = String::with_capacity(line.len() + 2);
    buf.push_str(line);
    buf.push_str("\r\n");
    let (encoded, _, _) = WINDOWS_1251.encode(&buf);
    writer.write_all(&encoded)?;
    Ok(())
}

/// Strip characters that would break the comma-separated properties line.
///
/// Mirrors `infrastructure::export::wpt::sanitise_text`, without the length
/// cap — the PLT track description has no documented length limit.
fn sanitise_name(input: &str) -> String {
    input
        .chars()
        .map(|ch| match ch {
            ',' | '\r' | '\n' => ' ',
            other => other,
        })
        .collect()
}

fn rgb_to_colorref_bgr(rgb: u32) -> u32 {
    let r = (rgb >> 16) & 0xFF;
    let g = (rgb >> 8) & 0xFF;
    let b = rgb & 0xFF;
    (b << 16) | (g << 8) | r
}

fn map_line_width(width: f64) -> u32 {
    if !width.is_finite() {
        return 1;
    }
    width.round().clamp(1.0, 7.0) as u32
}

fn datetime_to_ole_date(datetime: chrono::DateTime<chrono::Utc>) -> f64 {
    let naive = datetime.naive_utc();
    let day_delta = naive.date().num_days_from_ce() - OLE_BASE_DATE.num_days_from_ce();
    let seconds_in_day = naive.time().num_seconds_from_midnight() as f64;
    day_delta as f64 + (seconds_in_day / 86_400.0)
}

#[cfg(test)]
mod tests {
    use super::export_plt;
    use crate::domain::{Track, TrackId, TrackPoint, TrackPointId, TrackSegment, TrackSegmentId};
    use crate::infrastructure::import::plt::{import_plt_file, import_plt_text};
    use chrono::TimeZone as _;
    use encoding_rs::WINDOWS_1251;

    /// PLT is the format a crew hands to whoever is still on OziExplorer, and
    /// the one this application is replacing. Every piece of it is tested
    /// somewhere; nothing tested the whole trip, which is what the receiver
    /// actually gets.
    ///
    /// The colour and the width are the caller's, not the track's: the command
    /// pulls them off the style and packs the colour into an RGB `u32`, so the
    /// round trip is only honest if it goes through the same packing.
    #[test]
    fn a_track_survives_the_plt_trip_out_and_back() {
        let mut track = Track::new(TrackId::new(1), "20260708_Ветер2");

        let mut first = TrackSegment::new(TrackSegmentId::new(1));
        first.add_point(
            TrackPoint::new(TrackPointId::new(1), 59.952430, 31.596810)
                .with_elevation(150.0)
                .with_timestamp(
                    chrono::Utc
                        .with_ymd_and_hms(2026, 7, 8, 12, 0, 0)
                        .single()
                        .expect("timestamp"),
                ),
        );
        first.add_point(TrackPoint::new(TrackPointId::new(2), 59.953350, 31.601640));
        track.add_segment(first);

        // A second sitting: if the boundary is lost the track claims a
        // straight line between them that nobody walked.
        let mut second = TrackSegment::new(TrackSegmentId::new(2));
        second.add_point(TrackPoint::new(TrackPointId::new(3), 59.944550, 31.659270));
        second.add_point(TrackPoint::new(TrackPointId::new(4), 59.946590, 31.671080));
        track.add_segment(second);

        track.style_mut().color = [0, 0, 255, 255];
        track.style_mut().line_width = 3.0;

        let style = track.style();
        let [r, g, b, _] = style.color;
        let packed = (r as u32) << 16 | (g as u32) << 8 | (b as u32);

        let mut written = Vec::new();
        export_plt(&track, packed, style.line_width as f64, &mut written).expect("export");
        let (text, _, _) = WINDOWS_1251.decode(&written);
        let back = import_plt_text("round-trip.plt".to_owned(), &text).expect("import");

        assert_eq!(back.track.name(), "20260708_Ветер2", "the name");
        assert_eq!(back.track.segments().len(), 2, "both sittings");
        assert_eq!(back.track.segments()[0].points().len(), 2);
        assert_eq!(back.track.segments()[1].points().len(), 2);

        let first_point = &back.track.segments()[0].points()[0];
        assert!(
            (first_point.latitude() - 59.952430).abs() < 1e-6,
            "latitude"
        );
        assert!(
            (first_point.longitude() - 31.596810).abs() < 1e-6,
            "longitude"
        );
        // Metres out, feet on the wire, metres back: the rounding to whole
        // feet is the format's, and a third of a metre is not a hill.
        assert!(
            (first_point.elevation().expect("elevation") - 150.0).abs() < 0.2,
            "elevation, got {:?}",
            first_point.elevation()
        );
        assert_eq!(
            first_point.timestamp(),
            Some(
                chrono::Utc
                    .with_ymd_and_hms(2026, 7, 8, 12, 0, 0)
                    .single()
                    .expect("timestamp")
            ),
            "the timestamp"
        );
        assert_eq!(
            back.track.segments()[0].points()[1].timestamp(),
            None,
            "and the absence of one"
        );

        assert_eq!(back.track.style().color, [0, 0, 255, 255], "the colour");
        assert_eq!(back.track.style().line_width, 3.0, "the line width");
    }

    #[test]
    fn export_plt_writes_exact_header_and_first_data_lines() {
        let mut track = Track::new(TrackId::new(1), "Direct");
        let mut segment = TrackSegment::new(TrackSegmentId::new(1));
        let ts = chrono::Utc.with_ymd_and_hms(1899, 12, 30, 0, 0, 0).unwrap();
        segment.add_point(TrackPoint::new(TrackPointId::new(1), 55.0, 37.0).with_timestamp(ts));
        segment.add_point(TrackPoint::new(TrackPointId::new(2), 55.1, 37.1).with_elevation(100.0));
        track.add_segment(segment);

        let mut bytes = Vec::new();
        export_plt(&track, 0x112233, 3.6, &mut bytes).expect("export");

        let expected = concat!(
            "OziExplorer Track Point File Version 2.1\r\n",
            "WGS 84\r\n",
            "Altitude is in Feet\r\n",
            "Field 1 = Lat, Field 2 = Lon, Field 3 = Code, Field 4 = Alt, Field 5 = Date, Field 6 = Stop, Field 7 = Bearing\r\n",
            "0,4,3351057,Direct,0,0,2,8421376,-1,0\r\n",
            "2\r\n",
            "55.000000,37.000000,1,-777,0.0000000,30-12-1899,00:00:00\r\n",
            "55.100000,37.100000,0,328,0.0000000,,\r\n"
        );

        assert_eq!(bytes, expected.as_bytes());
    }

    #[test]
    fn export_plt_round_trip_import_export_import_preserves_points() {
        let fixture = concat!(
            "OziExplorer Track Point File Version 2.1\n",
            "WGS 84\n",
            "Altitude is in Feet\n",
            "Reserved 3\n",
            "0,2,255,Roundtrip,,0,0,8421376,-1,0\n",
            "0\n",
            "60.000000,30.000000,0,-777,44407.572553669,30-07-2021,13:44:28\n",
            "60.100000,30.100000,1,-777,44407.572553680,30-07-2021,13:44:28\n",
            "60.200000,30.200000,0,-777,44407.572553690,30-07-2021,13:44:28\n"
        );

        let first = import_plt_text("fixture.plt".to_owned(), fixture).expect("first import");

        let mut bytes = Vec::new();
        let style_color = first.track.style().color;
        let rgb =
            (style_color[2] as u32) << 16 | (style_color[1] as u32) << 8 | style_color[0] as u32;
        export_plt(
            &first.track,
            rgb,
            first.track.style().line_width as f64,
            &mut bytes,
        )
        .expect("export");

        let text = String::from_utf8(bytes).expect("utf8");
        let second = import_plt_text("roundtrip.plt".to_owned(), &text).expect("second import");

        let first_points: Vec<_> = first
            .track
            .segments()
            .iter()
            .flat_map(|segment| segment.points().iter())
            .collect();
        let second_points: Vec<_> = second
            .track
            .segments()
            .iter()
            .flat_map(|segment| segment.points().iter())
            .collect();

        assert_eq!(first_points.len(), second_points.len());
        for (a, b) in first_points.iter().zip(second_points.iter()) {
            assert!((a.latitude() - b.latitude()).abs() < 1e-9);
            assert!((a.longitude() - b.longitude()).abs() < 1e-9);
            match (a.timestamp(), b.timestamp()) {
                (Some(ta), Some(tb)) => {
                    let diff = (ta - tb).num_seconds().abs();
                    assert!(diff <= 1, "timestamps differ by more than 1s: {ta} vs {tb}");
                }
                (None, None) => {}
                _ => panic!(
                    "timestamp mismatch: {:?} vs {:?}",
                    a.timestamp(),
                    b.timestamp()
                ),
            }
        }
    }

    #[test]
    fn export_plt_uses_zero_date_when_timestamp_missing() {
        let mut track = Track::new(TrackId::new(1), "NoTime");
        let mut segment = TrackSegment::new(TrackSegmentId::new(1));
        segment.add_point(TrackPoint::new(TrackPointId::new(1), 10.0, 20.0));
        track.add_segment(segment);

        let mut bytes = Vec::new();
        export_plt(&track, 0x0000FF, 2.0, &mut bytes).expect("export");
        let text = String::from_utf8(bytes).expect("utf8");

        assert!(text.contains("10.000000,20.000000,1,-777,0.0000000,,\r\n"));
    }

    #[test]
    fn export_plt_marks_first_point_of_each_segment() {
        let mut track = Track::new(TrackId::new(1), "Segments");

        let mut segment_a = TrackSegment::new(TrackSegmentId::new(1));
        segment_a.add_point(TrackPoint::new(TrackPointId::new(1), 1.0, 2.0));
        segment_a.add_point(TrackPoint::new(TrackPointId::new(2), 1.1, 2.1));
        track.add_segment(segment_a);

        let mut segment_b = TrackSegment::new(TrackSegmentId::new(2));
        segment_b.add_point(TrackPoint::new(TrackPointId::new(3), 3.0, 4.0));
        track.add_segment(segment_b);

        let mut bytes = Vec::new();
        export_plt(&track, 0x00FF00, 2.0, &mut bytes).expect("export");
        let text = String::from_utf8(bytes).expect("utf8");
        let lines: Vec<&str> = text.split("\r\n").filter(|line| !line.is_empty()).collect();

        assert_eq!(lines[6], "1.000000,2.000000,1,-777,0.0000000,,");
        assert_eq!(lines[7], "1.100000,2.100000,0,-777,0.0000000,,");
        assert_eq!(lines[8], "3.000000,4.000000,1,-777,0.0000000,,");
    }

    /// Self round-trip through the real file path: a Cyrillic track name plus
    /// distinct color/width must survive export → `import_plt_file`.
    #[test]
    fn export_plt_round_trip_preserves_cyrillic_name_color_width() {
        let mut track = Track::new(TrackId::new(1), "20240601_Иванов");
        let mut segment = TrackSegment::new(TrackSegmentId::new(1));
        segment.add_point(TrackPoint::new(TrackPointId::new(1), 55.0, 37.0));
        segment.add_point(TrackPoint::new(TrackPointId::new(2), 55.1, 37.1));
        track.add_segment(segment);

        let mut bytes = Vec::new();
        export_plt(&track, 0x11AA33, 3.0, &mut bytes).expect("export");

        let path = std::env::temp_dir().join(format!(
            "ozi-rs-plt-export-roundtrip-{}.plt",
            std::process::id()
        ));
        std::fs::write(&path, &bytes).expect("write tempfile");
        let import = import_plt_file(&path);
        // Clean up first so a panic still leaves a tidy temp dir.
        let _ = std::fs::remove_file(&path);
        let import = import.expect("import");

        assert_eq!(import.track.name(), "20240601_Иванов");
        assert_eq!(import.track.style().line_width, 3.0);
        // RGB 0x11AA33 → [R, G, B, A].
        assert_eq!(import.track.style().color, [0x11, 0xAA, 0x33, 255]);
        let point_count: usize = import
            .track
            .segments()
            .iter()
            .map(|segment| segment.points().len())
            .sum();
        assert_eq!(point_count, 2);
    }

    /// The properties line must follow the layout our importer
    /// (`import/plt.rs::parse_track_style`) and OziExplorer expect:
    /// `visible, line_width, colorref, name, skip, type, fill_style, fill_color, closed, reserved`.
    #[test]
    fn export_plt_properties_line_matches_importer_field_order() {
        let mut track = Track::new(TrackId::new(1), "Order");
        let mut segment = TrackSegment::new(TrackSegmentId::new(1));
        segment.add_point(TrackPoint::new(TrackPointId::new(1), 55.0, 37.0));
        track.add_segment(segment);

        let mut bytes = Vec::new();
        export_plt(&track, 0xFF0000, 2.0, &mut bytes).expect("export");

        let (decoded, _, _) = WINDOWS_1251.decode(&bytes);
        let line = decoded.split("\r\n").nth(4).expect("properties line");
        let fields: Vec<&str> = line.split(',').collect();

        assert_eq!(fields.len(), 10, "properties line: {line}");
        assert_eq!(fields[0], "0", "visible flag (0 = shown)");
        assert_eq!(fields[1], "2", "line width");
        assert_eq!(fields[2], "255", "COLORREF (BGR) of RGB 0xFF0000");
        assert_eq!(fields[3], "Order", "track name");
    }

    /// Commas and line breaks in the track name must not break the
    /// comma-separated properties line.
    #[test]
    fn export_plt_sanitises_name_commas_and_line_breaks() {
        let mut track = Track::new(TrackId::new(1), "Ива,нов\r\n2024");
        let mut segment = TrackSegment::new(TrackSegmentId::new(1));
        segment.add_point(TrackPoint::new(TrackPointId::new(1), 55.0, 37.0));
        track.add_segment(segment);

        let mut bytes = Vec::new();
        export_plt(&track, 0x0000FF, 1.0, &mut bytes).expect("export");

        let (decoded, _, _) = WINDOWS_1251.decode(&bytes);
        let line = decoded.split("\r\n").nth(4).expect("properties line");
        let fields: Vec<&str> = line.split(',').collect();

        assert_eq!(fields.len(), 10, "properties line: {line}");
        assert_eq!(fields[3], "Ива нов  2024");
    }

    /// The output must be Windows-1251, not UTF-8: OziExplorer on Russian
    /// Windows reads cp1251 and would show mojibake for UTF-8 Cyrillic.
    #[test]
    fn export_plt_encodes_cyrillic_name_as_cp1251_not_utf8() {
        let mut track = Track::new(TrackId::new(1), "Иванов");
        let mut segment = TrackSegment::new(TrackSegmentId::new(1));
        segment.add_point(TrackPoint::new(TrackPointId::new(1), 55.0, 37.0));
        track.add_segment(segment);

        let mut bytes = Vec::new();
        export_plt(&track, 0x0000FF, 1.0, &mut bytes).expect("export");

        let (needle, _, had_errors) = WINDOWS_1251.encode("Иванов");
        assert!(!had_errors, "name must be cp1251-representable");
        assert!(
            bytes.windows(needle.len()).any(|w| w == needle.as_ref()),
            "output must contain the cp1251 byte sequence for the Cyrillic name"
        );

        let utf8_needle = "Иванов".as_bytes();
        assert!(
            !bytes.windows(utf8_needle.len()).any(|w| w == utf8_needle),
            "output must not contain the UTF-8 byte sequence for the Cyrillic name"
        );

        // cp1251 Cyrillic bytes are not valid UTF-8 — proves cp1251 was used.
        assert!(
            std::str::from_utf8(&bytes).is_err(),
            "output with Cyrillic must not decode as valid UTF-8"
        );
    }
}
