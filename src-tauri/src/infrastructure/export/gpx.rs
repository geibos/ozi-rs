use crate::domain::{Track, TrackLayer};
use std::fmt::Write as FmtWrite;
use std::io;
use std::path::Path;

/// Export a single track layer to a `.gpx` file on disk.
pub fn export_layer_to_gpx_file(layer: &TrackLayer, path: &Path) -> Result<(), io::Error> {
    let xml = build_gpx_xml(layer.tracks());
    std::fs::write(path, xml)
}

/// Build GPX XML for the given tracks, including Garmin color extensions.
pub fn build_gpx_xml(tracks: &[Track]) -> String {
    let mut out = String::new();

    out.push_str("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
    out.push_str("<gpx version=\"1.1\" creator=\"ozi-rs\"\n");
    out.push_str("  xmlns=\"http://www.topografix.com/GPX/1/1\"\n");
    out.push_str("  xmlns:gpxx=\"http://www.garmin.com/xmlschemas/GpxExtensions/v3\">\n");

    for track in tracks {
        write_track(&mut out, track);
    }

    out.push_str("</gpx>\n");
    out
}

fn write_track(out: &mut String, track: &Track) {
    out.push_str("  <trk>\n");
    out.push_str("    <name>");
    xml_escape_into(out, track.name());
    out.push_str("</name>\n");

    let color_name = rgba_to_garmin_color(track.style().color);
    out.push_str("    <extensions>\n");
    out.push_str("      <gpxx:TrackExtension>\n");
    let _ = writeln!(
        out,
        "        <gpxx:DisplayColor>{color_name}</gpxx:DisplayColor>"
    );
    out.push_str("      </gpxx:TrackExtension>\n");
    out.push_str("    </extensions>\n");

    for segment in track.segments() {
        out.push_str("    <trkseg>\n");
        for point in segment.points() {
            let _ = writeln!(
                out,
                "      <trkpt lat=\"{:.6}\" lon=\"{:.6}\">",
                point.latitude(),
                point.longitude()
            );
            if let Some(elev) = point.elevation() {
                let _ = writeln!(out, "        <ele>{:.1}</ele>", elev);
            }
            if let Some(ts) = point.timestamp() {
                let _ = writeln!(out, "        <time>{}</time>", ts.format("%+"));
            }
            out.push_str("      </trkpt>\n");
        }
        out.push_str("    </trkseg>\n");
    }

    out.push_str("  </trk>\n");
}

fn xml_escape_into(out: &mut String, s: &str) {
    for ch in s.chars() {
        match ch {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&apos;"),
            c => out.push(c),
        }
    }
}

/// Map an RGBA color to the nearest Garmin GPX display color name.
pub fn rgba_to_garmin_color(rgba: [u8; 4]) -> &'static str {
    const GARMIN_COLORS: &[(&str, u8, u8, u8)] = &[
        ("Black", 0, 0, 0),
        ("DarkRed", 128, 0, 0),
        ("DarkGreen", 0, 128, 0),
        ("DarkBlue", 0, 0, 128),
        ("DarkGray", 64, 64, 64),
        ("Gray", 128, 128, 128),
        ("LightGray", 192, 192, 192),
        ("White", 255, 255, 255),
        ("Red", 255, 0, 0),
        ("Green", 0, 255, 0),
        ("Blue", 0, 0, 255),
        ("Yellow", 255, 255, 0),
        ("Cyan", 0, 255, 255),
        ("Magenta", 255, 0, 255),
        ("Orange", 255, 165, 0),
        ("LightBlue", 135, 206, 235),
        ("Violet", 238, 130, 238),
        ("Purple", 128, 0, 128),
    ];

    let [r, g, b, _] = rgba;
    let r = r as i32;
    let g = g as i32;
    let b = b as i32;

    GARMIN_COLORS
        .iter()
        .min_by_key(|(_, cr, cg, cb)| {
            let dr = r - *cr as i32;
            let dg = g - *cg as i32;
            let db = b - *cb as i32;
            dr * dr + dg * dg + db * db
        })
        .map(|(name, _, _, _)| *name)
        .unwrap_or("Black")
}

