mod commands;
mod import;

pub use crate::infrastructure::import::PltImportError;
pub use commands::{CommandError, CommandStack, ProjectCommand};
pub use import::{ArchiveImportError, ArchiveImportReport};

use crate::domain::{LayerId, Project, TrackId};
use crate::infrastructure::import::{
    OziMapParseError, OziRasterKind, parse_ozi_map_metadata, read_ozi_map_text,
};
use crate::infrastructure::lizaalert;
use crate::infrastructure::persistence;
use std::collections::{HashSet, VecDeque};
use std::fmt;
use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct MapCenter {
    pub lat: f64,
    pub lon: f64,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct LizaProjectSummary {
    pub slug: String,
    pub name: String,
    pub url: String,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct LizaMapPackage {
    pub name: String,
    pub file_name: String,
    pub url: String,
    pub base_zoom: u8,
    pub local_path: Option<PathBuf>,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct LizaProject {
    pub summary: LizaProjectSummary,
    pub center: MapCenter,
    pub maps: Vec<LizaMapPackage>,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum ActiveMapKind {
    SqliteTiles,
    OziRaster,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ActiveMapSelection {
    pub kind: ActiveMapKind,
    pub project_name: String,
    pub package_name: String,
    pub remote_url: String,
    pub local_path: PathBuf,
    pub center: MapCenter,
    pub base_zoom: u8,
}

#[derive(Debug)]
pub enum OpenLocalMapError {
    Read(std::io::Error),
    Parse(OziMapParseError),
    UnsupportedRasterKind(OziRasterKind),
    Register(CommandError),
}

impl fmt::Display for OpenLocalMapError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Read(error) => write!(f, "failed to read OZI map file: {error}"),
            Self::Parse(error) => write!(f, "failed to parse OZI map metadata: {error}"),
            Self::UnsupportedRasterKind(kind) => {
                write!(f, "unsupported OZI raster kind for UI opening: {kind:?}")
            }
            Self::Register(error) => write!(f, "failed to register OZI map layer: {error:?}"),
        }
    }
}

impl std::error::Error for OpenLocalMapError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Read(error) => Some(error),
            Self::Parse(error) => Some(error),
            Self::UnsupportedRasterKind(_) => None,
            Self::Register(_) => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum DiagnosticLevel {
    Info,
    Error,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct DiagnosticEntry {
    level: DiagnosticLevel,
    message: String,
}

impl DiagnosticEntry {
    fn new(level: DiagnosticLevel, message: String) -> Self {
        Self { level, message }
    }

    pub const fn level(&self) -> DiagnosticLevel {
        self.level
    }

    pub fn message(&self) -> &str {
        &self.message
    }
}

#[derive(Debug)]
pub struct AppState {
    history: CommandStack,
    project: Project,
    project_path: Option<PathBuf>,
    bundles_root: PathBuf,
    lizaalert: LizaAlertState,
}

#[derive(Debug)]
struct LizaAlertState {
    projects: Vec<LizaProjectSummary>,
    selected_project_slug: Option<String>,
    selected_project: Option<LizaProject>,
    active_map: Option<ActiveMapSelection>,
    diagnostics: VecDeque<DiagnosticEntry>,
    status: String,
    /// True while project list or project metadata is loading (blocks re-entry).
    busy: bool,
    /// Package names currently being downloaded (allows parallel map downloads).
    downloading: HashSet<String>,
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}

impl AppState {
    pub fn new() -> Self {
        Self {
            history: CommandStack::default(),
            project: Project::default(),
            project_path: None,
            bundles_root: default_bundles_root(),
            lizaalert: LizaAlertState {
                projects: Vec::new(),
                selected_project_slug: None,
                selected_project: None,
                active_map: None,
                diagnostics: VecDeque::from([DiagnosticEntry::new(
                    DiagnosticLevel::Info,
                    "Load projects from maps.lizaalert.ru".to_owned(),
                )]),
                status: "Load projects from maps.lizaalert.ru".to_owned(),
                busy: false,
                downloading: HashSet::new(),
            },
        }
    }

    // ── Background-task handoff: "begin" sets busy and returns what the thread needs ──

    /// Returns `None` if already busy; otherwise sets busy and returns the bundles root path.
    pub fn begin_load_projects(&mut self) -> Option<PathBuf> {
        if self.lizaalert.busy {
            return None;
        }
        self.lizaalert.busy = true;
        self.update_status(DiagnosticLevel::Info, "Loading project list...");
        Some(self.bundles_root.clone())
    }

    /// Returns `None` if busy or project slug not found; otherwise returns data needed for thread.
    pub fn begin_load_project(
        &mut self,
        project_slug: &str,
    ) -> Option<(LizaProjectSummary, PathBuf)> {
        if self.lizaalert.busy {
            return None;
        }
        let summary = self
            .lizaalert
            .projects
            .iter()
            .find(|p| p.slug == project_slug)
            .cloned()?;

        self.lizaalert.busy = true;
        let status = if lizaalert::is_project_cached(&summary.slug, &self.bundles_root) {
            format!("Opening cached project {}...", summary.name)
        } else {
            format!("Downloading project {}...", summary.name)
        };
        self.update_status(DiagnosticLevel::Info, status);
        Some((summary, self.bundles_root.clone()))
    }

    /// Returns `None` if map/project not found or this package is already downloading.
    /// Multiple different packages can download in parallel.
    pub fn begin_open_map(&mut self, map_name: &str) -> Option<OpenMapRequest> {
        let project = self.lizaalert.selected_project.clone()?;
        let map = project.maps.iter().find(|m| m.name == map_name)?.clone();
        let selection = lizaalert::build_active_map_selection(&project, &map, &self.bundles_root);

        // If already local, handle synchronously (no dedup needed)
        if map.local_path.is_some() {
            return Some(OpenMapRequest::Local(selection));
        }

        // Prevent duplicate download of the same package
        if self.lizaalert.downloading.contains(&selection.package_name) {
            return None;
        }

        self.lizaalert
            .downloading
            .insert(selection.package_name.clone());
        self.update_status(
            DiagnosticLevel::Info,
            format!("Downloading {}...", selection.package_name),
        );
        Some(OpenMapRequest::Download(selection))
    }

    /// Returns the set of package names currently being downloaded.
    pub fn downloading_maps(&self) -> &HashSet<String> {
        &self.lizaalert.downloading
    }

    /// Returns `None` if busy; otherwise sets busy and returns directory for thread.
    pub fn begin_open_local_bundle(&mut self, dir: PathBuf) -> Option<PathBuf> {
        if self.lizaalert.busy {
            return None;
        }
        self.lizaalert.busy = true;
        self.update_status(
            DiagnosticLevel::Info,
            format!("Opening local bundle: {}", dir.display()),
        );
        Some(dir)
    }

    // ── Background-task completion: "apply" receives results and mutates state ──

    pub fn apply_projects_loaded(&mut self, result: Result<Vec<LizaProjectSummary>, String>) {
        self.lizaalert.busy = false;
        match result {
            Ok(_) => {
                let count = self.lizaalert.projects.len();
                self.update_status(DiagnosticLevel::Info, format!("Loaded {count} projects"));
            }
            Err(error) => {
                self.update_status(DiagnosticLevel::Error, error);
            }
        }
    }

    pub fn apply_projects_chunk(&mut self, chunk: Vec<LizaProjectSummary>) {
        for project in chunk {
            if self
                .lizaalert
                .projects
                .iter()
                .any(|existing| existing.slug == project.slug)
            {
                continue;
            }
            self.lizaalert.projects.push(project);
        }
    }

    pub fn apply_project_loaded(&mut self, result: Result<LizaProject, String>) {
        self.lizaalert.busy = false;
        match result {
            Ok(project) => {
                let name = project.summary.name.clone();
                self.lizaalert.selected_project_slug = Some(project.summary.slug.clone());
                self.lizaalert.selected_project = Some(project);
                self.update_status(DiagnosticLevel::Info, format!("Loaded project: {name}"));
            }
            Err(error) => {
                self.update_status(DiagnosticLevel::Error, error);
            }
        }
    }

    pub fn apply_map_downloaded(
        &mut self,
        package_name: &str,
        result: Result<ActiveMapSelection, String>,
    ) {
        self.lizaalert.downloading.remove(package_name);
        match result {
            Ok(selection) => {
                let status = match self.register_active_map_layer(&selection) {
                    Ok(true) => format!(
                        "Opened map: {} / {}",
                        selection.project_name, selection.package_name
                    ),
                    Ok(false) => format!(
                        "Opened map: {} / {} (already registered)",
                        selection.project_name, selection.package_name
                    ),
                    Err(error) => format!(
                        "Opened map: {} / {} (registration failed: {error:?})",
                        selection.project_name, selection.package_name
                    ),
                };
                self.lizaalert.active_map = Some(selection);
                self.update_status(DiagnosticLevel::Info, status);
            }
            Err(error) => {
                self.update_status(DiagnosticLevel::Error, error);
            }
        }
    }

    pub fn apply_progress(&mut self, message: String) {
        self.update_status(DiagnosticLevel::Info, message);
    }

    // ── Synchronous map-open helpers ──

    pub fn open_local_map_selection(&mut self, selection: ActiveMapSelection) {
        let status = match self.register_active_map_layer(&selection) {
            Ok(true) => format!(
                "Opened cached map: {} / {}",
                selection.project_name, selection.package_name
            ),
            Ok(false) => format!(
                "Opened cached map: {} / {} (already registered)",
                selection.project_name, selection.package_name
            ),
            Err(error) => format!(
                "Opened cached map: {} / {} (registration failed: {error:?})",
                selection.project_name, selection.package_name
            ),
        };
        self.lizaalert.active_map = Some(selection);
        self.update_status(DiagnosticLevel::Info, status);
    }

    pub fn open_local_ozi_map(
        &mut self,
        map_path: impl Into<PathBuf>,
    ) -> Result<(), OpenLocalMapError> {
        let map_path = map_path.into();
        let file_name = map_path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("local-ozi.map")
            .to_owned();
        self.open_ozi_map_selection("Local OZI".to_owned(), file_name, map_path)
    }

    fn open_ozi_map_selection(
        &mut self,
        project_name: String,
        package_name: String,
        map_path: PathBuf,
    ) -> Result<(), OpenLocalMapError> {
        let contents = read_ozi_map_text(&map_path).map_err(OpenLocalMapError::Read)?;
        let metadata =
            parse_ozi_map_metadata(&map_path, &contents).map_err(OpenLocalMapError::Parse)?;

        match metadata.raster_kind() {
            OziRasterKind::Ozf2 => {}
            kind => return Err(OpenLocalMapError::UnsupportedRasterKind(kind.clone())),
        }

        let selection = ActiveMapSelection {
            kind: ActiveMapKind::OziRaster,
            project_name,
            package_name,
            remote_url: String::new(),
            local_path: map_path,
            center: MapCenter { lat: 0.0, lon: 0.0 },
            base_zoom: 0,
        };

        match self.register_active_map_layer(&selection) {
            Ok(_) => {
                let status = format!(
                    "Opened OZI map: {} / {}",
                    selection.project_name, selection.package_name
                );
                self.lizaalert.active_map = Some(selection);
                self.update_status(DiagnosticLevel::Info, status);
                Ok(())
            }
            Err(error) => Err(OpenLocalMapError::Register(error)),
        }
    }

    // ── State accessors ──

    pub fn active_map(&self) -> Option<&ActiveMapSelection> {
        self.lizaalert.active_map.as_ref()
    }

    #[allow(dead_code)]
    pub fn map_layer_count(&self) -> usize {
        self.project.map_layers().len()
    }

    pub fn current_project(&self) -> Option<&LizaProject> {
        self.lizaalert.selected_project.as_ref()
    }

    pub fn lizaalert_projects(&self) -> &[LizaProjectSummary] {
        &self.lizaalert.projects
    }

    pub fn lizaalert_status(&self) -> &str {
        &self.lizaalert.status
    }

    pub fn recent_diagnostics(&self) -> impl DoubleEndedIterator<Item = &DiagnosticEntry> {
        self.lizaalert.diagnostics.iter()
    }

    pub fn lizaalert_busy(&self) -> bool {
        self.lizaalert.busy
    }

    pub fn project_name(&self) -> &str {
        self.project.name()
    }

    pub fn project_file_path(&self) -> Option<&std::path::Path> {
        self.project_path.as_deref()
    }

    #[allow(dead_code)]
    pub fn bundles_root(&self) -> &std::path::Path {
        &self.bundles_root
    }

    pub fn set_bundles_root(&mut self, path: PathBuf) {
        self.bundles_root = path;
    }

    pub fn track_layers(&self) -> &[crate::domain::TrackLayer] {
        self.project.track_layers()
    }

    pub fn track_layer_count(&self) -> usize {
        self.project.track_layers().len()
    }

    pub fn waypoint_layer_count(&self) -> usize {
        self.project.waypoint_layers().len()
    }

    // ── Mutations ──

    pub fn save_project_to(&mut self, path: PathBuf) {
        match persistence::save_project(&self.project, &path) {
            Ok(()) => {
                let display = path.display().to_string();
                self.project_path = Some(path);
                self.update_status(DiagnosticLevel::Info, format!("Saved: {display}"));
            }
            Err(error) => {
                self.update_status(DiagnosticLevel::Error, format!("Save failed: {error}"));
            }
        }
    }

    pub fn load_project_from(&mut self, path: PathBuf) {
        match persistence::load_project(&path) {
            Ok(project) => {
                let display = path.display().to_string();
                self.project = project;
                self.project_path = Some(path);
                self.history = CommandStack::default();
                self.lizaalert.active_map = None;
                self.update_status(DiagnosticLevel::Info, format!("Opened: {display}"));
            }
            Err(error) => {
                self.update_status(DiagnosticLevel::Error, format!("Open failed: {error}"));
            }
        }
    }

    pub fn import_gpx_file(
        &mut self,
        path: std::path::PathBuf,
    ) -> Result<ArchiveImportReport, ArchiveImportError> {
        import::import_gpx_file_into_project(&mut self.project, &mut self.history, &path)
    }

    pub fn import_plt_file(
        &mut self,
        path: std::path::PathBuf,
    ) -> Result<ArchiveImportReport, PltImportError> {
        import::import_plt_file_into_project(&mut self.project, &mut self.history, &path)
    }

    pub fn set_track_color(&mut self, layer_id: LayerId, track_id: TrackId, color: [u8; 4]) {
        if let Ok(track) = self.project.track_mut(layer_id.value(), track_id.value()) {
            track.style_mut().color = color;
        }
    }

    pub fn toggle_track_visible(&mut self, layer_id: LayerId, track_id: TrackId) {
        let visible = self
            .project
            .track_layers()
            .iter()
            .find(|l| l.id() == layer_id)
            .and_then(|l| l.tracks().iter().find(|t| t.id() == track_id))
            .map(|t| t.style().visible)
            .unwrap_or(true);
        self.project
            .set_track_visible_in_layer(layer_id, track_id, !visible);
    }

    pub fn rename_track(&mut self, layer_id: LayerId, track_id: TrackId, new_name: String) {
        let old_name = self
            .project
            .track_layers()
            .iter()
            .find(|l| l.id() == layer_id)
            .and_then(|l| l.tracks().iter().find(|t| t.id() == track_id))
            .map(|t| t.name().to_owned())
            .unwrap_or_default();

        let _ = self.history.apply(
            &mut self.project,
            &commands::ProjectCommand::rename_track(layer_id, track_id, old_name, new_name),
        );
    }

    pub fn export_layer_to_gpx(&mut self, layer_id: LayerId, path: std::path::PathBuf) {
        let Some(layer) = self
            .project
            .track_layers()
            .iter()
            .find(|l| l.id() == layer_id)
        else {
            self.update_status(DiagnosticLevel::Error, "Layer not found for export");
            return;
        };
        match crate::infrastructure::export::export_layer_to_gpx_file(layer, &path) {
            Ok(()) => self.update_status(
                DiagnosticLevel::Info,
                format!("Exported to {}", path.display()),
            ),
            Err(e) => {
                self.update_status(DiagnosticLevel::Error, format!("Export failed: {e}"));
            }
        }
    }

    pub fn undo(&mut self) {
        self.history.undo(&mut self.project);
    }

    pub fn redo(&mut self) {
        self.history.redo(&mut self.project);
    }

    #[allow(dead_code)]
    pub fn restore_active_map(&mut self, selection: ActiveMapSelection) {
        if selection.local_path.exists() {
            self.lizaalert.active_map = Some(selection);
        }
    }

    pub fn active_bundle_dir(&self) -> Option<PathBuf> {
        let map = self.lizaalert.active_map.as_ref()?;
        let slug = self
            .lizaalert
            .selected_project
            .as_ref()
            .map(|p| p.summary.slug.as_str())
            .or_else(|| map.local_path.ancestors().nth(2).map(|_| ""))?;

        if slug.is_empty() {
            return None;
        }
        Some(lizaalert::bundle_directory(&self.bundles_root, slug))
    }

    pub fn reveal_active_bundle(&self) {
        let Some(dir) = self.active_bundle_dir() else {
            return;
        };
        reveal_in_file_manager(&dir);
    }

    pub fn report_runtime_error(&mut self, message: impl Into<String>) {
        self.push_diagnostic(DiagnosticLevel::Error, message.into());
    }

    #[allow(dead_code)]
    pub fn report_runtime_info(&mut self, message: impl Into<String>) {
        self.push_diagnostic(DiagnosticLevel::Info, message.into());
    }

    // ── Private helpers ──

    fn register_active_map_layer(
        &mut self,
        selection: &ActiveMapSelection,
    ) -> Result<bool, CommandError> {
        if self
            .project
            .map_layers()
            .iter()
            .any(|layer| layer.source_path() == Some(selection.local_path.as_path()))
        {
            return Ok(false);
        }

        let layer_id = LayerId::new(self.project.map_layers().len() as u64 + 1);
        let layer_name = format!("{} / {}", selection.project_name, selection.package_name);
        self.history.apply(
            &mut self.project,
            &ProjectCommand::add_map_layer_with_source(
                layer_id,
                layer_name,
                selection.local_path.clone(),
            ),
        )?;

        Ok(true)
    }

    fn update_status(&mut self, level: DiagnosticLevel, message: impl Into<String>) {
        let message = message.into();
        self.lizaalert.status = message.clone();
        self.push_diagnostic(level, message);
    }

    fn push_diagnostic(&mut self, level: DiagnosticLevel, message: String) {
        match level {
            DiagnosticLevel::Error => tracing::error!("{message}"),
            DiagnosticLevel::Info => tracing::info!("{message}"),
        }
        self.lizaalert
            .diagnostics
            .push_back(DiagnosticEntry::new(level, message));
    }
}

/// Payload returned by `begin_open_map` to the command handler.
pub enum OpenMapRequest {
    /// Map is already local — no download needed, open synchronously.
    Local(ActiveMapSelection),
    /// Map must be downloaded — spawn a background thread.
    Download(ActiveMapSelection),
}

fn default_bundles_root() -> PathBuf {
    if let Ok(home) = std::env::var("HOME") {
        return PathBuf::from(home).join("Documents").join("LizaAlert Maps");
    }
    PathBuf::from("bundles")
}

fn reveal_in_file_manager(path: &std::path::Path) {
    #[cfg(target_os = "macos")]
    {
        let _ = std::process::Command::new("open").arg(path).spawn();
    }
    #[cfg(target_os = "linux")]
    {
        let _ = std::process::Command::new("xdg-open").arg(path).spawn();
    }
    #[cfg(target_os = "windows")]
    {
        let _ = std::process::Command::new("explorer").arg(path).spawn();
    }
}
