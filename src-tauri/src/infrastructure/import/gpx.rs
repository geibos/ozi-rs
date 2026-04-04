use crate::domain::{
    Track, TrackId, TrackPoint, TrackPointId, TrackSegment, TrackSegmentId, Waypoint, WaypointId,
};
use crate::infrastructure::import::archive::{
    ArchiveEntryKind, SupportedArchiveEntryKind, classify_archive_path,
};
use gpx::read;
use std::fmt;
use std::io::{BufReader, Cursor, Read, Seek};
use std::path::Path;
use zip::ZipArchive;
use zip::result::ZipError;

#[derive(Debug, Clone, PartialEq)]
pub struct ArchivedGpxImport {
    source_path: String,
    tracks: Vec<Track>,
    waypoints: Vec<Waypoint>,
}

impl ArchivedGpxImport {
    fn new(source_path: String, tracks: Vec<Track>, waypoints: Vec<Waypoint>) -> Self {
        Self {
            source_path,
            tracks,
            waypoints,
        }
    }

    pub fn source_path(&self) -> &str {
        &self.source_path
    }

    pub fn tracks(&self) -> &[Track] {
        &self.tracks
    }

    pub fn waypoints(&self) -> &[Waypoint] {
        &self.waypoints
    }
}

#[derive(Debug)]
pub enum ArchivedGpxImportError {
    OpenArchive(ZipError),
    ReadArchiveEntry {
        path: String,
        source: ZipError,
    },
    ReadArchiveEntryBytes {
        path: String,
        source: std::io::Error,
    },
    ParseGpx {
        path: String,
        source: gpx::errors::GpxError,
    },
}

impl fmt::Display for ArchivedGpxImportError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::OpenArchive(error) => write!(f, "failed to open ZIP archive: {error}"),
            Self::ReadArchiveEntry { path, source } => {
                write!(f, "failed to read ZIP archive entry {path}: {source}")
            }
            Self::ReadArchiveEntryBytes { path, source } => {
                write!(
                    f,
                    "failed to read ZIP archive entry bytes for {path}: {source}"
                )
            }
            Self::ParseGpx { path, source } => {
                write!(f, "failed to parse GPX entry {path}: {source}")
            }
        }
    }
}

impl std::error::Error for ArchivedGpxImportError {}

pub fn import_gpx_entries_from_archive<R>(
    reader: R,
) -> Result<Vec<ArchivedGpxImport>, ArchivedGpxImportError>
where
    R: Read + Seek,
{
    let mut archive = ZipArchive::new(reader).map_err(ArchivedGpxImportError::OpenArchive)?;
    let mut imports = Vec::new();

    for index in 0..archive.len() {
        let mut entry =
            archive
                .by_index(index)
                .map_err(|source| ArchivedGpxImportError::ReadArchiveEntry {
                    path: format!("index:{index}"),
                    source,
                })?;

        if entry.is_dir() {
            continue;
        }

        let path = entry.name().to_owned();
        if !matches!(
            classify_archive_path(&path),
            ArchiveEntryKind::Supported(SupportedArchiveEntryKind::Gpx)
        ) {
            continue;
        }

        let mut bytes = Vec::new();
        entry.read_to_end(&mut bytes).map_err(|source| {
            ArchivedGpxImportError::ReadArchiveEntryBytes {
                path: path.clone(),
                source,
            }
        })?;

        imports.push(parse_gpx_archive_entry(&path, &bytes)?);
    }

    Ok(imports)
}

/// Import a standalone `.gpx` file from disk.
pub fn import_gpx_file(path: &Path) -> Result<ArchivedGpxImport, ArchivedGpxImportError> {
    let file = std::fs::File::open(path).map_err(|source| {
        ArchivedGpxImportError::ReadArchiveEntryBytes {
            path: path.display().to_string(),
            source,
        }
    })?;
    let gpx = read(BufReader::new(file)).map_err(|source| ArchivedGpxImportError::ParseGpx {
        path: path.display().to_string(),
        source,
    })?;

    let file_stem = path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("track")
        .to_owned();
    let path_str = path.display().to_string();
    let tracks = gpx
        .tracks
        .into_iter()
        .enumerate()
        .map(|(i, t)| convert_track(i, &file_stem, t))
        .collect();
    let waypoints = gpx
        .waypoints
        .into_iter()
        .enumerate()
        .map(|(i, w)| convert_waypoint(i, &file_stem, w))
        .collect();

    Ok(ArchivedGpxImport::new(path_str, tracks, waypoints))
}

