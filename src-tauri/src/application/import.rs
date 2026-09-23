#![allow(dead_code)]

use crate::application::{CommandError, CommandStack, ProjectCommand};
use crate::domain::{LayerId, Project};
use crate::infrastructure::import::WptImportError;
use crate::infrastructure::import::{
    ArchivedGpxImport, ArchivedGpxImportError, PltImportError, import_gpx_entries_from_archive,
    import_gpx_file, import_plt_file,
};
use std::fmt;
use std::io::{Read, Seek};
use std::path::Path;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArchiveImportReport {
    imported_entries: usize,
    imported_track_layers: usize,
    imported_waypoint_layers: usize,
    imported_tracks: usize,
    imported_waypoints: usize,
}

impl ArchiveImportReport {
    fn new() -> Self {
        Self {
            imported_entries: 0,
            imported_track_layers: 0,
            imported_waypoint_layers: 0,
            imported_tracks: 0,
            imported_waypoints: 0,
        }
    }

    pub const fn imported_entries(&self) -> usize {
        self.imported_entries
    }

    pub const fn imported_track_layers(&self) -> usize {
        self.imported_track_layers
    }

    pub const fn imported_waypoint_layers(&self) -> usize {
        self.imported_waypoint_layers
    }

    pub const fn imported_tracks(&self) -> usize {
        self.imported_tracks
    }

    pub const fn imported_waypoints(&self) -> usize {
        self.imported_waypoints
    }
}

#[derive(Debug)]
pub enum ArchiveImportError {
    GpxImport(ArchivedGpxImportError),
    Command(CommandError),
}

impl fmt::Display for ArchiveImportError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::GpxImport(error) => write!(f, "failed to import GPX archive: {error}"),
            Self::Command(error) => write!(f, "failed to apply archive import command: {error:?}"),
        }
    }
}

impl std::error::Error for ArchiveImportError {}

impl From<ArchivedGpxImportError> for ArchiveImportError {
    fn from(value: ArchivedGpxImportError) -> Self {
        Self::GpxImport(value)
    }
}

impl From<CommandError> for ArchiveImportError {
    fn from(value: CommandError) -> Self {
        Self::Command(value)
    }
}

/// Import a single standalone `.gpx` file into the project as a new track layer.
pub fn import_gpx_file_into_project(
    project: &mut Project,
    history: &mut CommandStack,
    path: &Path,
) -> Result<ArchiveImportReport, ArchiveImportError> {
    // A .zip lands here from the unified Import dialog: route it to the
    // archive importer (which extracts every .gpx inside) instead of trying
    // to parse the zip bytes as GPX XML.
    let is_zip = path
        .extension()
        .and_then(|e| e.to_str())
        .is_some_and(|e| e.eq_ignore_ascii_case("zip"));
    if is_zip {
        let file = std::fs::File::open(path).map_err(|source| {
            ArchiveImportError::GpxImport(ArchivedGpxImportError::ReadArchiveEntryBytes {
                path: path.display().to_string(),
                source,
            })
        })?;
        return import_gpx_archive_into_project(project, history, std::io::BufReader::new(file));
    }
    let import = import_gpx_file(path)?;
    let mut report = ArchiveImportReport::new();
    apply_gpx_import(project, history, import, &mut report)?;
    Ok(report)
}

/// Import a single `.plt` file into the project as a new track layer.
pub fn import_plt_file_into_project(
    project: &mut Project,
    history: &mut CommandStack,
    path: &Path,
) -> Result<ArchiveImportReport, PltImportError> {
    let import = import_plt_file(path)?;
    let mut report = ArchiveImportReport::new();

    let layer_id = next_layer_id(project);
    let layer_name = source_file_label(&import.source_path);
    history
        .apply(
            project,
            &ProjectCommand::add_track_layer(layer_id, layer_name),
        )
        .ok();
    report.imported_track_layers += 1;
    history
        .apply(project, &ProjectCommand::add_track(layer_id, import.track))
        .ok();
    report.imported_tracks += 1;
    report.imported_entries += 1;

    Ok(report)
}

