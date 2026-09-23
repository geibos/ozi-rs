mod commands;
pub mod import;

pub use crate::infrastructure::import::PltImportError;
pub use commands::{CommandError, CommandStack, ProjectCommand};
pub use import::{ArchiveImportError, ArchiveImportReport};

use crate::domain::{
    LayerId, Project, ProjectLayerError, TrackId, TrackPointId, TrackSegmentId, Waypoint,
    WaypointId,
};
use crate::infrastructure::import::{
    OziMapParseError, OziRasterKind, parse_ozi_map_metadata, read_ozi_map_text,
};
use crate::infrastructure::lizaalert;
use crate::infrastructure::persistence::{self, PersistedActiveMap, PersistedAppSession};
use std::collections::{HashSet, VecDeque};
use std::fmt;
use std::path::{Path, PathBuf};

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
    /// Download size as the listing states it, when it states one.
    ///
    /// `None` means "not known", which is not the same as zero: a cached map
    /// read off disk and a listing without a size column both land here.
    #[serde(default)]
    pub size_bytes: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct LizaProject {
    pub summary: LizaProjectSummary,
    pub center: MapCenter,
    pub maps: Vec<LizaMapPackage>,
    /// The bundle's top level, as the site lists it.
    ///
    /// This is what the operator chooses from when deciding what not to
    /// download — print sheets and Android tile packs are most of the weight
    /// and this app opens neither.
    #[serde(default)]
    pub contents: Vec<BundleEntry>,
}

/// One entry at the top level of a bundle directory.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct BundleEntry {
    pub name: String,
    pub is_dir: bool,
    /// Size from the listing; folders do not state one.
    pub size_bytes: Option<u64>,
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
    Warning,
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
    session_path: Option<PathBuf>,
    bundles_root: PathBuf,
    lizaalert: LizaAlertState,
    /// Revision of non-undoable project mutations (style/visibility setters
    /// that bypass the CommandStack per ADR-0017). Together with
    /// `CommandStack::mutation_count` this is the complete mutation signal
    /// for dirty tracking.
    style_revision: u64,
    /// `(history.mutation_count(), style_revision)` at the last successful
    /// save / load / restore. `project_dirty()` compares against it.
    saved_mutation_state: (u64, u64),
}

#[derive(Debug)]
struct LizaAlertState {
    projects: Vec<LizaProjectSummary>,
    selected_project_slug: Option<String>,
    selected_project: Option<LizaProject>,
    active_map: Option<ActiveMapSelection>,
    diagnostics: VecDeque<DiagnosticEntry>,
    status: String,
    /// True while the catalogue walk is running (blocks a second walk).
    ///
    /// Separate from `bundle_busy` since 2026-09-22. One flag for both meant
    /// the launch-time walk — up to a thousand pages — disabled the only
    /// download button in the application for minutes, although the two do
    /// unrelated work: the walk reads a remote listing, a download fetches
    /// files into the bundles root.
    listing_busy: bool,
    /// True while a bundle is being downloaded or opened from disk (blocks a
    /// second one).
    bundle_busy: bool,
    /// Package names currently being downloaded (allows parallel map downloads).
    downloading: HashSet<String>,
    /// Files that have finished downloading in the active bundle. Lets the UI
    /// surface partial bundle availability before the whole download finishes.
    ready_bundle_files: Vec<ReadyBundleFile>,
    /// Stops the catalogue walk that is currently running, if one is.
    ///
    /// The walk is up to a thousand pages and holds `busy` for all of it, so on
    /// a field link the only download button in the app can be disabled for
    /// minutes after launch. This is how the operator says they have waited
    /// long enough.
    listing_cancel: Option<lizaalert::CancelToken>,
}

/// What a day's export wrote: the routes and the marks on them.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DayExport {
    pub tracks: usize,
    pub waypoints: usize,
}

/// A file that has been fully downloaded and fsync'd inside an in-progress
/// bundle download.
#[derive(Debug, Clone)]
pub struct ReadyBundleFile {
    #[allow(dead_code)]
    pub package_name: String,
    #[allow(dead_code)]
    pub local_path: PathBuf,
}

const MAX_DIAGNOSTICS: usize = 200;

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}

/// Why `begin_load_project` did not start.
///
/// The frontend needs the difference: "wait, the catalogue is still loading"
/// is a hint, "this project is not in the list" is an error.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LoadProjectRefusal {
    Busy,
    UnknownProject,
}

impl LoadProjectRefusal {
    pub const fn message(self) -> &'static str {
        match self {
            Self::Busy => "busy: another bundle is still being opened",
            Self::UnknownProject => "unknown project",
        }
    }
}

