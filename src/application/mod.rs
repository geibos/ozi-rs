mod commands;

pub use commands::{CommandError, CommandStack, ProjectCommand};

use crate::domain::{LayerId, Project};
use crate::infrastructure::lizaalert;
use std::path::PathBuf;
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;

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
}

#[derive(Debug, Clone, PartialEq)]
pub struct LizaProject {
    pub summary: LizaProjectSummary,
    pub center: MapCenter,
    pub maps: Vec<LizaMapPackage>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ActiveMapSelection {
    pub project_name: String,
    pub package_name: String,
    pub remote_url: String,
    pub local_path: PathBuf,
    pub center: MapCenter,
    pub base_zoom: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct MapDownloadProgress {
    downloaded_bytes: u64,
    total_bytes: Option<u64>,
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
                        self.lizaalert.status = format!("Loaded {count} projects");
                    }
                    Err(error) => {
                        self.lizaalert.status = error;
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
                        self.lizaalert.status = format!("Loaded project: {name}");
                    }
                    Err(error) => {
                        self.lizaalert.status = error;
                    }
                }
            }
            BackgroundMessage::MapDownloadProgress(progress) => {
                self.lizaalert.status = format_map_download_status(progress);
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

                        self.lizaalert.status = status;
                        self.lizaalert.active_map = Some(selection);
                    }
                    Err(error) => {
                        self.lizaalert.status = error;
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
        self.lizaalert.status = "Loading project list...".to_owned();
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
            self.lizaalert.status = "Project not found".to_owned();
            return;
        };

        self.lizaalert.busy = true;
        self.lizaalert.status = format!("Loading project {}...", summary.name);
        let sender = self.lizaalert.sender.clone();

        thread::spawn(move || {
            let _ = sender.send(BackgroundMessage::ProjectLoaded(lizaalert::fetch_project(
                summary,
            )));
        });
    }

    pub fn open_selected_map(&mut self, map_name: &str) {
        if self.lizaalert.busy {
            return;
        }

        let Some(project) = self.lizaalert.selected_project.clone() else {
            self.lizaalert.status = "Select a project first".to_owned();
            return;
        };

        let Some(map) = project
            .maps
            .iter()
            .find(|map| map.name == map_name)
            .cloned()
        else {
            self.lizaalert.status = "Map package not found".to_owned();
            return;
        };

        let selection = lizaalert::build_active_map_selection(&project, &map);
        self.lizaalert.busy = true;
        self.lizaalert.status = format!("Downloading {}...", selection.package_name);
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

    pub fn lizaalert_busy(&self) -> bool {
        self.lizaalert.busy
    }

    pub fn project_name(&self) -> &str {
        self.project.name()
    }

    pub fn window_title(&self) -> String {
        format!("ozi-rs - {}", self.project_name())
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
        ActiveMapSelection, AppState, BackgroundMessage, MapCenter, MapDownloadProgress,
        format_map_download_status,
    };
    use std::path::Path;
    use std::path::PathBuf;

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
}
