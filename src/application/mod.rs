mod commands;
mod import;

pub use commands::{CommandError, CommandStack, ProjectCommand};
pub use import::{ArchiveImportError, ArchiveImportReport, import_gpx_archive_into_project};

use crate::domain::{LayerId, Project};
use crate::infrastructure::import::{OziMapParseError, OziRasterKind, parse_ozi_map_metadata};
use crate::infrastructure::lizaalert;
use std::collections::VecDeque;
use std::fmt;
use std::path::PathBuf;
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;

const MAX_DIAGNOSTIC_ENTRIES: usize = 100;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MapCenter {
    pub lat: f64,
    pub lon: f64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LizaProjectSummary {
    pub slug: String,
    pub name: String,
    pub url: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LizaMapPackage {
    pub name: String,
    pub file_name: String,
    pub url: String,
    pub base_zoom: u8,
    pub local_path: Option<PathBuf>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct LizaProject {
    pub summary: LizaProjectSummary,
    pub center: MapCenter,
    pub maps: Vec<LizaMapPackage>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ActiveMapKind {
    SqliteTiles,
    OziRaster,
}

#[derive(Debug, Clone, PartialEq)]
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct MapDownloadProgress {
    downloaded_bytes: u64,
    total_bytes: Option<u64>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiagnosticLevel {
    Info,
    Error,
}

#[derive(Debug, Clone, PartialEq, Eq)]
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
    busy: bool,
    sender: Sender<BackgroundMessage>,
    receiver: Receiver<BackgroundMessage>,
}

#[derive(Debug)]
enum BackgroundMessage {
    ProjectsLoaded(Result<Vec<LizaProjectSummary>, String>),
    ProjectLoaded(Result<LizaProject, String>),
    MapDownloadProgress(MapDownloadProgress),
    MapDownloaded(Result<ActiveMapSelection, String>),
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}

impl AppState {
    pub fn new() -> Self {
        let (sender, receiver) = mpsc::channel();

        Self {
            history: CommandStack::default(),
            project: Project::default(),
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
                sender,
                receiver,
            },
        }
    }

    pub fn poll_background_tasks(&mut self) {
        while let Ok(message) = self.lizaalert.receiver.try_recv() {
            self.handle_background_message(message);
        }
    }

    fn handle_background_message(&mut self, message: BackgroundMessage) {
        match message {
            BackgroundMessage::ProjectsLoaded(result) => {
                self.lizaalert.busy = false;

                match result {
                    Ok(projects) => {
                        let count = projects.len();
                        self.lizaalert.projects = projects;
                        self.update_status(
                            DiagnosticLevel::Info,
                            format!("Loaded {count} projects"),
                        );
                    }
                    Err(error) => {
                        self.update_status(DiagnosticLevel::Error, error);
                    }
                }
            }
            BackgroundMessage::ProjectLoaded(result) => {
                self.lizaalert.busy = false;

                match result {
                    Ok(project) => {
                        let name = project.summary.name.clone();
                        self.lizaalert.selected_project_slug = Some(project.summary.slug.clone());
                        self.lizaalert.selected_project = Some(project);
                        self.update_status(
                            DiagnosticLevel::Info,
                            format!("Loaded project: {name}"),
                        );
                    }
                    Err(error) => {
                        self.update_status(DiagnosticLevel::Error, error);
                    }
                }
            }
            BackgroundMessage::MapDownloadProgress(progress) => {
                self.update_status(DiagnosticLevel::Info, format_map_download_status(progress));
            }
            BackgroundMessage::MapDownloaded(result) => {
                self.lizaalert.busy = false;

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
                                "Opened map: {} / {} (project layer registration failed: {error:?})",
                                selection.project_name, selection.package_name
                            ),
                        };

                        self.update_status(DiagnosticLevel::Info, status);
                        self.lizaalert.active_map = Some(selection);
                    }
                    Err(error) => {
                        self.update_status(DiagnosticLevel::Error, error);
                    }
                }
            }
        }
    }

    pub fn load_projects(&mut self) {
        if self.lizaalert.busy {
            return;
        }

        self.lizaalert.busy = true;
        self.update_status(DiagnosticLevel::Info, "Loading project list...");
        let sender = self.lizaalert.sender.clone();

        thread::spawn(move || {
            let _ = sender.send(BackgroundMessage::ProjectsLoaded(
                lizaalert::fetch_project_summaries(),
            ));
        });
    }

    pub fn load_project(&mut self, project_slug: &str) {
        if self.lizaalert.busy {
            return;
        }

        let Some(summary) = self
            .lizaalert
            .projects
            .iter()
            .find(|project| project.slug == project_slug)
            .cloned()
        else {
            self.update_status(DiagnosticLevel::Error, "Project not found");
            return;
        };

        self.lizaalert.busy = true;
        let status = if lizaalert::is_project_cached(&summary.slug) {
            format!("Opening cached project {}...", summary.name)
        } else {
            format!("Downloading project {}...", summary.name)
        };
        self.update_status(DiagnosticLevel::Info, status);
        let sender = self.lizaalert.sender.clone();

        thread::spawn(move || {
            let _ = sender.send(BackgroundMessage::ProjectLoaded(lizaalert::open_project(
                summary,
            )));
        });
    }

    pub fn open_selected_map(&mut self, map_name: &str) {
        if self.lizaalert.busy {
            return;
        }

        let Some(project) = self.lizaalert.selected_project.clone() else {
            self.update_status(DiagnosticLevel::Error, "Select a project first");
            return;
        };

        let Some(map) = project
            .maps
            .iter()
            .find(|map| map.name == map_name)
            .cloned()
        else {
            self.update_status(DiagnosticLevel::Error, "Map package not found");
            return;
        };

        let selection = lizaalert::build_active_map_selection(&project, &map);

        if map.local_path.is_some() {
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
                    "Opened cached map: {} / {} (project layer registration failed: {error:?})",
                    selection.project_name, selection.package_name
                ),
            };

            self.lizaalert.active_map = Some(selection);
            self.update_status(DiagnosticLevel::Info, status);
            return;
        }

        self.lizaalert.busy = true;
        self.update_status(
            DiagnosticLevel::Info,
            format!("Downloading {}...", selection.package_name),
        );
        let sender = self.lizaalert.sender.clone();

        thread::spawn(move || {
            let download_result = lizaalert::download_map(selection, |progress| {
                let _ = sender.send(BackgroundMessage::MapDownloadProgress(
                    MapDownloadProgress {
                        downloaded_bytes: progress.downloaded_bytes,
                        total_bytes: progress.total_bytes,
                    },
                ));
            });
            let _ = sender.send(BackgroundMessage::MapDownloaded(download_result));
        });
    }

    pub fn open_local_ozi_map(
        &mut self,
        map_path: impl Into<PathBuf>,
    ) -> Result<(), OpenLocalMapError> {
        let map_path = map_path.into();
        let contents = std::fs::read_to_string(&map_path).map_err(OpenLocalMapError::Read)?;
        let metadata =
            parse_ozi_map_metadata(&map_path, &contents).map_err(OpenLocalMapError::Parse)?;

        match metadata.raster_kind() {
            OziRasterKind::Ozf2 => {}
            kind => return Err(OpenLocalMapError::UnsupportedRasterKind(kind.clone())),
        }

        let file_name = map_path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("local-ozi.map")
            .to_owned();
        let selection = ActiveMapSelection {
            kind: ActiveMapKind::OziRaster,
            project_name: "Local OZI".to_owned(),
            package_name: file_name,
            remote_url: String::new(),
            local_path: map_path,
            center: MapCenter { lat: 0.0, lon: 0.0 },
            base_zoom: 0,
        };

        match self.register_active_map_layer(&selection) {
            Ok(true) => {
                self.lizaalert.active_map = Some(selection.clone());
                self.update_status(
                    DiagnosticLevel::Info,
                    format!("Opened local OZI map: {}", selection.package_name),
                );
                Ok(())
            }
            Ok(false) => {
                self.lizaalert.active_map = Some(selection.clone());
                self.update_status(
                    DiagnosticLevel::Info,
                    format!(
                        "Opened local OZI map: {} (already registered)",
                        selection.package_name
                    ),
                );
                Ok(())
            }
            Err(error) => Err(OpenLocalMapError::Register(error)),
        }
    }

    pub fn active_map(&self) -> Option<&ActiveMapSelection> {
        self.lizaalert.active_map.as_ref()
    }

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

    pub fn window_title(&self) -> String {
        format!("ozi-rs - {}", self.project_name())
    }

    pub fn import_gpx_archive<R>(
        &mut self,
        reader: R,
    ) -> Result<ArchiveImportReport, ArchiveImportError>
    where
        R: std::io::Read + std::io::Seek,
    {
        import::import_gpx_archive_into_project(&mut self.project, &mut self.history, reader)
    }

    pub fn track_layer_count(&self) -> usize {
        self.project.track_layers().len()
    }

    pub fn waypoint_layer_count(&self) -> usize {
        self.project.waypoint_layers().len()
    }

    pub fn report_runtime_error(&mut self, message: impl Into<String>) {
        self.push_diagnostic(DiagnosticLevel::Error, message.into());
    }

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
        self.lizaalert
            .diagnostics
            .push_back(DiagnosticEntry::new(level, message));

        while self.lizaalert.diagnostics.len() > MAX_DIAGNOSTIC_ENTRIES {
            self.lizaalert.diagnostics.pop_front();
        }
    }
}

