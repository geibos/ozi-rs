use ozi_rs::infrastructure::import::{ArchivedGpxImportError, import_gpx_entries_from_archive};
use std::io::{Cursor, Write};
use zip::write::SimpleFileOptions;
use zip::{CompressionMethod, ZipWriter};

#[test]
fn import_gpx_entries_from_archive_reads_tracks_and_waypoints() {
    let archive = build_archive(&[("field/mission.gpx", SAMPLE_GPX.as_bytes())]);

    let imports = import_gpx_entries_from_archive(Cursor::new(archive)).expect("imports");

    assert_eq!(imports.len(), 1);
    assert_eq!(imports[0].source_path(), "field/mission.gpx");
    assert_eq!(imports[0].tracks().len(), 1);
    assert_eq!(imports[0].tracks()[0].name(), "Evening route");
    assert_eq!(imports[0].tracks()[0].segments().len(), 1);
    assert_eq!(imports[0].tracks()[0].segments()[0].points().len(), 2);
    assert_eq!(
        imports[0].tracks()[0].segments()[0].points()[0].latitude(),
        54.1
    );
    assert_eq!(
        imports[0].tracks()[0].segments()[0].points()[0].longitude(),
        27.2
    );
    assert_eq!(imports[0].waypoints().len(), 1);
    assert_eq!(imports[0].waypoints()[0].name(), "Camp");
    assert_eq!(imports[0].waypoints()[0].latitude(), 54.3);
    assert_eq!(imports[0].waypoints()[0].longitude(), 27.4);
}

#[test]
fn import_gpx_entries_from_archive_skips_non_gpx_entries_in_mixed_archives() {
    let archive = build_archive(&[
        ("notes/readme.txt", b"ignore me"),
        ("maps/base.ozf2", b"binary"),
        ("field/mission.gpx", SAMPLE_GPX.as_bytes()),
    ]);

    let imports = import_gpx_entries_from_archive(Cursor::new(archive)).expect("imports");

    assert_eq!(imports.len(), 1);
    assert_eq!(imports[0].source_path(), "field/mission.gpx");
}

#[test]
fn import_gpx_entries_from_archive_returns_explicit_error_for_malformed_gpx() {
    let archive = build_archive(&[("field/bad.gpx", b"<gpx><trk></gpx>")]);

    let error = import_gpx_entries_from_archive(Cursor::new(archive)).expect_err("error");

    match error {
        ArchivedGpxImportError::ParseGpx { path, .. } => assert_eq!(path, "field/bad.gpx"),
        other => panic!("unexpected error: {other:?}"),
    }
}

fn build_archive(entries: &[(&str, &[u8])]) -> Vec<u8> {
    let mut buffer = Cursor::new(Vec::new());
    let mut writer = ZipWriter::new(&mut buffer);
    let options = SimpleFileOptions::default().compression_method(CompressionMethod::Stored);

    for (path, contents) in entries {
        writer.start_file(*path, options).expect("file");
        writer.write_all(contents).expect("contents");
    }

    writer.finish().expect("finish");
    buffer.into_inner()
}

const SAMPLE_GPX: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<gpx version="1.1" creator="ozi-rs-test" xmlns="http://www.topografix.com/GPX/1/1">
  <wpt lat="54.3" lon="27.4">
    <name>Camp</name>
  </wpt>
  <trk>
    <name>Evening route</name>
    <trkseg>
      <trkpt lat="54.1" lon="27.2"></trkpt>
      <trkpt lat="54.2" lon="27.3"></trkpt>
    </trkseg>
  </trk>
</gpx>
"#;