/// Import an OziExplorer waypoint file into a layer of its own.
///
/// A neighbouring headquarters running the original hands over a `.wpt`. Until
/// now the only way to take it was to find something that converts it to GPX
/// first, which is not a thing a coordinator does at four in the morning.
///
/// A layer per file, named after it, like every other import: the marks a
/// second штаб sends are theirs, and mixing them into ours would make them
/// impossible to hand back or hide.
pub fn import_wpt_file_into_project(
    project: &mut Project,
    history: &mut CommandStack,
    path: &Path,
) -> Result<ArchiveImportReport, WptImportError> {
    let import = crate::infrastructure::import::import_wpt_file(path)?;
    let mut report = ArchiveImportReport::new();

    let waypoints = import.waypoints().to_vec();
    if waypoints.is_empty() {
        return Ok(report);
    }

    let layer_id = next_layer_id(project);
    let layer_name = source_file_label(import.source_path());
    history
        .apply(
            project,
            &ProjectCommand::add_waypoint_layer(layer_id, layer_name),
        )
        .ok();
    report.imported_waypoint_layers += 1;

    for waypoint in waypoints {
        history
            .apply(project, &ProjectCommand::add_waypoint(layer_id, waypoint))
            .ok();
        report.imported_waypoints += 1;
    }
    report.imported_entries += 1;

    Ok(report)
}

pub fn import_gpx_archive_into_project<R>(
    project: &mut Project,
    history: &mut CommandStack,
    reader: R,
) -> Result<ArchiveImportReport, ArchiveImportError>
where
    R: Read + Seek,
{
    let imports = import_gpx_entries_from_archive(reader)?;
    let mut report = ArchiveImportReport::new();

    for imported_entry in imports {
        report.imported_entries += 1;
        apply_gpx_import(project, history, imported_entry, &mut report)?;
    }

    Ok(report)
}

fn apply_gpx_import(
    project: &mut Project,
    history: &mut CommandStack,
    import: ArchivedGpxImport,
    report: &mut ArchiveImportReport,
) -> Result<(), ArchiveImportError> {
    if !import.tracks().is_empty() {
        let layer_id = next_layer_id(project);
        let layer_name = source_file_label(import.source_path());
        history.apply(
            project,
            &ProjectCommand::add_track_layer(layer_id, layer_name),
        )?;
        report.imported_track_layers += 1;

        for (index, track) in import.tracks().iter().enumerate() {
            let mut track = track.clone();
            if !import.declared_colour(index) {
                track.style_mut().color = palette_colour(track_count(project));
            }
            history.apply(project, &ProjectCommand::add_track(layer_id, track))?;
            report.imported_tracks += 1;
        }
    }

    if !import.waypoints().is_empty() {
        let layer_id = next_layer_id(project);
        let layer_name = source_file_label(import.source_path());
        history.apply(
            project,
            &ProjectCommand::add_waypoint_layer(layer_id, layer_name),
        )?;
        report.imported_waypoint_layers += 1;

        for waypoint in import.waypoints() {
            history.apply(
                project,
                &ProjectCommand::add_waypoint(layer_id, waypoint.clone()),
            )?;
            report.imported_waypoints += 1;
        }
    }

    Ok(())
}

/// Colours handed to imported tracks that do not say what colour they are.
///
/// Almost no GPX says: the colour is a Garmin extension, written by Garmin's
/// own software and by little else, so a day's folder from phones and
/// navigators used to import as twenty identical red lines. Telling the crews
/// apart on the map is most of what a coordinator does with a day.
///
/// Chosen to stay apart from each other and from a topographic basemap, which
/// is mostly green, grey and pale yellow: no greens, nothing pale, and the
/// first few as far apart as the wheel allows, since most days need only the
/// first few.
pub const TRACK_PALETTE: [[u8; 4]; 12] = [
    [220, 38, 38, 255],  // red
    [37, 99, 235, 255],  // blue
    [217, 119, 6, 255],  // amber
    [147, 51, 234, 255], // violet
    [8, 145, 178, 255],  // cyan
    [219, 39, 119, 255], // pink
    [120, 53, 15, 255],  // brown
    [30, 64, 175, 255],  // deep blue
    [190, 24, 93, 255],  // crimson
    [126, 34, 206, 255], // purple
    [161, 98, 7, 255],   // bronze
    [15, 118, 110, 255], // teal
];

