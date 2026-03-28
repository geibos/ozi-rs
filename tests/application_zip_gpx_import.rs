use ozi_rs::application::{AppState, ArchiveImportError, import_gpx_archive_into_project};
use ozi_rs::domain::{LayerId, Project};
use ozi_rs::{application::CommandStack, application::ProjectCommand};
use std::io::{Cursor, Write};
use zip::write::SimpleFileOptions;
use zip::{CompressionMethod, ZipWriter};

#[test]
fn application_import_creates_independent_track_and_waypoint_layers() {
    let archive = build_archive(&[("field/mission.gpx", SAMPLE_GPX.as_bytes())]);
    let mut project = Project::default();
    let mut history = CommandStack::default();

    let report = import_gpx_archive_into_project(&mut project, &mut history, Cursor::new(archive))
        .expect("import report");

    assert_eq!(report.imported_entries(), 1);
    assert_eq!(report.imported_track_layers(), 1);
    assert_eq!(report.imported_waypoint_layers(), 1);
    assert_eq!(report.imported_tracks(), 1);
    assert_eq!(report.imported_waypoints(), 1);
    assert_eq!(project.track_layers().len(), 1);
    assert_eq!(
        project.track_layers()[0].name(),
        "Imported tracks: field/mission.gpx"
    );
    assert_eq!(
        project.track_layers()[0].tracks()[0].name(),
        "Evening route"
    );
    assert_eq!(project.waypoint_layers().len(), 1);
    assert_eq!(
        project.waypoint_layers()[0].name(),
        "Imported waypoints: field/mission.gpx"
    );
    assert_eq!(project.waypoint_layers()[0].waypoints()[0].name(), "Camp");
}

#[test]
fn application_import_surfaces_explicit_errors_for_malformed_gpx_archives() {
    let archive = build_archive(&[("field/bad.gpx", b"<gpx><trk></gpx>")]);
    let mut project = Project::default();
    let mut history = CommandStack::default();

    let error = import_gpx_archive_into_project(&mut project, &mut history, Cursor::new(archive))
        .expect_err("import error");

    match error {
        ArchiveImportError::GpxImport(_) => {}
        other => panic!("unexpected error: {other:?}"),
    }
}

#[test]
fn application_import_does_not_regress_existing_map_layer_registration() {
    let archive = build_archive(&[("field/mission.gpx", SAMPLE_GPX.as_bytes())]);
    let mut state = AppState::default();

    let inserted = state
        .import_gpx_archive(Cursor::new(archive))
        .expect("import report");

    assert_eq!(inserted.imported_entries(), 1);
    assert_eq!(state.map_layer_count(), 0);
    assert_eq!(state.track_layer_count(), 1);
    assert_eq!(state.waypoint_layer_count(), 1);
}

#[test]
fn application_import_keeps_preexisting_map_layers_intact() {
    let archive = build_archive(&[("field/mission.gpx", SAMPLE_GPX.as_bytes())]);
    let mut project = Project::default();
    let mut history = CommandStack::default();

    history
        .apply(
            &mut project,
            &ProjectCommand::add_map_layer_with_source(
                LayerId::new(5),
                "Base map",
                ".tmp/maps/base.sqlitedb",
            ),
        )
        .expect("map layer");

    import_gpx_archive_into_project(&mut project, &mut history, Cursor::new(archive))
        .expect("import report");

    assert_eq!(project.map_layers().len(), 1);
    assert_eq!(project.map_layers()[0].name(), "Base map");
    assert_eq!(project.track_layers().len(), 1);
    assert_eq!(project.waypoint_layers().len(), 1);
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
