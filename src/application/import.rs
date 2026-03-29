use crate::application::{CommandError, CommandStack, ProjectCommand};
use crate::domain::{LayerId, Project};
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
    let layer_name = format!("Imported tracks: {}", import.source_path);
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
        let layer_name = format!("Imported tracks: {}", import.source_path());
        history.apply(
            project,
            &ProjectCommand::add_track_layer(layer_id, layer_name),
        )?;
        report.imported_track_layers += 1;

        for track in import.tracks() {
            history.apply(project, &ProjectCommand::add_track(layer_id, track.clone()))?;
            report.imported_tracks += 1;
        }
    }

    if !import.waypoints().is_empty() {
        let layer_id = next_layer_id(project);
        let layer_name = format!("Imported waypoints: {}", import.source_path());
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

fn next_layer_id(project: &Project) -> LayerId {
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
