use crate::application::{
    ActiveMapKind, ActiveMapSelection, BundleEntry, LizaMapPackage, LizaProject,
    LizaProjectSummary, MapCenter,
};
use crate::infrastructure::import::{
    ArchiveEntryKind, SupportedArchiveEntryKind, extract_zip_entries_to_directory,
    inventory_zip_entries, parse_ozi_map_metadata, read_ozi_map_text,
};
use regex::Regex;
use reqwest::blocking::Client;
use std::fs::{self, File};
use std::io::{BufReader, Read, Write};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use tokio::sync::{Semaphore, mpsc};
use tokio::task::JoinSet;

const ROOT_URL: &str = "https://maps.lizaalert.ru/maps/";
const MOBILE_MAPS_DIR_NAME: &str = "8-Android&iOS";
const COORDINATES_FILE_NAME: &str = "2-Coordinates.txt";
const PROJECT_EXTRACTED_DIR: &str = "extracted";
const PROJECTS_CACHE_FILE_NAME: &str = "projects-cache.json";
const PROJECTS_CACHE_VERSION: u8 = 1;

#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct ProjectsCacheFile {
    version: u8,
    projects: Vec<LizaProjectSummaryCacheDto>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct LizaProjectSummaryCacheDto {
    slug: String,
    name: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DownloadProgress {
    pub downloaded_bytes: u64,
    pub total_bytes: Option<u64>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProjectOpenPhase {
    Scanning,
    Downloading,
    Extracting,
    Indexing,
}

impl ProjectOpenPhase {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Scanning => "scanning",
            Self::Downloading => "downloading",
            Self::Extracting => "extracting",
            Self::Indexing => "indexing",
        }
    }
}

/// What a progress message says, as something the interface can translate.
///
/// These sentences are built in Rust and reach the status bar verbatim, so
/// while they were plain strings they were English in a Russian window — the
/// one part of the download a crew watches that never spoke their language.
///
/// Each variant carries a key and its arguments in order; `english` is the
/// wording a diagnostic records, and what the interface falls back to when it
/// has no translation for the key.
#[derive(Debug, Clone)]
pub enum ProgressText {
    ScanningDirectory {
        path: String,
    },
    DownloadingBundle {
        name: String,
    },
    OpeningCachedBundle {
        name: String,
    },
    DownloadingInParallel {
        total: usize,
    },
    DownloadedOfFiles {
        completed: usize,
        total: usize,
    },
    DownloadedFiles {
        total: usize,
    },
    RetryingFile {
        name: String,
        attempt: usize,
        total: usize,
    },
    ExtractingOziIn {
        name: String,
    },
    ExtractingCachedOzi {
        name: String,
    },
    ExtractingInParallel {
        count: usize,
        names: String,
    },
    ExtractedOziArchives {
        count: usize,
    },
    IndexingMapsIn {
        name: String,
    },
    IndexingCachedMaps {
        name: String,
    },
}

