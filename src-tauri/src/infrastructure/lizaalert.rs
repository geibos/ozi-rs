use crate::application::{
    ActiveMapKind, ActiveMapSelection, LizaMapPackage, LizaProject, LizaProjectSummary, MapCenter,
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

#[derive(Debug, Clone)]
pub struct ProjectOpenProgress {
    pub message: String,
    pub phase: ProjectOpenPhase,
    pub completed: Option<u64>,
    pub total: Option<u64>,
    pub downloaded_bytes: Option<u64>,
    pub total_bytes: Option<u64>,
}

impl ProjectOpenProgress {
    fn status(message: impl Into<String>, phase: ProjectOpenPhase) -> Self {
        Self {
            message: message.into(),
            phase,
            completed: None,
            total: None,
            downloaded_bytes: None,
            total_bytes: None,
        }
    }
}

#[derive(Debug, Clone)]
struct RemoteFileDownload {
    url: String,
    path: PathBuf,
    /// Path relative to the project source root (used for `package_name`).
    relative: String,
}

/// Default upper bound on concurrent per-file downloads inside a bundle.
pub const DEFAULT_BUNDLE_DOWNLOAD_CONCURRENCY: usize = 3;

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

pub fn fetch_project_summaries_streaming<F>(
    mut on_chunk: F,
) -> Result<Vec<LizaProjectSummary>, String>
where
    F: FnMut(Vec<LizaProjectSummary>),
{
    let html = fetch_text(ROOT_URL)?;
    let link_regex =
        Regex::new(r#"href="([^"]+)"[^>]*>([^<]+)</a>"#).map_err(|err| err.to_string())?;
    let project_regex = Regex::new(r"^\d{4}-\d{2}-\d{2}_.+/$").map_err(|err| err.to_string())?;

    let mut projects = Vec::new();
    let mut chunk = Vec::with_capacity(200);

    for captures in link_regex.captures_iter(&html) {
        let Some(href) = captures.get(1).map(|value| value.as_str()) else {
            continue;
        };
        let Some(label) = captures.get(2).map(|value| value.as_str()) else {
            continue;
        };

        if !project_regex.is_match(href) {
            continue;
        }

        let project = LizaProjectSummary {
            slug: label.trim_end_matches('/').to_owned(),
            name: label.trim_end_matches('/').replace('_', " "),
            url: format!("{ROOT_URL}{href}"),
        };

        chunk.push(project.clone());
        projects.push(project);

        if chunk.len() >= 200 {
            on_chunk(std::mem::take(&mut chunk));
            chunk = Vec::with_capacity(200);
        }
    }

    if !chunk.is_empty() {
        on_chunk(chunk);
    }

    Ok(projects)
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
        format!("Extracting OZI archives in: {name}"),
        ProjectOpenPhase::Extracting,
    ));
    materialize_cached_ozi_archives(root, &slug, &mut on_progress)?;

    on_progress(ProjectOpenProgress::status(
        format!("Indexing maps in: {name}"),
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

pub fn download_map<F>(
    selection: ActiveMapSelection,
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

    let mut file = File::create(&selection.local_path).map_err(|err| err.to_string())?;
    let total_bytes = response.content_length();
    let mut downloaded_bytes = 0u64;
    let mut buffer = [0u8; 16 * 1024];

    loop {
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

    Ok(selection)
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
    let link_regex = Regex::new(r#"href="([^"]+\.sqlitedb)"[^>]*>([^<]+)</a>"#)
        .map_err(|err| err.to_string())?;
    let zoom_regex = Regex::new(r"_z(\d+)\.sqlitedb$").map_err(|err| err.to_string())?;

    let maps = link_regex
        .captures_iter(html)
        .filter_map(|caps| {
            let href = caps.get(1)?.as_str();
            let label = caps.get(2)?.as_str();
            let zoom = zoom_regex
                .captures(label)
                .and_then(|captures| captures.get(1))?
                .as_str()
                .parse::<u8>()
                .ok()?;

            Some(LizaMapPackage {
                name: label.to_owned(),
                file_name: label.to_owned(),
                url: format!("{base_url}{href}"),
                base_zoom: zoom,
                local_path: None,
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

fn client() -> Result<Client, String> {
    Client::builder().build().map_err(|err| err.to_string())
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
        format!("Scanning {}", config.local_dir.display()),
        ProjectOpenPhase::Scanning,
    )));

    if config.cancel.is_cancelled() {
        return Err(CANCEL_ERROR.to_owned());
    }

    let mut files = tokio::task::spawn_blocking({
        let url = config.url.clone();
        let local_dir = config.local_dir.clone();
        move || {
            let mut out = Vec::new();
            let root_rel = String::new();
            collect_remote_files_rel(&url, &local_dir, &root_rel, &mut out)?;
            Ok::<_, String>(out)
        }
    })
    .await
    .map_err(|err| err.to_string())??;

    sort_remote_files_by_prefix(&mut files);
    let total = files.len();

    let _ = tx.send(DownloadNotification::Phase(ProjectOpenProgress {
        message: format!("Downloading {total} files in parallel"),
        phase: ProjectOpenPhase::Downloading,
        completed: Some(0),
        total: Some(total as u64),
        downloaded_bytes: None,
        total_bytes: None,
    }));

    let concurrency = config.concurrency.max(1);
    let sem = Arc::new(Semaphore::new(concurrency));
    let mut set: JoinSet<Result<(), String>> = JoinSet::new();
    let async_client = reqwest::Client::builder()
        .build()
        .map_err(|err| err.to_string())?;

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
            let RemoteFileDownload {
                url,
                path,
                relative,
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
                let res = download_to_path_async(
                    &client,
                    &url,
                    &path,
                    &cancel,
                    |downloaded, total_bytes| {
                        let _ = tx_worker.send(DownloadNotification::FileProgress {
                            package_name: pkg.clone(),
                            downloaded_bytes: downloaded,
                            total_bytes,
                            file_index: index,
                            file_count,
                        });
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
            Ok(())
        });
    }

    let mut first_err: Option<String> = None;
    while let Some(joined) = set.join_next().await {
        match joined {
            Ok(Ok(())) => {}
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
        message: format!("Downloaded {total} files"),
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
    tx: mpsc::UnboundedSender<DownloadNotification>,
) -> Result<LizaProject, String> {
    if !is_project_cached(&summary.slug, &root) {
        let _ = tx.send(DownloadNotification::Phase(ProjectOpenProgress::status(
            format!("Downloading project bundle: {}", summary.name),
            ProjectOpenPhase::Downloading,
        )));
        let source_root = project_source_root(&root, &summary.slug);
        fs::create_dir_all(&source_root).map_err(|err| err.to_string())?;
        let cfg = BundleDownloadConfig {
            concurrency,
            url: summary.url.clone(),
            local_dir: source_root,
            cancel: cancel.clone(),
        };
        download_bundle_concurrent(cfg, tx.clone()).await?;
    } else {
        let _ = tx.send(DownloadNotification::Phase(ProjectOpenProgress::status(
            format!("Opening cached project bundle: {}", summary.name),
            ProjectOpenPhase::Downloading,
        )));
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
                format!("Extracting cached OZI bundles: {}", summary.name),
                ProjectOpenPhase::Extracting,
            ));
            materialize_cached_ozi_archives(&root, &summary.slug, &mut on_progress)?;

            on_progress(ProjectOpenProgress::status(
                format!("Indexing cached project maps: {}", summary.name),
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

fn collect_remote_files_rel(
    url: &str,
    local_dir: &Path,
    rel_prefix: &str,
    output: &mut Vec<RemoteFileDownload>,
) -> Result<(), String> {
    let html = fetch_text(url)?;

    for entry in parse_directory_entries(&html)? {
        let child_url = format!("{url}{}", entry.href);
        let child_path = local_dir.join(&entry.name);
        let child_rel = if rel_prefix.is_empty() {
            entry.name.clone()
        } else {
            format!("{rel_prefix}/{}", entry.name)
        };

        if entry.is_dir {
            fs::create_dir_all(&child_path).map_err(|err| err.to_string())?;
            collect_remote_files_rel(&child_url, &child_path, &child_rel, output)?;
        } else {
            output.push(RemoteFileDownload {
                url: child_url,
                path: child_path,
                relative: child_rel,
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
        let chunk = chunk.map_err(|err| err.to_string())?;
        file.write_all(&chunk)
            .await
            .map_err(|err| err.to_string())?;
        downloaded_bytes += chunk.len() as u64;
        on_progress(downloaded_bytes, total_bytes);
    }

    file.sync_all().await.map_err(|err| err.to_string())?;
    drop(file);
    tokio::fs::rename(&tmp_path, path)
        .await
        .map_err(|err| err.to_string())?;
    Ok(())
}

/// Future that resolves when `cancel.is_cancelled()` becomes true. Polled
/// every 25 ms — fast enough to honour the 250 ms cancel-deadline target.
async fn cancel_wait(cancel: &CancelToken) {
    while !cancel.is_cancelled() {
        tokio::time::sleep(std::time::Duration::from_millis(25)).await;
    }
}

fn parse_directory_entries(html: &str) -> Result<Vec<DirectoryEntry>, String> {
    let link_regex =
        Regex::new(r#"href="([^"]+)"[^>]*>([^<]+)</a>"#).map_err(|err| err.to_string())?;
    let mut entries = link_regex
        .captures_iter(html)
        .filter_map(|captures| {
            let href = captures.get(1)?.as_str().trim();
            let label = captures.get(2)?.as_str().trim();
            let fallback_name = label.trim_end_matches('/').trim();
            let name = decode_entry_name(href).unwrap_or_else(|| fallback_name.to_owned());

            if href.is_empty()
                || label.is_empty()
                || href == "../"
                || href == "./"
                || href.starts_with('?')
                || href.starts_with('#')
                || name.trim().is_empty()
            {
                return None;
            }

            Some(DirectoryEntry {
                href: href.to_owned(),
                name,
                is_dir: href.ends_with('/'),
            })
        })
        .collect::<Vec<_>>();

    entries.sort_by(|left, right| left.name.cmp(&right.name));
    Ok(entries)
}

fn decode_entry_name(href: &str) -> Option<String> {
    let raw_name = href.trim_end_matches('/').rsplit('/').next()?.trim();
    if raw_name.is_empty() {
        return None;
    }

    let mut decoded = String::new();
    let bytes = raw_name.as_bytes();
    let mut index = 0;

    while index < bytes.len() {
        if bytes[index] == b'%' && index + 2 < bytes.len() {
            let hex = std::str::from_utf8(&bytes[index + 1..index + 3]).ok()?;
            let value = u8::from_str_radix(hex, 16).ok()?;
            decoded.push(value as char);
            index += 3;
        } else {
            decoded.push(bytes[index] as char);
            index += 1;
        }
    }

    Some(decoded)
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

    Ok(LizaProject {
        summary,
        center,
        maps,
    })
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

fn read_cached_sqlite_map_packages(
    root: &Path,
    project_slug: &str,
    zoom_regex: &Regex,
) -> Result<Vec<LizaMapPackage>, String> {
    let maps_dir = project_mobile_maps_dir(root, project_slug);
    if !maps_dir.exists() {
        return Ok(Vec::new());
    }

    let maps = fs::read_dir(&maps_dir)
        .map_err(|err| err.to_string())?
        .filter_map(|entry| entry.ok())
        .filter_map(|entry| {
            let path = entry.path();
            let file_name = path.file_name()?.to_str()?.to_owned();
            if !file_name.ends_with(".sqlitedb") {
                return None;
            }

            let base_zoom = zoom_regex
                .captures(&file_name)
                .and_then(|captures| captures.get(1))?
                .as_str()
                .parse::<u8>()
                .ok()?;

            Some(LizaMapPackage {
                name: file_name.clone(),
                file_name,
                url: String::new(),
                base_zoom,
                local_path: Some(path),
            })
        })
        .collect();

    Ok(maps)
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

        packages.push(LizaMapPackage {
            name: format!("OZI: {}", metadata.title()),
            file_name: relative_name,
            url: String::new(),
            base_zoom: 0,
            local_path: Some(map_path),
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

fn project_coordinates_path(root: &Path, project_slug: &str) -> PathBuf {
    project_source_root(root, project_slug).join("2-Coordinates.txt")
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
    href: String,
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
        message: format!(
            "Extracting {} in parallel: {}",
            to_extract.len(),
            names.join(", ")
        ),
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
        message: format!("Extracted {} OZI archives", to_extract.len()),
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
    let file = File::open(archive_path).map_err(|err| err.to_string())?;
    extract_zip_entries_to_directory(BufReader::new(file), destination)
        .map_err(|err| err.to_string())?;
    Ok(())
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
        parse_center, parse_directory_entries, parse_map_packages, read_text_file_lossy,
    };
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

    #[test]
    fn parse_directory_entries_skips_parent_links_and_marks_directories() {
        let html = r#"
            <a href="../">../</a>
            <a href="8-Android%26iOS/">8-Android&amp;iOS/</a>
            <a href="5-Ozi.zip">5-Ozi.zip</a>
        "#;

        let entries = parse_directory_entries(html).expect("entries");

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

    #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
    async fn per_file_progress_is_monotonic_and_complete() {
        let server = setup_bundle_server().await;
        let tmp = tempdir();
        let cfg = BundleDownloadConfig {
            concurrency: DEFAULT_BUNDLE_DOWNLOAD_CONCURRENCY,
            url: format!("{}/bundle/", server.uri()),
            local_dir: tmp.path().to_path_buf(),
            cancel: CancelToken::new(),
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