#[cfg(test)]
mod tests {
    use super::{build_gpx_xml, rgba_to_garmin_color};
    use crate::domain::{Track, TrackId, TrackPoint, TrackPointId, TrackSegment, TrackSegmentId};

    #[test]
    fn rgba_to_garmin_color_maps_pure_red() {
        assert_eq!(rgba_to_garmin_color([255, 0, 0, 255]), "Red");
    }

    #[test]
    fn rgba_to_garmin_color_maps_pure_blue() {
        assert_eq!(rgba_to_garmin_color([0, 0, 255, 255]), "Blue");
    }

    #[test]
    fn rgba_to_garmin_color_maps_black() {
        assert_eq!(rgba_to_garmin_color([0, 0, 0, 255]), "Black");
    }

    #[test]
    fn build_gpx_xml_produces_valid_header_and_footer() {
        let xml = build_gpx_xml(&[]);

        assert!(xml.starts_with("<?xml version=\"1.0\""));
        assert!(xml.contains("<gpx version=\"1.1\""));
        assert!(xml.contains("xmlns:gpxx="));
        assert!(xml.ends_with("</gpx>\n"));
    }

    #[test]
    fn build_gpx_xml_includes_track_name_and_garmin_color() {
        let mut track = Track::new(TrackId::new(1), "Morning route");
        track.style_mut().color = [255, 0, 0, 255]; // red
        let mut seg = TrackSegment::new(TrackSegmentId::new(1));
        seg.add_point(TrackPoint::new(TrackPointId::new(1), 60.0, 30.0));
        track.add_segment(seg);

        let xml = build_gpx_xml(&[track]);

        assert!(xml.contains("<name>Morning route</name>"));
        assert!(xml.contains("<gpxx:DisplayColor>Red</gpxx:DisplayColor>"));
        assert!(xml.contains("lat=\"60.000000\" lon=\"30.000000\""));
    }

    #[test]
    fn build_gpx_xml_writes_elevation_and_timestamp() {
        use chrono::TimeZone as _;

        let ts = chrono::Utc.with_ymd_and_hms(2024, 6, 1, 10, 0, 0).unwrap();
        let mut track = Track::new(TrackId::new(1), "T");
        let mut seg = TrackSegment::new(TrackSegmentId::new(1));
        seg.add_point(
            TrackPoint::new(TrackPointId::new(1), 55.0, 37.0)
                .with_elevation(150.0)
                .with_timestamp(ts),
        );
        track.add_segment(seg);

        let xml = build_gpx_xml(&[track]);

        assert!(xml.contains("<ele>150.0</ele>"));
        assert!(xml.contains("<time>2024-06-01T10:00:00+00:00</time>"));
    }

    #[test]
    fn build_gpx_xml_escapes_special_characters_in_name() {
        let track = Track::new(TrackId::new(1), "Route & <test>");

        let xml = build_gpx_xml(&[track]);

        assert!(xml.contains("Route &amp; &lt;test&gt;"));
    }

    #[test]
    fn track_total_distance_km_computes_haversine_sum() {
        let mut track = Track::new(TrackId::new(1), "T");
        let mut seg = TrackSegment::new(TrackSegmentId::new(1));
        // ~111 km per degree latitude
        seg.add_point(TrackPoint::new(TrackPointId::new(1), 0.0, 0.0));
        seg.add_point(TrackPoint::new(TrackPointId::new(2), 1.0, 0.0));
        track.add_segment(seg);

        let dist = track.total_distance_km();
        assert!((dist - 111.195).abs() < 0.1, "distance={dist:.3} km");
    }

    #[test]
    fn track_duration_is_none_for_track_without_timestamps() {
        let mut track = Track::new(TrackId::new(1), "T");
        let mut seg = TrackSegment::new(TrackSegmentId::new(1));
        seg.add_point(TrackPoint::new(TrackPointId::new(1), 0.0, 0.0));
        track.add_segment(seg);

        assert!(track.total_duration().is_none());
    }
}