/// The colour for the `index`-th track a project holds.
///
/// Position in the project, not in the file, so a folder import continues the
/// sequence across files and the same day imported twice looks the same twice.
fn palette_colour(index: usize) -> [u8; 4] {
    TRACK_PALETTE[index % TRACK_PALETTE.len()]
}

/// How many tracks the project already holds, across every layer.
fn track_count(project: &Project) -> usize {
    project
        .track_layers()
        .iter()
        .map(|layer| layer.tracks().len())
        .sum()
}

/// Name an import-created layer after the source file.
///
/// The full path made every entry in the layer selector look the same: a column
/// of `Imported tracks: /Users/.../` with the distinguishing part cut off.
fn source_file_label(source_path: &str) -> String {
    source_path
        .rsplit(['/', '\\'])
        .next()
        .filter(|name| !name.is_empty())
        .unwrap_or(source_path)
        .to_owned()
}

/// Allocate a layer id that no layer of any kind currently uses.
///
/// Ids are unique per project, not per layer kind, and callers assign them
/// (ADR-0014: no central generator). Deriving one from a layer count breaks as
/// soon as a layer is removed — the count drops and the next allocation hands
/// out an id that is still in use.
pub(crate) fn next_layer_id(project: &Project) -> LayerId {
    let max_map = project
        .map_layers()
        .iter()
        .map(|layer| layer.id().value())
        .max();
    let max_track = project
        .track_layers()
        .iter()
        .map(|layer| layer.id().value())
        .max();
    let max_waypoint = project
        .waypoint_layers()
        .iter()
        .map(|layer| layer.id().value())
        .max();
    let next = [max_map, max_track, max_waypoint]
        .into_iter()
        .flatten()
        .max()
        .unwrap_or(0)
        + 1;

    LayerId::new(next)
}

/// Result of a recursive folder import (CJ-3: the field convention is a
/// `10-Tracks/` folder with per-date subfolders of GPX/PLT files).
#[derive(Debug)]
pub struct DirectoryImportReport {
    pub imported_files: usize,
    pub imported_tracks: usize,
    pub imported_waypoints: usize,
    /// Files that failed to import, with the reason. A single bad file must
    /// never abort the rest of the folder.
    pub skipped: Vec<(std::path::PathBuf, String)>,
}

/// Recursively import every `.gpx` / `.plt` under `dir` (case-insensitive
/// extensions, deterministic sorted order). Per-file failures are collected
/// into the report instead of aborting. Errors only when the directory is
/// unreadable or contains no candidate files at all.
pub fn import_tracks_directory_into_project(
    project: &mut Project,
    history: &mut CommandStack,
    dir: &Path,
) -> Result<DirectoryImportReport, String> {
    let mut candidates: Vec<std::path::PathBuf> = Vec::new();
    collect_track_files(dir, &mut candidates)
        .map_err(|e| format!("cannot read folder {}: {e}", dir.display()))?;
    candidates.sort();
    if candidates.is_empty() {
        return Err(format!(
            "no .gpx or .plt files found under {}",
            dir.display()
        ));
    }

    let mut report = DirectoryImportReport {
        imported_files: 0,
        imported_tracks: 0,
        imported_waypoints: 0,
        skipped: Vec::new(),
    };
    for path in candidates {
        let extension = path
            .extension()
            .and_then(|e| e.to_str())
            .map(str::to_ascii_lowercase)
            .unwrap_or_default();
        let outcome = match extension.as_str() {
            "gpx" => import_gpx_file_into_project(project, history, &path)
                .map(|r| (r.imported_tracks(), r.imported_waypoints()))
                .map_err(|e| e.to_string()),
            "plt" => import_plt_file_into_project(project, history, &path)
                .map(|r| (r.imported_tracks(), r.imported_waypoints()))
                .map_err(|e| e.to_string()),
            _ => unreachable!("collect_track_files only yields gpx/plt"),
        };
        match outcome {
            Ok((tracks, waypoints)) => {
                report.imported_files += 1;
                report.imported_tracks += tracks;
                report.imported_waypoints += waypoints;
            }
            Err(reason) => report.skipped.push((path, reason)),
        }
    }
    Ok(report)
}

