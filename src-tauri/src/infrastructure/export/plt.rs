use crate::domain::Track;
use chrono::{Datelike, NaiveDate, Timelike};
use std::io::Write;

const OLE_BASE_DATE: NaiveDate =
    NaiveDate::from_ymd_opt(1899, 12, 30).expect("valid OLE base date");

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
    let point_count: usize = track
        .segments()
        .iter()
        .map(|segment| segment.points().len())
        .sum();

    write!(writer, "OziExplorer Track Point File Version 2.1\r\n")?;
    write!(writer, "WGS 84\r\n")?;
    write!(writer, "Altitude is in Feet\r\n")?;
    write!(writer, "Field 1 = Lat, Field 2 = Lon, Field 3 = Code, Field 4 = Alt, Field 5 = Date, Field 6 = Stop, Field 7 = Bearing\r\n")?;
    write!(
        writer,
        "{},0,{colorref},{width_int},0,0,0\r\n",
        track.name()
    )?;
    write!(writer, "{point_count}\r\n")?;

    for segment in track.segments() {
        for (index, point) in segment.points().iter().enumerate() {
            let is_segment_start = index == 0;
            let segment_flag = if is_segment_start { 1 } else { 0 };
            let altitude_ft = point
                .elevation()
                .map_or(-777_i32, |meters| (meters * 3.28084).round() as i32);
            let ole_date = point.timestamp().map_or(0.0, datetime_to_ole_date);
            let time_field = match point.timestamp() {
                Some(ts) if is_segment_start => ts.format("%Y%m%d_%H%M%S").to_string(),
                Some(ts) => ts.format("%H%M%S").to_string(),
                None => "000000".to_owned(),
            };

            write!(
                writer,
                "{:.6},{:.6},0,{altitude_ft},{ole_date:.7},{time_field},{segment_flag}\r\n",
                point.latitude(),
                point.longitude(),
            )?;
        }
    }

    Ok(())
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
    use crate::infrastructure::import::plt::import_plt_text;
    use chrono::TimeZone as _;

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
            "Direct,0,3351057,4,0,0,0\r\n",
            "2\r\n",
            "55.000000,37.000000,0,-777,0.0000000,18991230_000000,1\r\n",
            "55.100000,37.100000,0,328,0.0000000,000000,0\r\n"
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
            assert_eq!(a.timestamp(), b.timestamp());
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

        assert!(text.contains("10.000000,20.000000,0,-777,0.0000000,000000,1\r\n"));
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

        assert_eq!(lines[6], "1.000000,2.000000,0,-777,0.0000000,000000,1");
        assert_eq!(lines[7], "1.100000,2.100000,0,-777,0.0000000,000000,0");
        assert_eq!(lines[8], "3.000000,4.000000,0,-777,0.0000000,000000,1");
    }
}
