use std::fmt;
use std::io::{Read, Seek};
use std::path::{Path, PathBuf};
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

#[derive(Debug)]
pub enum ArchiveExtractError {
    OpenArchive(ZipError),
    ReadEntry(ZipError),
    InvalidEntryPath(String),
    CreateDirectory(std::io::Error),
    CreateFile(std::io::Error),
    CopyEntry(std::io::Error),
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

impl fmt::Display for ArchiveExtractError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::OpenArchive(error) => {
                write!(f, "failed to open ZIP archive for extraction: {error}")
            }
            Self::ReadEntry(error) => {
                write!(f, "failed to read ZIP entry during extraction: {error}")
            }
            Self::InvalidEntryPath(path) => {
                write!(f, "ZIP entry path is unsafe to extract: {path}")
            }
            Self::CreateDirectory(error) => {
                write!(f, "failed to create extraction directory: {error}")
            }
            Self::CreateFile(error) => write!(f, "failed to create extracted file: {error}"),
            Self::CopyEntry(error) => write!(f, "failed to copy extracted ZIP entry: {error}"),
        }
    }
}

impl std::error::Error for ArchiveExtractError {}

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
        let kind = classify_archive_path(&path);

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

pub fn extract_zip_entries_to_directory<R>(
    reader: R,
    destination_root: &Path,
) -> Result<Vec<PathBuf>, ArchiveExtractError>
where
    R: Read + Seek,
{
    std::fs::create_dir_all(destination_root).map_err(ArchiveExtractError::CreateDirectory)?;

    let mut archive = ZipArchive::new(reader).map_err(ArchiveExtractError::OpenArchive)?;
    let mut extracted_paths = Vec::new();

    for index in 0..archive.len() {
        let mut entry = archive
            .by_index(index)
            .map_err(ArchiveExtractError::ReadEntry)?;

        let Some(relative_path) = entry.enclosed_name() else {
            return Err(ArchiveExtractError::InvalidEntryPath(
                entry.name().to_owned(),
            ));
        };

        let output_path = destination_root.join(relative_path);

        if entry.is_dir() {
            std::fs::create_dir_all(&output_path).map_err(ArchiveExtractError::CreateDirectory)?;
            continue;
        }

        if let Some(parent) = output_path.parent() {
            std::fs::create_dir_all(parent).map_err(ArchiveExtractError::CreateDirectory)?;
        }

        let mut output =
            std::fs::File::create(&output_path).map_err(ArchiveExtractError::CreateFile)?;
        std::io::copy(&mut entry, &mut output).map_err(ArchiveExtractError::CopyEntry)?;
        extracted_paths.push(output_path);
    }

    Ok(extracted_paths)
}

fn archive_file_name(path: &str) -> String {
    path.rsplit('/').next().unwrap_or(path).to_owned()
}

pub fn classify_archive_path(path: &str) -> ArchiveEntryKind {
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
        archive_file_name, classify_archive_path, extract_zip_entries_to_directory,
    };
    use std::fs;
    use std::io::{Cursor, Write};
    use std::time::{SystemTime, UNIX_EPOCH};
    use zip::write::SimpleFileOptions;
    use zip::{CompressionMethod, ZipWriter};

    #[test]
    fn archive_file_name_returns_last_path_segment() {
        assert_eq!(archive_file_name("nested/track.gpx"), "track.gpx");
    }

    #[test]
    fn classify_archive_path_matches_supported_extensions_case_insensitively() {
        assert_eq!(
            classify_archive_path("nested/FIELD.GPX"),
            ArchiveEntryKind::Supported(SupportedArchiveEntryKind::Gpx)
        );
        assert_eq!(
            classify_archive_path("maps/calibration.MAP"),
            ArchiveEntryKind::Supported(SupportedArchiveEntryKind::OziMap)
        );
    }

    #[test]
    fn classify_archive_path_marks_raster_and_unknown_payloads_as_unsupported() {
        assert_eq!(
            classify_archive_path("maps/base.ozf2"),
            ArchiveEntryKind::Unsupported(UnsupportedArchiveEntryKind::RasterPayload)
        );
        assert_eq!(
            classify_archive_path("mobile/cache.sqlitedb"),
            ArchiveEntryKind::Unsupported(UnsupportedArchiveEntryKind::SqliteTiles)
        );
        assert_eq!(
            classify_archive_path("notes/readme.txt"),
            ArchiveEntryKind::Unsupported(UnsupportedArchiveEntryKind::Unknown)
        );
    }

    #[test]
    fn extract_zip_entries_to_directory_writes_nested_files() {
        let archive = build_archive(&[
            ("maps/", b"".as_slice(), true),
            ("maps/demo.map", b"map".as_slice(), false),
            ("maps/demo.ozf2", b"ozf".as_slice(), false),
        ]);
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time")
            .as_nanos();
        let destination = std::env::temp_dir().join(format!("ozi-rs-zip-extract-{unique}"));

        let extracted = extract_zip_entries_to_directory(Cursor::new(archive), &destination)
            .expect("extract archive");

        assert_eq!(extracted.len(), 2);
        assert_eq!(
            fs::read(destination.join("maps/demo.map")).expect("map bytes"),
            b"map"
        );
        assert_eq!(
            fs::read(destination.join("maps/demo.ozf2")).expect("ozf bytes"),
            b"ozf"
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
}