fn collect_track_files(dir: &Path, into: &mut Vec<std::path::PathBuf>) -> std::io::Result<()> {
    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            // A broken subfolder should not kill the whole walk.
            let _ = collect_track_files(&path, into);
            continue;
        }
        let is_track = path
            .extension()
            .and_then(|e| e.to_str())
            .is_some_and(|e| e.eq_ignore_ascii_case("gpx") || e.eq_ignore_ascii_case("plt"));
        if is_track {
            into.push(path);
        }
    }
    Ok(())
}

#[cfg(test)]
mod directory_import_tests {
    use super::*;
    use crate::domain::{LayerId, TrackLayer, WaypointLayer};

    const MINIMAL_GPX: &str = r#"<?xml version="1.0"?>
<gpx version="1.1" creator="test" xmlns="http://www.topografix.com/GPX/1/1">
  <trk><name>20260709-ЛИСА15</name><trkseg>
    <trkpt lat="55.0" lon="37.0"></trkpt>
    <trkpt lat="55.1" lon="37.1"></trkpt>
  </trkseg></trk>
</gpx>"#;

    fn project_with_default_layers() -> Project {
        let mut project = Project::untitled();
        project.add_track_layer(TrackLayer::new(LayerId::new(1), "Tracks"));
        project.add_waypoint_layer(WaypointLayer::new(LayerId::new(1), "Waypoints"));
        project
    }

    #[test]
    fn imports_recursively_and_skips_broken_files() {
        let dir = tempfile::tempdir().expect("tempdir");
        let by_date = dir.path().join("20260709");
        std::fs::create_dir_all(&by_date).expect("subdir");
        std::fs::write(by_date.join("a.gpx"), MINIMAL_GPX).expect("gpx a");
        std::fs::write(by_date.join("b.GPX"), MINIMAL_GPX).expect("gpx b uppercase");
        std::fs::write(by_date.join("broken.gpx"), "not xml at all").expect("broken");
        std::fs::write(dir.path().join("notes.txt"), "ignore me").expect("txt");

        let mut project = project_with_default_layers();
        let mut history = CommandStack::default();
        let report = import_tracks_directory_into_project(&mut project, &mut history, dir.path())
            .expect("directory import succeeds");

        assert_eq!(report.imported_files, 2, "two valid gpx files");
        assert_eq!(report.imported_tracks, 2);
        assert_eq!(report.skipped.len(), 1, "broken file skipped, not fatal");
        assert!(report.skipped[0].0.ends_with("broken.gpx"));
    }

    #[test]
    fn errors_when_folder_has_no_track_files() {
        let dir = tempfile::tempdir().expect("tempdir");
        std::fs::write(dir.path().join("readme.txt"), "x").expect("txt");

        let mut project = project_with_default_layers();
        let mut history = CommandStack::default();
        let result = import_tracks_directory_into_project(&mut project, &mut history, dir.path());
        assert!(result.is_err(), "no candidates must be a clear error");
    }
}

#[cfg(test)]
mod zip_routing_tests {
    use super::*;
    use crate::domain::{LayerId, TrackLayer, WaypointLayer};
    use std::io::Write;