fn format_map_download_status(progress: MapDownloadProgress) -> String {
    match progress.total_bytes {
        Some(total_bytes) if total_bytes > 0 => {
            let percent = progress.downloaded_bytes as f64 / total_bytes as f64 * 100.0;
            format!(
                "Downloading map... {:.0}% ({}/{})",
                percent,
                format_bytes(progress.downloaded_bytes),
                format_bytes(total_bytes)
            )
        }
        _ => format!(
            "Downloading map... {}",
            format_bytes(progress.downloaded_bytes)
        ),
    }
}

fn format_bytes(bytes: u64) -> String {
    const KIB: u64 = 1024;
    const MIB: u64 = KIB * 1024;

    if bytes >= MIB {
        format!("{:.1} MiB", bytes as f64 / MIB as f64)
    } else if bytes >= KIB {
        format!("{:.1} KiB", bytes as f64 / KIB as f64)
    } else {
        format!("{bytes} B")
    }
}

#[cfg(test)]
mod tests {
    use super::{
        ActiveMapKind, ActiveMapSelection, AppState, BackgroundMessage, DiagnosticLevel, MapCenter,
        MapDownloadProgress, OpenLocalMapError, format_map_download_status,
    };
    use crate::infrastructure::import::OziRasterKind;
    use std::fs;
    use std::path::Path;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn default_window_title_uses_untitled_project() {
        let state = AppState::default();

        assert_eq!(state.window_title(), "ozi-rs - Untitled Project");
    }