fn parse_gpx_archive_entry(
    path: &str,
    bytes: &[u8],
) -> Result<ArchivedGpxImport, ArchivedGpxImportError> {
    let gpx = read(Cursor::new(bytes)).map_err(|source| ArchivedGpxImportError::ParseGpx {
        path: path.to_owned(),
        source,
    })?;

    let file_stem = archive_file_stem(path);
    let tracks = gpx
        .tracks
        .into_iter()
        .enumerate()
        .map(|(track_index, track)| convert_track(track_index, &file_stem, track))
        .collect();
    let waypoints = gpx
        .waypoints
        .into_iter()
        .enumerate()
        .map(|(waypoint_index, waypoint)| convert_waypoint(waypoint_index, &file_stem, waypoint))
        .collect();

    Ok(ArchivedGpxImport::new(path.to_owned(), tracks, waypoints))
}

fn convert_track(track_index: usize, file_stem: &str, track: gpx::Track) -> Track {
    let mut imported_track = Track::new(
        TrackId::new((track_index + 1) as u64),
        track
            .name
            .unwrap_or_else(|| format!("{file_stem} track {}", track_index + 1)),
    );

    for (segment_index, segment) in track.segments.into_iter().enumerate() {
        imported_track.add_segment(convert_segment(segment_index, segment));
    }

    imported_track
}

fn convert_segment(segment_index: usize, segment: gpx::TrackSegment) -> TrackSegment {
    let mut imported_segment = TrackSegment::new(TrackSegmentId::new((segment_index + 1) as u64));

    for (point_index, point) in segment.points.into_iter().enumerate() {
        imported_segment.add_point(convert_track_point(point_index, point));
    }

    imported_segment
}

fn convert_track_point(point_index: usize, point: gpx::Waypoint) -> TrackPoint {
    let coordinates = point.point();
    let mut tp = TrackPoint::new(
        TrackPointId::new((point_index + 1) as u64),
        coordinates.y(),
        coordinates.x(),
    );
    if let Some(elev) = point.elevation {
        tp = tp.with_elevation(elev);
    }
    if let Some(t) = point.time.and_then(gpx_time_to_chrono) {
        tp = tp.with_timestamp(t);
    }
    tp
}

/// Convert `gpx::Time` (wraps `time::OffsetDateTime`) to `chrono::DateTime<Utc>`.
/// Uses the ISO 8601 string representation as a bridge to avoid a direct `time` crate dependency.
fn gpx_time_to_chrono(t: gpx::Time) -> Option<chrono::DateTime<chrono::Utc>> {
    let s = t.format().ok()?;
    chrono::DateTime::parse_from_rfc3339(&s)
        .ok()
        .map(|dt| dt.with_timezone(&chrono::Utc))
}

fn convert_waypoint(waypoint_index: usize, file_stem: &str, waypoint: gpx::Waypoint) -> Waypoint {
    let coordinates = waypoint.point();
    let mut imported_waypoint = Waypoint::new(
        WaypointId::new((waypoint_index + 1) as u64),
        waypoint
            .name
            .unwrap_or_else(|| format!("{file_stem} waypoint {}", waypoint_index + 1)),
        coordinates.y(),
        coordinates.x(),
    );

    if let Some(symbol) = waypoint.symbol {
        let _ = imported_waypoint.set_symbol(Some(symbol));
    }

    imported_waypoint
}

fn archive_file_stem(path: &str) -> String {
    let file_name = path.rsplit('/').next().unwrap_or(path);

    file_name
        .rsplit_once('.')
        .map(|(stem, _)| stem)
        .unwrap_or(file_name)
        .to_owned()
}

#[cfg(test)]
mod tests {
    use super::{archive_file_stem, import_gpx_file};
    use std::path::PathBuf;

    #[test]
    fn archive_file_stem_uses_last_path_segment_without_extension() {
        assert_eq!(archive_file_stem("nested/field-track.gpx"), "field-track");
    }

    #[test]
    fn import_gpx_file_reads_waypoint_symbol() {
        let path = PathBuf::from(format!(
            "{}/ozi-rs-waypoint-symbol-{}.gpx",
            std::env::temp_dir().display(),
            std::process::id()
        ));
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<gpx version="1.1" creator="ozi-rs" xmlns="http://www.topografix.com/GPX/1/1">
  <wpt lat="55.000000" lon="37.000000">
    <name>Camp</name>
    <sym>Flag</sym>
  </wpt>
</gpx>
"#;

        std::fs::write(&path, xml).unwrap();
        let imported = import_gpx_file(&path).unwrap();
        let _ = std::fs::remove_file(&path);

        assert_eq!(imported.waypoints().len(), 1);
        assert_eq!(imported.waypoints()[0].name(), "Camp");
        assert_eq!(imported.waypoints()[0].symbol(), Some("Flag"));
    }
}
