use ozi_rs::infrastructure::import::{
    ArchiveEntryKind, SupportedArchiveEntryKind, UnsupportedArchiveEntryKind, inventory_zip_entries,
};
use std::io::{Cursor, Write};
use zip::write::SimpleFileOptions;
use zip::{CompressionMethod, ZipWriter};

#[test]
fn inventory_zip_entries_enumerates_files_and_skips_directories() {
    let archive = build_archive(&[
        ("field/", b"".as_slice(), true),
        ("field/track.GPX", b"<gpx></gpx>".as_slice(), false),
        ("field/waypoints.wpt", b"header\npoint".as_slice(), false),
        (
            "maps/calibration.map",
            b"OziExplorer Map Data File".as_slice(),
            false,
        ),
    ]);

    let entries = inventory_zip_entries(Cursor::new(archive)).expect("inventory");

    assert_eq!(entries.len(), 3);
    assert_eq!(entries[0].path(), "field/track.GPX");
    assert_eq!(entries[0].file_name(), "track.GPX");
    assert_eq!(
        entries[0].kind(),
        &ArchiveEntryKind::Supported(SupportedArchiveEntryKind::Gpx)
    );
    assert_eq!(
        entries[1].kind(),
        &ArchiveEntryKind::Supported(SupportedArchiveEntryKind::OziWaypoint)
    );
    assert_eq!(
        entries[2].kind(),
        &ArchiveEntryKind::Supported(SupportedArchiveEntryKind::OziMap)
    );
}

#[test]
fn inventory_zip_entries_marks_unsupported_payloads_deterministically() {
    let archive = build_archive(&[
        ("maps/base.ozf2", b"binary".as_slice(), false),
        ("mobile/offline.sqlitedb", b"sqlite".as_slice(), false),
        ("notes/readme.txt", b"note".as_slice(), false),
    ]);

    let entries = inventory_zip_entries(Cursor::new(archive)).expect("inventory");

    assert_eq!(entries.len(), 3);
    assert_eq!(
        entries[0].kind(),
        &ArchiveEntryKind::Unsupported(UnsupportedArchiveEntryKind::RasterPayload)
    );
    assert_eq!(
        entries[1].kind(),
        &ArchiveEntryKind::Unsupported(UnsupportedArchiveEntryKind::SqliteTiles)
    );
    assert_eq!(
        entries[2].kind(),
        &ArchiveEntryKind::Unsupported(UnsupportedArchiveEntryKind::Unknown)
    );
}

fn build_archive(entries: &[(&str, &[u8], bool)]) -> Vec<u8> {
    let mut buffer = Cursor::new(Vec::new());
    let mut writer = ZipWriter::new(&mut buffer);
    let options = SimpleFileOptions::default().compression_method(CompressionMethod::Stored);

    for (path, contents, is_directory) in entries {
        if *is_directory {
            writer.add_directory(*path, options).expect("directory");
            continue;
        }

        writer.start_file(*path, options).expect("file");
        writer.write_all(contents).expect("contents");
    }

    writer.finish().expect("finish");
    buffer.into_inner()
}
