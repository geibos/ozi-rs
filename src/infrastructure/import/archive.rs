use std::fmt;
use std::io::{Read, Seek};
use zip::ZipArchive;
use zip::result::ZipError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArchiveEntry {
    path: String,
    file_name: String,
    kind: ArchiveEntryKind,
    compressed_size: u64,
    uncompressed_size: u64,
}

impl ArchiveEntry {
    fn new(
        path: String,
        file_name: String,
        kind: ArchiveEntryKind,
        compressed_size: u64,
        uncompressed_size: u64,
    ) -> Self {
        Self {
            path,
            file_name,
            kind,
            compressed_size,
            uncompressed_size,
        }
    }

    pub fn path(&self) -> &str {
        &self.path
    }

    pub fn file_name(&self) -> &str {
        &self.file_name
    }

    pub fn kind(&self) -> &ArchiveEntryKind {
        &self.kind
    }

    pub const fn compressed_size(&self) -> u64 {
        self.compressed_size
    }

    pub const fn uncompressed_size(&self) -> u64 {
        self.uncompressed_size
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ArchiveEntryKind {
    Supported(SupportedArchiveEntryKind),
    Unsupported(UnsupportedArchiveEntryKind),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SupportedArchiveEntryKind {
    Gpx,
    Kml,
    OziMap,
    OziTrack,
    OziWaypoint,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnsupportedArchiveEntryKind {
    RasterPayload,
    SqliteTiles,
    Unknown,
}

#[derive(Debug)]
pub enum ArchiveInventoryError {
    OpenArchive(ZipError),
    ReadEntry(ZipError),
}

impl fmt::Display for ArchiveInventoryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::OpenArchive(error) => write!(f, "failed to open ZIP archive: {error}"),
            Self::ReadEntry(error) => write!(f, "failed to read ZIP archive entry: {error}"),
        }
    }
}

impl std::error::Error for ArchiveInventoryError {}

pub fn inventory_zip_entries<R>(reader: R) -> Result<Vec<ArchiveEntry>, ArchiveInventoryError>
where
    R: Read + Seek,
{
    let mut archive = ZipArchive::new(reader).map_err(ArchiveInventoryError::OpenArchive)?;
    let mut entries = Vec::new();

    for index in 0..archive.len() {
        let entry = archive
            .by_index(index)
            .map_err(ArchiveInventoryError::ReadEntry)?;

        if entry.is_dir() {
            continue;
        }

        let path = entry.name().to_owned();
        let file_name = archive_file_name(&path);
        let kind = classify_archive_entry(&path);

        entries.push(ArchiveEntry::new(
            path,
            file_name,
            kind,
            entry.compressed_size(),
            entry.size(),
        ));
    }

    Ok(entries)
}

fn archive_file_name(path: &str) -> String {
    path.rsplit('/').next().unwrap_or(path).to_owned()
}

fn classify_archive_entry(path: &str) -> ArchiveEntryKind {
    match extension(path).as_deref() {
        Some("gpx") => ArchiveEntryKind::Supported(SupportedArchiveEntryKind::Gpx),
        Some("kml") => ArchiveEntryKind::Supported(SupportedArchiveEntryKind::Kml),
        Some("map") => ArchiveEntryKind::Supported(SupportedArchiveEntryKind::OziMap),
        Some("plt") => ArchiveEntryKind::Supported(SupportedArchiveEntryKind::OziTrack),
        Some("wpt") => ArchiveEntryKind::Supported(SupportedArchiveEntryKind::OziWaypoint),
        Some("ozf2") | Some("ozfx3") => {
            ArchiveEntryKind::Unsupported(UnsupportedArchiveEntryKind::RasterPayload)
        }
        Some("sqlitedb") => ArchiveEntryKind::Unsupported(UnsupportedArchiveEntryKind::SqliteTiles),
        _ => ArchiveEntryKind::Unsupported(UnsupportedArchiveEntryKind::Unknown),
    }
}

fn extension(path: &str) -> Option<String> {
    let file_name = path.rsplit('/').next()?;
    let (_, extension) = file_name.rsplit_once('.')?;

    Some(extension.to_ascii_lowercase())
}

#[cfg(test)]
mod tests {
    use super::{
        ArchiveEntryKind, SupportedArchiveEntryKind, UnsupportedArchiveEntryKind,
        archive_file_name, classify_archive_entry,
    };

    #[test]
    fn archive_file_name_returns_last_path_segment() {
        assert_eq!(archive_file_name("nested/track.gpx"), "track.gpx");
    }

    #[test]
    fn classify_archive_entry_matches_supported_extensions_case_insensitively() {
        assert_eq!(
            classify_archive_entry("nested/FIELD.GPX"),
            ArchiveEntryKind::Supported(SupportedArchiveEntryKind::Gpx)
        );
        assert_eq!(
            classify_archive_entry("maps/calibration.MAP"),
            ArchiveEntryKind::Supported(SupportedArchiveEntryKind::OziMap)
        );
    }

    #[test]
    fn classify_archive_entry_marks_raster_and_unknown_payloads_as_unsupported() {
        assert_eq!(
            classify_archive_entry("maps/base.ozf2"),
            ArchiveEntryKind::Unsupported(UnsupportedArchiveEntryKind::RasterPayload)
        );
        assert_eq!(
            classify_archive_entry("mobile/cache.sqlitedb"),
            ArchiveEntryKind::Unsupported(UnsupportedArchiveEntryKind::SqliteTiles)
        );
        assert_eq!(
            classify_archive_entry("notes/readme.txt"),
            ArchiveEntryKind::Unsupported(UnsupportedArchiveEntryKind::Unknown)
        );
    }
}