impl AppState {
    pub fn new() -> Self {
        // `Project::default()` already satisfies the `layers` invariant with a
        // "Tracks" and a "Waypoints" layer, both id 1. Adding them again here
        // gave every fresh project two layers of each kind sharing one id: the
        // selector listed "Tracks" twice, the second was unreachable because
        // everything addresses a layer by id, and saving then loading silently
        // renumbered it (`deduplicate_layer_ids`).
        let project = Project::default();
        Self {
            history: CommandStack::default(),
            project,
            project_path: None,
            session_path: None,
            style_revision: 0,
            saved_mutation_state: (0, 0),
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
                listing_busy: false,
                bundle_busy: false,
                downloading: HashSet::new(),
                ready_bundle_files: Vec::new(),
                listing_cancel: None,
            },
        }
    }

    /// Runtime constructor: both locations are resolved by the caller
    /// (Tauri's path resolver in `lib.rs`) and injected, so this module
    /// never derives paths from environment variables itself. A `None`
    /// session path disables session persistence for this run.
    pub fn new_with_paths(session_path: Option<PathBuf>, bundles_root: PathBuf) -> Self {
        let mut state = Self::new();
        state.bundles_root = bundles_root;
        state.session_path = session_path;
        state.restore_session();
        state
    }

    // ── Background-task handoff: "begin" sets busy and returns what the thread needs ──

    /// Returns `None` if already busy; otherwise sets busy and returns the
    /// bundles root path together with the token that stops this walk.
    pub fn begin_load_projects(&mut self) -> Option<(PathBuf, lizaalert::CancelToken)> {
        if self.lizaalert.listing_busy {
            return None;
        }
        self.lizaalert.listing_busy = true;
        let cancel = lizaalert::CancelToken::new();
        self.lizaalert.listing_cancel = Some(cancel.clone());
        self.update_status(DiagnosticLevel::Info, "Loading project list...");
        Some((self.bundles_root.clone(), cancel))
    }

    /// Stop the catalogue walk that is running, if one is.
    ///
    /// Returns whether there was one to stop, so the command can say nothing
    /// happened rather than reporting a stop that did not occur.
    pub fn cancel_project_listing(&mut self) -> bool {
        let Some(cancel) = self.lizaalert.listing_cancel.as_ref() else {
            return false;
        };
        cancel.cancel();
        self.update_status(
            DiagnosticLevel::Info,
            "Stopping the project list refresh...",
        );
        true
    }

    /// Preview lookup: summary + bundles root WITHOUT the busy gate — a
    /// preview is read-only and must work even while the catalog is still
    /// streaming in (otherwise a click during startup silently no-ops).
    pub fn preview_data(&mut self, project_slug: &str) -> Option<(LizaProjectSummary, PathBuf)> {
        let summary = self
            .lizaalert
            .projects
            .iter()
            .find(|p| p.slug == project_slug)
            .cloned()?;
        self.lizaalert.selected_project_slug = Some(summary.slug.clone());
        self.update_status(
            DiagnosticLevel::Info,
            format!("Loading maps: {}", summary.name),
        );
        Some((summary, self.bundles_root.clone()))
    }

    /// Start opening a bundle, or say why it cannot start.
    ///
    /// Both refusals used to be the same `None`, which the command turned into
    /// an empty string and the loader into nothing at all: pressing the only
    /// download button during the catalogue refresh looked like a dead button.
    pub fn begin_load_project(
        &mut self,
        project_slug: &str,
    ) -> Result<(LizaProjectSummary, PathBuf), LoadProjectRefusal> {
        if self.lizaalert.bundle_busy {
            return Err(LoadProjectRefusal::Busy);
        }
        let Some(summary) = self
            .lizaalert
            .projects
            .iter()
            .find(|p| p.slug == project_slug)
            .cloned()
        else {
            return Err(LoadProjectRefusal::UnknownProject);
        };

        self.lizaalert.bundle_busy = true;
        self.lizaalert.ready_bundle_files.clear();
        let status = if lizaalert::is_project_cached(&summary.slug, &self.bundles_root) {
            format!("Opening cached project {}...", summary.name)
        } else {
            format!("Downloading project {}...", summary.name)
        };
        self.update_status(DiagnosticLevel::Info, status);
        Ok((summary, self.bundles_root.clone()))
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
        if self.lizaalert.bundle_busy {
            return None;
        }
        self.lizaalert.bundle_busy = true;
        self.update_status(
            DiagnosticLevel::Info,
            format!("Opening local bundle: {}", dir.display()),
        );
        Some(dir)
    }

    // ── Background-task completion: "apply" receives results and mutates state ──

    pub fn apply_projects_loaded(&mut self, result: Result<lizaalert::CatalogueWalk, String>) {
        self.lizaalert.listing_busy = false;
        self.lizaalert.listing_cancel = None;
        match result {
            Ok(walk) => {
                // A walk that ran to the end is the whole truth about what
                // exists, so it replaces the list rather than merging into it:
                // otherwise a search taken down upstream stayed for good, in
                // the list and then in the cache, and clicking it failed. A
                // stopped walk read only a prefix and may remove nothing.
                if !walk.cancelled {
                    self.lizaalert.projects = walk.projects.clone();
                }
                let count = self.lizaalert.projects.len();
                // A stopped walk is not a failure and not a complete list
                // either. Saying which it was is the difference between "there
                // are 412 searches" and "there are 412 so far".
                let status = if walk.cancelled {
                    format!("Stopped the refresh at {count} projects")
                } else {
                    format!("Loaded {count} projects")
                };
                // The launch-time walk finishes minutes after the operator
                // started a download, and used to write its count over the one
                // line that says how far that download has got. The walk has
                // the diagnostics log; the status bar belongs to the thing the
                // crew is waiting on. External review, 2026-09-22.
                if self.lizaalert.downloading.is_empty() {
                    self.update_status(DiagnosticLevel::Info, status);
                } else {
                    self.push_diagnostic(DiagnosticLevel::Info, status);
                }
            }
            Err(error) => {
                self.update_status(DiagnosticLevel::Error, error);
            }
        }
    }

    /// Seed the catalogue directly. Tests only: the real path is a walk.
    #[cfg(test)]
    pub fn push_project_summary_for_test(&mut self, summary: LizaProjectSummary) {
        self.lizaalert.projects.push(summary);
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

    /// Land a preview's map list, unless the operator has moved on.
    ///
    /// A preview runs in its own thread with nothing ordering two of them, so
    /// the project clicked first can answer last. Applied blindly it swapped
    /// the map list under a row the operator had already left, and its failure
    /// put an error in the status bar about a project nobody was looking at.
    /// The requested slug decides: only the newest preview may land.
    ///
    /// It also does not touch `busy`. A preview never takes that flag — that
    /// is deliberate, so a click during the catalogue walk is not swallowed —
    /// and releasing a flag it never took let a second download start while
    /// the first was still going.
    pub fn apply_preview_loaded(&mut self, slug: &str, result: Result<LizaProject, String>) {
        if self.lizaalert.selected_project_slug.as_deref() != Some(slug) {
            return;
        }
        match result {
            Ok(project) => {
                let name = project.summary.name.clone();
                self.lizaalert.selected_project = Some(project);
                self.update_status(DiagnosticLevel::Info, format!("Loaded project: {name}"));
            }
            Err(error) => {
                self.update_status(DiagnosticLevel::Error, error);
            }
        }
    }

    pub fn apply_project_loaded(&mut self, result: Result<LizaProject, String>) {
        self.lizaalert.bundle_busy = false;
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
                if let Some(project) = self.lizaalert.selected_project.as_mut()
                    && let Some(map) = project
                        .maps
                        .iter_mut()
                        .find(|map| map.name == selection.package_name)
                {
                    map.local_path = Some(selection.local_path.clone());
                }
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
                self.persist_session_snapshot();
            }
            Err(error) => {
                self.update_status(DiagnosticLevel::Error, error);
            }
        }
    }

    pub fn apply_progress(&mut self, message: String) {
        self.update_status(DiagnosticLevel::Info, message);
    }

    /// Record that a bundle file finished downloading so partial-bundle UIs
    /// can surface it before the rest of the bundle catches up.
    ///
    /// Currently mirrors the file into `ready_bundle_files` (used by status
    /// reports) and updates the per-map `local_path` if the file corresponds
    /// to a known map package. Importantly this never errors when the file
    /// is not (yet) a recognised map: a `.pdf` reference file should still
    /// register as ready without disrupting state.
    pub fn note_bundle_file_ready(&mut self, package_name: &str, local_path: &Path) {
        self.lizaalert.ready_bundle_files.push(ReadyBundleFile {
            package_name: package_name.to_owned(),
            local_path: local_path.to_path_buf(),
        });
        if let Some(project) = self.lizaalert.selected_project.as_mut() {
            // `package_name` is a path relative to the bundle root, a package's
            // `file_name` is the bare name, so the two meet at the last
            // component. Comparing with `ends_with` on the whole string instead
            // made `bigmap.ozf2` a match for `map.ozf2`, and the crew opened a
            // layer they had not downloaded. External review, 2026-09-22.
            let landed_name = package_name.rsplit('/').next().unwrap_or(package_name);
            for map in project.maps.iter_mut() {
                if landed_name == map.file_name {
                    map.local_path = Some(local_path.to_path_buf());
                }
            }
        }
        self.update_status(DiagnosticLevel::Info, format!("Ready: {package_name}"));
    }

    /// Snapshot of currently-known ready files, e.g. for diagnostics or tests.
    #[allow(dead_code)]
    pub fn ready_bundle_files(&self) -> &[ReadyBundleFile] {
        &self.lizaalert.ready_bundle_files
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
        self.persist_session_snapshot();
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

        // OZF2 is the optimised form; a `.map` beside an ordinary picture is
        // OziExplorer's own pairing and the commoner thing a headquarters is
        // handed — a scan of a sheet, a screenshot, a photograph of a paper map
        // on a table. Refusing those meant refusing a file OziExplorer opens
        // without comment, which is most of what "I cannot open my map here"
        // meant. `.ozfx3` stays refused: it is encrypted and nothing here reads
        // it.
        match metadata.raster_kind() {
            OziRasterKind::Ozf2 | OziRasterKind::DirectImage(_) => {}
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
                self.persist_session_snapshot();
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

    pub fn lizaalert_status(&self) -> &str {
        &self.lizaalert.status
    }

    pub fn recent_diagnostics(&self) -> impl DoubleEndedIterator<Item = &DiagnosticEntry> {
        self.lizaalert.diagnostics.iter()
    }

    /// Whether the catalogue walk is running.
    pub fn lizaalert_listing_busy(&self) -> bool {
        self.lizaalert.listing_busy
    }

    /// Whether a bundle is being downloaded or opened from disk.
    pub fn lizaalert_bundle_busy(&self) -> bool {
        self.lizaalert.bundle_busy
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

    /// Point the app at another bundles directory, and remember it.
    ///
    /// Without the write-through the choice lasted until the next launch,
    /// which silently sent downloads back to the default folder and made
    /// every already-fetched bundle look missing.
    pub fn set_bundles_root(&mut self, path: PathBuf) {
        self.bundles_root = path;
        self.persist_session_snapshot();
    }

    #[cfg(test)]
    /// Mutable project access, for building fixtures and tests.
    ///
    /// Production paths go through the command stack; this bypasses it on
    /// purpose, which is why it is crate-private.
    pub(crate) fn project_mut(&mut self) -> &mut Project {
        &mut self.project
    }

    /// Read back a catalogue installed by `set_fixture_catalogue`.
    ///
    /// The catalogue left the state snapshot, so the fixture writer can no
    /// longer read it out of a DTO and needs this instead.
    #[cfg(test)]
    pub(crate) fn fixture_catalogue(&self) -> &[LizaProjectSummary] {
        &self.lizaalert.projects
    }

    /// Install a catalogue state without a network, for fixtures and tests.
    ///
    /// The LizaAlert state is otherwise only reachable through the download
    /// paths, and a fixture that had to run those would need a server.
    #[cfg(test)]
    pub(crate) fn set_fixture_catalogue(
        &mut self,
        projects: Vec<LizaProjectSummary>,
        selected_project: Option<LizaProject>,
        active_map: Option<ActiveMapSelection>,
        status: impl Into<String>,
    ) {
        self.lizaalert.selected_project_slug =
            selected_project.as_ref().map(|p| p.summary.slug.clone());
        self.lizaalert.projects = projects;
        self.lizaalert.selected_project = selected_project;
        self.lizaalert.active_map = active_map;
        self.lizaalert.status = status.into();
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

    pub fn project_waypoint_layers(&self) -> &[crate::domain::WaypointLayer] {
        self.project.waypoint_layers()
    }

    // ── Mutations ──

    /// True when the project has mutations that are not persisted to disk.
    /// Structural inputs: `CommandStack::mutation_count` (all undoable edits,
    /// undo, redo) plus `style_revision` (non-undoable setters). Cleared by
    /// `mark_project_saved` on save / load / session restore.
    pub fn project_dirty(&self) -> bool {
        (self.history.mutation_count(), self.style_revision) != self.saved_mutation_state
    }

    fn mark_project_saved(&mut self) {
        self.saved_mutation_state = (self.history.mutation_count(), self.style_revision);
    }

    fn mark_style_mutation(&mut self) {
        self.style_revision += 1;
    }

    pub fn save_project_to(&mut self, path: PathBuf) -> Result<(), persistence::PersistenceError> {
        match persistence::save_project(&self.project, &path) {
            Ok(()) => {
                let display = path.display().to_string();
                self.project_path = Some(path);
                self.mark_project_saved();
                self.update_status(DiagnosticLevel::Info, format!("Saved: {display}"));
                self.persist_session_snapshot();
                Ok(())
            }
            Err(error) => {
                self.update_status(DiagnosticLevel::Error, format!("Save failed: {error}"));
                Err(error)
            }
        }
    }

    /// Open a project file.
    ///
    /// Returns the failure to the caller. It used to swallow it into the
    /// diagnostics and return `()`, so the command answered `Ok(())` for a
    /// file it had not opened: the interface then remembered the path in the
    /// recents and framed the map on a project that was never loaded. Saving
    /// already returned its errors; this is the other half of that pair.
    /// External review, 2026-09-22.
    /// Empty the project: the next search starts here.
    ///
    /// A project is one search. Nothing created an empty one until now, so a
    /// crew that finished an operation and began the next kept adding to the
    /// same document — yesterday's routes under today's, two operations mixed
    /// in one Tracks tab, and no way out short of quitting and deleting the
    /// session file.
    ///
    /// The history is cleared rather than carried over. Undo that could walk
    /// back into a finished search and put its tracks on the map again is not
    /// an undo anybody wants.
    ///
    /// The bundle and the active raster stay. The map is the ground; the
    /// project is the work on it, and a second search in the same district
    /// should not blank the screen.
    pub fn new_project(&mut self) {
        self.project = Project::untitled();
        self.project_path = None;
        self.history = CommandStack::default();
        self.style_revision = 0;
        self.mark_project_saved();
        self.update_status(DiagnosticLevel::Info, "New project");
        self.persist_session_snapshot();
    }

    pub fn load_project_from(&mut self, path: PathBuf) -> Result<(), String> {
        match persistence::load_project(&path) {
            Ok(project) => {
                let display = path.display().to_string();
                self.project = project;
                self.project_path = Some(path);
                self.history = CommandStack::default();
                self.mark_project_saved();
                self.lizaalert.active_map = None;
                self.update_status(DiagnosticLevel::Info, format!("Opened: {display}"));
                self.persist_session_snapshot();
                Ok(())
            }
            Err(error) => {
                let message = format!("Open failed: {error}");
                self.update_status(DiagnosticLevel::Error, message.clone());
                Err(message)
            }
        }
    }

    /// CJ-3: recursive folder import (10-Tracks convention).
    pub fn import_tracks_directory(
        &mut self,
        dir: std::path::PathBuf,
    ) -> Result<import::DirectoryImportReport, String> {
        import::import_tracks_directory_into_project(&mut self.project, &mut self.history, &dir)
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

    /// Import an OziExplorer waypoint file. See
    /// `import::import_wpt_file_into_project` for why it lands in its own
    /// layer.
    pub fn import_wpt_file(
        &mut self,
        path: std::path::PathBuf,
    ) -> Result<ArchiveImportReport, crate::infrastructure::import::WptImportError> {
        let report =
            import::import_wpt_file_into_project(&mut self.project, &mut self.history, &path)?;
        // A file from another headquarters may be in another datum. We do not
        // transform between them — that is a non-goal — but taking the
        // coordinates without a word puts the marks 100–150 m from where they
        // were meant, and nobody finds out until the crew is standing there.
        if let Some(warning) = report.datum_warning() {
            self.push_diagnostic(DiagnosticLevel::Warning, warning.to_owned());
        }
        Ok(report)
    }

    /// Set the note beside a mark, undoably.
    ///
    /// A mark called «улика» is the place; the note is what a crew is sent to,
    /// and it travels through GPX `<desc>` and WPT field 11 to the штаб next
    /// door.
    pub fn apply_set_waypoint_description(
        &mut self,
        layer_id: LayerId,
        waypoint_id: WaypointId,
        new_description: Option<String>,
    ) -> Result<(), ProjectLayerError> {
        let old_description = self
            .project
            .waypoint_layers()
            .iter()
            .find(|layer| layer.id() == layer_id)
            .ok_or(ProjectLayerError::WaypointLayerUnavailable(layer_id))?
            .waypoints()
            .iter()
            .find(|waypoint| waypoint.id() == waypoint_id)
            .ok_or(ProjectLayerError::WaypointNotFound(layer_id, waypoint_id))?
            .description()
            .map(str::to_owned);

        let cmd = commands::ProjectCommand::SetWaypointDescription {
            layer_id,
            waypoint_id,
            old_description,
            new_description,
        };
        self.history
            .apply(&mut self.project, &cmd)
            .map_err(|commands::CommandError::ProjectLayer(e)| e)
    }

    /// Replace the files that belong to a mark, as one undoable step.
    ///
    /// Paths, not bytes — see `Waypoint::attachments`. The list is replaced
    /// whole because the undo delta needs the previous one either way, and a
    /// pair of add/remove commands can get out of step with itself.
    pub fn apply_set_waypoint_attachments(
        &mut self,
        layer_id: LayerId,
        waypoint_id: WaypointId,
        new_attachments: Vec<String>,
    ) -> Result<(), ProjectLayerError> {
        let old_attachments = self
            .project
            .waypoint_layers()
            .iter()
            .find(|layer| layer.id() == layer_id)
            .ok_or(ProjectLayerError::WaypointLayerUnavailable(layer_id))?
            .waypoints()
            .iter()
            .find(|waypoint| waypoint.id() == waypoint_id)
            .ok_or(ProjectLayerError::WaypointNotFound(layer_id, waypoint_id))?
            .attachments()
            .to_vec();

        let cmd = commands::ProjectCommand::SetWaypointAttachments {
            layer_id,
            waypoint_id,
            old_attachments,
            new_attachments,
        };
        self.history
            .apply(&mut self.project, &cmd)
            .map_err(|commands::CommandError::ProjectLayer(e)| e)
    }

    pub fn apply_set_waypoint_color(
        &mut self,
        layer_id: LayerId,
        waypoint_id: WaypointId,
        new_color: Option<[u8; 4]>,
    ) -> Result<(), ProjectLayerError> {
        let old_color = self
            .project
            .waypoint_layers()
            .iter()
            .find(|layer| layer.id() == layer_id)
            .ok_or(ProjectLayerError::WaypointLayerUnavailable(layer_id))?
            .waypoints()
            .iter()
            .find(|waypoint| waypoint.id() == waypoint_id)
            .ok_or(ProjectLayerError::WaypointNotFound(layer_id, waypoint_id))?
            .color();

        let cmd = commands::ProjectCommand::set_waypoint_color(
            layer_id,
            waypoint_id,
            old_color,
            new_color,
        );
        self.history
            .apply(&mut self.project, &cmd)
            .map_err(|e| match e {
                commands::CommandError::ProjectLayer(pe) => pe,
            })
    }

    pub fn apply_set_waypoint_symbol(
        &mut self,
        layer_id: LayerId,
        waypoint_id: WaypointId,
        new_symbol: Option<String>,
    ) -> Result<(), ProjectLayerError> {
        let old_symbol = self
            .project
            .waypoint_layers()
            .iter()
            .find(|layer| layer.id() == layer_id)
            .ok_or(ProjectLayerError::WaypointLayerUnavailable(layer_id))?
            .waypoints()
            .iter()
            .find(|waypoint| waypoint.id() == waypoint_id)
            .ok_or(ProjectLayerError::WaypointNotFound(layer_id, waypoint_id))?
            .symbol()
            .map(str::to_owned);

        let cmd = commands::ProjectCommand::set_waypoint_symbol(
            layer_id,
            waypoint_id,
            old_symbol,
            new_symbol,
        );
        self.history
            .apply(&mut self.project, &cmd)
            .map_err(|e| match e {
                commands::CommandError::ProjectLayer(pe) => pe,
            })
    }

    pub fn set_track_color(&mut self, layer_id: LayerId, track_id: TrackId, color: [u8; 4]) {
        if let Ok(track) = self.project.track_mut(layer_id.value(), track_id.value()) {
            track.style_mut().color = color;
            self.mark_style_mutation();
        }
    }

    /// Show or hide every track in the project in one step.
    ///
    /// Visibility is a style mutation and deliberately bypasses the command
    /// stack (ADR-0017), so a bulk change marks the project dirty without
    /// filling undo with twenty-six entries.
    pub fn set_all_tracks_visible(&mut self, visible: bool) {
        self.project.set_all_tracks_visible(visible);
        self.mark_style_mutation();
    }

    /// Leave one track visible and hide the rest. No-op for a missing track.
    pub fn show_only_track(&mut self, layer_id: LayerId, track_id: TrackId) -> bool {
        let changed = self.project.show_only_track(layer_id, track_id);
        if changed {
            self.mark_style_mutation();
        }
        changed
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
        self.mark_style_mutation();
    }

    /// Flip a waypoint's visibility flag. Non-undoable — mirrors
    /// `toggle_track_visible` and never enters the undo stack.
    pub fn toggle_waypoint_visible(
        &mut self,
        layer_id: LayerId,
        waypoint_id: crate::domain::WaypointId,
    ) {
        if self
            .project
            .toggle_waypoint_visible_in_layer(layer_id, waypoint_id)
            .is_some()
        {
            self.mark_style_mutation();
        }
    }

    /// Show or hide every waypoint in the project in one step. Non-undoable,
    /// like every other visibility change (ADR-0017).
    pub fn set_all_waypoints_visible(&mut self, visible: bool) {
        self.project.set_all_waypoints_visible(visible);
        self.mark_style_mutation();
    }

    /// Leave one waypoint visible and hide the rest. No-op for a missing one.
    pub fn show_only_waypoint(
        &mut self,
        layer_id: LayerId,
        waypoint_id: crate::domain::WaypointId,
    ) -> bool {
        let changed = self.project.show_only_waypoint(layer_id, waypoint_id);
        if changed {
            self.mark_style_mutation();
        }
        changed
    }

    /// Throw away the commands that built a drawing in progress.
    ///
    /// Pressing Esc while drawing used to call undo once per command, which
    /// left the abandoned track in the redo stack — a later redo brought it
    /// back — and kept the project marked as changed. Discarding reverses the
    /// same commands without recording them.
    /// Abandon a drawing in progress, naming the track it created.
    ///
    /// Counting commands is not enough: the undo stack is bounded at
    /// `MAX_STACK_DEPTH` and drops its oldest entries, so a drawing longer
    /// than the stack has already lost the command that created the track.
    /// Discarding "the last N" then reversed only the surviving inserts and
    /// left an empty track behind — rubbish a crew could not remove with
    /// undo, because undo no longer knew about it. External review,
    /// 2026-09-22.
    ///
    /// The sweep is deliberate rather than another command: the drawing never
    /// happened, so there is nothing to put in the redo stack.
    pub fn cancel_drawing_of(
        &mut self,
        layer_id: LayerId,
        track_id: TrackId,
        command_count: usize,
    ) -> usize {
        let discarded = self.history.discard_last(command_count, &mut self.project);
        if let Ok(layer) = self.project.track_layer_mut(layer_id.value()) {
            let _ = layer.remove_track(track_id);
        }
        discarded
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

    // ── Layer management ─────────────────────────────────────────────────

    /// Create an empty track layer and answer with its identifier.
    ///
    /// The caller needs the id to make the layer active, and reading it back
    /// out of the state means matching on a name the operator chose — two
    /// layers may share one.
    pub fn create_track_layer(&mut self, name: String) -> Result<u64, String> {
        let id = import::next_layer_id(&self.project);
        self.history
            .apply(
                &mut self.project,
                &commands::ProjectCommand::add_track_layer(id, name),
            )
            .map_err(|commands::CommandError::ProjectLayer(e)| format!("{e}"))?;
        Ok(id.value())
    }

    pub fn create_waypoint_layer(&mut self, name: String) -> Result<u64, String> {
        let id = import::next_layer_id(&self.project);
        self.history
            .apply(
                &mut self.project,
                &commands::ProjectCommand::add_waypoint_layer(id, name),
            )
            .map_err(|commands::CommandError::ProjectLayer(e)| format!("{e}"))?;
        Ok(id.value())
    }

    pub fn rename_track_layer(
        &mut self,
        layer_id: LayerId,
        new_name: String,
    ) -> Result<(), String> {
        let old_name = self
            .project
            .track_layers()
            .iter()
            .find(|l| l.id() == layer_id)
            .map(|l| l.name().to_owned())
            .ok_or_else(|| format!("track layer {} not found", layer_id.value()))?;
        self.history
            .apply(
                &mut self.project,
                &commands::ProjectCommand::RenameTrackLayer {
                    layer_id,
                    old_name,
                    new_name,
                },
            )
            .map_err(|commands::CommandError::ProjectLayer(e)| format!("{e}"))
    }

    pub fn rename_waypoint_layer(
        &mut self,
        layer_id: LayerId,
        new_name: String,
    ) -> Result<(), String> {
        let old_name = self
            .project
            .waypoint_layers()
            .iter()
            .find(|l| l.id() == layer_id)
            .map(|l| l.name().to_owned())
            .ok_or_else(|| format!("waypoint layer {} not found", layer_id.value()))?;
        self.history
            .apply(
                &mut self.project,
                &commands::ProjectCommand::RenameWaypointLayer {
                    layer_id,
                    old_name,
                    new_name,
                },
            )
            .map_err(|commands::CommandError::ProjectLayer(e)| format!("{e}"))
    }

    /// Remove a track layer with everything in it.
    ///
    /// The command carries the whole layer, which is what makes the removal
    /// undoable: `RemoveTrackLayer` reverses to `RestoreTrackLayer`.
    pub fn delete_track_layer(&mut self, layer_id: LayerId) -> Result<(), String> {
        let layer = self
            .project
            .track_layers()
            .iter()
            .find(|l| l.id() == layer_id)
            .cloned()
            .ok_or_else(|| format!("track layer {} not found", layer_id.value()))?;
        self.history
            .apply(
                &mut self.project,
                &commands::ProjectCommand::RemoveTrackLayer { layer },
            )
            .map_err(|commands::CommandError::ProjectLayer(e)| format!("{e}"))
    }

    pub fn delete_waypoint_layer(&mut self, layer_id: LayerId) -> Result<(), String> {
        let layer = self
            .project
            .waypoint_layers()
            .iter()
            .find(|l| l.id() == layer_id)
            .cloned()
            .ok_or_else(|| format!("waypoint layer {} not found", layer_id.value()))?;
        self.history
            .apply(
                &mut self.project,
                &commands::ProjectCommand::RemoveWaypointLayer { layer },
            )
            .map_err(|commands::CommandError::ProjectLayer(e)| format!("{e}"))
    }

    /// Move a track point.
    ///
    /// `None` for the gesture: the frontend sends one command per completed
    /// drag, so each drop is its own undo step. Passing a gesture id is how a
    /// continuous drag — a command per pointer move — would collapse into one.
    pub fn apply_move_track_point(
        &mut self,
        layer_id: LayerId,
        track_id: TrackId,
        segment_id: TrackSegmentId,
        point_id: TrackPointId,
        lat: f64,
        lon: f64,
    ) -> Result<(), ProjectLayerError> {
        let (old_lat, old_lon) = self
            .project
            .track_layers()
            .iter()
            .find(|l| l.id() == layer_id)
            .and_then(|l| l.tracks().iter().find(|t| t.id() == track_id))
            .and_then(|t| t.segments().iter().find(|s| s.id() == segment_id))
            .and_then(|s| s.points().iter().find(|p| p.id() == point_id))
            .map(|p| (p.latitude(), p.longitude()))
            .unwrap_or((lat, lon));

        let cmd = commands::ProjectCommand::move_track_point(
            layer_id, track_id, segment_id, point_id, lat, lon, old_lat, old_lon,
        );
        self.history
            .apply_or_merge(cmd, None, &mut self.project)
            .map_err(|e| match e {
                commands::CommandError::ProjectLayer(pe) => pe,
            })
    }

    /// Delete a track point.
    pub fn apply_delete_track_point(
        &mut self,
        layer_id: LayerId,
        track_id: TrackId,
        segment_id: TrackSegmentId,
        point_id: TrackPointId,
    ) -> Result<(), ProjectLayerError> {
        let cmd =
            commands::ProjectCommand::delete_track_point(layer_id, track_id, segment_id, point_id);
        self.history
            .apply(&mut self.project, &cmd)
            .map_err(|e| match e {
                commands::CommandError::ProjectLayer(pe) => pe,
            })
    }

    /// Insert a track point at a given index. Generates a new point ID.
    pub fn apply_insert_track_point(
        &mut self,
        layer_id: LayerId,
        track_id: TrackId,
        segment_id: TrackSegmentId,
        index: usize,
        lat: f64,
        lon: f64,
    ) -> Result<(), ProjectLayerError> {
        use crate::domain::TrackPoint;

        let new_id = {
            let max_id = self
                .project
                .track_layers()
                .iter()
                .find(|l| l.id() == layer_id)
                .and_then(|l| l.tracks().iter().find(|t| t.id() == track_id))
                .map(|t| {
                    t.segments()
                        .iter()
                        .flat_map(|s| s.points().iter().map(|p| p.id().value()))
                        .max()
                        .unwrap_or(0)
                })
                .unwrap_or(0);
            TrackPointId::new(max_id + 1)
        };

        let point = TrackPoint::new(new_id, lat, lon);
        let cmd = commands::ProjectCommand::insert_track_point(
            layer_id, track_id, segment_id, index, point,
        );
        self.history
            .apply(&mut self.project, &cmd)
            .map_err(|e| match e {
                commands::CommandError::ProjectLayer(pe) => pe,
            })
    }

    /// Split a track segment at a given point. Pre-generates new segment ID.
    pub fn apply_split_segment(
        &mut self,
        layer_id: LayerId,
        track_id: TrackId,
        segment_id: TrackSegmentId,
        point_id: TrackPointId,
    ) -> Result<(), ProjectLayerError> {
        let new_segment_id = {
            let max_id = self
                .project
                .track_layers()
                .iter()
                .find(|l| l.id() == layer_id)
                .and_then(|l| l.tracks().iter().find(|t| t.id() == track_id))
                .map(|t| {
                    t.segments()
                        .iter()
                        .map(|s| s.id().value())
                        .max()
                        .unwrap_or(0)
                })
                .unwrap_or(0);
            TrackSegmentId::new(max_id + 1)
        };

        let cmd = commands::ProjectCommand::split_segment(
            layer_id,
            track_id,
            segment_id,
            point_id,
            new_segment_id,
        );
        self.history
            .apply(&mut self.project, &cmd)
            .map_err(|e| match e {
                commands::CommandError::ProjectLayer(pe) => pe,
            })
    }

    /// Join two track segments.
    pub fn apply_join_segments(
        &mut self,
        layer_id: LayerId,
        track_id: TrackId,
        segment_id_a: TrackSegmentId,
        segment_id_b: TrackSegmentId,
    ) -> Result<(), ProjectLayerError> {
        let cmd =
            commands::ProjectCommand::join_segments(layer_id, track_id, segment_id_a, segment_id_b);
        self.history
            .apply(&mut self.project, &cmd)
            .map_err(|e| match e {
                commands::CommandError::ProjectLayer(pe) => pe,
            })
    }

    /// Delete a track from a layer.
    pub fn apply_delete_track(
        &mut self,
        layer_id: LayerId,
        track_id: TrackId,
    ) -> Result<(), ProjectLayerError> {
        let cmd = commands::ProjectCommand::delete_track(layer_id, track_id);
        self.history
            .apply(&mut self.project, &cmd)
            .map_err(|e| match e {
                commands::CommandError::ProjectLayer(pe) => pe,
            })
    }

    /// Create a new empty track in a track layer and return its id.
    pub fn apply_create_empty_track(
        &mut self,
        layer_id: LayerId,
        name: String,
    ) -> Result<TrackId, ProjectLayerError> {
        let new_id = {
            let max = self
                .project
                .track_layers()
                .iter()
                .find(|l| l.id() == layer_id)
                .map(|l| l.tracks().iter().map(|t| t.id().value()).max().unwrap_or(0))
                .unwrap_or(0);
            TrackId::new(max + 1)
        };
        let cmd = commands::ProjectCommand::create_empty_track(layer_id, new_id, name);
        self.history
            .apply(&mut self.project, &cmd)
            .map_err(|e| match e {
                commands::CommandError::ProjectLayer(pe) => pe,
            })?;
        Ok(new_id)
    }

    pub fn apply_delete_waypoint(
        &mut self,
        layer_id: LayerId,
        waypoint_id: WaypointId,
    ) -> Result<(), ProjectLayerError> {
        let cmd = commands::ProjectCommand::delete_waypoint(layer_id, waypoint_id);
        self.history
            .apply(&mut self.project, &cmd)
            .map_err(|e| match e {
                commands::CommandError::ProjectLayer(pe) => pe,
            })
    }

    pub fn apply_add_waypoint(
        &mut self,
        layer_id: LayerId,
        lat: f64,
        lon: f64,
        name: String,
    ) -> Result<(), ProjectLayerError> {
        let new_id = {
            let max_id = self
                .project
                .waypoint_layers()
                .iter()
                .find(|l| l.id() == layer_id)
                .map(|l| {
                    l.waypoints()
                        .iter()
                        .map(|w| w.id().value())
                        .max()
                        .unwrap_or(0)
                })
                .unwrap_or(0);
            WaypointId::new(max_id + 1)
        };
        let waypoint = Waypoint::new(new_id, name, lat, lon);
        let cmd = commands::ProjectCommand::add_waypoint(layer_id, waypoint);
        self.history
            .apply(&mut self.project, &cmd)
            .map_err(|e| match e {
                commands::CommandError::ProjectLayer(pe) => pe,
            })
    }

    pub fn apply_rename_waypoint(
        &mut self,
        layer_id: LayerId,
        waypoint_id: WaypointId,
        new_name: String,
    ) -> Result<(), ProjectLayerError> {
        let old_name = self
            .project
            .waypoint_layers()
            .iter()
            .find(|l| l.id() == layer_id)
            .and_then(|l| {
                l.waypoints()
                    .iter()
                    .find(|w| w.id() == waypoint_id)
                    .map(|w| w.name().to_owned())
            })
            .unwrap_or_default();

        let cmd =
            commands::ProjectCommand::rename_waypoint(layer_id, waypoint_id, old_name, new_name);
        self.history
            .apply(&mut self.project, &cmd)
            .map_err(|e| match e {
                commands::CommandError::ProjectLayer(pe) => pe,
            })
    }

    pub fn apply_move_waypoint(
        &mut self,
        layer_id: LayerId,
        waypoint_id: WaypointId,
        lat: f64,
        lon: f64,
    ) -> Result<(), ProjectLayerError> {
        let cmd = commands::ProjectCommand::move_waypoint(layer_id, waypoint_id, lat, lon);
        self.history
            .apply(&mut self.project, &cmd)
            .map_err(|e| match e {
                commands::CommandError::ProjectLayer(pe) => pe,
            })
    }

    /// CJ-4 sort: reorder every segment's points by timestamp (untimed
    /// first, stable). No-op — and no undo entry — when already sorted.
    pub fn apply_sort_track_points(
        &mut self,
        layer_id: LayerId,
        track_id: TrackId,
    ) -> Result<(), ProjectLayerError> {
        let (current, sorted) = {
            let track = self
                .project
                .track_layers()
                .iter()
                .find(|l| l.id() == layer_id)
                .and_then(|l| l.tracks().iter().find(|t| t.id() == track_id))
                .ok_or(ProjectLayerError::MissingTrack {
                    layer_id: layer_id.value(),
                    track_id: track_id.value(),
                })?;
            let current: Vec<(u64, Vec<u64>)> = track
                .segments()
                .iter()
                .map(|s| {
                    (
                        s.id().value(),
                        s.points().iter().map(|p| p.id().value()).collect(),
                    )
                })
                .collect();
            let sorted = crate::domain::sorted_point_order_by_time(track.segments());
            (current, sorted)
        };
        if current == sorted {
            return Ok(());
        }
        self.history
            .apply(
                &mut self.project,
                &commands::ProjectCommand::reorder_track_points(layer_id, track_id, sorted),
            )
            .map_err(|e| match e {
                commands::CommandError::ProjectLayer(pe) => pe,
            })
    }

    /// CJ-4 crop: keep only points inside the bbox. No-op when nothing falls
    /// outside; error (from the command) when everything would be removed.
    pub fn apply_crop_track_to_extent(
        &mut self,
        layer_id: LayerId,
        track_id: TrackId,
        min_lat: f64,
        min_lon: f64,
        max_lat: f64,
        max_lon: f64,
    ) -> Result<usize, ProjectLayerError> {
        self.apply_crop_with(layer_id, track_id, |p| {
            p.latitude() < min_lat
                || p.latitude() > max_lat
                || p.longitude() < min_lon
                || p.longitude() > max_lon
        })
    }

    /// CJ-4 crop by time range (either bound optional). Untimed points are
    /// always KEPT — cropping must not silently destroy data that carries no
    /// timestamp to judge by.
    pub fn apply_crop_track_to_time(
        &mut self,
        layer_id: LayerId,
        track_id: TrackId,
        from: Option<chrono::DateTime<chrono::Utc>>,
        to: Option<chrono::DateTime<chrono::Utc>>,
    ) -> Result<usize, ProjectLayerError> {
        self.apply_crop_with(layer_id, track_id, |p| match p.timestamp() {
            None => false,
            Some(ts) => from.is_some_and(|f| ts < f) || to.is_some_and(|t| ts > t),
        })
    }

    /// Trim a track at one of its points, keeping that point.
    ///
    /// The commonest edit to a recording: the first twenty minutes are the
    /// drive to the start, so cut everything before where the walking begins.
    /// Crop by time can do it if the crew knows the time; this is the gesture
    /// they have — they can see the point on the map and in the table.
    ///
    /// `before` trims what came earlier, otherwise what came later. Either way
    /// the named point survives: it is where the walk starts or ends, and
    /// removing it would be off by one in the direction nobody checks.
    ///
    /// Returns how many points were removed; zero records no undo step,
    /// because trimming at the first or last point is not an edit.
    pub fn apply_trim_track_at_point(
        &mut self,
        layer_id: LayerId,
        track_id: TrackId,
        point_id: TrackPointId,
        before: bool,
    ) -> Result<usize, ProjectLayerError> {
        // Positional, so the predicate cannot judge a point on its own: walk
        // the track in order and flip once the named point is reached.
        let mut seen = false;
        self.apply_crop_with(layer_id, track_id, |p| {
            if p.id() == point_id {
                seen = true;
                return false;
            }
            if before { !seen } else { seen }
        })
    }

    /// Shared crop plumbing: collect points matching `remove`, apply one
    /// undoable CropTrackPoints. Returns how many points were removed.
    fn apply_crop_with(
        &mut self,
        layer_id: LayerId,
        track_id: TrackId,
        mut remove: impl FnMut(&crate::domain::TrackPoint) -> bool,
    ) -> Result<usize, ProjectLayerError> {
        let doomed: Vec<(TrackSegmentId, Vec<TrackPointId>)> = {
            let track = self
                .project
                .track_layers()
                .iter()
                .find(|l| l.id() == layer_id)
                .and_then(|l| l.tracks().iter().find(|t| t.id() == track_id))
                .ok_or(ProjectLayerError::MissingTrack {
                    layer_id: layer_id.value(),
                    track_id: track_id.value(),
                })?;
            track
                .segments()
                .iter()
                .map(|segment| {
                    (
                        segment.id(),
                        segment
                            .points()
                            .iter()
                            .filter(|p| remove(p))
                            .map(|p| p.id())
                            .collect::<Vec<_>>(),
                    )
                })
                .filter(|(_, ids)| !ids.is_empty())
                .collect()
        };
        let removed: usize = doomed.iter().map(|(_, ids)| ids.len()).sum();
        if removed == 0 {
            return Ok(0);
        }
        self.history
            .apply(
                &mut self.project,
                &commands::ProjectCommand::crop_track_points(layer_id, track_id, doomed),
            )
            .map_err(|e| match e {
                commands::CommandError::ProjectLayer(pe) => pe,
            })?;
        Ok(removed)
    }

    /// Simplify a track to the tolerance the operator set, in **metres**.
    ///
    /// Metres because that is what the slider says and what the number means
    /// by the time it gets here; the command and the algorithm below work in
    /// kilometres. The conversion lived nowhere for four months, so every
    /// setting of a 1–1000 m slider simplified at 1–1000 km. See
    /// `domain::track::simplify_track_points_m`.
    pub fn apply_simplify_track(
        &mut self,
        layer_id: LayerId,
        track_id: TrackId,
        tolerance_m: f64,
    ) -> Result<(), ProjectLayerError> {
        let cmd =
            commands::ProjectCommand::simplify_track(layer_id, track_id, tolerance_m / 1000.0);
        self.history
            .apply(&mut self.project, &cmd)
            .map_err(|e| match e {
                commands::CommandError::ProjectLayer(pe) => pe,
            })
    }

    pub fn set_track_line_width(&mut self, layer_id: LayerId, track_id: TrackId, width: f32) {
        if let Ok(track) = self.project.track_mut(layer_id.value(), track_id.value()) {
            track.style_mut().line_width = width.clamp(0.5, 20.0);
            self.mark_style_mutation();
        }
    }

    /// Export a track layer to GPX.
    ///
    /// Returns the failure as well as recording it: a status line at the
    /// bottom of the window is not an answer to "did my export happen?" —
    /// the caller needs to be able to show the error toast.
    pub fn export_layer_to_gpx(
        &mut self,
        layer_id: LayerId,
        path: std::path::PathBuf,
    ) -> Result<(), String> {
        let Some(layer) = self
            .project
            .track_layers()
            .iter()
            .find(|l| l.id() == layer_id)
        else {
            let message = "Layer not found for export".to_owned();
            self.update_status(DiagnosticLevel::Error, message.clone());
            return Err(message);
        };
        match crate::infrastructure::export::export_layer_to_gpx_file(layer, &path) {
            Ok(()) => {
                self.update_status(
                    DiagnosticLevel::Info,
                    format!("Exported to {}", path.display()),
                );
                Ok(())
            }
            Err(e) => {
                let message = format!("Export failed: {e}");
                self.update_status(DiagnosticLevel::Error, message.clone());
                Err(message)
            }
        }
    }

    /// Export all waypoints of the given layer to an OziExplorer `.wpt` file.
    pub fn export_wpt_waypoints(
        &mut self,
        layer_id: LayerId,
        path: std::path::PathBuf,
    ) -> Result<(), String> {
        let Some(layer) = self
            .project
            .waypoint_layers()
            .iter()
            .find(|l| l.id() == layer_id)
        else {
            let message = "Waypoint layer not found for export".to_owned();
            self.update_status(DiagnosticLevel::Error, message.clone());
            return Err(message);
        };
        let waypoints: Vec<Waypoint> = layer.waypoints().to_vec();

        let mut file = match std::fs::File::create(&path) {
            Ok(file) => file,
            Err(e) => {
                let message = format!("Export failed: {e}");
                self.update_status(DiagnosticLevel::Error, message.clone());
                return Err(message);
            }
        };

        match crate::infrastructure::export::wpt::write_wpt(waypoints, &mut file) {
            Ok(()) => {
                self.update_status(
                    DiagnosticLevel::Info,
                    format!("Exported waypoints to {}", path.display()),
                );
                Ok(())
            }
            Err(e) => {
                let message = format!("Export failed: {e}");
                self.update_status(DiagnosticLevel::Error, message.clone());
                Err(message)
            }
        }
    }

    /// Export every track in the project to one GPX file.
    ///
    /// A folder import makes one layer per file, so "today's tracks" is
    /// twenty-odd layers. Handing them over one dialog at a time is the same
    /// shape as hiding twenty-five tracks by hand.
    ///
    /// Returns the number of tracks written; an empty project is an error
    /// rather than a silently empty file.
    pub fn export_all_tracks_gpx(&mut self, path: std::path::PathBuf) -> Result<DayExport, String> {
        let tracks: Vec<crate::domain::Track> = self
            .project
            .track_layers()
            .iter()
            .flat_map(|layer| layer.tracks().iter().cloned())
            .collect();
        // The marks belong to the handover as much as the routes do: a crew
        // that found something put a waypoint there, and GPX carries both in
        // one document.
        let waypoints: Vec<crate::domain::Waypoint> = self
            .project
            .waypoint_layers()
            .iter()
            .flat_map(|layer| layer.waypoints().iter().cloned())
            .collect();

        if tracks.is_empty() && waypoints.is_empty() {
            let message = "Nothing to export".to_owned();
            self.update_status(DiagnosticLevel::Error, message.clone());
            return Err(message);
        }

        match crate::infrastructure::export::export_day_to_gpx_file(&tracks, &waypoints, &path) {
            Ok(()) => {
                self.update_status(
                    DiagnosticLevel::Info,
                    format!(
                        "Exported {} tracks and {} waypoints to {}",
                        tracks.len(),
                        waypoints.len(),
                        path.display()
                    ),
                );
                Ok(DayExport {
                    tracks: tracks.len(),
                    waypoints: waypoints.len(),
                })
            }
            Err(e) => {
                let message = format!("Export failed: {e}");
                self.update_status(DiagnosticLevel::Error, message.clone());
                Err(message)
            }
        }
    }

    /// Export all waypoints of the given layer to a GPX file.
    ///
    /// The waypoints capability has always required this and the XML builder
    /// has always existed; nothing reached it, so the only way out of the app
    /// was OziExplorer's own WPT — which the phones and the other groups'
    /// software do not read.
    pub fn export_gpx_waypoints(
        &mut self,
        layer_id: LayerId,
        path: std::path::PathBuf,
    ) -> Result<(), String> {
        let Some(layer) = self
            .project
            .waypoint_layers()
            .iter()
            .find(|l| l.id() == layer_id)
        else {
            let message = "Waypoint layer not found for export".to_owned();
            self.update_status(DiagnosticLevel::Error, message.clone());
            return Err(message);
        };

        match crate::infrastructure::export::export_waypoints_to_gpx_file(layer.waypoints(), &path)
        {
            Ok(()) => {
                self.update_status(
                    DiagnosticLevel::Info,
                    format!("Exported waypoints to {}", path.display()),
                );
                Ok(())
            }
            Err(e) => {
                let message = format!("Export failed: {e}");
                self.update_status(DiagnosticLevel::Error, message.clone());
                Err(message)
            }
        }
    }

    /// Default path suggestion for a waypoint export, in the given format.
    pub fn export_waypoints_default_path(
        &self,
        layer_id: LayerId,
        extension: &str,
    ) -> Option<PathBuf> {
        let layer = self
            .project
            .waypoint_layers()
            .iter()
            .find(|l| l.id() == layer_id)?;
        let file_name = format!("{}.{extension}", layer.name());
        match self.active_bundle_dir() {
            Some(dir) => Some(dir.join(file_name)),
            None => Some(PathBuf::from(file_name)),
        }
    }

    /// Default path suggestion for a WPT waypoint export.
    /// Returns `<bundle>/<layer_name>.wpt` when a bundle is active; otherwise
    /// `<layer_name>.wpt` (filename only, the dialog will pick a directory).
    pub fn export_wpt_default_path(&self, layer_id: LayerId) -> Option<PathBuf> {
        let layer = self
            .project
            .waypoint_layers()
            .iter()
            .find(|l| l.id() == layer_id)?;
        let file_name = format!("{}.wpt", layer.name());
        match self.active_bundle_dir() {
            Some(dir) => Some(dir.join(file_name)),
            None => Some(PathBuf::from(file_name)),
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
        if let Some(project) = self.lizaalert.selected_project.as_ref() {
            return Some(lizaalert::bundle_directory(
                &self.bundles_root,
                &project.summary.slug,
            ));
        }

        let map = self.lizaalert.active_map.as_ref()?;
        infer_bundle_dir_from_map_path(&map.local_path, &self.bundles_root)
    }

    pub fn export_default_tracks_dir_path(
        &self,
        track_name: &str,
        extension: &str,
    ) -> Option<PathBuf> {
        let extension = extension.trim_start_matches('.');
        if extension.is_empty() {
            return None;
        }

        Some(
            self.active_bundle_dir()?
                .join("10-Tracks")
                .join(format!("{track_name}.{extension}")),
        )
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

    fn restore_session(&mut self) {
        let Some(session_path) = self.session_path.as_deref() else {
            return;
        };

        let session = match persistence::load_app_session(session_path) {
            Ok(Some(session)) => session,
            Ok(None) => return,
            Err(error) => {
                self.update_status(
                    DiagnosticLevel::Error,
                    format!("Session restore skipped: {error}"),
                );
                return;
            }
        };

        // Restore the bundles root first: everything below resolves cached
        // bundles and the active map against it.
        if let Some(bundles_root) = session.bundles_root {
            self.bundles_root = bundles_root;
        }

        // The project and the map are independent things on disk, and what
        // follows restores each on its own.
        //
        // This used to be one run of early returns: a `.ozp` that had been
        // moved, renamed or deleted aborted the whole restore, so the crew
        // lost the raster as well and met a blank screen with everything
        // present on the disk underneath it. CJ-2 is a laptop switched on in a
        // field camp that has to be showing a working map inside a minute; the
        // map is the ground, the project is the work on it, and losing the
        // work is no reason to lose the ground.
        self.restore_session_project(session.last_project_path);
        self.restore_session_active_map(session.active_map);
    }

    fn restore_session_project(&mut self, project_path: Option<PathBuf>) {
        let Some(project_path) = project_path else {
            return;
        };
        if !project_path.exists() {
            self.update_status(
                DiagnosticLevel::Error,
                format!(
                    "Session restore skipped missing project: {}",
                    project_path.display()
                ),
            );
            return;
        }

        match persistence::load_project(&project_path) {
            Ok(project) => {
                self.project = project;
                self.project_path = Some(project_path.clone());
                self.history = CommandStack::default();
                self.mark_project_saved();
                self.update_status(
                    DiagnosticLevel::Info,
                    format!("Restored project: {}", project_path.display()),
                );
            }
            Err(error) => {
                self.update_status(
                    DiagnosticLevel::Error,
                    format!(
                        "Session restore skipped project {}: {error}",
                        project_path.display()
                    ),
                );
            }
        }
    }

    fn restore_session_active_map(&mut self, active_map: Option<PersistedActiveMap>) {
        let Some(active_map) = active_map else {
            return;
        };
        let Some(selection) = active_map_selection_from_persisted(active_map) else {
            self.update_status(
                DiagnosticLevel::Error,
                "Session restore skipped invalid active map kind",
            );
            return;
        };
        if !selection.local_path.exists() {
            self.update_status(
                DiagnosticLevel::Error,
                format!(
                    "Session restore skipped missing active map: {}",
                    selection.local_path.display()
                ),
            );
            return;
        }

        // Mirror the click-open path (`open_selected_map` /
        // `apply_map_downloaded` / `open_local_map_selection`) so the
        // frontend's `MapView` receives an `activeMapRef` change with the
        // backing map layer already registered. Without this call the
        // frontend would see `active_map` in `AppStateDto` but no
        // corresponding tile layer to fit-bounds against, leaving the
        // viewport at MapLibre's default `{ center: [0,0], zoom: 0 }`.
        let registration = self.register_active_map_layer(&selection);
        match registration {
            Ok(_) => {
                self.lizaalert.active_map = Some(selection.clone());
                self.update_status(
                    DiagnosticLevel::Info,
                    format!(
                        "Restored active map: {} / {}",
                        selection.project_name, selection.package_name
                    ),
                );
            }
            Err(error) => {
                self.lizaalert.active_map = None;
                self.update_status(
                    DiagnosticLevel::Error,
                    format!(
                        "Session restore failed to register active map {} / {}: {error:?}",
                        selection.project_name, selection.package_name
                    ),
                );
            }
        }
    }

    fn persist_session_snapshot(&mut self) {
        let Some(session_path) = self.session_path.as_deref() else {
            return;
        };
        let session = PersistedAppSession {
            last_project_path: self.project_path.clone(),
            active_map: self
                .lizaalert
                .active_map
                .as_ref()
                .map(persisted_active_map_from_selection),
            bundles_root: Some(self.bundles_root.clone()),
        };
        if let Err(error) = persistence::save_app_session(&session, session_path) {
            self.push_diagnostic(
                DiagnosticLevel::Error,
                format!("Session save failed: {error}"),
            );
        }
    }

    fn register_active_map_layer(
        &mut self,
        selection: &ActiveMapSelection,
    ) -> Result<bool, CommandError> {
        // All map-open paths (click-open, download completion, local .map,
        // session restore) funnel through this method, so the datum warning
        // fires exactly once per open — never per tile.
        if selection.kind == ActiveMapKind::OziRaster {
            self.warn_on_non_wgs84_datum(&selection.local_path);
        }

        if self
            .project
            .map_layers()
            .iter()
            .any(|layer| layer.source_path() == Some(selection.local_path.as_path()))
        {
            return Ok(false);
        }

        let layer_id = super::application::import::next_layer_id(&self.project);
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

    /// Push a warning diagnostic when the OZI `.map` at `map_path` is
    /// calibrated in a datum outside the WGS-84 family. Georeferencing uses
    /// calibration coordinates as-is (no datum transformation), so such maps
    /// shift ~100–150 m against GPS tracks. Read/parse failures are ignored
    /// here: the open and tile-serving paths report them on their own.
    fn warn_on_non_wgs84_datum(&mut self, map_path: &Path) {
        let Ok(contents) = read_ozi_map_text(map_path) else {
            return;
        };
        let Ok(metadata) = parse_ozi_map_metadata(map_path, &contents) else {
            return;
        };
        if let Some(warning) = datum_shift_warning(metadata.datum_name()) {
            self.push_diagnostic(DiagnosticLevel::Warning, warning);
        }
    }

    fn update_status(&mut self, level: DiagnosticLevel, message: impl Into<String>) {
        let message = message.into();
        self.lizaalert.status = message.clone();
        self.push_diagnostic(level, message);
    }

    fn push_diagnostic(&mut self, level: DiagnosticLevel, message: String) {
        match level {
            DiagnosticLevel::Error => tracing::error!("{message}"),
            DiagnosticLevel::Warning => tracing::warn!("{message}"),
            DiagnosticLevel::Info => tracing::info!("{message}"),
        }
        while self.lizaalert.diagnostics.len() >= MAX_DIAGNOSTICS {
            self.lizaalert.diagnostics.pop_front();
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

/// Placeholder used by `AppState::new()` (tests and `Default` only). The
/// runtime constructor `new_with_paths` receives the platform-resolved
/// directory from `lib.rs` instead of deriving it here.
fn default_bundles_root() -> PathBuf {
    PathBuf::from("bundles")
}

/// Returns a user-facing warning when `datum_name` is not in the WGS-84
/// family ("WGS 84", "WGS84", "WGS-84" in any case), `None` otherwise.
pub(crate) fn datum_shift_warning(datum_name: &str) -> Option<String> {
    let normalized: String = datum_name
        .chars()
        .filter(|c| !c.is_whitespace() && *c != '-')
        .map(|c| c.to_ascii_uppercase())
        .collect();
    if normalized == "WGS84" {
        return None;
    }
    Some(format!(
        "Карта откалибрована в датуме {datum_name}: возможно смещение ~100–150 м \
         относительно WGS-84 (треки/GPS). Преобразование датума пока не поддерживается."
    ))
}

fn persisted_active_map_from_selection(selection: &ActiveMapSelection) -> PersistedActiveMap {
    PersistedActiveMap {
        kind: match selection.kind {
            ActiveMapKind::SqliteTiles => "sqlite".to_owned(),
            ActiveMapKind::OziRaster => "ozi".to_owned(),
        },
        project_name: selection.project_name.clone(),
        package_name: selection.package_name.clone(),
        remote_url: selection.remote_url.clone(),
        local_path: selection.local_path.clone(),
        center_lat: selection.center.lat,
        center_lon: selection.center.lon,
        base_zoom: selection.base_zoom,
    }
}

fn active_map_selection_from_persisted(
    active_map: PersistedActiveMap,
) -> Option<ActiveMapSelection> {
    let kind = match active_map.kind.as_str() {
        "sqlite" => ActiveMapKind::SqliteTiles,
        "ozi" => ActiveMapKind::OziRaster,
        _ => return None,
    };

    Some(ActiveMapSelection {
        kind,
        project_name: active_map.project_name,
        package_name: active_map.package_name,
        remote_url: active_map.remote_url,
        local_path: active_map.local_path,
        center: MapCenter {
            lat: active_map.center_lat,
            lon: active_map.center_lon,
        },
        base_zoom: active_map.base_zoom,
    })
}

fn infer_bundle_dir_from_map_path(
    map_path: &std::path::Path,
    bundles_root: &std::path::Path,
) -> Option<PathBuf> {
    for ancestor in map_path.ancestors() {
        if ancestor.parent() == Some(bundles_root) {
            return Some(ancestor.to_path_buf());
        }
    }

    let parent = map_path.parent()?;
    if parent.file_name().and_then(|name| name.to_str()) == Some("8-Android&iOS") {
        return parent.parent().map(std::path::Path::to_path_buf);
    }

    Some(parent.to_path_buf())
}

pub(crate) fn reveal_in_file_manager(path: &std::path::Path) {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn project_dirty_tracks_all_mutation_classes_and_clears_on_save() {
        let dir = tempfile::tempdir().expect("tempdir");
        let save_path = dir.path().join("cj7.ozp");
        let mut state = AppState::new();
        assert!(!state.project_dirty(), "fresh state must be clean");

        let track_layer = state.project.track_layers()[0].id();
        let waypoint_layer = state.project.waypoint_layers()[0].id();

        // Undoable command class.
        let track_id = state
            .apply_create_empty_track(track_layer, "CJ7".into())
            .expect("create track");
        assert!(state.project_dirty(), "undoable command must mark dirty");

        state
            .save_project_to(save_path.clone())
            .expect("save clears dirty");
        assert!(!state.project_dirty(), "save must clear dirty");

        // Undo / redo class.
        state.undo();
        assert!(state.project_dirty(), "undo must mark dirty");
        state.redo();
        state.save_project_to(save_path.clone()).expect("resave");
        assert!(!state.project_dirty());

        // Non-undoable style setters (bypass CommandStack, ADR-0017).
        state.set_track_color(track_layer, track_id, [1, 2, 3, 255]);
        assert!(state.project_dirty(), "set_track_color must mark dirty");
        state.save_project_to(save_path.clone()).expect("resave");

        state.set_track_line_width(track_layer, track_id, 3.0);
        assert!(
            state.project_dirty(),
            "set_track_line_width must mark dirty"
        );
        state.save_project_to(save_path.clone()).expect("resave");

        state.toggle_track_visible(track_layer, track_id);
        assert!(
            state.project_dirty(),
            "toggle_track_visible must mark dirty"
        );
        state.save_project_to(save_path.clone()).expect("resave");

        state
            .apply_add_waypoint(waypoint_layer, 55.0, 37.0, "W1".into())
            .expect("add waypoint");
        state.save_project_to(save_path.clone()).expect("resave");
        let waypoint_id = state.project.waypoint_layers()[0].waypoints()[0].id();
        state.toggle_waypoint_visible(waypoint_layer, waypoint_id);
        assert!(
            state.project_dirty(),
            "toggle_waypoint_visible must mark dirty"
        );

        // A failed save must NOT clear dirty.
        let blocked = dir.path().join("blocked.ozp");
        std::fs::create_dir_all(&blocked).expect("blocking dir");
        let _ = state.save_project_to(blocked);
        assert!(state.project_dirty(), "failed save must keep dirty");
    }

    /// Loading a project (or restoring a session) starts clean: the user has
    /// not changed anything yet, so the close-guard must not fire.
    #[test]
    /// A find is photographed from three sides, and undo puts the list back.
    #[test]
    fn waypoint_attachments_are_one_undoable_step() {
        let mut state = AppState::new();
        let layer_id = LayerId::new(1);
        state
            .apply_add_waypoint(layer_id, 53.9, 27.5, "улика".to_owned())
            .expect("add waypoint");
        let waypoint_id = state
            .project_waypoint_layers()
            .iter()
            .find(|l| l.id() == layer_id)
            .expect("layer")
            .waypoints()
            .last()
            .expect("waypoint")
            .id();

        let files = vec![
            "находки/куртка-1.jpg".to_owned(),
            "находки/куртка-2.jpg".to_owned(),
        ];
        state
            .apply_set_waypoint_attachments(layer_id, waypoint_id, files.clone())
            .expect("attach");
        assert_eq!(attachments_of(&state, layer_id, waypoint_id), files);

        state.undo();
        assert!(
            attachments_of(&state, layer_id, waypoint_id).is_empty(),
            "undo restores the list as it was, not as it might have been"
        );

        state.redo();
        assert_eq!(attachments_of(&state, layer_id, waypoint_id), files);
    }

    fn attachments_of(state: &AppState, layer_id: LayerId, waypoint_id: WaypointId) -> Vec<String> {
        state
            .project_waypoint_layers()
            .iter()
            .find(|l| l.id() == layer_id)
            .expect("layer")
            .waypoints()
            .iter()
            .find(|w| w.id() == waypoint_id)
            .expect("waypoint")
            .attachments()
            .to_vec()
    }

    fn project_dirty_clears_on_load() {
        let dir = tempfile::tempdir().expect("tempdir");
        let save_path = dir.path().join("load.ozp");
        let mut state = AppState::new();
        let track_layer = state.project.track_layers()[0].id();
        state
            .apply_create_empty_track(track_layer, "T".into())
            .expect("create");
        state.save_project_to(save_path.clone()).expect("save");

        let mut fresh = AppState::new();
        let fresh_layer = fresh.project.track_layers()[0].id();
        fresh
            .apply_create_empty_track(fresh_layer, "X".into())
            .expect("create");
        assert!(fresh.project_dirty());
        fresh.load_project_from(save_path).expect("load");
        assert!(!fresh.project_dirty(), "loaded project starts clean");
    }
    use crate::infrastructure::persistence::{PersistedActiveMap, PersistedAppSession};
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_session_dir(label: &str) -> PathBuf {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time")
            .as_nanos();
        let dir =
            std::env::temp_dir().join(format!("ozi-rs-{label}-{}-{unique}", std::process::id()));
        std::fs::create_dir_all(&dir).expect("create temp session dir");
        dir
    }

    fn sample_project_with_remote_map() -> LizaProject {
        LizaProject {
            summary: LizaProjectSummary {
                slug: "demo-project".to_owned(),
                name: "Demo Project".to_owned(),
                url: "https://example.invalid/project".to_owned(),
            },
            center: MapCenter {
                lat: 55.0,
                lon: 37.0,
            },
            maps: vec![LizaMapPackage {
                name: "demo-map".to_owned(),
                file_name: "demo-map.sqlitedb".to_owned(),
                url: "https://example.invalid/demo-map.sqlitedb".to_owned(),
                base_zoom: 12,
                local_path: None,
                size_bytes: None,
            }],
            contents: Vec::new(),
        }
    }

    #[test]
    fn save_project_to_propagates_error_and_reports_diagnostic() {
        let dir = temp_session_dir("save-project-error");
        // A directory at the target path makes the save fail
        // deterministically without touching permissions.
        let target = dir.join("blocked.ozp");
        std::fs::create_dir_all(&target).expect("create blocking dir");

        let mut state = AppState::new();
        let result = state.save_project_to(target);

        assert!(
            result.is_err(),
            "save into a directory path must propagate the error to the caller"
        );
        assert!(
            state.lizaalert.status.starts_with("Save failed:"),
            "failure diagnostic must still be reported, got: {}",
            state.lizaalert.status
        );
        assert!(
            state.project_file_path().is_none(),
            "a failed save must not update the current project path"
        );
    }

    fn summary_for(slug: &str, name: &str) -> LizaProjectSummary {
        LizaProjectSummary {
            slug: slug.to_owned(),
            name: name.to_owned(),
            url: format!("https://example.invalid/{slug}/"),
        }
    }

    fn previewed_project(slug: &str, name: &str) -> LizaProject {
        LizaProject {
            summary: summary_for(slug, name),
            center: MapCenter {
                lat: 55.0,
                lon: 37.0,
            },
            maps: Vec::new(),
            contents: Vec::new(),
        }
    }

    /// Two previews in a row: the operator clicks one project, changes their
    /// mind and clicks the next. Each request runs in its own thread, so the
    /// first can land last — and it used to win, swapping the map list under a
    /// row the operator had already moved on from. Nothing ordered the two.
    #[test]
    fn a_preview_the_operator_has_moved_on_from_is_dropped() {
        let mut state = AppState::new();
        state.lizaalert.projects.push(summary_for("first", "First"));
        state
            .lizaalert
            .projects
            .push(summary_for("second", "Second"));

        state.preview_data("first").expect("first preview starts");
        state.preview_data("second").expect("second preview starts");

        state.apply_preview_loaded("first", Ok(previewed_project("first", "First")));
        assert!(
            state.lizaalert.selected_project.is_none(),
            "the abandoned preview must not become the shown project"
        );
        assert_eq!(
            state.lizaalert.selected_project_slug.as_deref(),
            Some("second"),
            "and it must not steal the selection back either"
        );

        state.apply_preview_loaded("second", Ok(previewed_project("second", "Second")));
        assert_eq!(
            state
                .lizaalert
                .selected_project
                .as_ref()
                .map(|p| p.summary.slug.as_str()),
            Some("second"),
            "the preview the operator is waiting for still lands"
        );
    }

    /// A failure from an abandoned preview is just as stale as a success: it
    /// would put an error in the status bar about a project nobody asked about
    /// any more.
    #[test]
    fn a_stale_preview_failure_is_not_reported() {
        let mut state = AppState::new();
        state.lizaalert.projects.push(summary_for("first", "First"));
        state
            .lizaalert
            .projects
            .push(summary_for("second", "Second"));

        state.preview_data("first").expect("first preview starts");
        state.preview_data("second").expect("second preview starts");
        let status_while_waiting = state.lizaalert.status.clone();

        state.apply_preview_loaded("first", Err("host unreachable".to_owned()));

        assert_eq!(
            state.lizaalert.status, status_while_waiting,
            "an abandoned preview must not report its failure"
        );
    }

    /// A preview deliberately runs without taking the busy flag, so that a
    /// click during the catalogue walk is not silently ignored. It must not
    /// clear the flag on the way out either: doing so let a second download
    /// start while the first was still running.
    #[test]
    fn a_finished_preview_leaves_the_busy_flag_alone() {
        let mut state = AppState::new();
        state.lizaalert.projects.push(summary_for("demo", "Demo"));

        state.preview_data("demo").expect("preview starts");
        state
            .begin_load_project("demo")
            .expect("a download starts while the preview is in flight");
        assert!(
            state.lizaalert.bundle_busy,
            "the download owns the bundle flag"
        );

        state.apply_preview_loaded("demo", Ok(previewed_project("demo", "Demo")));

        assert!(
            state.lizaalert.bundle_busy,
            "the preview did not take the bundle flag and must not release it"
        );
    }

    /// A search that has been taken down upstream used to stay in the list for
    /// good: chunks are merged, and nothing ever removed anything. Offline it
    /// then sat in the cache too, and clicking it failed.
    ///
    /// A walk that ran to the end knows exactly what exists. One that was
    /// stopped knows only a prefix, so it must not prune anything.
    #[test]
    fn a_complete_walk_drops_a_search_that_is_gone() {
        let mut state = AppState::new();
        state.apply_projects_chunk(vec![
            summary_for("2026-09-01_old", "Old"),
            summary_for("2026-09-20_current", "Current"),
        ]);

        state.apply_projects_loaded(Ok(lizaalert::CatalogueWalk {
            projects: vec![summary_for("2026-09-20_current", "Current")],
            pages: 3,
            cancelled: false,
        }));

        assert_eq!(
            state
                .lizaalert
                .projects
                .iter()
                .map(|p| p.slug.as_str())
                .collect::<Vec<_>>(),
            vec!["2026-09-20_current"],
            "a complete walk is the whole truth about what exists"
        );
    }

    #[test]
    fn a_stopped_walk_prunes_nothing() {
        let mut state = AppState::new();
        state.apply_projects_chunk(vec![
            summary_for("2026-09-01_old", "Old"),
            summary_for("2026-09-20_current", "Current"),
        ]);

        state.apply_projects_loaded(Ok(lizaalert::CatalogueWalk {
            projects: vec![summary_for("2026-09-20_current", "Current")],
            pages: 1,
            cancelled: true,
        }));

        assert_eq!(
            state.lizaalert.projects.len(),
            2,
            "a stopped walk read only a prefix and may not remove anything"
        );
    }

    /// Trimming a track at a point.
    ///
    /// The commonest edit a crew makes to a recording: the first twenty
    /// minutes are the drive to the start, so cut everything before the point
    /// where the walking begins. Crop by time can do it if they know the time;
    /// this is the gesture they actually have — they can see the point.
    ///
    /// The named point is kept in both directions. It is where the walk
    /// starts, or where it ends, and a crop that removed it would be off by
    /// one in the direction nobody checks.
    #[test]
    fn trimming_at_a_point_keeps_that_point_and_undoes_whole() {
        use crate::domain::{Track, TrackPoint, TrackPointId, TrackSegment, TrackSegmentId};

        let mut state = AppState::new();
        let layer_id = LayerId::new(1);
        let track_id = TrackId::new(1);

        let mut track = Track::new(track_id, "20260708_Ветер");
        let mut segment = TrackSegment::new(TrackSegmentId::new(1));
        for i in 1..=5u64 {
            segment.add_point(TrackPoint::new(
                TrackPointId::new(i),
                59.95 + i as f64 * 0.001,
                31.59,
            ));
        }
        track.add_segment(segment);
        state.project.add_track_to_layer(layer_id, track).unwrap();

        let ids = |s: &AppState| {
            s.project.track_layers()[0].tracks()[0].segments()[0]
                .points()
                .iter()
                .map(|p| p.id().value())
                .collect::<Vec<_>>()
        };

        let removed = state
            .apply_trim_track_at_point(layer_id, track_id, TrackPointId::new(3), true)
            .expect("trim before");
        assert_eq!(removed, 2, "the two points before it");
        assert_eq!(ids(&state), vec![3, 4, 5], "and the named point stays");

        state.undo();
        assert_eq!(
            ids(&state),
            vec![1, 2, 3, 4, 5],
            "one undo puts them all back"
        );

        let removed = state
            .apply_trim_track_at_point(layer_id, track_id, TrackPointId::new(3), false)
            .expect("trim after");
        assert_eq!(removed, 2);
        assert_eq!(ids(&state), vec![1, 2, 3]);
    }

    /// A crop that would leave nothing is refused elsewhere in this file; a
    /// trim at the first or last point removes nothing and must not pretend
    /// otherwise by recording an undo step for it.
    #[test]
    fn trimming_at_an_end_point_changes_nothing() {
        use crate::domain::{Track, TrackPoint, TrackPointId, TrackSegment, TrackSegmentId};

        let mut state = AppState::new();
        let layer_id = LayerId::new(1);
        let track_id = TrackId::new(1);

        let mut track = Track::new(track_id, "Короткий");
        let mut segment = TrackSegment::new(TrackSegmentId::new(1));
        for i in 1..=3u64 {
            segment.add_point(TrackPoint::new(TrackPointId::new(i), 59.95, 31.59));
        }
        track.add_segment(segment);
        state.project.add_track_to_layer(layer_id, track).unwrap();
        let before = state.history.mutation_count();

        assert_eq!(
            state
                .apply_trim_track_at_point(layer_id, track_id, TrackPointId::new(1), true)
                .expect("trim before the first point"),
            0
        );
        assert_eq!(
            state.history.mutation_count(),
            before,
            "nothing removed, nothing to undo"
        );
    }

    fn state_with_demo_project() -> AppState {
        let mut state = AppState::new();
        state.lizaalert.projects.push(LizaProjectSummary {
            slug: "2026-09-21_demo".to_owned(),
            name: "Demo".to_owned(),
            url: "https://example.invalid/demo/".to_owned(),
        });
        state
    }

    /// A second download, while one is already running, is still refused with
    /// a reason the loader can show.
    #[test]
    fn opening_a_second_bundle_while_one_downloads_is_refused_with_a_reason() {
        let mut state = state_with_demo_project();

        state.lizaalert.bundle_busy = true;
        assert_eq!(
            state.begin_load_project("2026-09-21_demo"),
            Err(LoadProjectRefusal::Busy)
        );

        state.lizaalert.bundle_busy = false;
        assert_eq!(
            state.begin_load_project("nothing-like-this"),
            Err(LoadProjectRefusal::UnknownProject)
        );

        assert!(state.begin_load_project("2026-09-21_demo").is_ok());
    }

    /// The catalogue walk is up to a thousand pages, and it used to share one
    /// `busy` flag with downloading. On a field link that disabled the only
    /// download button in the application for minutes after every launch, for
    /// no reason: the walk reads a remote listing, the download fetches files.
    #[test]
    fn a_download_may_start_while_the_catalogue_is_being_walked() {
        let mut state = state_with_demo_project();

        let walk = state.begin_load_projects();
        assert!(walk.is_some(), "the walk takes its own flag");
        assert!(state.lizaalert.listing_busy);

        assert!(
            state.begin_load_project("2026-09-21_demo").is_ok(),
            "a download must not wait for the listing"
        );
    }

    /// Two walks at once would fight over the same list, so the listing flag
    /// still guards its own re-entry.
    #[test]
    fn a_second_catalogue_walk_is_refused_while_one_runs() {
        let mut state = AppState::new();
        assert!(state.begin_load_projects().is_some());
        assert!(state.begin_load_projects().is_none());
    }

    /// Each flag is released by the completion of its own operation. Sharing
    /// one release point is how a finished preview used to free a download.
    #[test]
    fn finishing_one_operation_does_not_release_the_other() {
        let mut state = state_with_demo_project();
        state.begin_load_projects();
        state.begin_load_project("2026-09-21_demo").expect("start");

        state.apply_projects_loaded(Ok(lizaalert::CatalogueWalk {
            projects: Vec::new(),
            pages: 0,
            cancelled: true,
        }));
        assert!(!state.lizaalert.listing_busy, "the walk released its flag");
        assert!(
            state.lizaalert.bundle_busy,
            "the download still owns its own"
        );

        state.apply_project_loaded(Err("stopped".to_owned()));
        assert!(!state.lizaalert.bundle_busy);
    }

    /// Opening a bundle from disk is a bundle operation, not a listing one.
    #[test]
    fn opening_a_local_bundle_takes_the_bundle_flag() {
        let mut state = AppState::new();
        assert!(
            state
                .begin_open_local_bundle(PathBuf::from("/tmp/bundle"))
                .is_some()
        );
        assert!(state.lizaalert.bundle_busy);
        assert!(!state.lizaalert.listing_busy);
        assert!(
            state
                .begin_open_local_bundle(PathBuf::from("/tmp/other"))
                .is_none(),
            "a second one is refused"
        );
    }

    /// Длинное рисование должно отменяться целиком.
    ///
    /// Внешнее ревью 22.09: отмена считала число команд, а стек ограничен
    /// сотней записей и вытесняет старое. После сотни точек команда создания
    /// трека уже вытеснена — `discard_last` снимала только оставшиеся
    /// вставки, и пустой трек оставался в проекте. Крыло, которое передумало
    /// рисовать длинный маршрут, получало мусор без способа его убрать
    /// отменой.
    #[test]
    fn cancelling_a_long_drawing_leaves_no_scratch_track() {
        let mut state = AppState::new();
        let layer = LayerId::new(1);
        state
            .apply_create_empty_track(layer, "рисую".to_owned())
            .expect("create");
        let track = state
            .project
            .track_layers()
            .iter()
            .find(|l| l.id() == layer)
            .expect("layer")
            .tracks()
            .last()
            .expect("track")
            .id();

        // Больше, чем глубина стека: самые ранние команды вытесняются.
        let segment = state
            .project
            .track_layers()
            .iter()
            .find(|l| l.id() == layer)
            .expect("layer")
            .tracks()
            .last()
            .expect("track")
            .segments()
            .first()
            .expect("segment")
            .id();
        for i in 0..150usize {
            state
                .apply_insert_track_point(layer, track, segment, i, 59.9 + i as f64 * 1e-5, 31.5)
                .expect("point");
        }

        state.cancel_drawing_of(layer, track, 151);

        let tracks_left = state
            .project
            .track_layers()
            .iter()
            .find(|l| l.id() == layer)
            .expect("layer")
            .tracks()
            .len();
        assert_eq!(
            tracks_left, 0,
            "отменённое рисование не должно оставлять трек в проекте"
        );
    }

    /// Неудачное открытие должно доехать до вызывающего.
    ///
    /// Раньше `load_project_from` возвращала `()`, ошибка уходила в
    /// диагностику, а команда отвечала `Ok(())`. Интерфейс на это записывал
    /// путь в недавние и кадрировал карту по проекту, который не открылся.
    #[test]
    fn opening_a_file_that_is_not_a_project_reports_the_failure() {
        let dir = temp_session_dir("open-failure");
        let path = dir.join("broken.ozp");
        std::fs::write(&path, b"not json at all").expect("write");

        let mut state = AppState::new();
        let before = state.project_name().to_owned();

        let result = state.load_project_from(path);

        assert!(result.is_err(), "отказ должен вернуться вызывающему");
        assert_eq!(
            state.project_name(),
            before,
            "неудачное открытие не должно менять текущий проект"
        );
    }

    /// A fresh project must have exactly one layer of each kind. It used to
    /// have two of each, both claiming id 1 — the selector showed "Tracks"
    /// twice and the second was unreachable.
    #[test]
    fn a_fresh_project_has_one_layer_of_each_kind() {
        let state = AppState::new();

        assert_eq!(state.track_layers().len(), 1, "one track layer");
        assert_eq!(
            state.project_waypoint_layers().len(),
            1,
            "one waypoint layer"
        );
        assert_eq!(state.track_layers()[0].id().value(), 1);
        assert_eq!(state.project_waypoint_layers()[0].id().value(), 1);
    }

    /// Esc during a drawing is "forget this", not "undo this": the abandoned
    /// track must not be recoverable with redo, and a project that was saved
    /// before the drawing started must read as saved again.
    #[test]
    fn cancelling_a_drawing_leaves_no_redo_entry_and_no_unsaved_changes() {
        let dir = temp_session_dir("cancel-drawing");
        let path = dir.join("search.ozp");
        let mut state = AppState::new();
        state.save_project_to(path).expect("save");
        assert!(!state.project_dirty(), "a freshly saved project is clean");

        // Draw: create the track, then add two points to its first segment.
        let layer_id = LayerId::new(1);
        let track_id = state
            .apply_create_empty_track(layer_id, "drawing".to_owned())
            .expect("create track");
        let segment_id = state
            .project
            .track_layers()
            .iter()
            .find(|l| l.id() == layer_id)
            .and_then(|l| l.tracks().iter().find(|t| t.id() == track_id))
            .and_then(|t| t.segments().first())
            .map(|segment| segment.id())
            .expect("the new track has a segment");
        state
            .apply_insert_track_point(layer_id, track_id, segment_id, 0, 55.0, 37.0)
            .expect("point 1");
        state
            .apply_insert_track_point(layer_id, track_id, segment_id, 1, 55.1, 37.1)
            .expect("point 2");
        assert!(state.project_dirty(), "drawing marks the project changed");
        let drawn = state
            .project
            .track_layers()
            .iter()
            .find(|l| l.id() == layer_id)
            .expect("layer")
            .tracks()
            .len();
        assert_eq!(drawn, 1, "the drawing is on the project");

        let discarded = state.cancel_drawing_of(layer_id, track_id, 3);
        assert_eq!(discarded, 3);

        assert_eq!(
            state
                .project
                .track_layers()
                .iter()
                .find(|l| l.id() == layer_id)
                .expect("layer")
                .tracks()
                .len(),
            0,
            "the abandoned track SHALL be gone"
        );
        state.redo();
        assert_eq!(
            state
                .project
                .track_layers()
                .iter()
                .find(|l| l.id() == layer_id)
                .expect("layer")
                .tracks()
                .len(),
            0,
            "redo SHALL NOT resurrect an abandoned drawing"
        );
        assert!(
            !state.project_dirty(),
            "a cancelled drawing SHALL leave the project as saved as it was"
        );
    }

    /// The waypoints capability has always required a GPX export and the XML
    /// builder has always existed — nothing called it. WPT alone strands a
    /// crew whose counterpart runs a phone rather than OziExplorer.
    #[test]
    fn waypoints_export_to_gpx_that_a_phone_can_read() {
        let dir = temp_session_dir("waypoint-gpx-export");
        let mut state = AppState::new();
        let layer_id = LayerId::new(1);
        state
            .project
            .add_waypoint_to_layer(
                layer_id,
                Waypoint::new(WaypointId::new(1), "ЗАБРОС", 59.951938, 31.596359),
            )
            .unwrap();
        state
            .apply_set_waypoint_symbol(layer_id, WaypointId::new(1), Some("Flag".to_owned()))
            .unwrap();

        let path = dir.join("waypoints.gpx");
        state
            .export_gpx_waypoints(layer_id, path.clone())
            .expect("export");

        let xml = std::fs::read_to_string(&path).expect("read gpx");
        assert!(xml.contains("<gpx"), "a GPX document");
        assert!(
            xml.contains("59.951938") && xml.contains("31.596359"),
            "the waypoint's position: {xml}"
        );
        assert!(xml.contains("ЗАБРОС"), "its name, in Cyrillic: {xml}");
        assert!(xml.contains("Flag"), "and its symbol: {xml}");
    }

    #[test]
    fn a_failed_waypoint_gpx_export_reaches_the_caller() {
        let dir = temp_session_dir("waypoint-gpx-failure");
        let mut state = AppState::new();

        assert!(
            state
                .export_gpx_waypoints(LayerId::new(9999), dir.join("out.gpx"))
                .is_err(),
            "a missing layer SHALL be an error"
        );
        assert!(
            state
                .export_gpx_waypoints(LayerId::new(1), dir.join("no-such-dir").join("out.gpx"))
                .is_err(),
            "an unwritable path SHALL be an error"
        );
    }

    #[test]
    fn the_suggested_waypoint_file_name_follows_the_format() {
        let state = AppState::new();
        let gpx = state
            .export_waypoints_default_path(LayerId::new(1), "gpx")
            .expect("suggestion");
        let wpt = state
            .export_waypoints_default_path(LayerId::new(1), "wpt")
            .expect("suggestion");

        assert!(gpx.to_string_lossy().ends_with(".gpx"));
        assert!(wpt.to_string_lossy().ends_with(".wpt"));
        assert_eq!(
            state.export_waypoints_default_path(LayerId::new(9999), "gpx"),
            None,
            "no suggestion for a layer that is not there"
        );
    }

    /// A folder import makes one layer per navigator file, so handing the
    /// day's work to the штаб meant one export dialog per layer.
    #[test]
    fn every_track_in_the_project_exports_to_one_file() {
        let dir = temp_session_dir("export-all-tracks");
        let mut state = AppState::new();
        let first = LayerId::new(1);
        state
            .apply_create_empty_track(first, "20260708_Veter2".to_owned())
            .expect("track");
        state
            .apply_create_empty_track(first, "20260709-ЛИСА15".to_owned())
            .expect("track");
        // A second layer, the shape an import leaves behind.
        state
            .project
            .add_track_layer(crate::domain::TrackLayer::new(
                LayerId::new(2),
                "20260709_Veter3.gpx",
            ));
        state
            .apply_create_empty_track(LayerId::new(2), "20260709-ЛИСА16".to_owned())
            .expect("track");

        let path = dir.join("tracks.gpx");
        let written = state.export_all_tracks_gpx(path.clone()).expect("export");

        assert_eq!(written.tracks, 3, "every track, across every layer");
        let xml = std::fs::read_to_string(&path).expect("read gpx");
        assert_eq!(xml.matches("<trk>").count(), 3);
        assert!(xml.contains("20260709-ЛИСА15"), "Cyrillic names survive");
        assert!(
            xml.contains("20260709-ЛИСА16"),
            "including the second layer"
        );
    }

    /// Handing the day over means the tracks and the marks made on them. A
    /// crew that found something put a waypoint there; a file of tracks alone
    /// leaves the one thing the штаб most wants to see out of the handover,
    /// and GPX holds both in one document.
    #[test]
    fn the_days_export_carries_the_marks_as_well_as_the_tracks() {
        let dir = temp_session_dir("export-day");
        let mut state = AppState::new();
        state
            .apply_create_empty_track(LayerId::new(1), "20260708_Veter2".to_owned())
            .expect("track");
        state
            .apply_add_waypoint(LayerId::new(1), 59.95243, 31.59681, "ШТАБ".to_owned())
            .expect("waypoint");
        state
            .apply_add_waypoint(LayerId::new(1), 59.95194, 31.59636, "ЗАБРОС".to_owned())
            .expect("waypoint");

        let path = dir.join("day.gpx");
        let written = state.export_all_tracks_gpx(path.clone()).expect("export");

        assert_eq!(written.tracks, 1);
        assert_eq!(written.waypoints, 2);
        let xml = std::fs::read_to_string(&path).expect("read gpx");
        assert_eq!(xml.matches("<trk>").count(), 1);
        assert_eq!(xml.matches("<wpt ").count(), 2);
        assert!(xml.contains("ШТАБ") && xml.contains("ЗАБРОС"));
    }

    /// A project of marks and no tracks is a real state — the штаб's own
    /// project, before anybody has walked anywhere — and it has something
    /// worth handing over.
    #[test]
    fn a_project_of_marks_alone_still_exports() {
        let dir = temp_session_dir("export-marks-only");
        let mut state = AppState::new();
        state
            .apply_add_waypoint(LayerId::new(1), 59.95243, 31.59681, "ШТАБ".to_owned())
            .expect("waypoint");

        let path = dir.join("marks.gpx");
        let written = state.export_all_tracks_gpx(path.clone()).expect("export");
        assert_eq!(written.tracks, 0);
        assert_eq!(written.waypoints, 1);
    }

    #[test]
    fn exporting_an_empty_project_is_an_error_not_an_empty_file() {
        let dir = temp_session_dir("export-all-empty");
        let mut state = AppState::new();
        let path = dir.join("tracks.gpx");

        assert!(state.export_all_tracks_gpx(path.clone()).is_err());
        assert!(
            !path.exists(),
            "an empty export SHALL NOT leave a file that looks like a day's work"
        );
    }

    /// The owner's July note: a crew should be able to start on the topo
    /// layer while the 185 MiB satellite layer is still coming down. The
    /// plumbing was there — a file that lands mid-download gets its
    /// `local_path` — but nothing had ever checked that opening it works.
    #[test]
    fn a_map_that_lands_mid_download_can_be_opened_without_waiting() {
        let mut state = AppState::new();
        let mut project = sample_project_with_remote_map();
        project.maps[0].local_path = None;
        let package = project.maps[0].name.clone();
        let file_name = project.maps[0].file_name.clone();
        state.lizaalert.selected_project = Some(project);

        // Before its file lands, opening it has to fetch.
        assert!(matches!(
            state.begin_open_map(&package),
            Some(OpenMapRequest::Download(_))
        ));
        state.lizaalert.downloading.clear();

        // The bundle download reports that file ready.
        let landed = std::path::PathBuf::from(format!("/tmp/bundle/{file_name}"));
        state.note_bundle_file_ready(&format!("8-Android&iOS/{file_name}"), &landed);

        // Now it opens from disk, with the rest of the bundle still running.
        match state.begin_open_map(&package) {
            Some(OpenMapRequest::Local(selection)) => {
                assert_eq!(selection.local_path, landed);
            }
            _ => panic!("a map already on disk SHALL open without a download"),
        }
    }

    /// A crew finishes one search and starts the next. Until this existed,
    /// they kept adding to the same document.
    #[test]
    fn a_new_project_is_empty_and_undo_cannot_undo_it() {
        use crate::domain::{Track, TrackPoint, TrackPointId, TrackSegment, TrackSegmentId};
        let mut state = AppState::new();
        let layer_id = LayerId::new(state.create_track_layer("Day two".to_owned()).unwrap());
        let mut track = Track::new(TrackId::new(1), "ЛИСА15");
        let mut segment = TrackSegment::new(TrackSegmentId::new(1));
        segment.add_point(TrackPoint::new(TrackPointId::new(1), 53.9, 27.5));
        track.add_segment(segment);
        state
            .project_mut()
            .add_track_to_layer(layer_id, track)
            .expect("add track");
        assert!(state.track_layers().iter().any(|l| !l.tracks().is_empty()));

        state.new_project();

        assert!(
            state
                .track_layers()
                .iter()
                .all(|layer| layer.tracks().is_empty()),
            "the next search SHALL start with nothing drawn"
        );
        // The invariant the `layers` capability declares survives.
        assert!(!state.track_layers().is_empty());
        assert!(!state.project_waypoint_layers().is_empty());

        // Undo that walks back into a finished search and puts its tracks on
        // the map again is not an undo anybody wants.
        state.undo();
        assert!(
            state
                .track_layers()
                .iter()
                .all(|layer| layer.tracks().is_empty()),
            "undo SHALL NOT reach past the start of a new search"
        );
    }

    /// The map is the ground; the project is the work on it. A second search
    /// in the same district should not blank the screen.
    #[test]
    fn a_new_project_keeps_the_map_that_is_open() {
        let mut state = AppState::new();
        state.lizaalert.active_map = Some(ActiveMapSelection {
            kind: ActiveMapKind::SqliteTiles,
            project_name: "2026-07-08 Lavrovo".to_owned(),
            package_name: "topo".to_owned(),
            remote_url: "https://example.invalid/topo.sqlitedb".to_owned(),
            local_path: std::path::PathBuf::from("/tmp/topo.sqlitedb"),
            center: MapCenter {
                lat: 59.95,
                lon: 31.6,
            },
            base_zoom: 16,
        });

        state.new_project();

        assert!(
            state.lizaalert.active_map.is_some(),
            "the raster the crew is looking at SHALL survive a new search"
        );
    }

    /// The path an operator actually takes: delete the layer the import made,
    /// then think better of it. The command is built from the live layer, so
    /// this is what proves the tracks travel with it.
    #[test]
    fn deleting_a_layer_and_undoing_brings_its_tracks_back() {
        use crate::domain::{Track, TrackPoint, TrackPointId, TrackSegment, TrackSegmentId};
        let mut state = AppState::new();
        let layer_id = state
            .create_track_layer("Imported tracks: /tmp/day3.gpx".to_owned())
            .expect("create");
        let layer_id = LayerId::new(layer_id);

        for n in 0..4u64 {
            let mut track = Track::new(TrackId::new(n + 1), format!("ЛИСА{n}"));
            let mut segment = TrackSegment::new(TrackSegmentId::new(n + 1));
            segment.add_point(TrackPoint::new(TrackPointId::new(n + 1), 53.9, 27.5));
            track.add_segment(segment);
            state
                .project_mut()
                .add_track_to_layer(layer_id, track)
                .expect("add track");
        }

        state.delete_track_layer(layer_id).expect("delete");
        assert!(!state.track_layers().iter().any(|l| l.id() == layer_id));

        state.undo();
        let back = state
            .track_layers()
            .iter()
            .find(|l| l.id() == layer_id)
            .expect("the layer SHALL come back");
        assert_eq!(
            back.tracks().len(),
            4,
            "a day's tracks SHALL travel back with their layer"
        );
    }

    /// Renaming what the import called a path is the first thing anybody does.
    #[test]
    fn renaming_a_layer_through_the_application_is_undoable() {
        let mut state = AppState::new();
        let id = LayerId::new(
            state
                .create_track_layer("Imported tracks: /tmp/a.gpx".to_owned())
                .unwrap(),
        );

        state.rename_track_layer(id, "День 3".to_owned()).unwrap();
        let name_now = |s: &AppState| {
            s.track_layers()
                .iter()
                .find(|l| l.id() == id)
                .map(|l| l.name().to_owned())
                .unwrap()
        };
        assert_eq!(name_now(&state), "День 3");
        state.undo();
        assert_eq!(name_now(&state), "Imported tracks: /tmp/a.gpx");
    }

    /// `bigmap.ozf2` ends with `map.ozf2`. Matching the ready file against a
    /// package's name with `ends_with` therefore handed the wrong map a
    /// `local_path`, and the crew opened a layer they had not downloaded.
    /// External review, 2026-09-22.
    #[test]
    fn a_ready_file_lands_on_the_map_whose_name_it_actually_is() {
        let mut state = AppState::new();
        let mut project = sample_project_with_remote_map();
        project.maps[0].file_name = "map.ozf2".to_owned();
        project.maps.push(LizaMapPackage {
            name: "big".to_owned(),
            file_name: "bigmap.ozf2".to_owned(),
            url: "https://example.invalid/bigmap.ozf2".to_owned(),
            base_zoom: 12,
            local_path: None,
            size_bytes: None,
        });
        state.lizaalert.selected_project = Some(project);

        let landed = std::path::PathBuf::from("/tmp/bundle/bigmap.ozf2");
        state.note_bundle_file_ready("8-Android&iOS/bigmap.ozf2", &landed);

        let maps = &state.lizaalert.selected_project.as_ref().unwrap().maps;
        assert_eq!(
            maps[0].local_path, None,
            "`map.ozf2` SHALL NOT be marked ready by a file called `bigmap.ozf2`"
        );
        assert_eq!(maps[1].local_path, Some(landed));
    }

    /// A file directly in the bundle root, with no directory in front of it,
    /// still belongs to its package.
    #[test]
    fn a_ready_file_at_the_bundle_root_still_finds_its_map() {
        let mut state = AppState::new();
        let mut project = sample_project_with_remote_map();
        project.maps[0].file_name = "map.ozf2".to_owned();
        state.lizaalert.selected_project = Some(project);

        let landed = std::path::PathBuf::from("/tmp/bundle/map.ozf2");
        state.note_bundle_file_ready("map.ozf2", &landed);

        let maps = &state.lizaalert.selected_project.as_ref().unwrap().maps;
        assert_eq!(maps[0].local_path, Some(landed));
    }

    /// The launch-time walk finishes minutes after the operator started a
    /// download. It used to write "Loaded 412 projects" over the only line
    /// that says how far the download has got. The walk has the diagnostics
    /// log; the status bar belongs to the thing the crew is waiting on.
    /// External review, 2026-09-22.
    #[test]
    fn a_finished_walk_does_not_write_over_a_running_download() {
        let mut state = AppState::new();
        state.lizaalert.downloading.insert("demo-map".to_owned());
        state.apply_progress("Downloading demo-map: 40%".to_owned());

        state.apply_projects_loaded(Ok(lizaalert::CatalogueWalk {
            projects: Vec::new(),
            pages: 1,
            cancelled: false,
        }));

        assert_eq!(
            state.lizaalert.status, "Downloading demo-map: 40%",
            "the status bar SHALL keep reporting the download the crew is waiting on"
        );
        assert!(
            state
                .lizaalert
                .diagnostics
                .iter()
                .any(|entry| entry.message.contains("0 projects")),
            "the walk's result SHALL still reach the diagnostics log"
        );
        assert!(
            !state.lizaalert.listing_busy,
            "the walk SHALL still release its flag"
        );
    }

    /// With nothing downloading, the walk owns the line as before.
    #[test]
    fn a_finished_walk_reports_itself_when_nothing_is_downloading() {
        let mut state = AppState::new();
        state.apply_projects_loaded(Ok(lizaalert::CatalogueWalk {
            projects: Vec::new(),
            pages: 1,
            cancelled: false,
        }));
        assert!(state.lizaalert.status.contains("0 projects"));
    }

    /// An export that failed used to answer `Ok(())`, so the caller showed the
    /// success path and the only trace was a line in the status bar.
    #[test]
    fn a_failed_export_is_reported_to_the_caller() {
        let dir = temp_session_dir("export-failure");
        let mut state = AppState::new();

        let missing_layer = state.export_layer_to_gpx(LayerId::new(9999), dir.join("out.gpx"));
        assert!(missing_layer.is_err(), "a missing layer SHALL be an error");

        // A directory that does not exist cannot receive a file.
        let unwritable = dir.join("no-such-dir").join("out.gpx");
        let write_failure = state.export_layer_to_gpx(LayerId::new(1), unwritable);
        assert!(
            write_failure.is_err(),
            "a write failure SHALL reach the caller"
        );

        let missing_waypoint_layer =
            state.export_wpt_waypoints(LayerId::new(9999), dir.join("out.wpt"));
        assert!(missing_waypoint_layer.is_err());

        let wpt_write_failure =
            state.export_wpt_waypoints(LayerId::new(1), dir.join("no-such-dir").join("out.wpt"));
        assert!(wpt_write_failure.is_err());
    }

    #[test]
    fn a_successful_export_writes_the_file_and_answers_ok() {
        let dir = temp_session_dir("export-success");
        let mut state = AppState::new();
        let path = dir.join("tracks.gpx");

        assert!(
            state
                .export_layer_to_gpx(LayerId::new(1), path.clone())
                .is_ok()
        );
        assert!(path.exists(), "the GPX file SHALL be on disk");

        let wpt = dir.join("waypoints.wpt");
        assert!(
            state
                .export_wpt_waypoints(LayerId::new(1), wpt.clone())
                .is_ok()
        );
        assert!(wpt.exists(), "the WPT file SHALL be on disk");
    }

    /// The owner keeps bundles on an external disk. Choosing that folder had
    /// to be redone on every launch, and until it was, the app looked at the
    /// default folder and reported every downloaded bundle as missing.
    #[test]
    fn bundles_root_survives_a_restart() {
        let dir = temp_session_dir("session-bundles-root");
        let session_path = dir.join("session.json");
        let chosen = dir.join("External Disk").join("LizaAlert Maps");

        let mut first = AppState::new_with_paths(Some(session_path.clone()), dir.join("default"));
        first.set_bundles_root(chosen.clone());

        let second = AppState::new_with_paths(Some(session_path.clone()), dir.join("default"));
        assert_eq!(
            second.bundles_root(),
            chosen.as_path(),
            "the chosen bundles root SHALL outlive the process"
        );
    }

    /// Session files written before the field existed have no `bundles_root`.
    /// They must still restore — a parse failure would lose the project too.
    #[test]
    fn session_without_a_bundles_root_still_restores_and_keeps_the_injected_one() {
        let dir = temp_session_dir("session-bundles-root-legacy");
        let session_path = dir.join("session.json");
        let project_path = dir.join("search.ozp");
        persistence::save_project(&Project::untitled(), &project_path).expect("save project");
        std::fs::write(
            &session_path,
            format!(
                r#"{{"last_project_path":{:?},"active_map":null}}"#,
                project_path.display().to_string()
            ),
        )
        .expect("write legacy session");

        let injected = dir.join("injected-bundles");
        let state = AppState::new_with_paths(Some(session_path), injected.clone());

        assert_eq!(state.project_file_path(), Some(project_path.as_path()));
        assert_eq!(state.bundles_root(), injected.as_path());
    }

    #[test]
    fn session_restore_valid_restores_project_path_and_active_map() {
        let dir = temp_session_dir("session-restore-valid");
        let project_path = dir.join("search.ozp");
        let map_path = dir.join("demo-map.sqlitedb");
        let session_path = dir.join("session.json");
        persistence::save_project(&Project::untitled(), &project_path).expect("save project");
        std::fs::write(&map_path, b"sqlite placeholder").expect("save map placeholder");
        persistence::save_app_session(
            &PersistedAppSession {
                last_project_path: Some(project_path.clone()),
                bundles_root: None,
                active_map: Some(PersistedActiveMap {
                    kind: "sqlite".to_owned(),
                    project_name: "Demo Project".to_owned(),
                    package_name: "demo-map".to_owned(),
                    remote_url: "https://example.invalid/demo-map.sqlitedb".to_owned(),
                    local_path: map_path.clone(),
                    center_lat: 55.0,
                    center_lon: 37.0,
                    base_zoom: 12,
                }),
            },
            &session_path,
        )
        .expect("save session");

        let state = AppState::new_with_paths(Some(session_path), dir.join("bundles"));

        assert_eq!(state.project_file_path(), Some(project_path.as_path()));
        let active_map = state.active_map().expect("active map restored");
        assert_eq!(active_map.local_path, map_path);
        assert_eq!(active_map.package_name, "demo-map");
        // Session restore SHALL register the active map layer (parity with
        // the click-open path), so the frontend's `applyActiveMap` effect
        // has a registered tile source to fit-bounds against. Without this
        // assertion, regressions like the cold-start "viewport at zoom 0"
        // bug from `fix-redesign-functional-bugs` would slip through.
        assert!(
            state
                .project
                .map_layers()
                .iter()
                .any(|layer| layer.source_path() == Some(map_path.as_path())),
            "expected the restored active map's layer to be registered"
        );
    }

    #[test]
    fn session_restore_missing_degrades_to_fresh_state_with_warning() {
        let dir = temp_session_dir("session-restore-missing");
        let session_path = dir.join("session.json");
        persistence::save_app_session(
            &PersistedAppSession {
                last_project_path: Some(dir.join("missing.ozp")),
                bundles_root: None,
                active_map: Some(PersistedActiveMap {
                    kind: "sqlite".to_owned(),
                    project_name: "Missing Project".to_owned(),
                    package_name: "missing-map".to_owned(),
                    remote_url: "https://example.invalid/missing-map.sqlitedb".to_owned(),
                    local_path: dir.join("missing-map.sqlitedb"),
                    center_lat: 55.0,
                    center_lon: 37.0,
                    base_zoom: 12,
                }),
            },
            &session_path,
        )
        .expect("save session");

        let state = AppState::new_with_paths(Some(session_path), dir.join("bundles"));

        assert_eq!(state.project_file_path(), None);
        assert_eq!(state.active_map(), None);
        assert!(
            state
                .recent_diagnostics()
                .any(|entry| entry.level() == DiagnosticLevel::Error
                    && entry
                        .message()
                        .contains("Session restore skipped missing project")),
            "expected missing-project diagnostic"
        );
    }

    #[test]
    fn new_with_paths_uses_injected_bundles_root() {
        let dir = temp_session_dir("injected-bundles-root");
        let bundles_root = dir.join("Injected Bundles");

        let state = AppState::new_with_paths(None, bundles_root.clone());

        assert_eq!(state.bundles_root(), bundles_root.as_path());
    }

    #[test]
    fn new_with_paths_restores_session_from_injected_path() {
        let dir = temp_session_dir("injected-session-restore");
        let project_path = dir.join("mission.ozp");
        let session_path = dir.join("nested").join("session.json");
        persistence::save_project(&Project::untitled(), &project_path).expect("save project");
        persistence::save_app_session(
            &PersistedAppSession {
                last_project_path: Some(project_path.clone()),
                bundles_root: None,
                active_map: None,
            },
            &session_path,
        )
        .expect("save session");

        let state = AppState::new_with_paths(Some(session_path), dir.join("bundles"));

        assert_eq!(state.project_file_path(), Some(project_path.as_path()));
    }

    /// CJ-2: the laptop is switched on in a field camp and has to be showing a
    /// working map inside a minute.
    ///
    /// The project and the map are separate things on disk, and the session
    /// restored them in one run of early returns: a `.ozp` that had been moved,
    /// renamed or deleted aborted the whole restore, so the crew lost the
    /// raster as well — a blank screen with everything present on the disk
    /// underneath it. The map is the ground; the project is the work on it.
    #[test]
    fn a_missing_project_does_not_cost_the_map_as_well() {
        let dir = temp_session_dir("restore-missing-project");
        let map_path = dir.join("topo.sqlitedb");
        std::fs::write(&map_path, b"not really a tile store").expect("write map");
        let session_path = dir.join("session.json");

        persistence::save_app_session(
            &PersistedAppSession {
                // Never written, so it does not exist: the crew moved it.
                last_project_path: Some(dir.join("gone.ozp")),
                bundles_root: None,
                active_map: Some(persistence::PersistedActiveMap {
                    kind: "sqlite".to_owned(),
                    project_name: "2026-07-08 Lavrovo".to_owned(),
                    package_name: "topo".to_owned(),
                    remote_url: String::new(),
                    local_path: map_path.clone(),
                    center_lat: 59.95,
                    center_lon: 31.6,
                    base_zoom: 16,
                }),
            },
            &session_path,
        )
        .expect("save session");

        let state = AppState::new_with_paths(Some(session_path), dir.join("bundles"));

        let said: Vec<&str> = state
            .lizaalert
            .diagnostics
            .iter()
            .map(|entry| entry.message.as_str())
            .collect();
        assert!(
            said.iter().any(|m| m.contains("missing project")),
            "the missing project SHALL be reported: {said:?}"
        );
        assert!(
            said.iter().any(|m| m.to_lowercase().contains("active map")),
            "the restore SHALL go on to the map rather than stop at the project: {said:?}"
        );
    }

    /// The same, one step later: a project file that exists but cannot be read
    /// — a truncated save, or one written by a newer build — must not take the
    /// map down with it either.
    #[test]
    fn an_unreadable_project_does_not_cost_the_map_as_well() {
        let dir = temp_session_dir("restore-unreadable-project");
        let project_path = dir.join("broken.ozp");
        std::fs::write(&project_path, b"{ this is not a project").expect("write project");
        let map_path = dir.join("topo.sqlitedb");
        std::fs::write(&map_path, b"not really a tile store").expect("write map");
        let session_path = dir.join("session.json");

        persistence::save_app_session(
            &PersistedAppSession {
                last_project_path: Some(project_path),
                bundles_root: None,
                active_map: Some(persistence::PersistedActiveMap {
                    kind: "sqlite".to_owned(),
                    project_name: "2026-07-08 Lavrovo".to_owned(),
                    package_name: "topo".to_owned(),
                    remote_url: String::new(),
                    local_path: map_path,
                    center_lat: 59.95,
                    center_lon: 31.6,
                    base_zoom: 16,
                }),
            },
            &session_path,
        )
        .expect("save session");

        let state = AppState::new_with_paths(Some(session_path), dir.join("bundles"));

        let said: Vec<&str> = state
            .lizaalert
            .diagnostics
            .iter()
            .map(|entry| entry.message.as_str())
            .collect();
        assert!(
            said.iter().any(|m| m.contains("Restored active map")
                || m.contains("failed to register active map")),
            "an unreadable project SHALL NOT stop the restore reaching the map: {said:?}"
        );
    }

    #[test]
    fn new_with_paths_saves_session_to_injected_path() {
        let dir = temp_session_dir("injected-session-save");
        let session_path = dir.join("nested").join("session.json");
        let project_path = dir.join("mission.ozp");

        let mut state = AppState::new_with_paths(Some(session_path.clone()), dir.join("bundles"));
        state
            .save_project_to(project_path.clone())
            .expect("save project");

        let session = persistence::load_app_session(&session_path)
            .expect("read session file")
            .expect("session snapshot written on save");
        assert_eq!(session.last_project_path, Some(project_path));
    }

    #[test]
    fn apply_map_downloaded_marks_selected_project_map_as_cached() {
        let mut state = AppState::new();
        let project = sample_project_with_remote_map();
        let selection = ActiveMapSelection {
            kind: ActiveMapKind::SqliteTiles,
            project_name: project.summary.name.clone(),
            package_name: project.maps[0].name.clone(),
            remote_url: project.maps[0].url.clone(),
            local_path: PathBuf::from("/tmp/demo-map.sqlitedb"),
            center: project.center,
            base_zoom: project.maps[0].base_zoom,
        };

        state.lizaalert.selected_project = Some(project);
        state
            .lizaalert
            .downloading
            .insert(selection.package_name.clone());

        state.apply_map_downloaded(&selection.package_name, Ok(selection.clone()));

        let selected_project = state.lizaalert.selected_project.as_ref().expect("project");
        assert_eq!(
            selected_project.maps[0].local_path.as_deref(),
            Some(selection.local_path.as_path())
        );
        assert!(
            !state
                .lizaalert
                .downloading
                .contains(&selection.package_name)
        );
        assert_eq!(state.lizaalert.active_map.as_ref(), Some(&selection));
    }

    #[test]
    fn begin_open_map_returns_local_request_after_download_is_applied() {
        let mut state = AppState::new();
        let project = sample_project_with_remote_map();
        let selection = ActiveMapSelection {
            kind: ActiveMapKind::SqliteTiles,
            project_name: project.summary.name.clone(),
            package_name: project.maps[0].name.clone(),
            remote_url: project.maps[0].url.clone(),
            local_path: PathBuf::from("/tmp/demo-map.sqlitedb"),
            center: project.center,
            base_zoom: project.maps[0].base_zoom,
        };

        state.lizaalert.selected_project = Some(project);
        let first_request = state.begin_open_map(&selection.package_name);
        assert!(matches!(first_request, Some(OpenMapRequest::Download(_))));

        state
            .lizaalert
            .downloading
            .insert(selection.package_name.clone());
        state.apply_map_downloaded(&selection.package_name, Ok(selection.clone()));

        let second_request = state.begin_open_map(&selection.package_name);
        assert!(matches!(second_request, Some(OpenMapRequest::Local(_))));
    }

    #[test]
    fn toggle_waypoint_visible_flips_flag_and_is_not_undoable() {
        let mut state = AppState::new();
        let layer_id = LayerId::new(1);
        let waypoint_id = WaypointId::new(1);

        state
            .project
            .add_waypoint_to_layer(layer_id, Waypoint::new(waypoint_id, "Camp", 53.9, 27.5667))
            .unwrap();

        // A waypoint that has been added through the undoable add-waypoint
        // path would push an entry; here we add directly to the layer so
        // the stack starts empty. Use a real undoable mutation to make
        // the test meaningful.
        state
            .apply_set_waypoint_symbol(layer_id, waypoint_id, Some("camp".to_owned()))
            .unwrap();
        let undo_depth_before_toggle = state.history.undo_depth();
        assert_eq!(undo_depth_before_toggle, 1);

        // Default is visible == true.
        assert!(state.project.waypoint_layers()[0].waypoints()[0].visible());

        state.toggle_waypoint_visible(layer_id, waypoint_id);
        assert!(!state.project.waypoint_layers()[0].waypoints()[0].visible());
        assert_eq!(
            state.history.undo_depth(),
            undo_depth_before_toggle,
            "toggle_waypoint_visible SHALL NOT push an undo entry"
        );

        state.toggle_waypoint_visible(layer_id, waypoint_id);
        assert!(state.project.waypoint_layers()[0].waypoints()[0].visible());
        assert_eq!(
            state.history.undo_depth(),
            undo_depth_before_toggle,
            "toggle_waypoint_visible (second flip) SHALL NOT push an undo entry"
        );

        // Undo SHALL revert the symbol change (not the visibility toggles).
        state.undo();
        assert_eq!(
            state.project.waypoint_layers()[0].waypoints()[0].symbol(),
            None
        );
        // Visibility stays as last toggled.
        assert!(state.project.waypoint_layers()[0].waypoints()[0].visible());
    }

    /// Bulk visibility has to behave like the per-waypoint toggle: it marks
    /// the project dirty and leaves undo alone.
    #[test]
    fn bulk_waypoint_visibility_is_a_style_mutation() {
        let mut state = AppState::new();
        let layer_id = LayerId::new(1);
        for (id, name) in [(1u64, "Camp"), (2, "Task"), (3, "Find")] {
            state
                .project
                .add_waypoint_to_layer(
                    layer_id,
                    Waypoint::new(WaypointId::new(id), name, 53.9, 27.5667),
                )
                .unwrap();
        }
        let undo_depth_before = state.history.undo_depth();

        state.set_all_waypoints_visible(false);
        assert!(
            state.project.waypoint_layers()[0]
                .waypoints()
                .iter()
                .all(|w| !w.visible())
        );

        assert!(state.show_only_waypoint(layer_id, WaypointId::new(2)));
        let visible: Vec<&str> = state.project.waypoint_layers()[0]
            .waypoints()
            .iter()
            .filter(|w| w.visible())
            .map(|w| w.name())
            .collect();
        assert_eq!(visible, vec!["Task"]);

        assert_eq!(
            state.history.undo_depth(),
            undo_depth_before,
            "bulk waypoint visibility SHALL NOT push undo entries"
        );
        assert!(
            state.project_dirty(),
            "a style mutation marks the project dirty"
        );
    }

    #[test]
    fn show_only_waypoint_leaves_visibility_alone_when_the_waypoint_is_gone() {
        let mut state = AppState::new();
        let layer_id = LayerId::new(1);
        state
            .project
            .add_waypoint_to_layer(
                layer_id,
                Waypoint::new(WaypointId::new(1), "Camp", 53.9, 27.5),
            )
            .unwrap();

        assert!(!state.show_only_waypoint(layer_id, WaypointId::new(99)));
        assert!(state.project.waypoint_layers()[0].waypoints()[0].visible());
    }

    #[test]
    fn toggle_waypoint_visible_is_a_no_op_for_missing_waypoint() {
        let mut state = AppState::new();
        let layer_id = LayerId::new(1);
        state
            .project
            .add_waypoint_to_layer(
                layer_id,
                Waypoint::new(WaypointId::new(1), "Camp", 53.9, 27.5667),
            )
            .unwrap();

        let before = state.project.clone();
        state.toggle_waypoint_visible(layer_id, WaypointId::new(999));
        assert_eq!(state.project, before);

        state.toggle_waypoint_visible(LayerId::new(99), WaypointId::new(1));
        assert_eq!(state.project, before);
    }

    /// Colour is the other axis a search map is read on. The symbol says what
    /// a mark is; the colour says whose it is — group A's marks against group
    /// B's, on one map, at night, on a laptop. Tracks have had per-track
    /// colour since the beginning; waypoints have not.
    #[test]
    fn waypoint_colour_undo_restores_the_previous_colour_and_redo_reapplies() {
        let mut state = AppState::new();
        let layer_id = LayerId::new(1);
        let waypoint_id = WaypointId::new(1);

        state
            .project
            .add_waypoint_to_layer(layer_id, Waypoint::new(waypoint_id, "ШТАБ", 53.9, 27.5667))
            .unwrap();
        assert_eq!(
            state.project.waypoint_layers()[0].waypoints()[0].color(),
            None,
            "a waypoint starts with no colour of its own"
        );

        state
            .apply_set_waypoint_color(layer_id, waypoint_id, Some([37, 99, 235, 255]))
            .unwrap();
        state
            .apply_set_waypoint_color(layer_id, waypoint_id, Some([220, 38, 38, 255]))
            .unwrap();

        let colour = || state.project.waypoint_layers()[0].waypoints()[0].color();
        assert_eq!(colour(), Some([220, 38, 38, 255]));

        state.undo();
        assert_eq!(
            state.project.waypoint_layers()[0].waypoints()[0].color(),
            Some([37, 99, 235, 255])
        );

        state.redo();
        assert_eq!(
            state.project.waypoint_layers()[0].waypoints()[0].color(),
            Some([220, 38, 38, 255])
        );

        // Back to the default, which is not the same as "some default colour".
        state
            .apply_set_waypoint_color(layer_id, waypoint_id, None)
            .unwrap();
        assert_eq!(
            state.project.waypoint_layers()[0].waypoints()[0].color(),
            None
        );
        state.undo();
        assert_eq!(
            state.project.waypoint_layers()[0].waypoints()[0].color(),
            Some([220, 38, 38, 255])
        );
    }

    #[test]
    fn waypoint_symbol_undo_restores_previous_symbol_and_redo_reapplies_new_symbol() {
        let mut state = AppState::new();
        let layer_id = LayerId::new(1);
        let waypoint_id = WaypointId::new(1);

        state
            .project
            .add_waypoint_to_layer(layer_id, Waypoint::new(waypoint_id, "Camp", 53.9, 27.5667))
            .unwrap();
        state
            .project
            .set_waypoint_symbol_in_layer(layer_id, waypoint_id, Some("flag".to_owned()))
            .unwrap();

        state
            .apply_set_waypoint_symbol(layer_id, waypoint_id, Some("camp".to_owned()))
            .unwrap();

        let waypoint = &state.project.waypoint_layers()[0].waypoints()[0];
        assert_eq!(waypoint.symbol(), Some("camp"));

        state.undo();
        let waypoint = &state.project.waypoint_layers()[0].waypoints()[0];
        assert_eq!(waypoint.symbol(), Some("flag"));

        state.redo();
        let waypoint = &state.project.waypoint_layers()[0].waypoints()[0];
        assert_eq!(waypoint.symbol(), Some("camp"));
    }

    #[test]
    fn waypoint_symbol_missing_returns_error_without_mutating_state() {
        let mut state = AppState::new();
        let layer_id = LayerId::new(1);
        let waypoint_id = WaypointId::new(1);

        state
            .project
            .add_waypoint_to_layer(layer_id, Waypoint::new(waypoint_id, "Camp", 53.9, 27.5667))
            .unwrap();
        let before = state.project.clone();

        let missing_waypoint_error = state
            .apply_set_waypoint_symbol(layer_id, WaypointId::new(99), Some("camp".to_owned()))
            .unwrap_err();
        assert_eq!(
            missing_waypoint_error,
            ProjectLayerError::WaypointNotFound(layer_id, WaypointId::new(99))
        );
        assert_eq!(state.project, before);

        let missing_layer_error = state
            .apply_set_waypoint_symbol(LayerId::new(99), waypoint_id, Some("camp".to_owned()))
            .unwrap_err();
        assert_eq!(
            missing_layer_error,
            ProjectLayerError::WaypointLayerUnavailable(LayerId::new(99))
        );
        assert_eq!(state.project, before);
    }

    #[test]
    fn export_default_tracks_dir_uses_active_bundle_for_gpx() {
        let mut state = AppState::new();
        let project = sample_project_with_remote_map();
        state.lizaalert.selected_project = Some(project.clone());
        state.lizaalert.active_map = Some(ActiveMapSelection {
            kind: ActiveMapKind::SqliteTiles,
            project_name: project.summary.name.clone(),
            package_name: project.maps[0].name.clone(),
            remote_url: project.maps[0].url.clone(),
            local_path: PathBuf::from("/tmp/demo-map.sqlitedb"),
            center: project.center,
            base_zoom: project.maps[0].base_zoom,
        });

        let path = state
            .export_default_tracks_dir_path("20240601_Иванов", "gpx")
            .expect("active bundle default path");
        let components: Vec<_> = path
            .components()
            .map(|component| component.as_os_str().to_owned())
            .collect();

        assert!(components.ends_with(&[
            "demo-project".into(),
            "10-Tracks".into(),
            "20240601_Иванов.gpx".into(),
        ]));
    }

    #[test]
    fn export_default_tracks_dir_uses_active_bundle_for_plt() {
        let mut state = AppState::new();
        let project = sample_project_with_remote_map();
        state.lizaalert.selected_project = Some(project.clone());
        state.lizaalert.active_map = Some(ActiveMapSelection {
            kind: ActiveMapKind::SqliteTiles,
            project_name: project.summary.name.clone(),
            package_name: project.maps[0].name.clone(),
            remote_url: project.maps[0].url.clone(),
            local_path: PathBuf::from("/tmp/demo-map.sqlitedb"),
            center: project.center,
            base_zoom: project.maps[0].base_zoom,
        });

        let path = state
            .export_default_tracks_dir_path("20240601_Иванов", ".plt")
            .expect("active bundle default path");
        let file_name = path.file_name().and_then(|name| name.to_str());

        assert_eq!(file_name, Some("20240601_Иванов.plt"));
        assert_eq!(
            path.parent().and_then(|dir| dir.file_name()),
            Some("10-Tracks".as_ref())
        );
    }

    #[test]
    fn export_default_tracks_dir_is_none_without_active_bundle() {
        let state = AppState::new();

        assert_eq!(state.export_default_tracks_dir_path("Track 1", "gpx"), None);
    }

    #[test]
    fn export_wpt_default_path_uses_active_bundle_when_present() {
        let mut state = AppState::new();
        let project = sample_project_with_remote_map();
        state.lizaalert.selected_project = Some(project.clone());
        state.lizaalert.active_map = Some(ActiveMapSelection {
            kind: ActiveMapKind::SqliteTiles,
            project_name: project.summary.name.clone(),
            package_name: project.maps[0].name.clone(),
            remote_url: project.maps[0].url.clone(),
            local_path: PathBuf::from("/tmp/demo-map.sqlitedb"),
            center: project.center,
            base_zoom: project.maps[0].base_zoom,
        });

        let layer_id = LayerId::new(1);
        // Default waypoint layer "Waypoints" is created in AppState::new()
        let path = state
            .export_wpt_default_path(layer_id)
            .expect("default path returned");

        let file_name = path.file_name().and_then(|name| name.to_str());
        assert_eq!(file_name, Some("Waypoints.wpt"));
        // Parent directory should be the bundle (named after slug "demo-project").
        let parent = path.parent().and_then(|dir| dir.file_name());
        assert_eq!(parent.and_then(|s| s.to_str()), Some("demo-project"));
    }

    #[test]
    fn export_wpt_default_path_without_bundle_returns_filename_only() {
        let state = AppState::new();
        let layer_id = LayerId::new(1);

        let path = state
            .export_wpt_default_path(layer_id)
            .expect("default path returned");

        assert_eq!(path, PathBuf::from("Waypoints.wpt"));
        assert!(
            path.parent()
                .map(|p| p.as_os_str().is_empty())
                .unwrap_or(true)
        );
    }

    #[test]
    fn export_wpt_default_path_is_none_for_unknown_layer() {
        let state = AppState::new();
        assert!(state.export_wpt_default_path(LayerId::new(999)).is_none());
    }

    #[test]
    fn export_default_tracks_dir_recovers_bundle_from_restored_mobile_map_path() {
        let mut state = AppState::new();
        state.lizaalert.active_map = Some(ActiveMapSelection {
            kind: ActiveMapKind::SqliteTiles,
            project_name: "Restored Project".to_owned(),
            package_name: "Restored Map".to_owned(),
            remote_url: "https://example.invalid/restored.sqlitedb".to_owned(),
            local_path: state
                .bundles_root
                .join("restored-project")
                .join("8-Android&iOS")
                .join("restored.sqlitedb"),
            center: MapCenter {
                lat: 55.0,
                lon: 37.0,
            },
            base_zoom: 12,
        });

        let path = state
            .export_default_tracks_dir_path("20240601_Test", "gpx")
            .expect("restored active map default path");

        assert!(
            path.ends_with(
                PathBuf::from("restored-project")
                    .join("10-Tracks")
                    .join("20240601_Test.gpx")
            )
        );
    }

    // ── Datum warning on map open ──

    fn write_temp_ozi_map(dir: &Path, datum_name: &str) -> PathBuf {
        let path = dir.join("calibration.map");
        let contents = format!(
            "OziExplorer Map Data File Version 2.2\nField calibration\nmaps/base.ozf2\n1 ,Map Code,\n{datum_name},,   0.0000,   0.0000,WGS 84\nReserved 1\nReserved 2\nMagnetic Variation,,,E\nMap Projection,Mercator,PolyCal,No,AutoCalOnly,No,BSBUseWPX,No\nPoint01,xy,100,200,in, deg,54,30.000,N,48,24.000,E, grid, , , ,N\nPoint02,xy,300,400,in, deg,54,31.000,N,48,25.000,E, grid, , , ,N\n"
        );
        std::fs::write(&path, contents).expect("write temp ozi map");
        path
    }

    #[test]
    fn open_local_ozi_map_warns_about_non_wgs84_datum() {
        let dir = temp_session_dir("datum-warning");
        let map_path = write_temp_ozi_map(&dir, "Pulkovo 1942");

        let mut state = AppState::new();
        state.open_local_ozi_map(&map_path).expect("open ozi map");

        let warning = state
            .recent_diagnostics()
            .find(|entry| entry.message().contains("Pulkovo 1942"));
        assert!(
            warning.is_some(),
            "expected datum warning diagnostic, got: {:?}",
            state.recent_diagnostics().collect::<Vec<_>>()
        );
        assert_eq!(
            warning.map(DiagnosticEntry::level),
            Some(DiagnosticLevel::Warning)
        );
    }

    #[test]
    fn datum_shift_warning_accepts_wgs84_family_spellings() {
        assert!(datum_shift_warning("WGS 84").is_none());
        assert!(datum_shift_warning("WGS84").is_none());
        assert!(datum_shift_warning("wgs-84").is_none());
        assert!(datum_shift_warning("Pulkovo 1942").is_some());
        assert!(datum_shift_warning("WGS 72").is_some());
    }

    #[test]
    fn open_local_ozi_map_does_not_warn_for_wgs84_datum() {
        let dir = temp_session_dir("datum-no-warning");
        let map_path = write_temp_ozi_map(&dir, "WGS 84");

        let mut state = AppState::new();
        state.open_local_ozi_map(&map_path).expect("open ozi map");

        assert!(
            state
                .recent_diagnostics()
                .all(|entry| !entry.message().contains("датум")),
            "unexpected datum warning for WGS 84 map: {:?}",
            state.recent_diagnostics().collect::<Vec<_>>()
        );
    }
}