    const MINIMAL_GPX: &str = r#"<?xml version="1.0"?>
<gpx version="1.1" creator="test" xmlns="http://www.topografix.com/GPX/1/1">
  <trk><name>20260709-ЛИСА15</name><trkseg>
    <trkpt lat="55.0" lon="37.0"></trkpt>
    <trkpt lat="55.1" lon="37.1"></trkpt>
  </trkseg></trk>
</gpx>"#;

    /// The unified Import dialog sends .zip through the same command as
    /// .gpx - the file importer must route archives, not parse them as XML.
    #[test]
    fn import_gpx_file_routes_zip_archives() {
        let dir = tempfile::tempdir().expect("tempdir");
        let zip_path = dir.path().join("tracks.zip");
        let file = std::fs::File::create(&zip_path).expect("create zip");
        let mut writer = zip::ZipWriter::new(file);
        writer
            .start_file::<_, ()>("20260709/a.gpx", zip::write::FileOptions::default())
            .expect("start entry");
        writer
            .write_all(MINIMAL_GPX.as_bytes())
            .expect("write entry");
        writer.finish().expect("finish zip");

        let mut project = Project::untitled();
        project.add_track_layer(TrackLayer::new(LayerId::new(1), "Tracks"));
        project.add_waypoint_layer(WaypointLayer::new(LayerId::new(1), "Waypoints"));
        let mut history = CommandStack::default();

        let report = import_gpx_file_into_project(&mut project, &mut history, &zip_path)
            .expect("zip routed to archive importer");
        assert_eq!(report.imported_tracks(), 1);
    }
}

#[cfg(test)]
mod layer_id_tests {
    use super::next_layer_id;
    use crate::domain::{LayerId, MapLayer, Project, TrackLayer, WaypointLayer};

    #[test]
    fn next_layer_id_clears_every_layer_kind() {
        let mut project = Project::default();
        project.add_track_layer(TrackLayer::new(LayerId::new(1), "Tracks"));
        project.add_waypoint_layer(WaypointLayer::new(LayerId::new(7), "Waypoints"));
        project.add_map_layer(MapLayer::new(LayerId::new(4), "Map"));

        assert_eq!(next_layer_id(&project).value(), 8);
    }

    /// The failure this guards: allocating from a count reuses an id after a
    /// removal, so two layers end up sharing one identifier.
    #[test]
    fn next_layer_id_does_not_reuse_an_id_after_a_removal() {
        let mut project = Project::default();
        project.add_map_layer(MapLayer::new(LayerId::new(1), "First"));
        project.add_map_layer(MapLayer::new(LayerId::new(2), "Second"));
        assert!(project.remove_map_layer(LayerId::new(1)));

        let next = next_layer_id(&project);
        assert_eq!(
            project.map_layers().len(),
            1,
            "count-based allocation would say 2"
        );
        assert_eq!(next.value(), 3);
        assert!(
            project.map_layers().iter().all(|l| l.id() != next),
            "allocated id must not already be in use"
        );
    }
}

#[cfg(test)]
mod track_colour_tests {
    use super::{TRACK_PALETTE, import_tracks_directory_into_project};
    use crate::application::CommandStack;
    use crate::domain::Project;