    #[test]
    fn registering_active_map_adds_project_map_layer() {
        let mut state = AppState::default();

        let inserted = state
            .register_active_map_layer(&ActiveMapSelection {
                kind: ActiveMapKind::SqliteTiles,
                project_name: "Search Demo".to_owned(),
                package_name: "demo_z16.sqlitedb".to_owned(),
                remote_url: "https://example.com/demo_z16.sqlitedb".to_owned(),
                local_path: PathBuf::from(".tmp/lizaalert-maps/demo/demo_z16.sqlitedb"),
                center: MapCenter {
                    lat: 54.0,
                    lon: 27.0,
                },
                base_zoom: 16,
            })
            .expect("map registration should succeed");

        assert!(inserted);
        assert_eq!(state.map_layer_count(), 1);
        assert_eq!(
            state.project.map_layers()[0].source_path(),
            Some(Path::new(".tmp/lizaalert-maps/demo/demo_z16.sqlitedb"))
        );
    }

    #[test]
    fn registering_same_active_map_twice_does_not_duplicate_layer() {
        let mut state = AppState::default();
        let selection = ActiveMapSelection {
            kind: ActiveMapKind::SqliteTiles,
            project_name: "Search Demo".to_owned(),
            package_name: "demo_z16.sqlitedb".to_owned(),
            remote_url: "https://example.com/demo_z16.sqlitedb".to_owned(),
            local_path: PathBuf::from(".tmp/lizaalert-maps/demo/demo_z16.sqlitedb"),
            center: MapCenter {
                lat: 54.0,
                lon: 27.0,
            },
            base_zoom: 16,
        };

        assert!(
            state
                .register_active_map_layer(&selection)
                .expect("first registration should succeed")
        );
        assert!(
            !state
                .register_active_map_layer(&selection)
                .expect("duplicate registration should be ignored")
        );
        assert_eq!(state.map_layer_count(), 1);
    }

    #[test]
    fn download_progress_status_reports_percentage_when_total_is_known() {
        let status = format_map_download_status(MapDownloadProgress {
            downloaded_bytes: 512,
            total_bytes: Some(1024),
        });

        assert_eq!(status, "Downloading map... 50% (512 B/1.0 KiB)");
    }