impl ProgressText {
    pub fn key(&self) -> &'static str {
        match self {
            Self::ScanningDirectory { .. } => "progress.scanningDirectory",
            Self::DownloadingBundle { .. } => "progress.downloadingBundle",
            Self::OpeningCachedBundle { .. } => "progress.openingCachedBundle",
            Self::DownloadingInParallel { .. } => "progress.downloadingInParallel",
            Self::DownloadedOfFiles { .. } => "progress.downloadedOfFiles",
            Self::DownloadedFiles { .. } => "progress.downloadedFiles",
            Self::RetryingFile { .. } => "progress.retryingFile",
            Self::ExtractingOziIn { .. } => "progress.extractingOziIn",
            Self::ExtractingCachedOzi { .. } => "progress.extractingCachedOzi",
            Self::ExtractingInParallel { .. } => "progress.extractingInParallel",
            Self::ExtractedOziArchives { .. } => "progress.extractedOziArchives",
            Self::IndexingMapsIn { .. } => "progress.indexingMapsIn",
            Self::IndexingCachedMaps { .. } => "progress.indexingCachedMaps",
        }
    }

    pub fn args(&self) -> Vec<String> {
        match self {
            Self::ScanningDirectory { path } => vec![path.clone()],
            Self::DownloadingBundle { name }
            | Self::OpeningCachedBundle { name }
            | Self::ExtractingOziIn { name }
            | Self::ExtractingCachedOzi { name }
            | Self::IndexingMapsIn { name }
            | Self::IndexingCachedMaps { name } => vec![name.clone()],
            Self::DownloadingInParallel { total } | Self::DownloadedFiles { total } => {
                vec![total.to_string()]
            }
            Self::DownloadedOfFiles { completed, total } => {
                vec![completed.to_string(), total.to_string()]
            }
            Self::RetryingFile {
                name,
                attempt,
                total,
            } => vec![name.clone(), attempt.to_string(), total.to_string()],
            Self::ExtractingInParallel { count, names } => {
                vec![count.to_string(), names.clone()]
            }
            Self::ExtractedOziArchives { count } => vec![count.to_string()],
        }
    }

    pub fn english(&self) -> String {
        match self {
            Self::ScanningDirectory { path } => format!("Scanning {path}"),
            Self::DownloadingBundle { name } => format!("Downloading project bundle: {name}"),
            Self::OpeningCachedBundle { name } => {
                format!("Opening cached project bundle: {name}")
            }
            Self::DownloadingInParallel { total } => {
                format!("Downloading {total} files in parallel")
            }
            Self::DownloadedOfFiles { completed, total } => {
                format!("Downloaded {completed} of {total} files")
            }
            Self::DownloadedFiles { total } => format!("Downloaded {total} files"),
            Self::RetryingFile {
                name,
                attempt,
                total,
            } => format!("Retrying {name} (attempt {attempt} of {total})"),
            Self::ExtractingOziIn { name } => format!("Extracting OZI archives in: {name}"),
            Self::ExtractingCachedOzi { name } => {
                format!("Extracting cached OZI bundles: {name}")
            }
            Self::ExtractingInParallel { count, names } => {
                format!("Extracting {count} in parallel: {names}")
            }
            Self::ExtractedOziArchives { count } => format!("Extracted {count} OZI archives"),
            Self::IndexingMapsIn { name } => format!("Indexing maps in: {name}"),
            Self::IndexingCachedMaps { name } => format!("Indexing cached project maps: {name}"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ProjectOpenProgress {
    pub text: ProgressText,
    pub phase: ProjectOpenPhase,
    pub completed: Option<u64>,
    pub total: Option<u64>,
    pub downloaded_bytes: Option<u64>,
    pub total_bytes: Option<u64>,
}

impl ProjectOpenProgress {
    fn status(text: ProgressText, phase: ProjectOpenPhase) -> Self {
        Self {
            text,
            phase,
            completed: None,
            total: None,
            downloaded_bytes: None,
            total_bytes: None,
        }
    }

    /// The English wording, for diagnostics and as the interface's fallback.
    pub fn message(&self) -> String {
        self.text.english()
    }
}

#[derive(Debug, Clone)]
struct RemoteFileDownload {
    url: String,
    path: PathBuf,
    /// Path relative to the project source root (used for `package_name`).
    relative: String,
    /// Size from the listing, when it states one.
    size_bytes: Option<u64>,
}

/// Default upper bound on concurrent per-file downloads inside a bundle.
pub const DEFAULT_BUNDLE_DOWNLOAD_CONCURRENCY: usize = 6;

/// Per-file notifications emitted by the multi-file download path.
#[derive(Debug, Clone)]
pub enum DownloadNotification {
    /// Phase update for the overall bundle (Scanning, Downloading, Extracting, Indexing).
    Phase(ProjectOpenProgress),
    /// Per-file progress; emitted at least once per file plus on each chunk.
    FileProgress {
        package_name: String,
        downloaded_bytes: u64,
        total_bytes: Option<u64>,
        file_index: usize,
        file_count: usize,
    },
    /// A file has been fully downloaded and fsync'd to its final path.
    FileReady {
        package_name: String,
        local_path: PathBuf,
        file_index: usize,
        file_count: usize,
    },
}

/// Cooperative cancellation token shared by the orchestrator and its workers.
#[derive(Debug, Clone, Default)]
pub struct CancelToken {
    flag: Arc<AtomicBool>,
}

impl CancelToken {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn cancel(&self) {
        self.flag.store(true, Ordering::SeqCst);
    }

    pub fn is_cancelled(&self) -> bool {
        self.flag.load(Ordering::SeqCst)
    }
}

/// Error returned when a download was aborted via [`CancelToken`].
pub const CANCEL_ERROR: &str = "download cancelled";

/// Sort a list of remote file descriptors by their leading numeric prefix.
///
/// Files whose top-level directory or basename starts with `<digits>-` are
/// ordered by the numeric value of those digits. Files without such a prefix
/// are placed after all prefixed ones, sorted lexicographically.
fn sort_remote_files_by_prefix(files: &mut [RemoteFileDownload]) {
    files.sort_by(|a, b| {
        let (pa, ra) = leading_prefix(&a.relative);
        let (pb, rb) = leading_prefix(&b.relative);
        match (pa, pb) {
            (Some(x), Some(y)) => x.cmp(&y).then_with(|| ra.cmp(rb)),
            (Some(_), None) => std::cmp::Ordering::Less,
            (None, Some(_)) => std::cmp::Ordering::Greater,
            (None, None) => a.relative.cmp(&b.relative),
        }
    });
}

/// Extract the leading numeric prefix from a path component such as
/// `10-Tracks/foo.gpx` → (Some(10), "10-Tracks/foo.gpx").
fn leading_prefix(rel: &str) -> (Option<u32>, &str) {
    let head = rel.split('/').next().unwrap_or(rel);
    let digits: String = head.chars().take_while(|c| c.is_ascii_digit()).collect();
    if digits.is_empty() {
        return (None, rel);
    }
    let rest = &head[digits.len()..];
    if rest.starts_with('-') || rest.starts_with('_') || rest.is_empty() {
        digits
            .parse::<u32>()
            .ok()
            .map_or((None, rel), |p| (Some(p), rel))
    } else {
        (None, rel)
    }
}

/// Read one page of the root listing: the dated project directories on it and
/// the URL of the next page, when the catalogue continues.
fn parse_project_listing(
    base_url: &str,
    html: &str,
) -> Result<(Vec<LizaProjectSummary>, Option<String>), String> {
    let link_regex = Regex::new(r#"href="([^"]+)""#).map_err(|err| err.to_string())?;
    let project_regex = Regex::new(r"^\d{4}-\d{2}-\d{2}_.+$").map_err(|err| err.to_string())?;

    let mut projects = Vec::new();
    let mut next_page = None;
    let mut seen: Vec<String> = Vec::new();

    for captures in link_regex.captures_iter(html) {
        let Some(raw_href) = captures.get(1).map(|m| m.as_str()) else {
            continue;
        };
        let href = html_unescape(raw_href);

        // The pagination cursor points at the listing itself with a query, so
        // it is not a child entry and `resolve_listing_href` rejects it.
        if next_page.is_none()
            && let Some(query) = href.split_once("?after=").map(|(_, cursor)| cursor)
        {
            next_page = Some(format!("{base_url}?after={query}"));
        }

        let Some(url) = resolve_listing_href(base_url, raw_href) else {
            continue;
        };
        if !url.ends_with('/') {
            continue;
        }
        let Some(slug) = decode_entry_name(&url) else {
            continue;
        };
        if !project_regex.is_match(&slug) || seen.contains(&slug) {
            continue;
        }
        seen.push(slug.clone());
        projects.push(LizaProjectSummary {
            name: slug.replace('_', " "),
            slug,
            url,
        });
    }

    Ok((projects, next_page))
}

/// What one walk of the catalogue listing read.
///
/// `cancelled` is the part that matters to the caller: a stopped walk is not a
/// failure, but the projects it holds are only the pages it got through, so
/// they must not be written over the cache as if they were the whole
/// catalogue.
#[derive(Debug, Default)]
pub struct CatalogueWalk {
    pub projects: Vec<LizaProjectSummary>,
    pub pages: usize,
    pub cancelled: bool,
}

/// Walk the paginated root listing, reporting each page as it arrives.
///
/// The catalogue is served a page at a time with an opaque cursor; stopping at
/// the first page silently truncates it to the newest few dozen projects.
///
/// The walk is cancellable because it is long: a thousand pages at worst, and
/// it holds the application busy for its whole length. A crew that needs a
/// bundle now has to be able to stop waiting for it.
pub fn fetch_project_summaries_streaming<F>(
    cancel: &CancelToken,
    on_chunk: F,
) -> Result<CatalogueWalk, String>
where
    F: FnMut(Vec<LizaProjectSummary>, usize),
{
    walk_project_listing(ROOT_URL, cancel, on_chunk)
}

/// The walk itself, against a given listing root so it can be driven by a test
/// server rather than the live catalogue.
fn walk_project_listing<F>(
    root_url: &str,
    cancel: &CancelToken,
    mut on_chunk: F,
) -> Result<CatalogueWalk, String>
where
    F: FnMut(Vec<LizaProjectSummary>, usize),
{
    /// Guard against a server that keeps handing out cursors.
    const MAX_PAGES: usize = 1000;

    let mut walk = CatalogueWalk::default();
    let mut page_url = root_url.to_owned();

    loop {
        if cancel.is_cancelled() {
            walk.cancelled = true;
            break;
        }

        let html = fetch_text(&page_url)?;
        let (page_projects, next_page) = parse_project_listing(root_url, &html)?;

        walk.pages += 1;
        if !page_projects.is_empty() {
            walk.projects.extend(page_projects.iter().cloned());
            on_chunk(page_projects, walk.pages);
        }

        // Checked again here so that stopping during the last page is reported
        // as a stop rather than as a complete walk.
        if cancel.is_cancelled() {
            walk.cancelled = true;
            break;
        }

        match next_page {
            Some(next) if walk.pages < MAX_PAGES && next != page_url => page_url = next,
            _ => break,
        }
    }

    Ok(walk)
}

pub fn load_project_summaries_cache(root: &Path) -> Result<Vec<LizaProjectSummary>, String> {
    let cache_path = root.join(PROJECTS_CACHE_FILE_NAME);
    if !cache_path.exists() {
        return Ok(Vec::new());
    }

    let cache_text = fs::read_to_string(&cache_path).map_err(|err| err.to_string())?;
    let cache: ProjectsCacheFile =
        serde_json::from_str(&cache_text).map_err(|err| err.to_string())?;

    if cache.version != PROJECTS_CACHE_VERSION {
        return Err(format!(
            "Unsupported projects cache version: {}",
            cache.version
        ));
    }

    Ok(cache
        .projects
        .into_iter()
        .map(|project| LizaProjectSummary {
            slug: project.slug.clone(),
            name: project.name,
            url: format!("{ROOT_URL}{}/", project.slug),
        })
        .collect())
}

pub fn save_project_summaries_cache(
    root: &Path,
    projects: &[LizaProjectSummary],
) -> Result<(), String> {
    fs::create_dir_all(root).map_err(|err| err.to_string())?;

    let cache = ProjectsCacheFile {
        version: PROJECTS_CACHE_VERSION,
        projects: projects
            .iter()
            .map(|project| LizaProjectSummaryCacheDto {
                slug: project.slug.clone(),
                name: project.name.clone(),
            })
            .collect(),
    };

    let cache_text = serde_json::to_string(&cache).map_err(|err| err.to_string())?;
    fs::write(root.join(PROJECTS_CACHE_FILE_NAME), cache_text).map_err(|err| err.to_string())
}

/// The slugs of every bundle already on disk under `root`.
///
/// Computed once per catalogue read rather than asking per project: the
/// catalogue is about thirteen thousand entries and the bundles root holds a
/// handful of directories, so one `read_dir` answers for all of them.
pub fn cached_project_slugs(root: &Path) -> std::collections::HashSet<String> {
    let Ok(entries) = fs::read_dir(root) else {
        return std::collections::HashSet::new();
    };

    entries
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.path().is_dir())
        .filter_map(|entry| {
            let slug = entry.file_name().to_str()?.to_owned();
            is_project_cached(&slug, root).then_some(slug)
        })
        .collect()
}

pub fn is_project_cached(project_slug: &str, root: &Path) -> bool {
    project_coordinates_path(root, project_slug).exists()
}

/// Open an arbitrary local LizaAlert bundle directory.
///
/// The directory must be named like `YYYY-MM-DD-Place` and contain `2-Coordinates.txt`,
/// `8-Android&iOS/`, and OZI map files/archives directly inside (no `source/` subdir).
/// A `10-Tracks/` subfolder is created if absent.
pub fn open_bundle_directory<F>(dir: &Path, mut on_progress: F) -> Result<LizaProject, String>
where
    F: FnMut(ProjectOpenProgress),
{
    let slug = dir
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("bundle")
        .to_owned();
    let root = dir.parent().unwrap_or(dir);
    let name = slug.replace('_', " ");
    let summary = LizaProjectSummary {
        slug: slug.clone(),
        name: name.clone(),
        url: String::new(),
    };

    on_progress(ProjectOpenProgress::status(
        ProgressText::ExtractingOziIn { name: name.clone() },
        ProjectOpenPhase::Extracting,
    ));
    materialize_cached_ozi_archives(root, &slug, &mut on_progress)?;

    on_progress(ProjectOpenProgress::status(
        ProgressText::IndexingMapsIn { name: name.clone() },
        ProjectOpenPhase::Indexing,
    ));
    let project = load_cached_project_from_root(summary, root)?;
    ensure_tracks_dir(root, &slug);
    Ok(project)
}

/// Return the root directory of the bundle for the given slug.
pub fn bundle_directory(bundles_root: &Path, project_slug: &str) -> PathBuf {
    bundles_root.join(project_slug)
}

fn ensure_tracks_dir(root: &Path, project_slug: &str) {
    let tracks_dir = project_source_root(root, project_slug).join("10-Tracks");
    let _ = fs::create_dir_all(tracks_dir);
}

/// Download one map package, with cooperative cancellation.
///
/// A single-map download used to have no token at all, so a row that had
/// started downloading could not be stopped and the UI disabled it for the
/// duration. The token is checked between chunks; cancelling removes the
/// partial file.
pub fn download_map<F>(
    selection: ActiveMapSelection,
    cancel: &CancelToken,
    mut on_progress: F,
) -> Result<ActiveMapSelection, String>
where
    F: FnMut(DownloadProgress),
{
    let client = client()?;
    let mut response = client
        .get(&selection.remote_url)
        .send()
        .and_then(|response| response.error_for_status())
        .map_err(|err| err.to_string())?;

    if let Some(parent) = selection.local_path.parent() {
        fs::create_dir_all(parent).map_err(|err| err.to_string())?;
    }

    // Stream into a sibling `.part` file and atomically rename on success
    // (mirrors `download_to_path_async`) so an interrupted download never
    // leaves a truncated file at the canonical path — the cached-map listing
    // treats any file at that path as a fully downloaded map.
    let tmp_path = selection.local_path.with_extension("part");
    if let Err(err) = stream_response_to_file(&mut response, &tmp_path, cancel, &mut on_progress) {
        let _ = fs::remove_file(&tmp_path);
        return Err(err);
    }
    if let Err(err) = fs::rename(&tmp_path, &selection.local_path) {
        let _ = fs::remove_file(&tmp_path);
        return Err(err.to_string());
    }

    Ok(selection)
}

fn stream_response_to_file<F>(
    response: &mut reqwest::blocking::Response,
    path: &Path,
    cancel: &CancelToken,
    on_progress: &mut F,
) -> Result<(), String>
where
    F: FnMut(DownloadProgress),
{
    let mut file = File::create(path).map_err(|err| err.to_string())?;
    let total_bytes = response.content_length();
    let mut downloaded_bytes = 0u64;
    let mut buffer = [0u8; 16 * 1024];

    loop {
        if cancel.is_cancelled() {
            return Err(CANCEL_ERROR.to_owned());
        }
        let read_bytes = response.read(&mut buffer).map_err(|err| err.to_string())?;
        if read_bytes == 0 {
            break;
        }

        file.write_all(&buffer[..read_bytes])
            .map_err(|err| err.to_string())?;
        downloaded_bytes += read_bytes as u64;
        on_progress(DownloadProgress {
            downloaded_bytes,
            total_bytes,
        });
    }

    file.sync_all().map_err(|err| err.to_string())
}

pub fn build_active_map_selection(
    project: &LizaProject,
    map: &LizaMapPackage,
    bundles_root: &Path,
) -> ActiveMapSelection {
    let local_path = map.local_path.clone().unwrap_or_else(|| {
        project_mobile_maps_dir(bundles_root, &project.summary.slug).join(&map.file_name)
    });

    ActiveMapSelection {
        kind: map_kind_from_local_path(&local_path),
        project_name: project.summary.name.clone(),
        package_name: map.name.clone(),
        remote_url: map.url.clone(),
        local_path,
        center: project.center,
        base_zoom: map.base_zoom,
    }
}

fn parse_center(text: &str) -> Result<MapCenter, String> {
    let regex = Regex::new(r"([0-9]{1,3}\.[0-9]+)").map_err(|err| err.to_string())?;
    let numbers = regex
        .captures_iter(text)
        .filter_map(|captures| captures.get(1))
        .filter_map(|value| value.as_str().parse::<f64>().ok())
        .collect::<Vec<_>>();

    if numbers.len() < 2 {
        return Err("Could not parse project coordinates".to_owned());
    }

    Ok(MapCenter {
        lat: numbers[0],
        lon: numbers[1],
    })
}

#[allow(dead_code)]
fn parse_map_packages(html: &str, base_url: &str) -> Result<Vec<LizaMapPackage>, String> {
    let zoom_regex = Regex::new(r"_z(\d+)\.sqlitedb$").map_err(|err| err.to_string())?;
    // Sizes come from the same HTML, keyed by the href they sit beside.
    let sizes = parse_entry_sizes(html);
    let sizes_by_url: std::collections::HashMap<String, u64> = sizes
        .into_iter()
        .filter_map(|(href, bytes)| Some((resolve_listing_href(base_url, &href)?, bytes)))
        .collect();

    let maps = parse_directory_entries(base_url, html)?
        .into_iter()
        .filter(|entry| !entry.is_dir && entry.name.ends_with(".sqlitedb"))
        .filter_map(|entry| {
            let zoom = zoom_regex
                .captures(&entry.name)
                .and_then(|captures| captures.get(1))?
                .as_str()
                .parse::<u8>()
                .ok()?;
            let size_bytes = sizes_by_url.get(&entry.url).copied();
            Some(LizaMapPackage {
                name: entry.name.clone(),
                file_name: entry.name,
                url: entry.url,
                base_zoom: zoom,
                local_path: None,
                size_bytes,
            })
        })
        .collect::<Vec<_>>();

    Ok(maps)
}

fn fetch_text(url: &str) -> Result<String, String> {
    let bytes = client()?
        .get(url)
        .send()
        .and_then(|response| response.error_for_status())
        .map_err(|err| err.to_string())?
        .bytes()
        .map_err(|err| err.to_string())?;

    Ok(decode_text_bytes(bytes.as_ref()))
}

/// How long to wait for a TCP connection before giving up.
///
/// A штаб link goes through a phone; a host that never answers must fail
/// rather than hold the app.
const CONNECT_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(15);

/// How long a transfer may go without delivering any bytes.
///
/// This is a per-read timeout, reset by every successful read, so a slow but
/// moving download is not interrupted — only a stalled one. A whole-request
/// timeout would be wrong here: bundles take minutes.
const READ_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(60);

fn client() -> Result<Client, String> {
    // The blocking builder's `timeout` already defaults to 30s for connect,
    // read and write; the connect timeout is stated explicitly so it does not
    // depend on that default.
    Client::builder()
        .connect_timeout(CONNECT_TIMEOUT)
        .build()
        .map_err(|err| err.to_string())
}

/// The async client used by the bundle downloader.
///
/// It had no timeouts at all: a connection that stopped delivering bytes held
/// the download open indefinitely, with the progress panel frozen on the file
/// that stalled and no way to tell it apart from a slow link.
fn async_client(
    connect_timeout: std::time::Duration,
    read_timeout: std::time::Duration,
) -> Result<reqwest::Client, String> {
    reqwest::Client::builder()
        .connect_timeout(connect_timeout)
        .read_timeout(read_timeout)
        .build()
        .map_err(|err| err.to_string())
}

/// Configuration for the multi-file download orchestrator.
#[derive(Debug, Clone)]
pub struct BundleDownloadConfig {
    /// Maximum number of in-flight HTTP requests at any time. Always ≥ 1.
    pub concurrency: usize,
    /// HTTP base used to list and fetch directory contents.
    pub url: String,
    /// Filesystem destination for the project bundle root.
    pub local_dir: PathBuf,
    /// Cancellation token; when triggered, in-flight downloads abort and the
    /// orchestrator returns [`CANCEL_ERROR`].
    pub cancel: CancelToken,
    /// Top-level entries to leave on the server.
    ///
    /// A bundle carries print sheets and Android tile packs this app cannot
    /// open, and on a phone tether they are most of the transfer. The choice
    /// of what to skip belongs to the operator, so nothing is skipped unless
    /// it is named here.
    pub skip_top_level: Vec<String>,
}

/// Download a LizaAlert project bundle with per-file notifications.
///
/// Files are enumerated, sorted by prefix (`00-`, `10-`, …), and downloaded
/// concurrently with the bound given by `config.concurrency`. The caller
/// receives a [`DownloadNotification`] for every chunk and every file-ready
/// transition through the supplied unbounded channel.
///
/// Resuming a previously cancelled download is supported: files that already
/// exist on disk under `local_dir` are skipped (their `bundle-file-ready`
/// notification is still emitted so downstream listeners can rebuild their
/// view, mirroring real-world fresh state).
pub async fn download_bundle_concurrent(
    config: BundleDownloadConfig,
    tx: mpsc::UnboundedSender<DownloadNotification>,
) -> Result<(), String> {
    fs::create_dir_all(&config.local_dir).map_err(|err| err.to_string())?;
    let _ = tx.send(DownloadNotification::Phase(ProjectOpenProgress::status(
        ProgressText::ScanningDirectory {
            path: config.local_dir.display().to_string(),
        },
        ProjectOpenPhase::Scanning,
    )));

    if config.cancel.is_cancelled() {
        return Err(CANCEL_ERROR.to_owned());
    }

    let mut files = tokio::task::spawn_blocking({
        let url = config.url.clone();
        let local_dir = config.local_dir.clone();
        let skip = config.skip_top_level.clone();
        move || {
            let mut out = Vec::new();
            let root_rel = String::new();
            collect_remote_files_rel(&url, &local_dir, &root_rel, &skip, &mut out)?;
            Ok::<_, String>(out)
        }
    })
    .await
    .map_err(|err| err.to_string())??;

    sort_remote_files_by_prefix(&mut files);
    let total = files.len();
    // What the operator is committing to. Files already on disk are skipped
    // below, so they are left out of the figure — otherwise resuming a
    // download would announce the whole bundle again.
    let total_bytes: u64 = files
        .iter()
        .filter(|file| !file.path.exists())
        .filter_map(|file| file.size_bytes)
        .sum();

    let _ = tx.send(DownloadNotification::Phase(ProjectOpenProgress {
        text: ProgressText::DownloadingInParallel {
            total: total as usize,
        },
        phase: ProjectOpenPhase::Downloading,
        completed: Some(0),
        total: Some(total as u64),
        downloaded_bytes: Some(0),
        total_bytes: (total_bytes > 0).then_some(total_bytes),
    }));

    let concurrency = config.concurrency.max(1);
    let sem = Arc::new(Semaphore::new(concurrency));
    // Each worker answers with the bytes it actually fetched, so the
    // aggregate byte progress is a sum and not a guess.
    let mut set: JoinSet<Result<u64, String>> = JoinSet::new();
    let async_client = async_client(CONNECT_TIMEOUT, READ_TIMEOUT)?;

    for (index, file) in files.into_iter().enumerate() {
        if config.cancel.is_cancelled() {
            break;
        }
        let permit_sem = Arc::clone(&sem);
        let tx_worker = tx.clone();
        let cancel = config.cancel.clone();
        let file_count = total;
        let client = async_client.clone();
        set.spawn(async move {
            let permit = permit_sem
                .acquire_owned()
                .await
                .map_err(|e| e.to_string())?;
            if cancel.is_cancelled() {
                drop(permit);
                return Err(CANCEL_ERROR.to_owned());
            }
            let mut fetched_bytes: u64 = 0;
            let RemoteFileDownload {
                url,
                path,
                relative,
                size_bytes: _,
            } = file;
            let pkg = relative.clone();
            if path.exists() {
                // Already on disk (resume). Emit one synthetic progress + ready.
                let size = tokio::fs::metadata(&path).await.ok().map(|m| m.len());
                let _ = tx_worker.send(DownloadNotification::FileProgress {
                    package_name: pkg.clone(),
                    downloaded_bytes: size.unwrap_or(0),
                    total_bytes: size,
                    file_index: index,
                    file_count,
                });
            } else {
                let retry_tx = tx_worker.clone();
                let retry_pkg = pkg.clone();
                let res = download_file_with_retries(
                    &client,
                    &url,
                    &path,
                    &cancel,
                    |downloaded, total_bytes| {
                        fetched_bytes = downloaded;
                        let _ = tx_worker.send(DownloadNotification::FileProgress {
                            package_name: pkg.clone(),
                            downloaded_bytes: downloaded,
                            total_bytes,
                            file_index: index,
                            file_count,
                        });
                    },
                    |attempt| {
                        // Say so: a retry that looks like a stall is the same
                        // as a stall to the person watching the bar.
                        let _ = retry_tx.send(DownloadNotification::Phase(
                            ProjectOpenProgress::status(
                                ProgressText::RetryingFile {
                                    name: retry_pkg.clone(),
                                    attempt: attempt + 1,
                                    total: FILE_DOWNLOAD_ATTEMPTS,
                                },
                                ProjectOpenPhase::Downloading,
                            ),
                        ));
                    },
                )
                .await;
                res?;
            }
            // Emit ready
            let _ = tx_worker.send(DownloadNotification::FileReady {
                package_name: pkg,
                local_path: path,
                file_index: index,
                file_count,
            });
            drop(permit);
            Ok(fetched_bytes)
        });
    }

    let mut first_err: Option<String> = None;
    let mut completed_files: u64 = 0;
    let mut downloaded_bytes: u64 = 0;
    while let Some(joined) = set.join_next().await {
        match joined {
            Ok(Ok(bytes)) => {
                // Per-file completion tick so the aggregate progress moves
                // continuously instead of jumping 0 -> N at the very end.
                completed_files += 1;
                downloaded_bytes += bytes;
                let _ = tx.send(DownloadNotification::Phase(ProjectOpenProgress {
                    text: ProgressText::DownloadedOfFiles {
                        completed: completed_files as usize,
                        total: total as usize,
                    },
                    phase: ProjectOpenPhase::Downloading,
                    completed: Some(completed_files),
                    total: Some(total as u64),
                    downloaded_bytes: Some(downloaded_bytes),
                    total_bytes: (total_bytes > 0).then_some(total_bytes),
                }));
            }
            Ok(Err(e)) => {
                if first_err.is_none() {
                    first_err = Some(e);
                }
            }
            Err(e) if e.is_cancelled() => {
                if first_err.is_none() {
                    first_err = Some(CANCEL_ERROR.to_owned());
                }
            }
            Err(e) => {
                if first_err.is_none() {
                    first_err = Some(e.to_string());
                }
            }
        }
    }

    if config.cancel.is_cancelled() {
        return Err(CANCEL_ERROR.to_owned());
    }
    if let Some(e) = first_err {
        return Err(e);
    }

    let _ = tx.send(DownloadNotification::Phase(ProjectOpenProgress {
        text: ProgressText::DownloadedFiles {
            total: total as usize,
        },
        phase: ProjectOpenPhase::Downloading,
        completed: Some(total as u64),
        total: Some(total as u64),
        downloaded_bytes: None,
        total_bytes: None,
    }));

    Ok(())
}

/// Asynchronous, notification-driven counterpart to [`open_project`].
///
/// Returns the [`LizaProject`] descriptor once download + extract + indexing
/// have all completed. Per-file events stream through `tx` while work is in
/// progress.
pub async fn open_project_async(
    summary: LizaProjectSummary,
    root: PathBuf,
    cancel: CancelToken,
    concurrency: usize,
    skip_top_level: Vec<String>,
    tx: mpsc::UnboundedSender<DownloadNotification>,
) -> Result<LizaProject, String> {
    // Probe the remote listing first. While it is reachable we always run the
    // concurrent download: its resume logic skips files already on disk, so a
    // fully cached bundle costs only the listing fetches, while a partially
    // cached one (cancelled or failed mid-download earlier) gets its missing
    // files back. Gating on `is_project_cached` alone would freeze a partial
    // bundle forever, because `2-Coordinates.txt` — the cache marker — lands
    // on disk within the first seconds of a download. The cached-only branch
    // is reserved for the offline case.
    let listing_probe = tokio::task::spawn_blocking({
        let url = summary.url.clone();
        move || fetch_text(&url).map(|_| ())
    })
    .await
    .map_err(|err| err.to_string())?;

    match listing_probe {
        Ok(()) => {
            let _ = tx.send(DownloadNotification::Phase(ProjectOpenProgress::status(
                ProgressText::DownloadingBundle {
                    name: summary.name.clone(),
                },
                ProjectOpenPhase::Downloading,
            )));
            let source_root = project_source_root(&root, &summary.slug);
            fs::create_dir_all(&source_root).map_err(|err| err.to_string())?;
            let cfg = BundleDownloadConfig {
                concurrency,
                url: summary.url.clone(),
                local_dir: source_root,
                cancel: cancel.clone(),
                skip_top_level,
            };
            download_bundle_concurrent(cfg, tx.clone()).await?;
        }
        Err(err) => {
            if !is_project_cached(&summary.slug, &root) {
                return Err(err);
            }
            let _ = tx.send(DownloadNotification::Phase(ProjectOpenProgress::status(
                ProgressText::OpeningCachedBundle {
                    name: summary.name.clone(),
                },
                ProjectOpenPhase::Downloading,
            )));
        }
    }

    if cancel.is_cancelled() {
        return Err(CANCEL_ERROR.to_owned());
    }

    let project = tokio::task::spawn_blocking({
        let summary = summary.clone();
        let root = root.clone();
        let tx = tx.clone();
        move || {
            let mut on_progress = |p: ProjectOpenProgress| {
                let _ = tx.send(DownloadNotification::Phase(p));
            };
            on_progress(ProjectOpenProgress::status(
                ProgressText::ExtractingCachedOzi {
                    name: summary.name.clone(),
                },
                ProjectOpenPhase::Extracting,
            ));
            materialize_cached_ozi_archives(&root, &summary.slug, &mut on_progress)?;

            on_progress(ProjectOpenProgress::status(
                ProgressText::IndexingCachedMaps {
                    name: summary.name.clone(),
                },
                ProjectOpenPhase::Indexing,
            ));
            let project = load_cached_project_from_root(summary.clone(), &root)?;
            ensure_tracks_dir(&root, &summary.slug);
            Ok::<_, String>(project)
        }
    })
    .await
    .map_err(|err| err.to_string())??;
    Ok(project)
}

/// CJ-onboarding: build the map list of a bundle WITHOUT downloading it.
/// Online: root listing is parsed for sqlite map packages (cached flag from
/// local file presence) and the center file (a few hundred bytes) is the
/// only download. Offline: falls back to the fully cached view. OZI rasters
/// appear in the preview only when already extracted locally — remote OZI
/// archives are listed after a real open.
/// Collect the downloadable `.sqlitedb` maps a project offers.
///
/// A project page lists folders and archives, not maps: the Android/iOS tile
/// databases live one level down (`8-Android&iOS/` by the bundle convention).
/// Reading only the project page therefore returned nothing, and selecting a
/// project showed an empty map list.
fn fetch_remote_sqlite_maps(
    project_url: &str,
    project_listing: &str,
) -> Result<Vec<LizaMapPackage>, String> {
    /// A bundle has a handful of folders; the cap only stops a pathological page.
    const MAX_SUBDIRECTORIES: usize = 8;

    let mut maps = parse_map_packages(project_listing, project_url)?;
    if !maps.is_empty() {
        return Ok(maps);
    }

    let subdirectories: Vec<DirectoryEntry> =
        parse_directory_entries(project_url, project_listing)?
            .into_iter()
            .filter(|entry| entry.is_dir)
            .take(MAX_SUBDIRECTORIES)
            .collect();

    for directory in subdirectories {
        // A folder that cannot be read is not fatal: another may hold the maps.
        let Ok(listing) = fetch_text(&directory.url) else {
            continue;
        };
        maps.extend(parse_map_packages(&listing, &directory.url)?);
    }

    Ok(maps)
}

pub fn preview_project(summary: LizaProjectSummary, root: &Path) -> Result<LizaProject, String> {
    match fetch_text(&summary.url) {
        Ok(listing) => {
            let mut maps = fetch_remote_sqlite_maps(&summary.url, &listing)?;
            let zoom_regex = Regex::new(r"_z(\d+)\.sqlitedb$").map_err(|err| err.to_string())?;
            let cached: Vec<LizaMapPackage> =
                read_cached_sqlite_map_packages(root, &summary.slug, &zoom_regex)?;
            for map in &mut maps {
                if let Some(local) = cached.iter().find(|c| c.file_name == map.file_name) {
                    map.local_path = local.local_path.clone();
                }
            }
            let bundle_dir = project_source_root(root, &summary.slug);
            let mut ozi = read_cached_ozi_map_packages(&bundle_dir)?;
            ozi.sort_by(|a, b| a.name.cmp(&b.name));
            ozi.extend(maps);

            let center_href = Regex::new(r#"href="([^"]*Coordinates\.txt)""#)
                .map_err(|err| err.to_string())?
                .captures(&listing)
                .and_then(|c| c.get(1).map(|m| m.as_str().to_owned()));
            let center =
                match center_href.and_then(|href| resolve_listing_href(&summary.url, &href)) {
                    Some(url) => parse_center(&fetch_text(&url)?)?,
                    None => {
                        let local = project_coordinates_path(root, &summary.slug);
                        parse_center(&read_text_file_lossy(&local).map_err(|e| e.to_string())?)?
                    }
                };
            // The same listing already fetched, as the operator's menu of
            // what not to download.
            let contents = parse_bundle_contents(&summary.url, &listing);
            Ok(LizaProject {
                summary,
                center,
                maps: ozi,
                contents,
            })
        }
        Err(err) => {
            if !is_project_cached(&summary.slug, root) {
                return Err(format!("bundle listing unreachable and not cached: {err}"));
            }
            load_cached_project_from_root(summary, root)
        }
    }
}

fn collect_remote_files_rel(
    url: &str,
    local_dir: &Path,
    rel_prefix: &str,
    skip_top_level: &[String],
    output: &mut Vec<RemoteFileDownload>,
) -> Result<(), String> {
    let html = fetch_text(url)?;
    // The listing states each file's size beside it, so the scan learns what
    // the whole bundle weighs without a single extra request.
    let sizes_by_url: std::collections::HashMap<String, u64> = parse_entry_sizes(&html)
        .into_iter()
        .filter_map(|(href, bytes)| Some((resolve_listing_href(url, &href)?, bytes)))
        .collect();

    for entry in parse_directory_entries(url, &html)? {
        // Only the top level is skippable: that is the granularity the
        // listing shows the operator, so it is the granularity they choose in.
        if rel_prefix.is_empty() && skip_top_level.iter().any(|name| name == &entry.name) {
            continue;
        }
        let child_url = entry.url.clone();
        let child_path = local_dir.join(&entry.name);
        let child_rel = if rel_prefix.is_empty() {
            entry.name.clone()
        } else {
            format!("{rel_prefix}/{}", entry.name)
        };

        if entry.is_dir {
            fs::create_dir_all(&child_path).map_err(|err| err.to_string())?;
            collect_remote_files_rel(&child_url, &child_path, &child_rel, skip_top_level, output)?;
        } else {
            let size_bytes = sizes_by_url.get(&child_url).copied();
            output.push(RemoteFileDownload {
                url: child_url,
                path: child_path,
                relative: child_rel,
                size_bytes,
            });
        }
    }

    Ok(())
}

/// Streaming async HTTP GET that periodically calls `on_progress`, fsyncs on
/// completion, and aborts early when `cancel` is triggered.
///
/// Bytes are written to a sibling `.part` file and atomically renamed on
/// success — a cancel mid-stream therefore never leaves a half-written file
/// at the canonical path, so `path.exists()` is a reliable "fully done"
/// signal for resume logic.
/// How many times a single file is fetched before the bundle gives up on it.
///
/// A field link drops transfers. With timeouts in place a stalled one fails
/// rather than hanging, and before this the whole bundle failed with it —
/// after however many files had already come down. Three is enough to ride out
/// a dropped connection without keeping a crew waiting on a link that is
/// genuinely gone.
const FILE_DOWNLOAD_ATTEMPTS: usize = 3;

/// Wait between attempts. Short: the operator is standing there.
const RETRY_BACKOFF: std::time::Duration = std::time::Duration::from_millis(500);

/// Fetch one file, retrying a failed transfer.
///
/// `on_retry` is called with the attempt that just failed (1-based), so the
/// interface can say it is retrying rather than appear stuck.
///
/// Cancellation is not a transport failure and is never retried: retrying it
/// would ignore the operator and keep the link busy after they asked it to
/// stop.
async fn download_file_with_retries<F, R>(
    client: &reqwest::Client,
    url: &str,
    path: &Path,
    cancel: &CancelToken,
    mut on_progress: F,
    mut on_retry: R,
) -> Result<(), String>
where
    F: FnMut(u64, Option<u64>),
    R: FnMut(usize),
{
    let mut last_error = String::new();
    for attempt in 1..=FILE_DOWNLOAD_ATTEMPTS {
        if cancel.is_cancelled() {
            return Err(CANCEL_ERROR.to_owned());
        }

        match download_to_path_async(client, url, path, cancel, &mut on_progress).await {
            Ok(()) => return Ok(()),
            Err(error) if error == CANCEL_ERROR => return Err(error),
            Err(error) => {
                last_error = error;
                if attempt == FILE_DOWNLOAD_ATTEMPTS {
                    break;
                }
                on_retry(attempt);
                tokio::time::sleep(RETRY_BACKOFF).await;
            }
        }
    }
    Err(last_error)
}

async fn download_to_path_async<F>(
    client: &reqwest::Client,
    url: &str,
    path: &Path,
    cancel: &CancelToken,
    mut on_progress: F,
) -> Result<(), String>
where
    F: FnMut(u64, Option<u64>),
{
    let send_fut = client.get(url).send();
    tokio::pin!(send_fut);
    let response = tokio::select! {
        biased;
        _ = cancel_wait(cancel) => return Err(CANCEL_ERROR.to_owned()),
        r = &mut send_fut => r.and_then(|r| r.error_for_status()).map_err(|err| err.to_string())?,
    };

    if let Some(parent) = path.parent() {
        tokio::fs::create_dir_all(parent)
            .await
            .map_err(|err| err.to_string())?;
    }

    let tmp_path = path.with_extension("part");
    let mut file = tokio::fs::File::create(&tmp_path)
        .await
        .map_err(|err| err.to_string())?;
    let total_bytes = response.content_length();
    let mut downloaded_bytes = 0u64;

    // Always emit at least one progress event per file so a UI can render the
    // currently-downloading label even if the body is empty.
    on_progress(0, total_bytes);

    let mut stream = response.bytes_stream();
    use futures_util::StreamExt;
    use tokio::io::AsyncWriteExt;
    loop {
        let next = tokio::select! {
            biased;
            _ = cancel_wait(cancel) => {
                drop(file);
                let _ = tokio::fs::remove_file(&tmp_path).await;
                return Err(CANCEL_ERROR.to_owned());
            }
            n = stream.next() => n,
        };
        let Some(chunk) = next else { break };
        // A stream or write error leaves a partial `.part` behind unless it is
        // removed here. Cancellation already cleaned up; a stalled or broken
        // transfer did not, and the leftovers accumulate in the bundle folder.
        let chunk = match chunk {
            Ok(chunk) => chunk,
            Err(err) => {
                drop(file);
                let _ = tokio::fs::remove_file(&tmp_path).await;
                return Err(err.to_string());
            }
        };
        if let Err(err) = file.write_all(&chunk).await {
            drop(file);
            let _ = tokio::fs::remove_file(&tmp_path).await;
            return Err(err.to_string());
        }
        downloaded_bytes += chunk.len() as u64;
        on_progress(downloaded_bytes, total_bytes);
    }

    if let Err(err) = file.sync_all().await {
        drop(file);
        let _ = tokio::fs::remove_file(&tmp_path).await;
        return Err(err.to_string());
    }
    drop(file);
    if let Err(err) = tokio::fs::rename(&tmp_path, path).await {
        let _ = tokio::fs::remove_file(&tmp_path).await;
        return Err(err.to_string());
    }
    Ok(())
}

/// Future that resolves when `cancel.is_cancelled()` becomes true. Polled
/// every 25 ms — fast enough to honour the 250 ms cancel-deadline target.
async fn cancel_wait(cancel: &CancelToken) {
    while !cancel.is_cancelled() {
        tokio::time::sleep(std::time::Duration::from_millis(25)).await;
    }
}

fn parse_directory_entries(base_url: &str, html: &str) -> Result<Vec<DirectoryEntry>, String> {
    // Match the href only. The listing wraps the file name in an icon
    // `<span>`, so any pattern that expects text straight after `<a ...>`
    // matches nothing on the markup the site serves.
    let link_regex = Regex::new(r#"href="([^"]+)""#).map_err(|err| err.to_string())?;

    let mut entries: Vec<DirectoryEntry> = Vec::new();
    for captures in link_regex.captures_iter(html) {
        let Some(raw_href) = captures.get(1).map(|m| m.as_str()) else {
            continue;
        };
        let Some(url) = resolve_listing_href(base_url, raw_href) else {
            continue;
        };
        let Some(name) = decode_entry_name(&url) else {
            continue;
        };
        if name.trim().is_empty() || entries.iter().any(|e| e.url == url) {
            continue;
        }
        entries.push(DirectoryEntry {
            is_dir: url.ends_with('/'),
            url,
            name,
        });
    }

    entries.sort_by(|left, right| left.name.cmp(&right.name));
    Ok(entries)
}

/// The download sizes the listing prints beside each file.
///
/// Keyed by href, because that is what `parse_directory_entries` resolves
/// against. The listing renders one table row per entry with the size in the
/// next cell ("15.9 МиБ"), so the sizes cost no extra request — and without
/// them the operator commits to a download with no idea whether it is fifteen
/// megabytes or two hundred, which on a phone tether is the whole decision.
fn parse_entry_sizes(html: &str) -> std::collections::HashMap<String, u64> {
    let mut sizes = std::collections::HashMap::new();
    let Ok(row_regex) = Regex::new(r"(?s)<tr>(.*?)</tr>") else {
        return sizes;
    };
    let Ok(href_regex) = Regex::new(r#"href="([^"]+)""#) else {
        return sizes;
    };
    let Ok(cell_regex) = Regex::new(r"(?s)<td>(.*?)</td>") else {
        return sizes;
    };

    for row in row_regex.captures_iter(html) {
        let Some(row) = row.get(1).map(|m| m.as_str()) else {
            continue;
        };
        let Some(href) = href_regex
            .captures(row)
            .and_then(|c| c.get(1))
            .map(|m| m.as_str().to_owned())
        else {
            continue;
        };
        for cell in cell_regex.captures_iter(row).skip(1) {
            let Some(text) = cell.get(1).map(|m| m.as_str()) else {
                continue;
            };
            if let Some(bytes) = parse_human_size(text) {
                sizes.insert(href, bytes);
                break;
            }
        }
    }

    sizes
}

/// Read "15.9 МиБ" / "118.6 КиБ" / "742 Б" as a byte count.
///
/// The site prints binary units in Russian; anything else (a folder's "папка",
/// a dash, a stray cell) is not a size and yields `None`.
fn parse_human_size(text: &str) -> Option<u64> {
    let text = text.trim();
    let mut parts = text.split_whitespace();
    let number: f64 = parts.next()?.replace(',', ".").parse().ok()?;
    if number < 0.0 {
        return None;
    }
    let unit = parts.next()?;
    if parts.next().is_some() {
        return None;
    }
    let multiplier: f64 = match unit {
        "Б" | "B" => 1.0,
        "КиБ" | "KiB" | "КБ" | "KB" => 1024.0,
        "МиБ" | "MiB" | "МБ" | "MB" => 1024.0 * 1024.0,
        "ГиБ" | "GiB" | "ГБ" | "GB" => 1024.0 * 1024.0 * 1024.0,
        _ => return None,
    };
    Some((number * multiplier).round() as u64)
}

fn decode_entry_name(href: &str) -> Option<String> {
    let raw_name = href.trim_end_matches('/').rsplit('/').next()?.trim();
    if raw_name.is_empty() {
        return None;
    }
    Some(percent_decode(raw_name))
}

/// Percent-decode into bytes first, then interpret as UTF-8.
///
/// Decoding byte by byte into `char` treats each byte as a code point, which
/// turns every Cyrillic name in the catalogue into mojibake — `%D0%9B` is one
/// letter, not two.
fn percent_decode(raw: &str) -> String {
    let bytes = raw.as_bytes();
    let mut out: Vec<u8> = Vec::with_capacity(bytes.len());
    let mut index = 0;
    while index < bytes.len() {
        if bytes[index] == b'%'
            && index + 2 < bytes.len()
            && let Some(value) = std::str::from_utf8(&bytes[index + 1..index + 3])
                .ok()
                .and_then(|hex| u8::from_str_radix(hex, 16).ok())
        {
            out.push(value);
            index += 3;
            continue;
        }
        out.push(bytes[index]);
        index += 1;
    }
    String::from_utf8_lossy(&out).into_owned()
}

/// Undo the HTML escaping the listing applies to hrefs and labels.
fn html_unescape(value: &str) -> String {
    value
        .replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
}

/// Turn a listing href into the absolute URL of a child entry, or `None` when
/// the link is not one.
///
/// The catalogue serves absolute hrefs (`/maps/<project>/<file>`); it used to
/// serve relative ones, and both must work. Anything that is not below
/// `base_url` — the parent listing, `/login`, stylesheets, the pagination
/// cursor, another host — is navigation, not content.
fn resolve_listing_href(base_url: &str, href: &str) -> Option<String> {
    let href = html_unescape(href.trim());
    if href.is_empty()
        || href.starts_with('#')
        || href.starts_with('?')
        || href.starts_with("mailto:")
        || href.starts_with("javascript:")
        || href == "../"
        || href == "./"
    {
        return None;
    }

    let absolute = if href.starts_with("http://") || href.starts_with("https://") {
        href
    } else if let Some(path) = href.strip_prefix('/') {
        let origin_end = base_url.find("://").map(|i| i + 3)?;
        let origin = match base_url[origin_end..].find('/') {
            Some(slash) => &base_url[..origin_end + slash],
            None => base_url,
        };
        format!("{origin}/{path}")
    } else {
        format!("{base_url}{href}")
    };

    // A child entry lives below the listing; equal-or-shorter means parent.
    if absolute.len() <= base_url.len() || !absolute.starts_with(base_url) {
        return None;
    }
    Some(absolute)
}

fn read_text_file_lossy(path: &Path) -> Result<String, std::io::Error> {
    let bytes = fs::read(path)?;
    Ok(decode_text_bytes(&bytes))
}

fn decode_text_bytes(bytes: &[u8]) -> String {
    match String::from_utf8(bytes.to_vec()) {
        Ok(text) => text,
        Err(error) => String::from_utf8_lossy(&error.into_bytes()).into_owned(),
    }
}

fn load_cached_project_from_root(
    summary: LizaProjectSummary,
    root: &Path,
) -> Result<LizaProject, String> {
    let coordinates_text = read_text_file_lossy(&project_coordinates_path(root, &summary.slug))
        .map_err(|err| err.to_string())?;
    let center = parse_center(&coordinates_text)?;
    let maps = read_cached_map_packages(root, &summary.slug)?;

    let contents = read_cached_bundle_contents(&project_source_root(root, &summary.slug));

    Ok(LizaProject {
        summary,
        center,
        maps,
        contents,
    })
}

/// The bundle's top level, from a listing.
fn parse_bundle_contents(base_url: &str, html: &str) -> Vec<BundleEntry> {
    let sizes: std::collections::HashMap<String, u64> = parse_entry_sizes(html)
        .into_iter()
        .filter_map(|(href, bytes)| Some((resolve_listing_href(base_url, &href)?, bytes)))
        .collect();

    parse_directory_entries(base_url, html)
        .unwrap_or_default()
        .into_iter()
        .map(|entry| BundleEntry {
            size_bytes: sizes.get(&entry.url).copied(),
            name: entry.name,
            is_dir: entry.is_dir,
        })
        .collect()
}

/// The bundle's top level, from disk — what an offline open can report.
fn read_cached_bundle_contents(bundle_dir: &Path) -> Vec<BundleEntry> {
    let Ok(entries) = fs::read_dir(bundle_dir) else {
        return Vec::new();
    };
    let mut contents: Vec<BundleEntry> = entries
        .filter_map(|entry| entry.ok())
        .filter_map(|entry| {
            let path = entry.path();
            let name = path.file_name()?.to_str()?.to_owned();
            let metadata = entry.metadata().ok();
            let is_dir = metadata.as_ref().is_some_and(|m| m.is_dir());
            Some(BundleEntry {
                name,
                is_dir,
                size_bytes: metadata.filter(|m| m.is_file()).map(|m| m.len()),
            })
        })
        .collect();
    contents.sort_by(|a, b| a.name.cmp(&b.name));
    contents
}

fn read_cached_map_packages(
    root: &Path,
    project_slug: &str,
) -> Result<Vec<LizaMapPackage>, String> {
    let zoom_regex = Regex::new(r"_z(\d+)\.sqlitedb$").map_err(|err| err.to_string())?;
    let bundle_dir = project_source_root(root, project_slug);
    // OZI maps: recursive scan of the whole bundle dir (includes extracted/ subdir)
    let mut ozi_maps = read_cached_ozi_map_packages(&bundle_dir)?;
    ozi_maps.sort_by(|left, right| left.name.cmp(&right.name));

    let mut sqlite_maps = read_cached_sqlite_map_packages(root, project_slug, &zoom_regex)?;
    sqlite_maps.sort_by(|left, right| left.name.cmp(&right.name));

    ozi_maps.extend(sqlite_maps);
    Ok(ozi_maps)
}

/// Find the `.sqlitedb` tile databases already on disk for this bundle.
///
/// Searched over the whole bundle directory rather than the `8-Android&iOS`
/// folder alone: the remote walk stopped assuming that name when the site
/// changed, and the downloader writes every file under its real relative
/// path. Reading one fixed folder made a downloaded bundle stored anywhere
/// else look missing, so the map showed a "not downloaded" badge and a click
/// fetched it a second time.
fn read_cached_sqlite_map_packages(
    root: &Path,
    project_slug: &str,
    zoom_regex: &Regex,
) -> Result<Vec<LizaMapPackage>, String> {
    let bundle_dir = project_source_root(root, project_slug);
    let mut files = Vec::new();
    collect_cached_sqlite_map_files(&bundle_dir, &mut files)?;

    let mut maps = Vec::new();
    for path in files {
        let Some(file_name) = path.file_name().and_then(|name| name.to_str()) else {
            continue;
        };
        let file_name = file_name.to_owned();
        let Some(base_zoom) = zoom_regex
            .captures(&file_name)
            .and_then(|captures| captures.get(1))
            .and_then(|zoom| zoom.as_str().parse::<u8>().ok())
        else {
            continue;
        };

        // A cached map's size is the file on disk; the listing is not needed
        // and may not be reachable.
        let size_bytes = fs::metadata(&path).ok().map(|meta| meta.len());
        maps.push(LizaMapPackage {
            name: file_name.clone(),
            file_name,
            url: String::new(),
            base_zoom,
            local_path: Some(path),
            size_bytes,
        });
    }

    Ok(maps)
}

fn collect_cached_sqlite_map_files(dir: &Path, output: &mut Vec<PathBuf>) -> Result<(), String> {
    if !dir.exists() {
        return Ok(());
    }

    for entry in fs::read_dir(dir).map_err(|err| err.to_string())? {
        let entry = entry.map_err(|err| err.to_string())?;
        let path = entry.path();

        if path.is_dir() {
            collect_cached_sqlite_map_files(&path, output)?;
            continue;
        }

        if path
            .extension()
            .and_then(|ext| ext.to_str())
            .is_some_and(|ext| ext.eq_ignore_ascii_case("sqlitedb"))
        {
            output.push(path);
        }
    }

    Ok(())
}

fn read_cached_ozi_map_packages(source_root: &Path) -> Result<Vec<LizaMapPackage>, String> {
    let mut map_files = Vec::new();
    collect_cached_ozi_map_files(source_root, &mut map_files)?;

    let mut packages = Vec::new();

    for map_path in map_files {
        let contents = read_ozi_map_text(&map_path).map_err(|err| err.to_string())?;
        let metadata =
            parse_ozi_map_metadata(&map_path, &contents).map_err(|err| err.to_string())?;
        let relative_name = map_path
            .strip_prefix(source_root)
            .ok()
            .and_then(|path| path.to_str())
            .map(|path| path.replace(std::path::MAIN_SEPARATOR, "/"))
            .unwrap_or_else(|| {
                map_path
                    .file_name()
                    .and_then(|name| name.to_str())
                    .unwrap_or("unknown.map")
                    .to_owned()
            });

        let size_bytes = fs::metadata(&map_path).ok().map(|meta| meta.len());
        packages.push(LizaMapPackage {
            name: format!("OZI: {}", metadata.title()),
            file_name: relative_name,
            url: String::new(),
            base_zoom: 0,
            local_path: Some(map_path),
            size_bytes,
        });
    }

    Ok(packages)
}

fn collect_cached_ozi_map_files(dir: &Path, output: &mut Vec<PathBuf>) -> Result<(), String> {
    if !dir.exists() {
        return Ok(());
    }

    for entry in fs::read_dir(dir).map_err(|err| err.to_string())? {
        let entry = entry.map_err(|err| err.to_string())?;
        let path = entry.path();

        if path.is_dir() {
            collect_cached_ozi_map_files(&path, output)?;
            continue;
        }

        if is_ozi_map_path(&path) {
            output.push(path);
        }
    }

    Ok(())
}

fn is_ozi_map_path(path: &Path) -> bool {
    matches!(
        path.extension().and_then(|ext| ext.to_str()),
        Some(ext) if ext.eq_ignore_ascii_case("map")
    )
}

fn is_zip_path(path: &Path) -> bool {
    matches!(
        path.extension().and_then(|ext| ext.to_str()),
        Some(ext) if ext.eq_ignore_ascii_case("zip")
    )
}

fn map_kind_from_local_path(path: &Path) -> ActiveMapKind {
    if is_ozi_map_path(path) {
        ActiveMapKind::OziRaster
    } else {
        ActiveMapKind::SqliteTiles
    }
}

/// The bundle's coordinates file on disk.
///
/// The online side finds it by matching `*Coordinates.txt` in the listing, so
/// the local side matches the same way instead of insisting on the
/// `2-Coordinates.txt` spelling — otherwise a bundle previews online and then
/// fails to open from cache, and `is_project_cached` calls it absent. The
/// conventional path is returned when nothing matches, so the error message
/// still names the file that was expected.
fn project_coordinates_path(root: &Path, project_slug: &str) -> PathBuf {
    let bundle_dir = project_source_root(root, project_slug);
    let conventional = bundle_dir.join(COORDINATES_FILE_NAME);
    if conventional.exists() {
        return conventional;
    }

    let found = fs::read_dir(&bundle_dir).ok().and_then(|entries| {
        entries
            .filter_map(|entry| entry.ok())
            .map(|entry| entry.path())
            .find(|path| {
                path.file_name()
                    .and_then(|name| name.to_str())
                    .is_some_and(|name| name.to_ascii_lowercase().ends_with("coordinates.txt"))
            })
    });

    found.unwrap_or(conventional)
}

fn project_mobile_maps_dir(root: &Path, project_slug: &str) -> PathBuf {
    project_source_root(root, project_slug).join(MOBILE_MAPS_DIR_NAME)
}

fn project_source_root(root: &Path, project_slug: &str) -> PathBuf {
    root.join(project_slug)
}

fn project_extracted_root(root: &Path, project_slug: &str) -> PathBuf {
    root.join(project_slug).join(PROJECT_EXTRACTED_DIR)
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct DirectoryEntry {
    /// Absolute URL of the entry, already resolved against the listing.
    url: String,
    name: String,
    is_dir: bool,
}

fn materialize_cached_ozi_archives<F>(
    root: &Path,
    project_slug: &str,
    on_progress: &mut F,
) -> Result<(), String>
where
    F: FnMut(ProjectOpenProgress),
{
    let source_root = project_source_root(root, project_slug);
    let extracted_root = project_extracted_root(root, project_slug);
    let mut zip_files = Vec::new();
    collect_cached_zip_files(&source_root, &mut zip_files)?;

    // Filter to archives that exist, are OZI archives, and haven't been extracted yet.
    // is_ozi_archive_file now uses BufReader so no full file load.
    let to_extract: Vec<(PathBuf, PathBuf)> = zip_files
        .into_iter()
        .filter_map(|zip_path| {
            if !is_ozi_archive_file(&zip_path).unwrap_or(false) {
                return None;
            }
            let dest = extraction_destination_for_archive(&extracted_root, &zip_path);
            if dest.exists() {
                return None;
            }
            Some((zip_path, dest))
        })
        .collect();

    if to_extract.is_empty() {
        return Ok(());
    }

    let names: Vec<&str> = to_extract
        .iter()
        .filter_map(|(p, _)| p.file_name()?.to_str())
        .collect();
    on_progress(ProjectOpenProgress {
        text: ProgressText::ExtractingInParallel {
            count: to_extract.len(),
            names: names.join(", "),
        },
        phase: ProjectOpenPhase::Extracting,
        completed: Some(0),
        total: Some(to_extract.len() as u64),
        downloaded_bytes: None,
        total_bytes: None,
    });

    // Extract all archives concurrently; progress callback is not called from threads
    // (it is not Send), so we collect errors and report them after joining.
    let mut first_error: Option<String> = None;
    std::thread::scope(|s| {
        let handles: Vec<_> = to_extract
            .iter()
            .map(|(zip_path, dest)| s.spawn(|| extract_cached_archive(zip_path, dest)))
            .collect();

        for handle in handles {
            if let Ok(Err(e)) = handle.join()
                && first_error.is_none()
            {
                first_error = Some(e);
            }
        }
    });

    if let Some(e) = first_error {
        return Err(e);
    }

    on_progress(ProjectOpenProgress {
        text: ProgressText::ExtractedOziArchives {
            count: to_extract.len(),
        },
        phase: ProjectOpenPhase::Extracting,
        completed: Some(to_extract.len() as u64),
        total: Some(to_extract.len() as u64),
        downloaded_bytes: None,
        total_bytes: None,
    });

    Ok(())
}

fn collect_cached_zip_files(dir: &Path, output: &mut Vec<PathBuf>) -> Result<(), String> {
    if !dir.exists() {
        return Ok(());
    }

    for entry in fs::read_dir(dir).map_err(|err| err.to_string())? {
        let entry = entry.map_err(|err| err.to_string())?;
        let path = entry.path();

        if path.is_dir() {
            collect_cached_zip_files(&path, output)?;
            continue;
        }

        if is_zip_path(&path) {
            output.push(path);
        }
    }

    Ok(())
}

fn is_ozi_archive_file(path: &Path) -> Result<bool, String> {
    let file = File::open(path).map_err(|err| err.to_string())?;
    let entries = inventory_zip_entries(BufReader::new(file)).map_err(|err| err.to_string())?;

    Ok(entries.iter().any(|entry| {
        matches!(
            entry.kind(),
            ArchiveEntryKind::Supported(SupportedArchiveEntryKind::OziMap)
                | ArchiveEntryKind::Unsupported(_)
        ) && !matches!(
            entry.kind(),
            ArchiveEntryKind::Unsupported(
                crate::infrastructure::import::UnsupportedArchiveEntryKind::SqliteTiles
            ) | ArchiveEntryKind::Unsupported(
                crate::infrastructure::import::UnsupportedArchiveEntryKind::Unknown
            )
        )
    }))
}

fn extract_cached_archive(archive_path: &Path, destination: &Path) -> Result<(), String> {
    // Extract into a sibling staging directory and rename into place on
    // success, so the `dest.exists()` skip check in
    // `materialize_cached_ozi_archives` can only ever observe fully-extracted
    // archives — a failed or interrupted extraction leaves nothing behind and
    // the next open retries it.
    let staging = staging_destination_for_archive(destination)?;
    if staging.exists() {
        // Stale leftover from a previous interrupted run.
        fs::remove_dir_all(&staging).map_err(|err| err.to_string())?;
    }

    let file = File::open(archive_path).map_err(|err| err.to_string())?;
    if let Err(err) = extract_zip_entries_to_directory(BufReader::new(file), &staging) {
        let _ = fs::remove_dir_all(&staging);
        return Err(err.to_string());
    }

    if let Err(err) = fs::rename(&staging, destination) {
        let _ = fs::remove_dir_all(&staging);
        return Err(err.to_string());
    }
    Ok(())
}

fn staging_destination_for_archive(destination: &Path) -> Result<PathBuf, String> {
    let name = destination
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| {
            format!(
                "invalid archive extraction destination: {}",
                destination.display()
            )
        })?;
    Ok(destination.with_file_name(format!("{name}.extracting")))
}

fn extraction_destination_for_archive(extracted_root: &Path, archive_path: &Path) -> PathBuf {
    let stem = archive_path
        .file_stem()
        .and_then(|stem| stem.to_str())
        .unwrap_or("archive");

    extracted_root.join(stem)
}

#[cfg(test)]
mod tests {
    use super::{
        decode_text_bytes, load_cached_project_from_root, materialize_cached_ozi_archives,
        parse_center, parse_directory_entries, parse_map_packages, read_cached_sqlite_map_packages,
        read_text_file_lossy,
    };
    use super::{parse_project_listing, resolve_listing_href};
    use crate::application::LizaProjectSummary;
    use std::fs;
    use std::io::{Cursor, Write};
    use std::time::{SystemTime, UNIX_EPOCH};
    use zip::write::SimpleFileOptions;
    use zip::{CompressionMethod, ZipWriter};

    #[test]
    fn parse_center_reads_decimal_coordinates() {
        let center = parse_center("N 54.32821 E 048.40917").expect("center");

        assert_eq!(center.lat, 54.32821);
        assert_eq!(center.lon, 48.40917);
    }

    #[test]
    fn parse_map_packages_reads_sqlite_entries() {
        let html =
            r#"<a href="foo_z16.sqlitedb">foo_z16.sqlitedb</a><a href="bar.txt">bar.txt</a>"#;
        let maps = parse_map_packages(html, "https://example.com/").expect("maps");

        assert_eq!(maps.len(), 1);
        assert_eq!(maps[0].base_zoom, 16);
        assert_eq!(maps[0].url, "https://example.com/foo_z16.sqlitedb");
        assert_eq!(maps[0].local_path, None);
    }

    /// Markup served by maps.lizaalert.ru as of 2026-09-21: the file name sits
    /// after an icon `<span>` inside the anchor, and every href is absolute.
    /// The previous parser matched `>text</a>` directly after the tag, so it
    /// found nothing at all and the app reported an empty catalogue.
    const PROJECT_PAGE_HTML: &str = r##"
        <tbody><tr>
          <td><a href="/maps/2026-09-20_Schuvalovo/9-Map_4_print/"><span aria-hidden="true">📁</span> 9-Map_4_print</a></td>
          <td>Папка</td><td><time>20.09.2026 17:09 UTC</time></td>
        </tr><tr>
          <td><a href="/maps/2026-09-20_Schuvalovo/8-Android&amp;iOS/"><span aria-hidden="true">📁</span> 8-Android&amp;iOS</a></td>
          <td>Папка</td><td><time>20.09.2026 17:09 UTC</time></td>
        </tr><tr>
          <td><a href="/maps/2026-09-20_Schuvalovo/6-Ozi%28Win&amp;Android%29_Satell.zip"><span aria-hidden="true">↓</span> 6-Ozi(Win&amp;Android)_Satell.zip</a></td>
          <td>54.0 МиБ</td><td><time>20.09.2026 17:26 UTC</time></td>
        </tr></tbody>
        <a href="/maps/">Все файлы</a>
        <a href="/login">Администрирование</a>
        <link rel="stylesheet" href="/assets/app.css">
    "##;

    /// Hits the live catalogue, so it is not part of `just ci`.
    /// Run with: cargo test --manifest-path src-tauri/Cargo.toml --lib
    ///   live_catalogue -- --ignored --nocapture
    #[test]
    #[ignore = "requires network access to maps.lizaalert.ru"]
    fn live_catalogue_lists_projects_and_a_project_lists_its_maps() {
        let walk = super::fetch_project_summaries_streaming(
            &super::CancelToken::new(),
            |chunk: Vec<LizaProjectSummary>, page: usize| {
                if page == 1 {
                    println!("first chunk: {} projects", chunk.len());
                }
            },
        )
        .expect("catalogue");
        let pages = walk.pages;
        let projects = walk.projects;
        println!("catalogue: {} projects over {pages} chunks", projects.len());
        assert!(
            projects.len() > 1000,
            "the catalogue is paginated; stopping at page one truncates it to {}",
            projects.len()
        );

        let newest = projects.first().expect("at least one project").clone();
        println!("previewing {} ({})", newest.slug, newest.url);
        let temp = std::env::temp_dir().join("ozi-rs-live-preview");
        let project = super::preview_project(newest, &temp).expect("preview");
        println!(
            "maps: {:?}",
            project
                .maps
                .iter()
                .map(|m| &m.file_name)
                .collect::<Vec<_>>()
        );
        assert!(
            !project.maps.is_empty(),
            "selecting a project must list its maps"
        );
    }

    #[test]
    fn directory_entries_read_the_listing_markup_the_site_serves_now() {
        let base = "https://maps.lizaalert.ru/maps/2026-09-20_Schuvalovo/";
        let entries = parse_directory_entries(base, PROJECT_PAGE_HTML).expect("entries");

        let names: Vec<&str> = entries.iter().map(|e| e.name.as_str()).collect();
        assert_eq!(
            names,
            vec![
                "6-Ozi(Win&Android)_Satell.zip",
                "8-Android&iOS",
                "9-Map_4_print"
            ],
            "names come from the href, decoded, not from the anchor text"
        );

        let zip = entries
            .iter()
            .find(|e| e.name.ends_with(".zip"))
            .expect("zip entry");
        assert!(!zip.is_dir);
        assert_eq!(
            zip.url,
            "https://maps.lizaalert.ru/maps/2026-09-20_Schuvalovo/6-Ozi%28Win&Android%29_Satell.zip",
            "an absolute href must not be concatenated onto the base url"
        );
        assert!(entries.iter().filter(|e| e.is_dir).count() == 2);
    }

    #[test]
    fn directory_entries_ignore_navigation_assets_and_the_parent_listing() {
        let base = "https://maps.lizaalert.ru/maps/2026-09-20_Schuvalovo/";
        let entries = parse_directory_entries(base, PROJECT_PAGE_HTML).expect("entries");
        assert!(
            entries
                .iter()
                .all(|e| !e.url.contains("/login") && !e.url.contains("/assets/")),
            "navigation and stylesheets are not directory entries"
        );
        assert!(
            entries.iter().all(|e| e.url.len() > base.len()),
            "the parent listing link is not an entry"
        );
    }

    #[test]
    fn resolve_listing_href_handles_absolute_relative_and_foreign_links() {
        let base = "https://maps.lizaalert.ru/maps/2026-09-20_X/";
        assert_eq!(
            resolve_listing_href(base, "/maps/2026-09-20_X/file.zip").as_deref(),
            Some("https://maps.lizaalert.ru/maps/2026-09-20_X/file.zip")
        );
        // Relative hrefs are what the site used to serve; they must keep working.
        assert_eq!(
            resolve_listing_href(base, "file.zip").as_deref(),
            Some("https://maps.lizaalert.ru/maps/2026-09-20_X/file.zip")
        );
        assert_eq!(resolve_listing_href(base, "/maps/"), None, "parent");
        assert_eq!(resolve_listing_href(base, "../"), None, "parent");
        assert_eq!(resolve_listing_href(base, "/login"), None, "outside base");
        assert_eq!(resolve_listing_href(base, "?after=x"), None, "pagination");
        assert_eq!(resolve_listing_href(base, "#top"), None, "fragment");
        assert_eq!(
            resolve_listing_href(base, "https://example.test/evil.zip"),
            None,
            "another host"
        );
    }

    /// The root listing is paginated now: one page carries part of the
    /// catalogue and a "Следующая страница" link with an opaque cursor.
    const ROOT_LISTING_HTML: &str = r##"
        <tbody><tr>
          <td><a href="/maps/2026-09-20_Schuvalovo/"><span aria-hidden="true">📁</span> 2026-09-20_Schuvalovo</a></td>
        </tr><tr>
          <td><a href="/maps/2026-09-20_Orlovo/"><span aria-hidden="true">📁</span> 2026-09-20_Orlovo</a></td>
        </tr><tr>
          <td><a href="/maps/%21RAZNOE/"><span aria-hidden="true">📁</span> !RAZNOE</a></td>
        </tr><tr>
          <td><a href="/maps/tracks/"><span aria-hidden="true">📁</span> tracks</a></td>
        </tr></tbody>
        <a class="button-link secondary-link" href="/maps/?after=eyJ2IjoxfQ">Следующая страница</a>
    "##;

    #[test]
    fn project_listing_reads_dated_projects_and_the_next_page_cursor() {
        let base = "https://maps.lizaalert.ru/maps/";
        let (projects, next) = parse_project_listing(base, ROOT_LISTING_HTML).expect("listing");

        let slugs: Vec<&str> = projects.iter().map(|p| p.slug.as_str()).collect();
        assert_eq!(
            slugs,
            vec!["2026-09-20_Schuvalovo", "2026-09-20_Orlovo"],
            "only dated project directories, in page order"
        );
        assert_eq!(projects[0].name, "2026-09-20 Schuvalovo");
        assert_eq!(
            projects[0].url,
            "https://maps.lizaalert.ru/maps/2026-09-20_Schuvalovo/"
        );
        assert_eq!(
            next.as_deref(),
            Some("https://maps.lizaalert.ru/maps/?after=eyJ2IjoxfQ"),
            "the walk must follow the cursor or it stops at the first page"
        );
    }

    #[test]
    fn project_listing_reports_no_cursor_on_the_last_page() {
        let base = "https://maps.lizaalert.ru/maps/";
        let html = r##"<a href="/maps/2026-09-20_Last/"><span>📁</span> 2026-09-20_Last</a>"##;
        let (projects, next) = parse_project_listing(base, html).expect("listing");
        assert_eq!(projects.len(), 1);
        assert!(next.is_none());
    }

    #[test]
    fn map_packages_are_read_from_the_current_markup() {
        let base = "https://maps.lizaalert.ru/maps/2026-09-20_Schuvalovo/8-Android&iOS/";
        let html = r##"
          <td><a href="/maps/2026-09-20_Schuvalovo/8-Android&amp;iOS/2026-09-20_Schuvalovo_Topo_GGC_z16.sqlitedb"><span aria-hidden="true">↓</span> 2026-09-20_Schuvalovo_Topo_GGC_z16.sqlitedb</a></td>
          <td><a href="/maps/2026-09-20_Schuvalovo/8-Android&amp;iOS/2026-09-20_Schuvalovo_Satell_z17.sqlitedb"><span aria-hidden="true">↓</span> 2026-09-20_Schuvalovo_Satell_z17.sqlitedb</a></td>
        "##;

        let maps = parse_map_packages(html, base).expect("maps");
        assert_eq!(maps.len(), 2);
        // Entries come back sorted by name, not in page order.
        let satell = maps
            .iter()
            .find(|m| m.file_name.contains("Satell"))
            .expect("satellite map");
        let topo = maps
            .iter()
            .find(|m| m.file_name.contains("Topo"))
            .expect("topo map");
        assert_eq!(satell.base_zoom, 17);
        assert_eq!(topo.base_zoom, 16);
        assert_eq!(
            topo.url,
            "https://maps.lizaalert.ru/maps/2026-09-20_Schuvalovo/8-Android&iOS/2026-09-20_Schuvalovo_Topo_GGC_z16.sqlitedb"
        );
    }

    #[test]
    fn parse_directory_entries_skips_parent_links_and_marks_directories() {
        let html = r#"
            <a href="../">../</a>
            <a href="8-Android%26iOS/">8-Android&amp;iOS/</a>
            <a href="5-Ozi.zip">5-Ozi.zip</a>
        "#;

        let entries =
            parse_directory_entries("https://maps.lizaalert.ru/maps/proj/", html).expect("entries");

        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].name, "5-Ozi.zip");
        assert!(!entries[0].is_dir);
        assert_eq!(entries[1].name, "8-Android&iOS");
        assert!(entries[1].is_dir);
    }

    #[test]
    fn load_cached_project_from_root_reads_local_coordinates_and_ozi_maps() {
        let root = write_cached_project_fixture();
        let summary = LizaProjectSummary {
            slug: "2026-03-29_demo".to_owned(),
            name: "2026-03-29 demo".to_owned(),
            url: "https://example.test/project/".to_owned(),
        };

        let project = load_cached_project_from_root(summary.clone(), &root).expect("project");

        assert_eq!(project.summary, summary);
        assert_eq!(project.center.lat, 54.32821);
        assert_eq!(project.center.lon, 48.40917);
        assert_eq!(project.maps.len(), 2);
        assert_eq!(project.maps[0].base_zoom, 0);
        assert_eq!(project.maps[0].name, "OZI: Demo topo");
        let expected_ozi_path = root.join("2026-03-29_demo/5-Ozi/Maps/demo.map");
        assert_eq!(
            project.maps[0].local_path.as_deref(),
            Some(expected_ozi_path.as_path())
        );
        assert_eq!(project.maps[1].base_zoom, 16);
        let expected_sqlite_path = root.join("2026-03-29_demo/8-Android&iOS/demo_z16.sqlitedb");
        assert_eq!(
            project.maps[1].local_path.as_deref(),
            Some(expected_sqlite_path.as_path())
        );
    }

    #[test]
    fn materialize_cached_ozi_archives_extracts_zip_for_cached_project_indexing() {
        let root = write_cached_project_zip_fixture();

        materialize_cached_ozi_archives(&root, "2026-03-29_demo", &mut |_| {})
            .expect("extract ozi archives");

        let summary = LizaProjectSummary {
            slug: "2026-03-29_demo".to_owned(),
            name: "2026-03-29 demo".to_owned(),
            url: "https://example.test/project/".to_owned(),
        };
        let project = load_cached_project_from_root(summary, &root).expect("project");

        assert_eq!(project.maps.len(), 2);
        assert_eq!(project.maps[0].name, "OZI: Demo topo");
        assert!(
            project.maps[0]
                .local_path
                .as_ref()
                .expect("ozi path")
                .to_string_lossy()
                .contains("extracted/5-Ozi(Win&Android)_Topo/Maps/demo.map")
        );
    }

    /// Bundle progress reaches the status bar verbatim, so as long as it is a
    /// sentence built in Rust it is English in a Russian window. Every message
    /// carries a key and its arguments now; the English text stays as what a
    /// diagnostic reads and as the fallback when a build has no translation.
    #[test]
    fn every_progress_message_carries_a_key_and_its_arguments() {
        use super::ProgressText;

        let cases = [
            (
                ProgressText::ScanningDirectory {
                    path: "/bundles/2026-09-21_demo".to_owned(),
                },
                "progress.scanningDirectory",
                vec!["/bundles/2026-09-21_demo".to_owned()],
                "Scanning /bundles/2026-09-21_demo",
            ),
            (
                ProgressText::DownloadingInParallel { total: 3 },
                "progress.downloadingInParallel",
                vec!["3".to_owned()],
                "Downloading 3 files in parallel",
            ),
            (
                ProgressText::DownloadedOfFiles {
                    completed: 2,
                    total: 3,
                },
                "progress.downloadedOfFiles",
                vec!["2".to_owned(), "3".to_owned()],
                "Downloaded 2 of 3 files",
            ),
            (
                ProgressText::OpeningCachedBundle {
                    name: "Ветер".to_owned(),
                },
                "progress.openingCachedBundle",
                vec!["Ветер".to_owned()],
                "Opening cached project bundle: Ветер",
            ),
        ];

        for (text, key, args, english) in cases {
            assert_eq!(text.key(), key);
            assert_eq!(text.args(), args, "arguments for {key}");
            assert_eq!(text.english(), english, "english text for {key}");
        }
    }

    /// The catalogue walk is a thousand pages deep at worst, and it holds the
    /// application busy for its whole length. A crew that needs a bundle now
    /// must be able to stop waiting for it — and stopping must not pass a
    /// half-walked catalogue off as the complete one.
    #[test]
    fn a_stopped_catalogue_walk_says_so_and_keeps_what_it_read() {
        use std::io::Read;
        use std::net::TcpListener;

        let listener = TcpListener::bind("127.0.0.1:0").expect("bind");
        let addr = listener.local_addr().expect("addr");

        // Two pages, each pointing at the next through the cursor the real
        // listing uses. The walk must stop after the first one.
        let server = std::thread::spawn(move || {
            for page in 0..2 {
                let Ok((mut stream, _)) = listener.accept() else {
                    return;
                };
                let mut request = [0u8; 1024];
                let _ = stream.read(&mut request);
                let body = if page == 0 {
                    r#"<a href="2026-09-21_first/">first</a>
                       <a href="?after=2026-09-21_first">next</a>"#
                } else {
                    r#"<a href="2026-09-20_second/">second</a>"#
                };
                let _ = stream.write_all(
                    format!(
                        "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
                        body.len()
                    )
                    .as_bytes(),
                );
            }
        });

        let cancel = super::CancelToken::new();
        let mut seen_pages = 0usize;
        let walk =
            super::walk_project_listing(&format!("http://{addr}/"), &cancel, |_chunk, page| {
                seen_pages = page;
                // The operator presses stop while the first page is on screen.
                cancel.cancel();
            })
            .expect("a stopped walk is not a failure");

        // Deliberately not joined: the point of the test is that the second
        // page is never requested, so the server thread is still blocked in
        // `accept` and joining it would hang here instead of failing.
        drop(server);

        assert!(walk.cancelled, "the walk must report that it was stopped");
        assert_eq!(walk.pages, 1, "it must not fetch the page after the stop");
        assert_eq!(seen_pages, 1);
        assert_eq!(
            walk.projects
                .iter()
                .map(|p| p.slug.as_str())
                .collect::<Vec<_>>(),
            vec!["2026-09-21_first"],
            "what it did read stays; only the rest is missing"
        );
    }

    #[test]
    fn download_map_failure_leaves_no_file_at_final_path() {
        use super::download_map;
        use crate::application::{ActiveMapKind, ActiveMapSelection, MapCenter};
        use std::io::Read;
        use std::net::TcpListener;

        // Raw TCP server that announces 1000 body bytes but sends only 100
        // and then drops the connection — a deterministic mid-body failure.
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind");
        let addr = listener.local_addr().expect("addr");
        let server = std::thread::spawn(move || {
            let (mut stream, _) = listener.accept().expect("accept");
            let mut request = [0u8; 1024];
            let _ = stream.read(&mut request);
            let _ = stream
                .write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 1000\r\nConnection: close\r\n\r\n");
            let _ = stream.write_all(&[0xABu8; 100]);
        });

        let dir = tempfile::tempdir().expect("tempdir");
        let local_path = dir.path().join("demo_z16.sqlitedb");
        let selection = ActiveMapSelection {
            kind: ActiveMapKind::SqliteTiles,
            project_name: "demo".to_owned(),
            package_name: "demo_z16.sqlitedb".to_owned(),
            remote_url: format!("http://{addr}/demo_z16.sqlitedb"),
            local_path: local_path.clone(),
            center: MapCenter {
                lat: 54.0,
                lon: 48.0,
            },
            base_zoom: 16,
        };

        let result = download_map(selection, &super::CancelToken::new(), |_| {});
        server.join().expect("server thread");

        assert!(result.is_err(), "truncated download must fail");
        assert!(
            !local_path.exists(),
            "interrupted download must not leave a truncated file at the final path"
        );
        assert!(
            !local_path.with_extension("part").exists(),
            "interrupted download must clean up its .part file"
        );
    }

    #[test]
    fn failed_archive_extraction_leaves_no_destination_and_allows_retry() {
        let root = write_cached_project_corrupt_zip_fixture();
        let dest = root.join("2026-03-29_demo/extracted/5-Ozi(Win&Android)_Topo");

        let result = materialize_cached_ozi_archives(&root, "2026-03-29_demo", &mut |_| {});

        assert!(result.is_err(), "corrupt archive extraction must fail");
        assert!(
            !dest.exists(),
            "failed extraction must not leave a partial destination directory"
        );

        // Repair the archive; the retry must actually extract instead of
        // treating the previous partial output as already extracted.
        fs::write(
            root.join("2026-03-29_demo/5-Ozi(Win&Android)_Topo.zip"),
            build_archive(&[
                ("Maps/demo.map", sample_ozi_map().as_bytes(), false),
                ("Maps/demo.ozf2", b"ozf-placeholder".as_slice(), false),
            ]),
        )
        .expect("write repaired zip");
        materialize_cached_ozi_archives(&root, "2026-03-29_demo", &mut |_| {})
            .expect("retry extraction");
        assert!(dest.join("Maps/demo.map").exists());
        assert!(dest.join("Maps/demo.ozf2").exists());
    }

    fn write_cached_project_corrupt_zip_fixture() -> std::path::PathBuf {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time")
            .as_nanos();
        let root = std::env::temp_dir().join(format!("ozi-rs-lizaalert-corrupt-zip-{unique}"));
        let bundle_dir = root.join("2026-03-29_demo");
        fs::create_dir_all(&bundle_dir).expect("create bundle dir");
        fs::write(
            bundle_dir.join("2-Coordinates.txt"),
            "N 54.32821 E 048.40917",
        )
        .expect("write coordinates");

        // Valid central directory and first entry, but the second entry's
        // stored payload is corrupted so its CRC check fails mid-extraction —
        // after the first entry has already been written to disk.
        let payload = b"ozf2-corruptible-payload-0123456789";
        let mut zip_bytes = build_archive(&[
            ("Maps/demo.map", sample_ozi_map().as_bytes(), false),
            ("Maps/demo.ozf2", payload.as_slice(), false),
        ]);
        let position = zip_bytes
            .windows(payload.len())
            .position(|window| window == payload)
            .expect("stored payload bytes present in zip");
        zip_bytes[position + 4] ^= 0xFF;
        fs::write(bundle_dir.join("5-Ozi(Win&Android)_Topo.zip"), zip_bytes).expect("write zip");
        root
    }

    #[test]
    fn decode_text_bytes_falls_back_lossy_for_non_utf8() {
        let text = decode_text_bytes(b"demo \xFF bundle");

        assert!(text.contains("demo "));
        assert!(text.contains("bundle"));
    }

    #[test]
    fn read_text_file_lossy_reads_non_utf8_coordinates_file() {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time")
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "ozi-rs-non-utf8-coordinates-{}-{unique}.txt",
            std::process::id()
        ));
        fs::write(&path, b"N 54.32821 E 048.40917 \xFF").expect("write coordinates bytes");

        let text = read_text_file_lossy(&path).expect("lossy coordinates text");

        assert!(text.contains("54.32821"));
        assert!(text.contains("048.40917"));
    }

    /// The remote walk stopped assuming a folder name when the site changed —
    /// it now descends into whatever subdirectory holds the `.sqlitedb`. The
    /// local mirror kept reading only `8-Android&iOS`, so a bundle stored
    /// anywhere else read as "not downloaded" and was fetched again.
    #[test]
    fn cached_sqlite_maps_are_found_in_any_subdirectory() {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time")
            .as_nanos();
        let root = std::env::temp_dir().join(format!("ozi-rs-cached-sqlite-{unique}"));
        let bundle = root.join("2026-09-21_demo");
        let odd_dir = bundle.join("9-Mobile").join("maps");
        fs::create_dir_all(&odd_dir).expect("create nested dir");
        fs::write(odd_dir.join("demo_z16.sqlitedb"), []).expect("write sqlite placeholder");
        // A file that is not a map must not become one.
        fs::write(odd_dir.join("readme.txt"), b"hello").expect("write txt");

        let zoom_regex = regex::Regex::new(r"_z(\d+)\.sqlitedb$").expect("regex");
        let maps = read_cached_sqlite_map_packages(&root, "2026-09-21_demo", &zoom_regex)
            .expect("read cached sqlite maps");

        assert_eq!(maps.len(), 1, "the nested .sqlitedb SHALL be found");
        assert_eq!(maps[0].file_name, "demo_z16.sqlitedb");
        assert_eq!(maps[0].base_zoom, 16);
        assert!(maps[0].local_path.is_some());

        let _ = fs::remove_dir_all(&root);
    }

    /// The online preview finds the coordinates file by pattern; the cache
    /// used to demand the exact `2-Coordinates.txt` spelling, so a bundle that
    /// previewed online was reported as not cached at all.
    #[test]
    fn a_cached_bundle_is_recognised_by_a_differently_named_coordinates_file() {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time")
            .as_nanos();
        let root = std::env::temp_dir().join(format!("ozi-rs-coords-name-{unique}"));
        let bundle = root.join("2026-09-21_demo");
        fs::create_dir_all(&bundle).expect("create bundle dir");
        fs::write(bundle.join("1-Coordinates.txt"), "N 54.32821 E 048.40917")
            .expect("write coordinates");

        assert!(
            super::is_project_cached("2026-09-21_demo", &root),
            "a bundle with a differently numbered coordinates file is still cached"
        );

        let _ = fs::remove_dir_all(&root);
    }

    /// Offline, an undifferentiated list of thirteen thousand projects is a
    /// guess. The loader needs to know which of them are already on disk.
    #[test]
    fn cached_project_slugs_lists_only_bundles_that_are_actually_there() {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time")
            .as_nanos();
        let root = std::env::temp_dir().join(format!("ozi-rs-cached-slugs-{unique}"));
        let complete = root.join("2026-09-21_complete");
        let partial = root.join("2026-09-20_partial");
        fs::create_dir_all(&complete).expect("create complete");
        fs::create_dir_all(partial.join("8-Android&iOS")).expect("create partial");
        fs::write(complete.join("2-Coordinates.txt"), "N 54.3 E 048.4").expect("write coords");
        fs::write(root.join("stray.txt"), b"not a bundle").expect("write stray file");

        let slugs = super::cached_project_slugs(&root);

        assert!(slugs.contains("2026-09-21_complete"));
        assert!(
            !slugs.contains("2026-09-20_partial"),
            "a directory without coordinates is not an openable bundle"
        );
        assert_eq!(slugs.len(), 1);

        let _ = fs::remove_dir_all(&root);
    }

    /// A field link goes through a phone, and a stalled TCP connection used to
    /// hold a bundle download open forever: the panel froze on the file that
    /// stopped and nothing told it apart from a slow link. The read timeout is
    /// per-read, so a slow download still finishes; only a dead one fails.
    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn a_stalled_transfer_fails_instead_of_hanging() {
        use std::io::Read;
        use std::net::TcpListener;

        // Announces a body and then never sends it, holding the socket open.
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind");
        let addr = listener.local_addr().expect("addr");
        let stop = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
        let stop_for_server = std::sync::Arc::clone(&stop);
        let server = std::thread::spawn(move || {
            let (mut stream, _) = listener.accept().expect("accept");
            let mut request = [0u8; 1024];
            let _ = stream.read(&mut request);
            let _ = stream
                .write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 1000\r\nConnection: close\r\n\r\n");
            let _ = stream.write_all(&[0xABu8; 10]);
            // Hold the connection open with nothing more to say.
            while !stop_for_server.load(std::sync::atomic::Ordering::Relaxed) {
                std::thread::sleep(std::time::Duration::from_millis(20));
            }
        });

        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("stalled.sqlitedb");
        let client = super::async_client(
            std::time::Duration::from_secs(5),
            std::time::Duration::from_millis(300),
        )
        .expect("client");

        let result = super::download_to_path_async(
            &client,
            &format!("http://{addr}/stalled.sqlitedb"),
            &path,
            &super::CancelToken::new(),
            |_, _| {},
        )
        .await;

        stop.store(true, std::sync::atomic::Ordering::Relaxed);
        let _ = server.join();

        assert!(result.is_err(), "a stalled transfer SHALL fail");
        assert!(
            !path.exists(),
            "a stalled transfer SHALL NOT leave a file at the final path"
        );
        assert!(
            !path.with_extension("part").exists(),
            "a stalled transfer SHALL clean up its .part file"
        );
    }

    /// Committing to a download without knowing its size is the difference
    /// between fifteen megabytes and two hundred on a phone tether.
    #[test]
    fn map_packages_carry_the_size_the_listing_prints() {
        let html = r#"
            <table><tbody>
            <tr>
              <td><a href="/maps/demo/8-Android&amp;iOS/demo_Topo_z16.sqlitedb">demo_Topo_z16.sqlitedb</a></td>
              <td>15,9 МиБ</td><td><time>08.07.2026</time></td>
            </tr>
            <tr>
              <td><a href="/maps/demo/8-Android&amp;iOS/demo_Satell_z17.sqlitedb">demo_Satell_z17.sqlitedb</a></td>
              <td>185.1 МиБ</td><td><time>08.07.2026</time></td>
            </tr>
            <tr>
              <td><a href="/maps/demo/8-Android&amp;iOS/notes.txt">notes.txt</a></td>
              <td>742 Б</td><td><time>08.07.2026</time></td>
            </tr>
            </tbody></table>
        "#;

        let maps = super::parse_map_packages(html, "https://example.com/maps/demo/8-Android&iOS/")
            .expect("maps");

        assert_eq!(maps.len(), 2);
        let topo = maps
            .iter()
            .find(|m| m.file_name.contains("Topo"))
            .expect("topo map");
        assert_eq!(
            topo.size_bytes,
            Some((15.9_f64 * 1024.0 * 1024.0).round() as u64),
            "a comma decimal separator is still a number"
        );
        let satell = maps
            .iter()
            .find(|m| m.file_name.contains("Satell"))
            .expect("satellite map");
        assert_eq!(
            satell.size_bytes,
            Some((185.1_f64 * 1024.0 * 1024.0).round() as u64)
        );
    }

    #[test]
    fn a_listing_without_sizes_leaves_them_unknown() {
        let html = r#"<a href="foo_z16.sqlitedb">foo_z16.sqlitedb</a>"#;
        let maps = super::parse_map_packages(html, "https://example.com/").expect("maps");

        assert_eq!(maps.len(), 1);
        assert_eq!(
            maps[0].size_bytes, None,
            "an unknown size SHALL stay unknown rather than become zero"
        );
    }

    #[test]
    fn human_sizes_are_read_in_both_alphabets_and_rejected_when_they_are_not_sizes() {
        assert_eq!(super::parse_human_size("742 Б"), Some(742));
        assert_eq!(super::parse_human_size("1 КиБ"), Some(1024));
        assert_eq!(super::parse_human_size("1 GiB"), Some(1024 * 1024 * 1024));
        assert_eq!(super::parse_human_size("папка"), None);
        assert_eq!(super::parse_human_size("—"), None);
        assert_eq!(super::parse_human_size("08.07.2026 14:58 UTC"), None);
    }

    fn write_cached_project_fixture() -> std::path::PathBuf {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time")
            .as_nanos();
        let root = std::env::temp_dir().join(format!("ozi-rs-lizaalert-cache-{unique}"));
        // Flat structure: files directly in {root}/{slug}/, no source/ subdir
        let bundle_dir = root.join("2026-03-29_demo");
        let mobile_dir = bundle_dir.join("8-Android&iOS");
        let ozi_dir = bundle_dir.join("5-Ozi/Maps");
        fs::create_dir_all(&mobile_dir).expect("create mobile dir");
        fs::create_dir_all(&ozi_dir).expect("create ozi dir");
        fs::write(
            bundle_dir.join("2-Coordinates.txt"),
            "N 54.32821 E 048.40917",
        )
        .expect("write coordinates");
        fs::write(mobile_dir.join("demo_z16.sqlitedb"), []).expect("write sqlite placeholder");
        fs::write(ozi_dir.join("demo.map"), sample_ozi_map()).expect("write ozi map");
        root
    }

    fn write_cached_project_zip_fixture() -> std::path::PathBuf {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time")
            .as_nanos();
        let root = std::env::temp_dir().join(format!("ozi-rs-lizaalert-cache-zip-{unique}"));
        // Flat structure: files directly in {root}/{slug}/, no source/ subdir
        let bundle_dir = root.join("2026-03-29_demo");
        let mobile_dir = bundle_dir.join("8-Android&iOS");
        fs::create_dir_all(&mobile_dir).expect("create mobile dir");
        fs::write(
            bundle_dir.join("2-Coordinates.txt"),
            "N 54.32821 E 048.40917",
        )
        .expect("write coordinates");
        fs::write(mobile_dir.join("demo_z16.sqlitedb"), []).expect("write sqlite placeholder");
        fs::write(
            bundle_dir.join("5-Ozi(Win&Android)_Topo.zip"),
            build_archive(&[
                ("Maps/demo.map", sample_ozi_map().as_bytes(), false),
                ("Maps/demo.ozf2", b"ozf-placeholder".as_slice(), false),
            ]),
        )
        .expect("write ozi zip");
        root
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

    fn sample_ozi_map() -> &'static str {
        "OziExplorer Map Data File Version 2.2\nDemo topo\ndemo.ozf2\n1 ,Map Code,\nWGS 84\nReserved 1\nReserved 2\nMagnetic Variation,,,E\nMap Projection,Latitude/Longitude,PolyCal,No,AutoCalOnly,No,BSBUseWPX,No\nPoint01,xy,10,20,in, deg,54,30.000,N,48,24.000,E, grid, , , ,N\nProjection Setup,,,,,,,,,,\n"
    }

    #[test]
    fn leading_prefix_extracts_numeric_prefix() {
        use super::leading_prefix;
        assert_eq!(leading_prefix("10-Tracks/file.gpx").0, Some(10));
        assert_eq!(leading_prefix("00-manifest.json").0, Some(0));
        assert_eq!(leading_prefix("99-refs.pdf").0, Some(99));
        assert_eq!(leading_prefix("readme.md").0, None);
        assert_eq!(leading_prefix("3D-Models/foo").0, None);
        assert_eq!(leading_prefix("123_legacy.txt").0, Some(123));
    }

    #[test]
    fn cancel_token_propagates() {
        use super::CancelToken;
        let t = CancelToken::new();
        assert!(!t.is_cancelled());
        let c = t.clone();
        c.cancel();
        assert!(t.is_cancelled());
    }

    #[test]
    fn sort_remote_files_by_prefix_orders_numeric_first() {
        use super::{RemoteFileDownload, sort_remote_files_by_prefix};
        let mk = |rel: &str| RemoteFileDownload {
            url: format!("https://example.test/{rel}"),
            path: std::path::PathBuf::from(rel),
            relative: rel.to_owned(),
            size_bytes: None,
        };
        let mut files = vec![
            mk("99-refs.pdf"),
            mk("readme.md"),
            mk("00-manifest.json"),
            mk("10-Tracks/a.ozf2"),
            mk("20-overlay.zip"),
            mk("10-Tracks/b.ozf2"),
            mk("alpha.txt"),
        ];
        sort_remote_files_by_prefix(&mut files);
        let order: Vec<_> = files.iter().map(|f| f.relative.as_str()).collect();
        assert_eq!(
            order,
            vec![
                "00-manifest.json",
                "10-Tracks/a.ozf2",
                "10-Tracks/b.ozf2",
                "20-overlay.zip",
                "99-refs.pdf",
                "alpha.txt",
                "readme.md",
            ]
        );
    }
}

#[cfg(test)]
mod bundle_download_tests {
    //! Integration tests for the multi-file download orchestrator.
    //!
    //! Uses `wiremock` to stand up a fake Apache-style directory listing and
    //! per-file responses; exercises the public async API end to end (no
    //! mocking of the inside).

    use super::{
        BundleDownloadConfig, CANCEL_ERROR, CancelToken, DEFAULT_BUNDLE_DOWNLOAD_CONCURRENCY,
        DownloadNotification, download_bundle_concurrent,
    };
    use std::sync::Arc;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::time::Duration;
    use tokio::sync::mpsc;
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, Request, ResponseTemplate};

    /// Build a minimal Apache-style HTML index that the parser will accept.
    fn index_html(entries: &[(&str, bool)]) -> String {
        // Apache appends a trailing slash for directories. The current parser
        // looks at `href="..."` so we match that shape.
        let mut html = String::from("<html><body><pre>\n");
        html.push_str(r#"<a href="../">../</a>\n"#);
        for (name, is_dir) in entries {
            let href = if *is_dir {
                format!("{name}/")
            } else {
                (*name).to_string()
            };
            html.push_str(&format!(r#"<a href="{href}">{href}</a>"#));
            html.push('\n');
        }
        html.push_str("</pre></body></html>\n");
        html
    }

    async fn collect_notifications(
        mut rx: mpsc::UnboundedReceiver<DownloadNotification>,
    ) -> Vec<DownloadNotification> {
        let mut out = Vec::new();
        while let Some(n) = rx.recv().await {
            out.push(n);
        }
        out
    }

    fn body_for(name: &str) -> Vec<u8> {
        // Distinct, deterministic content per file. Size also varies so the
        // monotonic-progress assertion is meaningful.
        match name {
            "00-manifest.json" => b"{\"v\":1}".repeat(2),
            "10-Tracks/a.ozf2" => vec![0xAAu8; 3 * 1024],
            "10-Tracks/b.ozf2" => vec![0xBBu8; 4 * 1024],
            "20-overlay.zip" => vec![0xCCu8; 8 * 1024],
            "99-refs.pdf" => vec![0xDDu8; 16 * 1024],
            _ => b"x".to_vec(),
        }
    }

    /// A field link drops transfers. With timeouts in place a stalled one now
    /// fails instead of hanging — and then the whole bundle failed with it,
    /// after however many files had already come down. One flaky file should
    /// not cost a crew the bundle.
    #[tokio::test]
    async fn a_dropped_transfer_is_retried_before_the_file_is_given_up() {
        use std::sync::atomic::{AtomicUsize, Ordering};

        let server = MockServer::start().await;
        let attempts = Arc::new(AtomicUsize::new(0));
        let seen = Arc::clone(&attempts);

        Mock::given(method("GET"))
            .and(path("/flaky.sqlitedb"))
            .respond_with(move |_: &Request| {
                // The first attempt dies; the second is served.
                if seen.fetch_add(1, Ordering::SeqCst) == 0 {
                    ResponseTemplate::new(503)
                } else {
                    ResponseTemplate::new(200).set_body_bytes(vec![7u8; 1024])
                }
            })
            .mount(&server)
            .await;

        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("flaky.sqlitedb");
        let client =
            super::async_client(super::CONNECT_TIMEOUT, super::READ_TIMEOUT).expect("client");

        let result = super::download_file_with_retries(
            &client,
            &format!("{}/flaky.sqlitedb", server.uri()),
            &path,
            &CancelToken::new(),
            |_, _| {},
            |_attempt| {},
        )
        .await;

        assert!(result.is_ok(), "the second attempt succeeded: {result:?}");
        assert_eq!(attempts.load(Ordering::SeqCst), 2, "one retry, not more");
        assert_eq!(
            tokio::fs::metadata(&path).await.expect("file").len(),
            1024,
            "and the file that landed is the whole file"
        );
    }

    /// Cancelling is not a transport failure: retrying it would ignore the
    /// operator and keep the link busy after they asked it to stop.
    #[tokio::test]
    async fn a_cancelled_transfer_is_not_retried() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/never.sqlitedb"))
            .respond_with(ResponseTemplate::new(200).set_body_bytes(vec![0u8; 16]))
            .mount(&server)
            .await;

        let dir = tempfile::tempdir().expect("tempdir");
        let cancel = CancelToken::new();
        cancel.cancel();

        let mut retries = 0usize;
        let result = super::download_file_with_retries(
            &super::async_client(super::CONNECT_TIMEOUT, super::READ_TIMEOUT).expect("client"),
            &format!("{}/never.sqlitedb", server.uri()),
            &dir.path().join("never.sqlitedb"),
            &cancel,
            |_, _| {},
            |_attempt| retries += 1,
        )
        .await;

        assert!(result.is_err(), "a cancelled transfer fails");
        assert_eq!(retries, 0, "and is not retried");
    }

    async fn setup_bundle_server() -> MockServer {
        let server = MockServer::start().await;

        // Root directory listing.
        Mock::given(method("GET"))
            .and(path("/bundle/"))
            .respond_with(ResponseTemplate::new(200).set_body_string(index_html(&[
                ("00-manifest.json", false),
                ("10-Tracks", true),
                ("20-overlay.zip", false),
                ("99-refs.pdf", false),
            ])))
            .mount(&server)
            .await;

        // Nested directory listing.
        Mock::given(method("GET"))
            .and(path("/bundle/10-Tracks/"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_string(index_html(&[("a.ozf2", false), ("b.ozf2", false)])),
            )
            .mount(&server)
            .await;

        for name in [
            "00-manifest.json",
            "10-Tracks/a.ozf2",
            "10-Tracks/b.ozf2",
            "20-overlay.zip",
            "99-refs.pdf",
        ] {
            Mock::given(method("GET"))
                .and(path(format!("/bundle/{name}")))
                .respond_with(ResponseTemplate::new(200).set_body_bytes(body_for(name)))
                .mount(&server)
                .await;
        }

        server
    }

    /// The owner's July note: the Android tile packs and the print sheets are
    /// most of a bundle's weight and this app cannot open either. What to
    /// leave behind is the operator's call, so nothing is skipped unless they
    /// name it.
    #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
    async fn a_named_top_level_entry_is_left_on_the_server() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/bundle/"))
            .respond_with(ResponseTemplate::new(200).set_body_string(index_html(&[
                ("00-manifest.json", false),
                ("9-Map_4_print", true),
                ("10-Tracks", true),
            ])))
            .mount(&server)
            .await;
        Mock::given(method("GET"))
            .and(path("/bundle/9-Map_4_print/"))
            .respond_with(
                ResponseTemplate::new(200).set_body_string(index_html(&[("sheet-1.pdf", false)])),
            )
            .mount(&server)
            .await;
        Mock::given(method("GET"))
            .and(path("/bundle/10-Tracks/"))
            .respond_with(
                ResponseTemplate::new(200).set_body_string(index_html(&[("a.gpx", false)])),
            )
            .mount(&server)
            .await;
        for file in [
            "00-manifest.json",
            "10-Tracks/a.gpx",
            "9-Map_4_print/sheet-1.pdf",
        ] {
            Mock::given(method("GET"))
                .and(path(format!("/bundle/{file}")))
                .respond_with(ResponseTemplate::new(200).set_body_bytes(vec![1u8; 8]))
                .mount(&server)
                .await;
        }

        let tmp = tempdir();
        let cfg = BundleDownloadConfig {
            concurrency: DEFAULT_BUNDLE_DOWNLOAD_CONCURRENCY,
            url: format!("{}/bundle/", server.uri()),
            local_dir: tmp.path().to_path_buf(),
            cancel: CancelToken::new(),
            skip_top_level: vec!["9-Map_4_print".to_owned()],
        };
        let (tx, rx) = mpsc::unbounded_channel();
        let handle = tokio::spawn(download_bundle_concurrent(cfg, tx));
        let _ = collect_notifications(rx).await;
        handle.await.expect("join").expect("download");

        assert!(
            tmp.path().join("00-manifest.json").exists(),
            "what was not skipped still arrives"
        );
        assert!(
            tmp.path().join("10-Tracks/a.gpx").exists(),
            "including the folders below the top level"
        );
        assert!(
            !tmp.path().join("9-Map_4_print/sheet-1.pdf").exists(),
            "the named entry SHALL be left on the server"
        );
    }

    /// The operator presses one button and the app fetches the whole project
    /// directory. Until the scan reported a total, the panel could say "3 of
    /// 47 files" and nothing about whether that was ten megabytes or two
    /// gigabytes — which on a tethered phone is the whole question.
    #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
    async fn the_scan_reports_what_the_bundle_weighs() {
        let server = MockServer::start().await;
        // A listing with sizes, in the shape the site serves them.
        let listing = r#"<table><tbody>
            <tr><td><a href="/bundle/00-manifest.json">00-manifest.json</a></td><td>2 КиБ</td></tr>
            <tr><td><a href="/bundle/20-overlay.zip">20-overlay.zip</a></td><td>1,5 МиБ</td></tr>
        </tbody></table>"#;
        Mock::given(method("GET"))
            .and(path("/bundle/"))
            .respond_with(ResponseTemplate::new(200).set_body_string(listing))
            .mount(&server)
            .await;
        for name in ["00-manifest.json", "20-overlay.zip"] {
            Mock::given(method("GET"))
                .and(path(format!("/bundle/{name}")))
                .respond_with(ResponseTemplate::new(200).set_body_bytes(vec![7u8; 64]))
                .mount(&server)
                .await;
        }

        let tmp = tempdir();
        let cfg = BundleDownloadConfig {
            concurrency: DEFAULT_BUNDLE_DOWNLOAD_CONCURRENCY,
            url: format!("{}/bundle/", server.uri()),
            local_dir: tmp.path().to_path_buf(),
            cancel: CancelToken::new(),
            skip_top_level: Vec::new(),
        };
        let (tx, rx) = mpsc::unbounded_channel();
        let handle = tokio::spawn(download_bundle_concurrent(cfg, tx));
        let notifications = collect_notifications(rx).await;
        handle.await.expect("join").expect("download");

        let expected = 2 * 1024 + (1.5_f64 * 1024.0 * 1024.0).round() as u64;
        let announced: Vec<u64> = notifications
            .iter()
            .filter_map(|n| match n {
                DownloadNotification::Phase(p) => p.total_bytes,
                _ => None,
            })
            .collect();
        assert!(
            announced.iter().all(|total| *total == expected),
            "the scan SHALL announce the bundle's total size ({expected}), got {announced:?}"
        );
        assert!(
            !announced.is_empty(),
            "a bundle whose listing states sizes SHALL report a total"
        );
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
    async fn per_file_progress_is_monotonic_and_complete() {
        let server = setup_bundle_server().await;
        let tmp = tempdir();
        let cfg = BundleDownloadConfig {
            concurrency: DEFAULT_BUNDLE_DOWNLOAD_CONCURRENCY,
            url: format!("{}/bundle/", server.uri()),
            local_dir: tmp.path().to_path_buf(),
            cancel: CancelToken::new(),
            skip_top_level: Vec::new(),
        };

        let (tx, rx) = mpsc::unbounded_channel();
        let handle = tokio::spawn(download_bundle_concurrent(cfg, tx));
        let notifications = collect_notifications(rx).await;
        handle.await.unwrap().unwrap();

        // Collect last `downloaded_bytes` per file and check monotonicity.
        use std::collections::HashMap;
        let mut last_bytes: HashMap<String, u64> = HashMap::new();
        let mut ready: Vec<String> = Vec::new();
        for n in &notifications {
            match n {
                DownloadNotification::FileProgress {
                    package_name,
                    downloaded_bytes,
                    ..
                } => {
                    if let Some(prev) = last_bytes.get(package_name) {
                        assert!(
                            *downloaded_bytes >= *prev,
                            "progress regression for {package_name}: {prev} -> {downloaded_bytes}"
                        );
                    }
                    last_bytes.insert(package_name.clone(), *downloaded_bytes);
                }
                DownloadNotification::FileReady { package_name, .. } => {
                    ready.push(package_name.clone());
                }
                _ => {}
            }
        }

        // Every file got at least one progress event …
        for name in [
            "00-manifest.json",
            "10-Tracks/a.ozf2",
            "10-Tracks/b.ozf2",
            "20-overlay.zip",
            "99-refs.pdf",
        ] {
            assert!(
                last_bytes.contains_key(name),
                "no progress for {name}; got {:?}",
                last_bytes.keys()
            );
            assert_eq!(
                last_bytes[name],
                body_for(name).len() as u64,
                "final progress mismatch for {name}"
            );
            assert!(ready.iter().any(|p| p == name), "{name} not ready");
        }
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
    async fn prefix_ordered_scheduling_starts_smallest_first() {
        // Slow down the high-prefix file so we can prove the small-prefix
        // file starts (and finishes) ahead of it even though listing order
        // does not enforce that.
        let server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/bundle/"))
            .respond_with(ResponseTemplate::new(200).set_body_string(index_html(&[
                ("99-refs.pdf", false),
                ("00-manifest.json", false),
            ])))
            .mount(&server)
            .await;

        Mock::given(method("GET"))
            .and(path("/bundle/00-manifest.json"))
            .respond_with(ResponseTemplate::new(200).set_body_bytes(b"manifest".to_vec()))
            .mount(&server)
            .await;

        Mock::given(method("GET"))
            .and(path("/bundle/99-refs.pdf"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_bytes(vec![0xDD; 4096])
                    .set_delay(Duration::from_millis(300)),
            )
            .mount(&server)
            .await;

        let tmp = tempdir();
        let cfg = BundleDownloadConfig {
            concurrency: 2,
            url: format!("{}/bundle/", server.uri()),
            local_dir: tmp.path().to_path_buf(),
            cancel: CancelToken::new(),
            skip_top_level: Vec::new(),
        };

        let (tx, mut rx) = mpsc::unbounded_channel();
        let handle = tokio::spawn(download_bundle_concurrent(cfg, tx));

        let mut first_ready: Option<String> = None;
        while let Some(n) = rx.recv().await {
            if let DownloadNotification::FileReady { package_name, .. } = n
                && first_ready.is_none()
            {
                first_ready = Some(package_name);
            }
        }
        handle.await.unwrap().unwrap();
        assert_eq!(first_ready.as_deref(), Some("00-manifest.json"));
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
    async fn missing_content_length_still_emits_progress() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/bundle/"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_string(index_html(&[("00-no-clen.bin", false)])),
            )
            .mount(&server)
            .await;
        // Use chunked transfer (no Content-Length).
        Mock::given(method("GET"))
            .and(path("/bundle/00-no-clen.bin"))
            .respond_with(
                ResponseTemplate::new(200)
                    .insert_header("Transfer-Encoding", "chunked")
                    .set_body_bytes(b"hello".to_vec()),
            )
            .mount(&server)
            .await;

        let tmp = tempdir();
        let cfg = BundleDownloadConfig {
            concurrency: 1,
            url: format!("{}/bundle/", server.uri()),
            local_dir: tmp.path().to_path_buf(),
            cancel: CancelToken::new(),
            skip_top_level: Vec::new(),
        };
        let (tx, rx) = mpsc::unbounded_channel();
        let handle = tokio::spawn(download_bundle_concurrent(cfg, tx));
        let notifications = collect_notifications(rx).await;
        handle.await.unwrap().unwrap();

        // Should not panic and should emit at least one progress event.
        let saw_progress = notifications.iter().any(|n| {
            matches!(
                n,
                DownloadNotification::FileProgress {
                    package_name,
                    total_bytes: None,
                    ..
                }
                if package_name == "00-no-clen.bin"
            )
        });
        assert!(
            saw_progress,
            "expected FileProgress with total_bytes=None, got {notifications:?}"
        );
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
    async fn small_file_ready_before_large_file_completes() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/bundle/"))
            .respond_with(ResponseTemplate::new(200).set_body_string(index_html(&[
                ("10-small.bin", false),
                ("99-large.pdf", false),
            ])))
            .mount(&server)
            .await;
        Mock::given(method("GET"))
            .and(path("/bundle/10-small.bin"))
            .respond_with(ResponseTemplate::new(200).set_body_bytes(b"tiny".to_vec()))
            .mount(&server)
            .await;
        Mock::given(method("GET"))
            .and(path("/bundle/99-large.pdf"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_bytes(vec![0xDD; 8 * 1024])
                    .set_delay(Duration::from_millis(400)),
            )
            .mount(&server)
            .await;

        let tmp = tempdir();
        let cfg = BundleDownloadConfig {
            concurrency: 2,
            url: format!("{}/bundle/", server.uri()),
            local_dir: tmp.path().to_path_buf(),
            cancel: CancelToken::new(),
            skip_top_level: Vec::new(),
        };
        let (tx, mut rx) = mpsc::unbounded_channel();
        let handle = tokio::spawn(download_bundle_concurrent(cfg, tx));

        let mut small_ready_at: Option<std::time::Instant> = None;
        let mut large_ready_at: Option<std::time::Instant> = None;
        while let Some(n) = rx.recv().await {
            if let DownloadNotification::FileReady { package_name, .. } = n {
                let now = std::time::Instant::now();
                if package_name == "10-small.bin" {
                    small_ready_at = Some(now);
                } else if package_name == "99-large.pdf" {
                    large_ready_at = Some(now);
                }
            }
        }
        handle.await.unwrap().unwrap();
        let s = small_ready_at.expect("small ready");
        let l = large_ready_at.expect("large ready");
        assert!(s < l, "small file should be ready before large file");
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
    async fn cancel_aborts_within_deadline_and_resume_only_fetches_missing() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/bundle/"))
            .respond_with(ResponseTemplate::new(200).set_body_string(index_html(&[
                ("00-fast.bin", false),
                ("99-slow.bin", false),
            ])))
            .mount(&server)
            .await;
        Mock::given(method("GET"))
            .and(path("/bundle/00-fast.bin"))
            .respond_with(ResponseTemplate::new(200).set_body_bytes(b"done".to_vec()))
            .mount(&server)
            .await;
        // Very slow to give us a clean cancel window.
        Mock::given(method("GET"))
            .and(path("/bundle/99-slow.bin"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_bytes(vec![0xEE; 16 * 1024])
                    .set_delay(Duration::from_secs(3)),
            )
            .mount(&server)
            .await;

        let tmp = tempdir();
        let cancel = CancelToken::new();
        let cfg = BundleDownloadConfig {
            concurrency: 2,
            url: format!("{}/bundle/", server.uri()),
            local_dir: tmp.path().to_path_buf(),
            cancel: cancel.clone(),
            skip_top_level: Vec::new(),
        };
        let (tx, mut rx) = mpsc::unbounded_channel();
        let handle = tokio::spawn(download_bundle_concurrent(cfg, tx));

        // Wait until the fast file is ready, then cancel.
        let mut fast_seen = false;
        while let Some(n) = rx.recv().await {
            if matches!(
                &n,
                DownloadNotification::FileReady { package_name, .. } if package_name == "00-fast.bin"
            ) {
                fast_seen = true;
                break;
            }
        }
        assert!(fast_seen, "fast file should have completed");

        let cancel_at = std::time::Instant::now();
        cancel.cancel();
        // Drain remaining notifications.
        while rx.recv().await.is_some() {}
        let res = handle.await.unwrap();
        assert_eq!(res, Err(CANCEL_ERROR.to_owned()));
        // 250ms target with comfortable headroom for test infra.
        let elapsed = cancel_at.elapsed();
        assert!(
            elapsed < Duration::from_millis(1500),
            "cancel took too long: {elapsed:?}"
        );

        // Already-downloaded file remains on disk
        assert!(tmp.path().join("00-fast.bin").exists());
        // Half-downloaded file should NOT be at the canonical path
        assert!(!tmp.path().join("99-slow.bin").exists());

        // Resume: swap the slow file for a fast one and re-run.
        // Reset wiremock by adding a higher-priority mock would require a
        // server reset. Easier: build a brand-new server with the working
        // file and verify only the missing one is fetched.
        let resume_server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/bundle/"))
            .respond_with(ResponseTemplate::new(200).set_body_string(index_html(&[
                ("00-fast.bin", false),
                ("99-slow.bin", false),
            ])))
            .mount(&resume_server)
            .await;
        let resume_fast_hits = Arc::new(AtomicUsize::new(0));
        let resume_slow_hits = Arc::new(AtomicUsize::new(0));
        let fast_hits_clone = resume_fast_hits.clone();
        let slow_hits_clone = resume_slow_hits.clone();
        Mock::given(method("GET"))
            .and(path("/bundle/00-fast.bin"))
            .respond_with(move |_: &Request| {
                fast_hits_clone.fetch_add(1, Ordering::SeqCst);
                ResponseTemplate::new(200).set_body_bytes(b"done".to_vec())
            })
            .mount(&resume_server)
            .await;
        Mock::given(method("GET"))
            .and(path("/bundle/99-slow.bin"))
            .respond_with(move |_: &Request| {
                slow_hits_clone.fetch_add(1, Ordering::SeqCst);
                ResponseTemplate::new(200).set_body_bytes(b"now-ok".to_vec())
            })
            .mount(&resume_server)
            .await;

        let cfg2 = BundleDownloadConfig {
            concurrency: 2,
            url: format!("{}/bundle/", resume_server.uri()),
            local_dir: tmp.path().to_path_buf(),
            cancel: CancelToken::new(),
            skip_top_level: Vec::new(),
        };
        let (tx2, rx2) = mpsc::unbounded_channel();
        let handle2 = tokio::spawn(download_bundle_concurrent(cfg2, tx2));
        let _ = collect_notifications(rx2).await;
        handle2.await.unwrap().unwrap();

        assert_eq!(
            resume_fast_hits.load(Ordering::SeqCst),
            0,
            "already-downloaded file must not be re-fetched"
        );
        assert_eq!(
            resume_slow_hits.load(Ordering::SeqCst),
            1,
            "missing file should be fetched exactly once"
        );
        assert!(tmp.path().join("99-slow.bin").exists());
    }

    fn sample_ozi_map_text() -> &'static str {
        "OziExplorer Map Data File Version 2.2\nDemo topo\ndemo.ozf2\n1 ,Map Code,\nWGS 84\nReserved 1\nReserved 2\nMagnetic Variation,,,E\nMap Projection,Latitude/Longitude,PolyCal,No,AutoCalOnly,No,BSBUseWPX,No\nPoint01,xy,10,20,in, deg,54,30.000,N,48,24.000,E, grid, , , ,N\nProjection Setup,,,,,,,,,,\n"
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
    async fn open_project_async_fetches_missing_files_for_partially_cached_bundle() {
        use super::open_project_async;
        use crate::application::LizaProjectSummary;

        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/bundle/"))
            .respond_with(ResponseTemplate::new(200).set_body_string(index_html(&[
                ("2-Coordinates.txt", false),
                ("demo.map", false),
            ])))
            .mount(&server)
            .await;

        let coordinates_hits = Arc::new(AtomicUsize::new(0));
        let map_hits = Arc::new(AtomicUsize::new(0));
        let coordinates_hits_clone = coordinates_hits.clone();
        let map_hits_clone = map_hits.clone();
        Mock::given(method("GET"))
            .and(path("/bundle/2-Coordinates.txt"))
            .respond_with(move |_: &Request| {
                coordinates_hits_clone.fetch_add(1, Ordering::SeqCst);
                ResponseTemplate::new(200).set_body_bytes(b"N 54.32821 E 048.40917".to_vec())
            })
            .mount(&server)
            .await;
        Mock::given(method("GET"))
            .and(path("/bundle/demo.map"))
            .respond_with(move |_: &Request| {
                map_hits_clone.fetch_add(1, Ordering::SeqCst);
                ResponseTemplate::new(200).set_body_bytes(sample_ozi_map_text().as_bytes().to_vec())
            })
            .mount(&server)
            .await;

        // Interrupted earlier download: the coordinates marker landed on disk
        // but the map file did not.
        let tmp = tempdir();
        let bundle_dir = tmp.path().join("2026-03-29_demo");
        std::fs::create_dir_all(&bundle_dir).expect("create bundle dir");
        std::fs::write(
            bundle_dir.join("2-Coordinates.txt"),
            "N 54.32821 E 048.40917",
        )
        .expect("write coordinates");

        let summary = LizaProjectSummary {
            slug: "2026-03-29_demo".to_owned(),
            name: "2026-03-29 demo".to_owned(),
            url: format!("{}/bundle/", server.uri()),
        };
        let (tx, rx) = mpsc::unbounded_channel();
        let open = tokio::spawn(open_project_async(
            summary,
            tmp.path().to_path_buf(),
            CancelToken::new(),
            DEFAULT_BUNDLE_DOWNLOAD_CONCURRENCY,
            Vec::new(),
            tx,
        ));
        let _ = collect_notifications(rx).await;
        let project = open.await.expect("join").expect("open project");

        assert!(
            bundle_dir.join("demo.map").exists(),
            "missing bundle file must be downloaded on open while the remote listing is reachable"
        );
        assert_eq!(
            map_hits.load(Ordering::SeqCst),
            1,
            "missing file should be fetched exactly once"
        );
        assert_eq!(
            coordinates_hits.load(Ordering::SeqCst),
            0,
            "already-cached file must not be re-fetched"
        );
        assert!(
            project.maps.iter().any(|m| m.name == "OZI: Demo topo"),
            "recovered map must be indexed; got {:?}",
            project.maps
        );
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
    async fn open_project_async_opens_cached_bundle_when_remote_unreachable() {
        use super::open_project_async;
        use crate::application::LizaProjectSummary;

        let tmp = tempdir();
        let bundle_dir = tmp.path().join("2026-03-29_demo");
        std::fs::create_dir_all(&bundle_dir).expect("create bundle dir");
        std::fs::write(
            bundle_dir.join("2-Coordinates.txt"),
            "N 54.32821 E 048.40917",
        )
        .expect("write coordinates");
        std::fs::write(bundle_dir.join("demo.map"), sample_ozi_map_text()).expect("write map");

        let summary = LizaProjectSummary {
            slug: "2026-03-29_demo".to_owned(),
            name: "2026-03-29 demo".to_owned(),
            // Nothing listens on port 9 (discard); the connection is refused.
            url: "http://127.0.0.1:9/bundle/".to_owned(),
        };
        let (tx, rx) = mpsc::unbounded_channel();
        let open = tokio::spawn(open_project_async(
            summary,
            tmp.path().to_path_buf(),
            CancelToken::new(),
            DEFAULT_BUNDLE_DOWNLOAD_CONCURRENCY,
            Vec::new(),
            tx,
        ));
        let _ = collect_notifications(rx).await;
        let project = open
            .await
            .expect("join")
            .expect("cached bundle must open offline");

        assert!(
            project.maps.iter().any(|m| m.name == "OZI: Demo topo"),
            "cached map must be indexed offline; got {:?}",
            project.maps
        );
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
    async fn concurrency_cap_is_respected() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/bundle/"))
            .respond_with(ResponseTemplate::new(200).set_body_string(index_html(&[
                ("00-a.bin", false),
                ("10-b.bin", false),
                ("20-c.bin", false),
                ("30-d.bin", false),
                ("40-e.bin", false),
                ("50-f.bin", false),
            ])))
            .mount(&server)
            .await;

        let in_flight = Arc::new(AtomicUsize::new(0));
        let peak = Arc::new(AtomicUsize::new(0));
        for name in [
            "00-a.bin", "10-b.bin", "20-c.bin", "30-d.bin", "40-e.bin", "50-f.bin",
        ] {
            let in_flight = in_flight.clone();
            let peak = peak.clone();
            Mock::given(method("GET"))
                .and(path(format!("/bundle/{name}")))
                .respond_with(move |_: &Request| {
                    let now = in_flight.fetch_add(1, Ordering::SeqCst) + 1;
                    let mut best = peak.load(Ordering::SeqCst);
                    while now > best
                        && let Err(actual) =
                            peak.compare_exchange(best, now, Ordering::SeqCst, Ordering::SeqCst)
                    {
                        best = actual;
                    }
                    std::thread::sleep(Duration::from_millis(100));
                    in_flight.fetch_sub(1, Ordering::SeqCst);
                    ResponseTemplate::new(200).set_body_bytes(vec![0u8; 128])
                })
                .mount(&server)
                .await;
        }

        let tmp = tempdir();
        let cfg = BundleDownloadConfig {
            concurrency: 3,
            url: format!("{}/bundle/", server.uri()),
            local_dir: tmp.path().to_path_buf(),
            cancel: CancelToken::new(),
            skip_top_level: Vec::new(),
        };
        let (tx, rx) = mpsc::unbounded_channel();
        let handle = tokio::spawn(download_bundle_concurrent(cfg, tx));
        let _ = collect_notifications(rx).await;
        handle.await.unwrap().unwrap();

        let observed = peak.load(Ordering::SeqCst);
        assert!(
            observed <= 3,
            "peak in-flight {observed} exceeded concurrency cap of 3"
        );
        // Sanity: at least some parallelism actually happened.
        assert!(observed >= 1);
    }

    fn tempdir() -> tempfile::TempDir {
        tempfile::tempdir().expect("tempdir")
    }
}