    /// A day's recordings arrive as a folder of GPX from phones and
    /// navigators, and almost none of them carry a colour — the Garmin
    /// extension is written by Garmin's own software and by little else. Every
    /// one of them used to import as the domain's default red, so twenty
    /// crews' routes drew one red smear on the map, and telling them apart is
    /// most of what a coordinator does with a day.
    ///
    /// A track that says what colour it is keeps it. That half already worked;
    /// this is the other half, and it is the common one.
    #[test]
    fn imported_tracks_without_a_colour_get_different_ones() {
        let dir = tempfile::tempdir().expect("tempdir");
        for (index, name) in ["lisa15", "veter2", "sokol7"].iter().enumerate() {
            std::fs::write(
                dir.path().join(format!("{name}.gpx")),
                format!(
                    r#"<?xml version="1.0"?><gpx version="1.1"><trk><name>{name}</name>
<trkseg><trkpt lat="59.9{index}" lon="31.5{index}"/><trkpt lat="59.91" lon="31.51"/></trkseg>
</trk></gpx>"#
                ),
            )
            .expect("write");
        }

        let mut project = Project::default();
        let mut history = CommandStack::default();
        import_tracks_directory_into_project(&mut project, &mut history, dir.path())
            .expect("import");

        let colours: Vec<[u8; 4]> = project
            .track_layers()
            .iter()
            .flat_map(|layer| layer.tracks().iter().map(|t| t.style().color))
            .collect();
        assert_eq!(colours.len(), 3);
        let distinct: std::collections::HashSet<_> = colours.iter().collect();
        assert_eq!(
            distinct.len(),
            3,
            "three routes, three colours: {colours:?}"
        );
    }

    #[test]
    fn an_imported_track_that_declares_a_colour_keeps_it() {
        let dir = tempfile::tempdir().expect("tempdir");
        std::fs::write(
            dir.path().join("blue.gpx"),
            r#"<?xml version="1.0"?><gpx version="1.1" creator="ozi-rs"
  xmlns="http://www.topografix.com/GPX/1/1"
  xmlns:gpxx="http://www.garmin.com/xmlschemas/GpxExtensions/v3">
<trk><name>blue</name>
<extensions><gpxx:TrackExtension>
  <gpxx:DisplayColor>Blue</gpxx:DisplayColor>
</gpxx:TrackExtension></extensions>
<trkseg><trkpt lat="59.9" lon="31.5"/></trkseg></trk></gpx>"#,
        )
        .expect("write");

        let mut project = Project::default();
        let mut history = CommandStack::default();
        let report = import_tracks_directory_into_project(&mut project, &mut history, dir.path())
            .expect("import");
        assert_eq!(
            report.imported_tracks, 1,
            "the file did not import at all: {:?}",
            report.skipped
        );

        // Across the layers: a fresh project already has an empty one, and the
        // import adds its own beside it.
        let colours: Vec<[u8; 4]> = project
            .track_layers()
            .iter()
            .flat_map(|layer| layer.tracks().iter().map(|t| t.style().color))
            .collect();
        assert_eq!(colours, vec![[0, 0, 255, 255]]);
    }

    /// The same folder imported into a fresh project twice gives the same
    /// colours, so a coordinator's screenshot still matches the screen after a
    /// reopen, and two machines looking at the same day agree.
    #[test]
    fn the_colours_are_the_same_every_time_the_same_day_is_imported() {
        let dir = tempfile::tempdir().expect("tempdir");
        for name in ["a", "b", "c", "d"] {
            std::fs::write(
                dir.path().join(format!("{name}.gpx")),
                r#"<?xml version="1.0"?><gpx version="1.1"><trk><name>t</name>
<trkseg><trkpt lat="59.9" lon="31.5"/></trkseg></trk></gpx>"#,
            )
            .expect("write");
        }

        let colours_of = || {
            let mut project = Project::default();
            let mut history = CommandStack::default();
            import_tracks_directory_into_project(&mut project, &mut history, dir.path())
                .expect("import");
            project
                .track_layers()
                .iter()
                .flat_map(|l| l.tracks().iter().map(|t| t.style().color))
                .collect::<Vec<_>>()
        };
        assert_eq!(colours_of(), colours_of());
    }

    #[test]
    fn the_palette_holds_distinct_colours() {
        let distinct: std::collections::HashSet<_> = TRACK_PALETTE.iter().collect();
        assert_eq!(
            distinct.len(),
            TRACK_PALETTE.len(),
            "a repeated entry would hand two crews the same colour early"
        );
        assert!(TRACK_PALETTE.len() >= 8, "a day has more than a few crews");
    }
}