    #[test]
    fn progress_message_keeps_background_work_marked_busy() {
        let mut state = AppState::default();
        state.lizaalert.busy = true;

        state.handle_background_message(BackgroundMessage::MapDownloadProgress(
            MapDownloadProgress {
                downloaded_bytes: 2048,
                total_bytes: Some(4096),
            },
        ));

        assert!(state.lizaalert.busy);
        assert_eq!(
            state.lizaalert.status,
            "Downloading map... 50% (2.0 KiB/4.0 KiB)"
        );
    }

    #[test]
    fn missing_project_error_is_recorded_in_diagnostics() {
        let mut state = AppState::default();

        state.load_project("missing-project");

        let latest = state
            .recent_diagnostics()
            .next_back()
            .expect("latest diagnostic entry");
        assert_eq!(latest.level(), DiagnosticLevel::Error);
        assert_eq!(latest.message(), "Project not found");
    }

    #[test]
    fn diagnostics_history_keeps_recent_entries_bounded() {
        let mut state = AppState::default();

        for index in 0..120 {
            state.report_runtime_error(format!("diagnostic {index}"));
        }

        assert_eq!(state.recent_diagnostics().count(), 100);
        let oldest = state
            .recent_diagnostics()
            .next()
            .expect("oldest diagnostic entry");
        let latest = state
            .recent_diagnostics()
            .next_back()
            .expect("latest diagnostic entry");
        assert_eq!(oldest.message(), "diagnostic 20");
        assert_eq!(latest.message(), "diagnostic 119");
    }

    #[test]
    fn open_local_ozi_map_sets_active_ozi_selection() {
        let mut state = AppState::default();
        let map_path = write_temp_ozi_map(sample_ozi_map("bundle/sample.ozf2"));

        state
            .open_local_ozi_map(&map_path)
            .expect("local ozi map should open");

        let active_map = state.active_map().expect("active map");
        assert_eq!(active_map.kind, ActiveMapKind::OziRaster);
        assert_eq!(active_map.local_path, map_path);
        assert_eq!(state.map_layer_count(), 1);
    }

    #[test]
    fn open_local_ozi_map_rejects_unsupported_ozfx3_payloads() {
        let mut state = AppState::default();
        let map_path = write_temp_ozi_map(sample_ozi_map("bundle/sample.ozfx3"));

        let error = state
            .open_local_ozi_map(&map_path)
            .expect_err("ozfx3 should stay unsupported");

        assert!(matches!(
            error,
            OpenLocalMapError::UnsupportedRasterKind(OziRasterKind::Ozfx3)
        ));
    }

    #[test]
    fn open_selected_map_prefers_cached_project_map_without_downloading() {
        let mut state = AppState::default();
        let local_map_path = std::env::temp_dir().join("cached-project-map.sqlitedb");

        state.lizaalert.selected_project = Some(super::LizaProject {
            summary: super::LizaProjectSummary {
                slug: "2026-03-29_demo".to_owned(),
                name: "2026-03-29 demo".to_owned(),
                url: "https://example.test/project/".to_owned(),
            },
            center: MapCenter {
                lat: 54.0,
                lon: 48.0,
            },
            maps: vec![super::LizaMapPackage {
                name: "demo_z16.sqlitedb".to_owned(),
                file_name: "demo_z16.sqlitedb".to_owned(),
                url: String::new(),
                base_zoom: 16,
                local_path: Some(local_map_path.clone()),
            }],
        });

        state.open_selected_map("demo_z16.sqlitedb");

        let active_map = state.active_map().expect("active map");
        assert_eq!(active_map.kind, ActiveMapKind::SqliteTiles);
        assert_eq!(active_map.local_path, local_map_path);
        assert!(!state.lizaalert_busy());
    }

    fn write_temp_ozi_map(contents: String) -> PathBuf {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time")
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "ozi-rs-open-local-ozi-map-{}-{unique}.map",
            std::process::id()
        ));
        fs::write(&path, contents).expect("write temp map");
        path
    }

    fn sample_ozi_map(raster_reference: &str) -> String {
        format!(
            "OziExplorer Map Data File Version 2.2\nForest map\n{raster_reference}\n1 ,Map Code,\nWGS 84,,0.0000,N,0.0000,E,0.000000,0.000000,WGS 84\nReserved 1\nReserved 2\nMagnetic Variation,,,E\nMap Projection,Latitude/Longitude,PolyCal,No,AutoCalOnly,No,BSBUseWPX,No\nProjection Setup,,,,,,,,,,\nPoint01,xy,10,20,in, deg,54,30.000,N,48,24.000,E, grid, , , ,N\n"
        )
    }
}
