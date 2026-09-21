use crate::domain::{Track, TrackLayer, Waypoint};
use std::fmt::Write as FmtWrite;
use std::io;
use std::path::Path;

/// Export a single track layer to a `.gpx` file on disk.
pub fn export_layer_to_gpx_file(layer: &TrackLayer, path: &Path) -> Result<(), io::Error> {
    let xml = build_gpx_xml(layer.tracks());
    std::fs::write(path, xml)
}

/// Export a day's work — the tracks and the marks made on them — to one file.
///
/// GPX holds `<trk>` and `<wpt>` in the same document, and a crew that found
/// something put a waypoint there. A handover of tracks alone leaves out the
/// one thing the штаб most wants to see.
pub fn export_day_to_gpx_file(
    tracks: &[Track],
    waypoints: &[Waypoint],
    path: &Path,
) -> Result<(), io::Error> {
    std::fs::write(path, build_day_gpx_xml(tracks, waypoints))
}

/// GPX XML for a day: the waypoints first, as every writer of the format puts
/// them, then the tracks.
pub fn build_day_gpx_xml(tracks: &[Track], waypoints: &[Waypoint]) -> String {
    let mut out = String::new();

    out.push_str("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
    out.push_str("<gpx version=\"1.1\" creator=\"ozi-rs\"\n");
    out.push_str("  xmlns=\"http://www.topografix.com/GPX/1/1\"\n");
    out.push_str("  xmlns:gpxx=\"http://www.garmin.com/xmlschemas/GpxExtensions/v3\">\n");

    for waypoint in waypoints {
        write_waypoint(&mut out, waypoint);
    }
    for track in tracks {
        write_track(&mut out, track);
    }

    out.push_str("</gpx>\n");
    out
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

/// Export a waypoint layer to a `.gpx` file on disk.
///
/// GPX is what a phone, a navigator and the other groups' software all read;
/// WPT is OziExplorer's own format. A crew that marks a найденный объект has
/// to be able to hand it over in the format the receiver has.
pub fn export_waypoints_to_gpx_file(waypoints: &[Waypoint], path: &Path) -> Result<(), io::Error> {
    std::fs::write(path, build_waypoint_gpx_xml(waypoints))
}

pub fn build_waypoint_gpx_xml(waypoints: &[Waypoint]) -> String {
    let mut out = String::new();

    out.push_str("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
    out.push_str("<gpx version=\"1.1\" creator=\"ozi-rs\"\n");
    out.push_str("  xmlns=\"http://www.topografix.com/GPX/1/1\">\n");

    for waypoint in waypoints {
        write_waypoint(&mut out, waypoint);
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

fn write_waypoint(out: &mut String, waypoint: &Waypoint) {
    let _ = writeln!(
        out,
        "  <wpt lat=\"{:.6}\" lon=\"{:.6}\">",
        waypoint.latitude(),
        waypoint.longitude()
    );
    out.push_str("    <name>");
    xml_escape_into(out, waypoint.name());
    out.push_str("</name>\n");
    if let Some(symbol) = waypoint.symbol() {
        out.push_str("    <sym>");
        xml_escape_into(out, symbol);
        out.push_str("</sym>\n");
    }
    out.push_str("  </wpt>\n");
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

/// The colour names GPX allows, with the RGB each one stands for.
///
/// One table for both directions: writing picks the nearest name, reading
/// turns a name back into bytes. Two tables would drift, and the drift would
/// show up as a track that changes colour on a round trip.
pub const GARMIN_COLORS: &[(&str, u8, u8, u8)] = &[
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

/// Turn a GPX colour name back into RGBA.
///
/// `None` for a name this table does not hold — another program's extension,
/// or a typo. That is not a reason to fail an import, and not a reason to
/// guess: the track keeps its default colour.
pub fn garmin_color_to_rgba(name: &str) -> Option<[u8; 4]> {
    GARMIN_COLORS
        .iter()
        .find(|(candidate, _, _, _)| candidate.eq_ignore_ascii_case(name))
        .map(|(_, r, g, b)| [*r, *g, *b, 255])
}

/// Map an RGBA color to the nearest Garmin GPX display color name.
pub fn rgba_to_garmin_color(rgba: [u8; 4]) -> &'static str {
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
    use super::{build_gpx_xml, build_waypoint_gpx_xml, rgba_to_garmin_color};
    use crate::domain::{
        Track, TrackId, TrackPoint, TrackPointId, TrackSegment, TrackSegmentId, Waypoint,
        WaypointId,
    };

    /// What survives a trip out of the app and back in.
    ///
    /// FTP upload of tracks is planned, and what goes up is this file. Until
    /// now both halves were tested apart — the writer's XML, the reader's
    /// parse — and nothing checked that a track handed to one comes back from
    /// the other. A field record is a record: names, the break between two
    /// sittings, the coordinates, and the times that make a track a timeline
    /// rather than a shape.
    #[test]
    fn a_track_survives_the_trip_out_and_back() {
        use crate::infrastructure::import::gpx::import_gpx_file;
        use chrono::{TimeZone, Utc};

        let mut track = Track::new(TrackId::new(1), "20260708_Ветер2");
        let mut first = TrackSegment::new(TrackSegmentId::new(1));
        for (i, (lat, lon)) in [(59.95243, 31.59681), (59.95335, 31.60164)]
            .into_iter()
            .enumerate()
        {
            first.add_point(
                TrackPoint::new(TrackPointId::new(i as u64 + 1), lat, lon)
                    .with_timestamp(Utc.with_ymd_and_hms(2026, 7, 8, 9, i as u32, 0).unwrap()),
            );
        }
        // A second sitting: the gap between them is part of the record.
        let mut second = TrackSegment::new(TrackSegmentId::new(2));
        for (i, (lat, lon)) in [(59.94455, 31.65927), (59.94659, 31.67108)]
            .into_iter()
            .enumerate()
        {
            second.add_point(TrackPoint::new(TrackPointId::new(i as u64 + 10), lat, lon));
        }
        track.add_segment(first);
        track.add_segment(second);

        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("out.gpx");
        super::export_day_to_gpx_file(std::slice::from_ref(&track), &[], &path).expect("export");

        let back = import_gpx_file(&path).expect("import what we just wrote");
        let tracks = back.tracks();
        assert_eq!(tracks.len(), 1, "one track out, one track back");
        let returned = &tracks[0];

        assert_eq!(
            returned.name(),
            "20260708_Ветер2",
            "the name, Cyrillic and all"
        );
        assert_eq!(
            returned.segments().len(),
            2,
            "the break between two sittings is not a detail: joined, the track \
             claims a straight line across the gap"
        );

        let out: Vec<(f64, f64)> = track
            .segments()
            .iter()
            .flat_map(|s| s.points().iter().map(|p| (p.latitude(), p.longitude())))
            .collect();
        let home: Vec<(f64, f64)> = returned
            .segments()
            .iter()
            .flat_map(|s| s.points().iter().map(|p| (p.latitude(), p.longitude())))
            .collect();
        assert_eq!(home, out, "every point, in order, to the same precision");

        let times: Vec<_> = returned.segments()[0]
            .points()
            .iter()
            .map(|p| p.timestamp())
            .collect();
        assert_eq!(
            times,
            vec![
                Some(Utc.with_ymd_and_hms(2026, 7, 8, 9, 0, 0).unwrap()),
                Some(Utc.with_ymd_and_hms(2026, 7, 8, 9, 1, 0).unwrap()),
            ],
            "the times that make a track a timeline"
        );
        assert!(
            returned.segments()[1]
                .points()
                .iter()
                .all(|p| p.timestamp().is_none()),
            "and points that never had one still do not"
        );
    }

    /// The same question for waypoints, which is the half of a search record
    /// that names things: the task point, what was found, where the danger is.
    ///
    /// A waypoint carries a name and a symbol, and both are what the other
    /// groups read when the file reaches them. The symbol is the one a reader
    /// is most likely to drop, since it is optional.
    #[test]
    fn a_waypoint_survives_the_trip_out_and_back() {
        use crate::infrastructure::import::gpx::import_gpx_file;

        let mut headquarters = Waypoint::new(WaypointId::new(1), "ШТАБ", 59.95243, 31.59681);
        // `set_symbol` hands back the previous symbol, not a result.
        let _ = headquarters.set_symbol(Some("flag".to_owned()));
        // One without a symbol: absent must stay absent rather than become a
        // default, which would put a mark on the map nobody placed.
        let plain = Waypoint::new(WaypointId::new(2), "Задача 1", 59.8, 31.7);

        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("waypoints.gpx");
        super::export_waypoints_to_gpx_file(&[headquarters, plain], &path).expect("export");

        let back = import_gpx_file(&path).expect("import what we just wrote");
        let waypoints = back.waypoints();
        assert_eq!(waypoints.len(), 2, "both came back");

        assert_eq!(waypoints[0].name(), "ШТАБ");
        assert_eq!(waypoints[0].symbol(), Some("flag"));
        assert!(
            (waypoints[0].latitude() - 59.95243).abs() < 1e-9
                && (waypoints[0].longitude() - 31.59681).abs() < 1e-9,
            "to the precision a search area needs"
        );

        assert_eq!(waypoints[1].name(), "Задача 1");
        assert_eq!(
            waypoints[1].symbol(),
            None,
            "a waypoint with no symbol must not acquire one on the way back"
        );
    }

    /// The colour the writer emits is read back — pinned on the reading side,
    /// in `infrastructure::import::gpx`, where the second pass that recovers
    /// it lives. This used to be a test asserting the colour was *lost*: the
    /// gap was real, so it was recorded rather than left to be rediscovered,
    /// and fixing it is what made that test fail.

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
    fn build_waypoint_gpx_xml_writes_symbol_when_present() {
        let mut waypoint = Waypoint::new(WaypointId::new(1), "Camp", 55.0, 37.0);
        waypoint.set_symbol(Some("Flag".to_owned()));

        let xml = build_waypoint_gpx_xml(&[waypoint]);

        assert!(xml.contains("<name>Camp</name>"));
        assert!(xml.contains("<sym>Flag</sym>"));
    }

    #[test]
    fn build_waypoint_gpx_xml_skips_symbol_when_absent() {
        let waypoint = Waypoint::new(WaypointId::new(1), "Camp", 55.0, 37.0);

        let xml = build_waypoint_gpx_xml(&[waypoint]);

        assert!(!xml.contains("<sym>"));
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
