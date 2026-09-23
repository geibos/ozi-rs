pub mod tiles;

use crate::application::{
    ActiveMapKind, AppState, DiagnosticLevel, LizaProjectSummary, OpenMapRequest,
};
use crate::infrastructure::lizaalert::{
    self, CancelToken, DEFAULT_BUNDLE_DOWNLOAD_CONCURRENCY, DownloadNotification,
};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::thread;
use tauri::{AppHandle, Emitter, State};

pub type SharedState = Arc<Mutex<AppState>>;

/// Active downloads registry: `download_id` → cancel token + abort handle.
#[derive(Default)]
pub struct DownloadRegistry {
    inner: Mutex<HashMap<String, CancelToken>>,
}

impl DownloadRegistry {
    pub fn register(&self, id: String, token: CancelToken) {
        if let Ok(mut map) = self.inner.lock() {
            map.insert(id, token);
        }
    }

    pub fn remove(&self, id: &str) -> Option<CancelToken> {
        self.inner.lock().ok().and_then(|mut map| map.remove(id))
    }

    pub fn cancel(&self, id: &str) -> bool {
        let Ok(map) = self.inner.lock() else {
            return false;
        };
        if let Some(token) = map.get(id) {
            token.cancel();
            true
        } else {
            false
        }
    }
}

pub type SharedDownloads = Arc<DownloadRegistry>;

fn lock_app_state<'a>(
    state: &'a SharedState,
) -> Result<std::sync::MutexGuard<'a, AppState>, String> {
    state
        .lock()
        .map_err(|e| format!("State lock poisoned: {}", e))
}

// ── Serializable DTOs ────────────────────────────────────────────────────────

#[derive(serde::Serialize, specta::Type, Clone)]
pub struct DiagnosticDto {
    pub level: &'static str,
    pub message: String,
}

#[derive(serde::Serialize, specta::Type)]
pub struct AppStateDto {
    pub project_name: String,
    pub project_saved: bool,
    /// True when the project has edits not yet persisted to disk (drives the
    /// dirty indicator and the close guard, CJ-7).
    pub project_dirty: bool,
    /// Current .ozp path when the project has been saved/loaded; lets the
    /// frontend quick-save (Cmd+S) without a dialog.
    pub project_path: Option<String>,
    pub status: String,
    /// The catalogue walk is running. Blocks another refresh; blocks nothing
    /// else — see `LizaAlertState::listing_busy`.
    pub listing_busy: bool,
    /// A bundle is being downloaded or opened from disk. Blocks another one.
    pub bundle_busy: bool,
    pub downloading_maps: Vec<String>,
    // The catalogue is deliberately absent. It is thirteen thousand rows, and
    // this DTO is fetched on every `state-changed` — once per file during a
    // bundle download. The frontend gets the catalogue from its own cache and
    // from the `projects-chunk` stream, which is where a list that size
    // belongs.
    pub current_project: Option<LizaProjectDto>,
    pub active_map: Option<ActiveMapDto>,
    pub diagnostics: Vec<DiagnosticDto>,
    pub track_layers: Vec<LayerSummaryDto>,
    pub waypoint_layers: Vec<LayerSummaryDto>,
    pub track_layer_count: usize,
    pub waypoint_layer_count: usize,
    pub tracks: Vec<TrackSummaryDto>,
}

#[derive(serde::Serialize, specta::Type, Clone)]
pub struct LayerSummaryDto {
    pub id: u64,
    pub name: String,
}

/// Per-track summary surfaced to the UI for the Tracks panel rows.
///
/// Includes derived statistics (distance, duration, point count) computed
/// from the domain `Track` so the frontend can render them without re-walking
/// the segment data.
#[derive(serde::Serialize, specta::Type, Clone)]
pub struct TrackSummaryDto {
    pub layer_id: u64,
    pub track_id: u64,
    pub name: String,
    pub color: String,
    pub line_width: f32,
    pub visible: bool,
    pub distance_km: f64,
    pub duration_seconds: Option<u64>,
    pub point_count: u32,
}

#[derive(serde::Serialize, specta::Type, Clone)]
pub struct LizaProjectSummaryDto {
    pub slug: String,
    pub name: String,
    /// Whether this bundle is already on disk and openable offline.
    ///
    /// Without it the catalogue is thirteen thousand identical rows and a
    /// crew with no signal cannot tell which of them they can still open.
    pub cached: bool,
}

#[derive(serde::Serialize, specta::Type)]
pub struct LizaProjectDto {
    /// The bundle's identifier, as `load_project` takes it.
    ///
    /// Without it a loader opened on an already-previewed project had a name
    /// and no way to ask for that project, so its download button did
    /// nothing at all.
    pub slug: String,
    pub name: String,
    pub center_lat: f64,
    pub center_lon: f64,
    pub maps: Vec<LizaMapPackageDto>,
    /// The bundle's top level — what the operator chooses from when deciding
    /// what not to download.
    pub contents: Vec<BundleEntryDto>,
}

#[derive(serde::Serialize, specta::Type)]
pub struct BundleEntryDto {
    pub name: String,
    pub is_dir: bool,
    pub size_bytes: Option<u64>,
}

#[derive(serde::Serialize, specta::Type)]
pub struct LizaMapPackageDto {
    pub name: String,
    pub base_zoom: u8,
    pub downloaded: bool,
    /// Size in bytes when known — from the listing for a remote map, from the
    /// file for a cached one. `None` is "unknown", not zero.
    pub size_bytes: Option<u64>,
}

#[derive(serde::Serialize, specta::Type)]
pub struct ActiveMapDto {
    pub kind: &'static str,
    pub project_name: String,
    pub package_name: String,
    pub local_path: String,
    pub center_lat: f64,
    pub center_lon: f64,
    pub base_zoom: u8,
}

fn format_track_color(style: &crate::domain::TrackStyle) -> String {
    let [r, g, b, a] = style.color;
    format!(
        "rgba({r},{g},{b},{:.3})",
        a as f64 / 255.0 * style.opacity as f64
    )
}

fn to_track_summary_dto(layer_id: u64, track: &crate::domain::Track) -> TrackSummaryDto {
    let style = track.style();
    let duration_seconds = track.total_duration().and_then(|d| {
        let secs = d.num_seconds();
        if secs < 0 { None } else { Some(secs as u64) }
    });
    TrackSummaryDto {
        layer_id,
        track_id: track.id().value(),
        name: track.name().to_owned(),
        color: format_track_color(style),
        line_width: style.line_width,
        visible: style.visible,
        distance_km: track.total_distance_km(),
        duration_seconds,
        point_count: track.point_count() as u32,
    }
}

pub fn to_project_summary_dtos(
    projects: &[LizaProjectSummary],
    cached: &std::collections::HashSet<String>,
) -> Vec<LizaProjectSummaryDto> {
    projects
        .iter()
        .map(|project| LizaProjectSummaryDto {
            slug: project.slug.clone(),
            name: project.name.clone(),
            cached: cached.contains(&project.slug),
        })
        .collect()
}

// Events
/// Emitted once, by whichever path owns a download, when it stops running.
///
/// Both the bundle download and the single-map download used to end with
/// nothing but a `state-changed`, so the frontend could not tell whose
/// progress it was still showing. The panel is tied to this id.
#[derive(serde::Serialize, specta::Type, Clone)]
pub struct DownloadFinishedPayload {
    pub download_id: String,
    pub ok: bool,
    pub message: Option<String>,
}

#[derive(serde::Serialize, specta::Type, Clone)]
struct DownloadProgressPayload {
    download_id: String,
    package_name: String,
    downloaded_bytes: u64,
    total_bytes: Option<u64>,
    /// Position of this file in the prefix-sorted bundle (zero-based).
    file_index: Option<usize>,
    /// Total number of files in the bundle download.
    file_count: Option<usize>,
}

#[derive(serde::Serialize, specta::Type, Clone)]
struct BundleProgressPayload {
    download_id: String,
    /// The English wording. The interface shows it only when it has no
    /// translation for `message_key` — it is the fallback, not the text.
    message: String,
    /// Translation key and its arguments in order, so the status bar can speak
    /// the interface's language instead of the backend's.
    message_key: &'static str,
    message_args: Vec<String>,
    phase: &'static str,
    completed: Option<u64>,
    total: Option<u64>,
    downloaded_bytes: Option<u64>,
    total_bytes: Option<u64>,
}

/// Emitted when a catalogue walk ends.
///
/// `complete` is what lets the interface prune: only a walk that ran to the
/// end knows what no longer exists. A stopped one read a prefix.
#[derive(serde::Serialize, specta::Type, Clone)]
struct CatalogueRefreshFinishedPayload {
    complete: bool,
}

#[derive(serde::Serialize, specta::Type, Clone)]
struct BundleFileReadyPayload {
    download_id: String,
    package_name: String,
    local_path: String,
    file_index: usize,
    file_count: usize,
}

// ── State snapshot ────────────────────────────────────────────────────────────

#[tauri::command]
#[specta::specta]
pub fn get_app_state(state: State<SharedState>) -> Result<AppStateDto, String> {
    let s = lock_app_state(state.inner())?;
    // No directory walk here any more: the only thing that needed the set of
    // downloaded bundles was the catalogue, and the catalogue has left. This
    // command runs on every `state-changed`, once per file during a download.
    Ok(app_state_dto(&s))
}

/// Build the state snapshot the frontend renders.
///
/// Split out of the command so it can be driven without a Tauri runtime: the
/// test fixtures the frontend renders against come from this exact function,
/// which is the only way a mock cannot drift from what the app really sends.
pub fn app_state_dto(s: &crate::application::AppState) -> AppStateDto {
    let current_project = s.current_project().map(|p| LizaProjectDto {
        slug: p.summary.slug.clone(),
        name: p.summary.name.clone(),
        center_lat: p.center.lat,
        center_lon: p.center.lon,
        maps: p
            .maps
            .iter()
            .map(|m| LizaMapPackageDto {
                name: m.name.clone(),
                base_zoom: m.base_zoom,
                downloaded: m.local_path.is_some(),
                size_bytes: m.size_bytes,
            })
            .collect(),
        contents: p
            .contents
            .iter()
            .map(|entry| BundleEntryDto {
                name: entry.name.clone(),
                is_dir: entry.is_dir,
                size_bytes: entry.size_bytes,
            })
            .collect(),
    });

    let active_map = s.active_map().map(|m| ActiveMapDto {
        kind: match m.kind {
            ActiveMapKind::SqliteTiles => "sqlite",
            ActiveMapKind::OziRaster => "ozi",
        },
        project_name: m.project_name.clone(),
        package_name: m.package_name.clone(),
        local_path: m.local_path.display().to_string(),
        center_lat: m.center.lat,
        center_lon: m.center.lon,
        base_zoom: m.base_zoom,
    });

    let diagnostics = s
        .recent_diagnostics()
        .map(|d| DiagnosticDto {
            level: match d.level() {
                DiagnosticLevel::Info => "info",
                DiagnosticLevel::Warning => "warning",
                DiagnosticLevel::Error => "error",
            },
            message: d.message().to_owned(),
        })
        .collect();

    let track_layers = s
        .track_layers()
        .iter()
        .map(|layer| LayerSummaryDto {
            id: layer.id().value(),
            name: layer.name().to_owned(),
        })
        .collect();

    let tracks: Vec<TrackSummaryDto> = s
        .track_layers()
        .iter()
        .flat_map(|layer| {
            let layer_id = layer.id().value();
            layer
                .tracks()
                .iter()
                .map(move |track| to_track_summary_dto(layer_id, track))
        })
        .collect();

    let waypoint_layers = s
        .project_waypoint_layers()
        .iter()
        .map(|layer| LayerSummaryDto {
            id: layer.id().value(),
            name: layer.name().to_owned(),
        })
        .collect();

    AppStateDto {
        project_name: s.project_name().to_owned(),
        project_saved: s.project_file_path().is_some(),
        project_dirty: s.project_dirty(),
        project_path: s.project_file_path().map(|p| p.display().to_string()),
        status: s.lizaalert_status().to_owned(),
        listing_busy: s.lizaalert_listing_busy(),
        bundle_busy: s.lizaalert_bundle_busy(),
        downloading_maps: s.downloading_maps().iter().cloned().collect(),
        current_project,
        active_map,
        diagnostics,
        track_layers,
        waypoint_layers,
        track_layer_count: s.track_layer_count(),
        waypoint_layer_count: s.waypoint_layer_count(),
        tracks,
    }
}

// ── Track GeoJSON ─────────────────────────────────────────────────────────────

/// Build the track `FeatureCollection` the map renders.
///
/// Each track is one Feature whose geometry is a `MultiLineString` carrying one
/// part per segment. Segment boundaries are real breaks in the recording — a
/// different day, a lost fix, a split the user made — so flattening every
/// segment into a single `LineString` drew a straight line across the map
/// between the end of one segment and the start of the next, and made `split`
/// and `join` invisible on the map.
///
/// A segment with fewer than two points cannot be drawn as a line and is
/// skipped; a track left with no parts is omitted rather than emitted with an
/// empty geometry.
/// The rows the Tracks tab shows: one per track, with no geometry.
///
/// The tab used to build its rows from `build_tracks_geojson`, which cost it
/// two things. Every coordinate of every track was serialized, sent over IPC
/// and parsed so that a list of names could be drawn — a day's folder of
/// recordings is hundreds of thousands of points, re-sent whenever anything
/// changed. And that GeoJSON deliberately omits a track with nothing drawable,
/// which is right for the map and wrong for a list: a track of one point was
/// in the project, counted towards it and exported with it, but had no row, so
/// it could not be seen, renamed or deleted.
pub fn list_track_summaries(layers: &[crate::domain::TrackLayer]) -> Vec<TrackSummaryDto> {
    layers
        .iter()
        .flat_map(|layer| {
            let layer_id = layer.id().value();
            layer
                .tracks()
                .iter()
                .map(move |track| to_track_summary_dto(layer_id, track))
        })
        .collect()
}

pub fn build_tracks_geojson(layers: &[crate::domain::TrackLayer]) -> serde_json::Value {
    let mut features = Vec::new();

    for layer in layers {
        // LayerId has .value(); TrackId is #[serde(transparent)] so it serializes as u64
        let layer_id_val = layer.id().value();
        for track in layer.tracks() {
            let parts: Vec<serde_json::Value> = track
                .segments()
                .iter()
                .filter(|seg| seg.points().len() >= 2)
                .map(|seg| {
                    serde_json::Value::Array(
                        seg.points()
                            .iter()
                            .map(|pt| serde_json::json!([pt.longitude(), pt.latitude()]))
                            .collect(),
                    )
                })
                .collect();

            if parts.is_empty() {
                continue;
            }

            let summary = to_track_summary_dto(layer_id_val, track);
            features.push(serde_json::json!({
                "type": "Feature",
                "properties": {
                    "layer_id": summary.layer_id,
                    "track_id": summary.track_id,
                    "name": summary.name,
                    "color": summary.color,
                    "line_width": summary.line_width,
                    "visible": summary.visible,
                    "distance_km": summary.distance_km,
                    "duration_seconds": summary.duration_seconds,
                    "point_count": summary.point_count,
                },
                "geometry": {
                    "type": "MultiLineString",
                    "coordinates": parts,
                }
            }));
        }
    }

    serde_json::json!({
        "type": "FeatureCollection",
        "features": features,
    })
}

#[tauri::command]
#[specta::specta]
pub fn get_tracks_geojson(state: State<SharedState>) -> Result<serde_json::Value, String> {
    let s = lock_app_state(state.inner())?;
    Ok(build_tracks_geojson(s.track_layers()))
}

/// Every track in the project as a row: what the Tracks tab shows, typed, and
/// without the geometry it was paying for. See `list_track_summaries`.
#[tauri::command]
#[specta::specta]
pub fn list_tracks(state: State<SharedState>) -> Result<Vec<TrackSummaryDto>, String> {
    let s = lock_app_state(state.inner())?;
    Ok(list_track_summaries(s.track_layers()))
}

// ── LizaAlert project loading ─────────────────────────────────────────────────

#[tauri::command]
#[specta::specta]
pub fn load_projects(state: State<SharedState>, app: AppHandle) -> Result<(), String> {
    let Some((bundles_root, cancel)) = lock_app_state(state.inner())?.begin_load_projects() else {
        return Ok(());
    };
    // One read of the bundles root answers "is it downloaded?" for every one
    // of the catalogue's thousands of rows.
    let cached_slugs = lizaalert::cached_project_slugs(&bundles_root);
    let cached_slugs_for_chunks = cached_slugs.clone();

    if let Ok(cached_projects) = lizaalert::load_project_summaries_cache(&bundles_root)
        && !cached_projects.is_empty()
    {
        if let Ok(mut s) = lock_app_state(state.inner()) {
            s.apply_projects_chunk(cached_projects.clone());
        }
        let _ = app.emit(
            "projects-chunk",
            to_project_summary_dtos(&cached_projects, &cached_slugs),
        );
    }

    // After the cached chunk and before the walk, so the interface collects
    // only what this walk sends and does not count yesterday's cache as proof
    // that a search still exists.
    let _ = app.emit("catalogue-refresh-started", ());

    let state_arc = Arc::clone(&state);
    thread::spawn(move || {
        let chunk_state = Arc::clone(&state_arc);
        let chunk_app = app.clone();
        let result = lizaalert::fetch_project_summaries_streaming(&cancel, move |chunk, _page| {
            let chunk_payload = to_project_summary_dtos(&chunk, &cached_slugs_for_chunks);
            if let Ok(mut s) = lock_app_state(&chunk_state) {
                s.apply_projects_chunk(chunk);
            }
            let _ = chunk_app.emit("projects-chunk", chunk_payload);
        });

        // A stopped walk read only the first pages. Writing those over the
        // cache would leave a crew offline tomorrow with the newest few dozen
        // searches and no sign that the rest ever existed.
        if let Ok(walk) = &result
            && !walk.cancelled
        {
            let _ = lizaalert::save_project_summaries_cache(&bundles_root, &walk.projects);
        }

        let complete = matches!(&result, Ok(walk) if !walk.cancelled);
        if let Ok(mut s) = lock_app_state(&state_arc) {
            s.apply_projects_loaded(result);
        }
        let _ = app.emit(
            "catalogue-refresh-finished",
            CatalogueRefreshFinishedPayload { complete },
        );
        let _ = app.emit("state-changed", ());
    });

    Ok(())
}

/// Preview a bundle: fetch its map list without downloading anything.
/// Selecting a project in the loader calls this; the actual download starts
/// only from the explicit open action (`load_project`).
#[tauri::command]
#[specta::specta]
pub fn preview_project(
    slug: String,
    state: State<SharedState>,
    app: AppHandle,
) -> Result<(), String> {
    let data = {
        let mut s = lock_app_state(state.inner())?;
        s.preview_data(&slug)
    };
    let Some((summary, bundles_root)) = data else {
        return Ok(());
    };
    let _ = app.emit("state-changed", ());

    let state_arc = Arc::clone(&state);
    let app_handle = app.clone();
    std::thread::spawn(move || {
        let result = crate::infrastructure::lizaalert::preview_project(summary, &bundles_root);
        if let Ok(mut s) = state_arc.lock() {
            // By slug, because nothing orders two previews: the one clicked
            // first can answer last, and it must not land on a row the
            // operator has already left.
            s.apply_preview_loaded(&slug, result);
        }
        let _ = app_handle.emit("state-changed", ());
    });
    Ok(())
}

#[tauri::command]
#[specta::specta]
/// Open a bundle, downloading what is missing.
///
/// `skip` names top-level entries to leave on the server — print sheets and
/// Android tile packs are most of the transfer and this app cannot open them.
/// Nothing is skipped unless the caller names it: the choice is the
/// operator's, not a default this code decides for them.
pub fn load_project(
    slug: String,
    skip: Vec<String>,
    state: State<SharedState>,
    downloads: State<SharedDownloads>,
    app: AppHandle,
) -> Result<String, String> {
    let (summary, bundles_root) = lock_app_state(state.inner())?
        .begin_load_project(&slug)
        .map_err(|refusal| refusal.message().to_owned())?;
    let _ = app.emit("state-changed", ());

    let download_id = uuid::Uuid::new_v4().to_string();
    let cancel = CancelToken::new();
    downloads.register(download_id.clone(), cancel.clone());

    let state_arc = Arc::clone(&state);
    let downloads_arc = Arc::clone(&downloads);
    let download_id_for_task = download_id.clone();

    tauri::async_runtime::spawn(async move {
        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<DownloadNotification>();
        // Event forwarder task — workers funnel notifications through `tx`,
        // this task is the sole writer to the Tauri Emitter so the UI sees a
        // coherent ordered stream.
        let forwarder = {
            let app = app.clone();
            let state_arc = Arc::clone(&state_arc);
            let download_id = download_id_for_task.clone();
            tokio::spawn(async move {
                while let Some(n) = rx.recv().await {
                    match n {
                        DownloadNotification::Phase(p) => {
                            if let Ok(mut s) = lock_app_state(&state_arc) {
                                s.apply_progress(p.message());
                            }
                            let _ = app.emit(
                                "bundle-progress",
                                BundleProgressPayload {
                                    download_id: download_id.clone(),
                                    message: p.message(),
                                    message_key: p.text.key(),
                                    message_args: p.text.args(),
                                    phase: p.phase.as_str(),
                                    completed: p.completed,
                                    total: p.total,
                                    downloaded_bytes: p.downloaded_bytes,
                                    total_bytes: p.total_bytes,
                                },
                            );
                        }
                        DownloadNotification::FileProgress {
                            package_name,
                            downloaded_bytes,
                            total_bytes,
                            file_index,
                            file_count,
                        } => {
                            let _ = app.emit(
                                "download-progress",
                                DownloadProgressPayload {
                                    download_id: download_id.clone(),
                                    package_name,
                                    downloaded_bytes,
                                    total_bytes,
                                    file_index: Some(file_index),
                                    file_count: Some(file_count),
                                },
                            );
                        }
                        DownloadNotification::FileReady {
                            package_name,
                            local_path,
                            file_index,
                            file_count,
                        } => {
                            if let Ok(mut s) = lock_app_state(&state_arc) {
                                s.note_bundle_file_ready(&package_name, &local_path);
                            }
                            let _ = app.emit(
                                "bundle-file-ready",
                                BundleFileReadyPayload {
                                    download_id: download_id.clone(),
                                    package_name,
                                    local_path: local_path.display().to_string(),
                                    file_index,
                                    file_count,
                                },
                            );
                            // Push the freshly-updated AppStateDto so the Maps
                            // column can flip the per-row badge from blue % to
                            // green cached while the rest of the bundle is
                            // still being fetched.
                            let _ = app.emit("state-changed", ());
                        }
                    }
                }
            })
        };

        let result = lizaalert::open_project_async(
            summary,
            bundles_root,
            cancel,
            DEFAULT_BUNDLE_DOWNLOAD_CONCURRENCY,
            skip,
            tx,
        )
        .await;

        // Drop the sender side by waiting for forwarder to drain; sender drops
        // automatically when open_project_async returns.
        let _ = forwarder.await;

        downloads_arc.remove(&download_id_for_task);

        let finished = DownloadFinishedPayload {
            download_id: download_id_for_task.clone(),
            ok: result.is_ok(),
            message: result.as_ref().err().cloned(),
        };
        if let Ok(mut s) = lock_app_state(&state_arc) {
            s.apply_project_loaded(result);
        }
        let _ = app.emit("download-finished", finished);
        let _ = app.emit("state-changed", ());
    });

    Ok(download_id)
}

/// Stop the catalogue refresh that is running.
///
/// The walk is up to a thousand pages and holds the application busy for all
/// of it, which on a field link disables the only download button in the app
/// for minutes after launch. Returns whether there was a refresh to stop.
#[tauri::command]
#[specta::specta]
pub fn cancel_project_listing(state: State<SharedState>) -> Result<bool, String> {
    Ok(lock_app_state(state.inner())?.cancel_project_listing())
}

#[tauri::command]
#[specta::specta]
pub fn cancel_download(
    download_id: String,
    downloads: State<SharedDownloads>,
) -> Result<bool, String> {
    Ok(downloads.cancel(&download_id))
}

#[tauri::command]
#[specta::specta]
/// Open a map, downloading it first when it is not on disk.
///
/// Returns the download id when a download started, so the caller can show
/// progress for it and cancel it; an empty string when the map opened from
/// disk or the request was a no-op.
pub fn open_selected_map(
    map_name: String,
    state: State<SharedState>,
    downloads: State<SharedDownloads>,
    app: AppHandle,
) -> Result<String, String> {
    let request = lock_app_state(state.inner())?.begin_open_map(&map_name);
    let Some(request) = request else {
        return Ok(String::new());
    };

    match request {
        OpenMapRequest::Local(selection) => {
            // OZI raster — open synchronously
            if selection.kind == ActiveMapKind::OziRaster {
                let path = selection.local_path.clone();
                match lock_app_state(state.inner())?.open_local_ozi_map(path) {
                    Ok(()) => {}
                    Err(e) => lock_app_state(state.inner())?.report_runtime_error(e.to_string()),
                }
            } else {
                lock_app_state(state.inner())?.open_local_map_selection(selection);
            }
            let _ = app.emit("state-changed", ());
        }
        OpenMapRequest::Download(selection) => {
            let _ = app.emit("state-changed", ());
            let state_arc = Arc::clone(&state);
            let downloads_arc = Arc::clone(&downloads);
            let package_name = selection.package_name.clone();
            let download_id = uuid::Uuid::new_v4().to_string();
            let cancel = CancelToken::new();
            downloads.register(download_id.clone(), cancel.clone());
            let download_id_for_task = download_id.clone();
            thread::spawn(move || {
                let pkg = package_name.clone();
                let progress_id = download_id_for_task.clone();
                let result = lizaalert::download_map(selection, &cancel, |progress| {
                    let _ = app.emit(
                        "download-progress",
                        DownloadProgressPayload {
                            download_id: progress_id.clone(),
                            package_name: pkg.clone(),
                            downloaded_bytes: progress.downloaded_bytes,
                            total_bytes: progress.total_bytes,
                            file_index: None,
                            file_count: None,
                        },
                    );
                });
                downloads_arc.remove(&download_id_for_task);
                let finished = DownloadFinishedPayload {
                    download_id: download_id_for_task.clone(),
                    ok: result.is_ok(),
                    message: result.as_ref().err().cloned(),
                };
                if let Ok(mut s) = lock_app_state(&state_arc) {
                    s.apply_map_downloaded(&package_name, result);
                }
                let _ = app.emit("download-finished", finished);
                let _ = app.emit("state-changed", ());
            });
            return Ok(download_id);
        }
    }

    Ok(String::new())
}

#[tauri::command]
#[specta::specta]
pub fn open_local_bundle(
    dir: String,
    state: State<SharedState>,
    app: AppHandle,
) -> Result<String, String> {
    let data = lock_app_state(state.inner())?.begin_open_local_bundle(PathBuf::from(&dir));
    let Some(dir_path) = data else {
        return Ok(String::new());
    };
    let _ = app.emit("state-changed", ());

    let download_id = uuid::Uuid::new_v4().to_string();
    let state_arc = Arc::clone(&state);
    let download_id_for_task = download_id.clone();
    thread::spawn(move || {
        let result = lizaalert::open_bundle_directory(&dir_path, |progress| {
            if let Ok(mut s) = lock_app_state(&state_arc) {
                s.apply_progress(progress.message());
            }
            let _ = app.emit(
                "bundle-progress",
                BundleProgressPayload {
                    download_id: download_id_for_task.clone(),
                    message: progress.message(),
                    message_key: progress.text.key(),
                    message_args: progress.text.args(),
                    phase: progress.phase.as_str(),
                    completed: progress.completed,
                    total: progress.total,
                    downloaded_bytes: progress.downloaded_bytes,
                    total_bytes: progress.total_bytes,
                },
            );
        });
        if let Ok(mut s) = lock_app_state(&state_arc) {
            s.apply_project_loaded(result);
        }
        let _ = app.emit("state-changed", ());
    });

    Ok(download_id)
}

// ── Settings ──────────────────────────────────────────────────────────────────

#[tauri::command]
#[specta::specta]
pub fn set_bundles_root(
    path: String,
    state: State<SharedState>,
    app: AppHandle,
) -> Result<(), String> {
    lock_app_state(state.inner())?.set_bundles_root(PathBuf::from(path));
    let _ = app.emit("state-changed", ());
    Ok(())
}

// ── Project persistence ───────────────────────────────────────────────────────

#[tauri::command]
#[specta::specta]
pub fn save_project(path: String, state: State<SharedState>, app: AppHandle) -> Result<(), String> {
    let result = lock_app_state(state.inner())?.save_project_to(PathBuf::from(path));
    // Emit even on failure so the diagnostics panel picks up the error entry.
    let _ = app.emit("state-changed", ());
    result.map_err(|e| e.to_string())
}

#[tauri::command]
#[specta::specta]
pub fn load_project_file(
    path: String,
    state: State<SharedState>,
    app: AppHandle,
) -> Result<(), String> {
    let result = lock_app_state(state.inner())?.load_project_from(PathBuf::from(path));
    // Emit either way: a failed open still produced a diagnostic entry.
    let _ = app.emit("state-changed", ());
    result
}

// ── Import / export ───────────────────────────────────────────────────────────

/// CJ-3: recursively import every GPX/PLT under a folder (per-date
/// subfolders included). Per-file failures are reported, not fatal.
#[tauri::command]
#[specta::specta]
pub fn import_tracks_directory(
    path: String,
    state: State<SharedState>,
    app: AppHandle,
) -> Result<ImportReportDto, String> {
    let report = lock_app_state(state.inner())?.import_tracks_directory(PathBuf::from(path))?;
    let _ = app.emit("state-changed", ());
    Ok(ImportReportDto::from(report))
}

/// What a folder import did, for the interface to put into words.
///
/// This used to be an English sentence built here and toasted verbatim, so a
/// Russian crew read "Imported 12 tracks and 3 waypoints from 4 files" after
/// the most common import there is. Same rule as the bundle progress: the
/// backend sends what happened, the interface says it.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, specta::Type)]
pub struct ImportReportDto {
    pub files: u32,
    pub tracks: u32,
    pub waypoints: u32,
    /// Names of the files that could not be read. Names, not paths: a toast
    /// has no room for a directory tree and the name is what is recognised.
    pub skipped: Vec<String>,
}

impl From<crate::application::import::DirectoryImportReport> for ImportReportDto {
    fn from(report: crate::application::import::DirectoryImportReport) -> Self {
        Self {
            files: report.imported_files as u32,
            tracks: report.imported_tracks as u32,
            waypoints: report.imported_waypoints as u32,
            skipped: report
                .skipped
                .iter()
                .map(|(path, _)| {
                    path.file_name()
                        .map(|name| name.to_string_lossy().into_owned())
                        .unwrap_or_default()
                })
                .collect(),
        }
    }
}

#[tauri::command]
#[specta::specta]
pub fn import_gpx(
    path: String,
    state: State<SharedState>,
    app: AppHandle,
) -> Result<String, String> {
    let result = lock_app_state(state.inner())?
        .import_gpx_file(PathBuf::from(path))
        .map_err(|e| e.to_string())?;
    let _ = app.emit("state-changed", ());
    Ok(format!(
        "Imported {} tracks in {} layers",
        result.imported_tracks(),
        result.imported_track_layers()
    ))
}

#[tauri::command]
#[specta::specta]
pub fn import_plt(
    path: String,
    state: State<SharedState>,
    app: AppHandle,
) -> Result<String, String> {
    let result = lock_app_state(state.inner())?
        .import_plt_file(PathBuf::from(path))
        .map_err(|e| e.to_string())?;
    let _ = app.emit("state-changed", ());
    Ok(format!(
        "Imported {} tracks in {} layers",
        result.imported_tracks(),
        result.imported_track_layers()
    ))
}

/// Import an OziExplorer waypoint file (`.wpt`).
///
/// We have written this format since ADR-0022 and never read it, which is the
/// wrong way round: the штаб next door runs the original and hands over `.wpt`.
#[tauri::command]
#[specta::specta]
pub fn import_wpt(
    path: String,
    state: State<SharedState>,
    app: AppHandle,
) -> Result<String, String> {
    let result = lock_app_state(state.inner())?
        .import_wpt_file(PathBuf::from(path))
        .map_err(|e| e.to_string())?;
    let _ = app.emit("state-changed", ());
    Ok(format!(
        "Imported {} waypoints in {} layers",
        result.imported_waypoints(),
        result.imported_waypoint_layers()
    ))
}

#[tauri::command]
#[specta::specta]
pub fn export_gpx(
    layer_id: u64,
    path: String,
    state: State<SharedState>,
    app: AppHandle,
) -> Result<(), String> {
    use crate::domain::LayerId;
    lock_app_state(state.inner())?
        .export_layer_to_gpx(LayerId::new(layer_id), PathBuf::from(path))?;
    let _ = app.emit("state-changed", ());
    Ok(())
}

/// What a day's export wrote, so the interface can say it.
#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize, specta::Type)]
pub struct DayExportDto {
    pub tracks: u32,
    pub waypoints: u32,
}

/// Export a day's work — every track and every mark — to one GPX file.
#[tauri::command]
#[specta::specta]
pub fn export_all_tracks_gpx(
    path: String,
    state: State<SharedState>,
    app: AppHandle,
) -> Result<DayExportDto, String> {
    let written = lock_app_state(state.inner())?.export_all_tracks_gpx(PathBuf::from(path))?;
    let _ = app.emit("state-changed", ());
    Ok(DayExportDto {
        tracks: written.tracks as u32,
        waypoints: written.waypoints as u32,
    })
}

#[tauri::command]
#[specta::specta]
pub fn get_track_export_default_path(
    track_name: String,
    extension: String,
    state: State<SharedState>,
) -> Result<Option<String>, String> {
    Ok(lock_app_state(state.inner())?
        .export_default_tracks_dir_path(&track_name, &extension)
        .map(|path| path.display().to_string()))
}

#[tauri::command]
#[specta::specta]
pub fn export_wpt_waypoints(
    layer_id: u64,
    path: String,
    state: State<SharedState>,
    app: AppHandle,
) -> Result<(), String> {
    use crate::domain::LayerId;
    lock_app_state(state.inner())?
        .export_wpt_waypoints(LayerId::new(layer_id), PathBuf::from(path))?;
    let _ = app.emit("state-changed", ());
    Ok(())
}

/// Export a waypoint layer to GPX — the format phones, navigators and the
/// other groups' software read.
#[tauri::command]
#[specta::specta]
pub fn export_gpx_waypoints(
    layer_id: u64,
    path: String,
    state: State<SharedState>,
    app: AppHandle,
) -> Result<(), String> {
    use crate::domain::LayerId;
    lock_app_state(state.inner())?
        .export_gpx_waypoints(LayerId::new(layer_id), PathBuf::from(path))?;
    let _ = app.emit("state-changed", ());
    Ok(())
}

/// Suggested file name for a waypoint export in the given format.
#[tauri::command]
#[specta::specta]
pub fn get_waypoints_export_default_path(
    layer_id: u64,
    extension: String,
    state: State<SharedState>,
) -> Result<Option<String>, String> {
    use crate::domain::LayerId;
    Ok(lock_app_state(state.inner())?
        .export_waypoints_default_path(LayerId::new(layer_id), &extension)
        .map(|path| path.display().to_string()))
}

#[tauri::command]
#[specta::specta]
pub fn get_wpt_export_default_path(
    layer_id: u64,
    state: State<SharedState>,
) -> Result<Option<String>, String> {
    use crate::domain::LayerId;
    Ok(lock_app_state(state.inner())?
        .export_wpt_default_path(LayerId::new(layer_id))
        .map(|path| path.display().to_string()))
}

#[tauri::command]
#[specta::specta]
pub fn export_track_plt(
    layer_id: u64,
    track_id: u64,
    path: String,
    state: State<SharedState>,
) -> Result<(), String> {
    use crate::domain::{LayerId, TrackId};
    use crate::infrastructure::export::plt::export_plt;

    let app_state = lock_app_state(state.inner())?;
    let lid = LayerId::new(layer_id);
    let tid = TrackId::new(track_id);

    let layer = app_state
        .track_layers()
        .iter()
        .find(|l| l.id() == lid)
        .ok_or_else(|| format!("track layer {layer_id} not found"))?;

    let track = layer
        .tracks()
        .iter()
        .find(|t| t.id() == tid)
        .ok_or_else(|| format!("track {track_id} not found in layer {layer_id}"))?;

    let style = track.style();
    let [r, g, b, _a] = style.color;
    let color: u32 = (r as u32) << 16 | (g as u32) << 8 | (b as u32);
    let width = style.line_width as f64;

    let mut file = std::fs::File::create(PathBuf::from(&path))
        .map_err(|e| format!("failed to create file {path}: {e}"))?;

    export_plt(track, color, width, &mut file).map_err(|e| format!("{e}"))?;

    Ok(())
}

// ── Undo / redo ───────────────────────────────────────────────────────────────

#[tauri::command]
#[specta::specta]
pub fn undo(state: State<SharedState>, app: AppHandle) -> Result<(), String> {
    lock_app_state(state.inner())?.undo();
    let _ = app.emit("state-changed", ());
    Ok(())
}

#[tauri::command]
#[specta::specta]
pub fn redo(state: State<SharedState>, app: AppHandle) -> Result<(), String> {
    lock_app_state(state.inner())?.redo();
    let _ = app.emit("state-changed", ());
    Ok(())
}

/// Start the next search.
///
/// A project is one search, and nothing created an empty one until now: a crew
/// that finished an operation and began the next kept adding to the same
/// document. Asking before discarding unsaved work is the caller's job — the
/// frontend already owns that question for the close guard.
#[tauri::command]
#[specta::specta]
pub fn new_project(state: State<SharedState>, app: AppHandle) -> Result<(), String> {
    lock_app_state(state.inner())?.new_project();
    let _ = app.emit("state-changed", ());
    Ok(())
}

/// Set the note beside a mark.
///
/// The place travels between headquarters; until 2026-09-23 the reason for it
/// did not, in either direction.
#[tauri::command]
#[specta::specta]
pub fn set_waypoint_description(
    layer_id: u64,
    waypoint_id: u64,
    description: Option<String>,
    state: State<SharedState>,
    app: AppHandle,
) -> Result<(), String> {
    use crate::domain::{LayerId, WaypointId};
    let mut app_state = lock_app_state(state.inner())?;
    app_state
        .apply_set_waypoint_description(
            LayerId::new(layer_id),
            WaypointId::new(waypoint_id),
            description,
        )
        .map_err(|e| format!("{e}"))?;
    let _ = app.emit("state-changed", ());
    Ok(())
}

// ── Layer management ──────────────────────────────────────────────────────────
//
// A day of recordings arrives as files and every file becomes a layer named
// after its path. The domain has had these commands since the first commit;
// until now nothing exposed them, so the operator could toggle a layer's
// tracks and nothing else.

#[tauri::command]
#[specta::specta]
pub fn create_track_layer(
    name: String,
    state: State<SharedState>,
    app: AppHandle,
) -> Result<u64, String> {
    let mut app_state = lock_app_state(state.inner())?;
    let id = app_state.create_track_layer(name)?;
    let _ = app.emit("state-changed", ());
    Ok(id)
}

#[tauri::command]
#[specta::specta]
pub fn create_waypoint_layer(
    name: String,
    state: State<SharedState>,
    app: AppHandle,
) -> Result<u64, String> {
    let mut app_state = lock_app_state(state.inner())?;
    let id = app_state.create_waypoint_layer(name)?;
    let _ = app.emit("state-changed", ());
    Ok(id)
}

#[tauri::command]
#[specta::specta]
pub fn rename_track_layer(
    layer_id: u64,
    new_name: String,
    state: State<SharedState>,
    app: AppHandle,
) -> Result<(), String> {
    use crate::domain::LayerId;
    let mut app_state = lock_app_state(state.inner())?;
    app_state.rename_track_layer(LayerId::new(layer_id), new_name)?;
    let _ = app.emit("state-changed", ());
    Ok(())
}

#[tauri::command]
#[specta::specta]
pub fn rename_waypoint_layer(
    layer_id: u64,
    new_name: String,
    state: State<SharedState>,
    app: AppHandle,
) -> Result<(), String> {
    use crate::domain::LayerId;
    let mut app_state = lock_app_state(state.inner())?;
    app_state.rename_waypoint_layer(LayerId::new(layer_id), new_name)?;
    let _ = app.emit("state-changed", ());
    Ok(())
}

#[tauri::command]
#[specta::specta]
pub fn delete_track_layer(
    layer_id: u64,
    state: State<SharedState>,
    app: AppHandle,
) -> Result<(), String> {
    use crate::domain::LayerId;
    let mut app_state = lock_app_state(state.inner())?;
    app_state.delete_track_layer(LayerId::new(layer_id))?;
    let _ = app.emit("state-changed", ());
    Ok(())
}

#[tauri::command]
#[specta::specta]
pub fn delete_waypoint_layer(
    layer_id: u64,
    state: State<SharedState>,
    app: AppHandle,
) -> Result<(), String> {
    use crate::domain::LayerId;
    let mut app_state = lock_app_state(state.inner())?;
    app_state.delete_waypoint_layer(LayerId::new(layer_id))?;
    let _ = app.emit("state-changed", ());
    Ok(())
}

// ── Track mutations ───────────────────────────────────────────────────────────

#[tauri::command]
#[specta::specta]
pub fn rename_track(
    layer_id: u64,
    track_id: u64,
    new_name: String,
    state: State<SharedState>,
    app: AppHandle,
) -> Result<(), String> {
    use crate::domain::{LayerId, TrackId};
    let mut app_state = lock_app_state(state.inner())?;
    app_state.rename_track(LayerId::new(layer_id), TrackId::new(track_id), new_name);
    let _ = app.emit("state-changed", ());
    Ok(())
}

#[tauri::command]
#[specta::specta]
pub fn set_track_color(
    layer_id: u64,
    track_id: u64,
    color: [u8; 4],
    state: State<SharedState>,
    app: AppHandle,
) -> Result<(), String> {
    use crate::domain::{LayerId, TrackId};
    let mut app_state = lock_app_state(state.inner())?;
    app_state.set_track_color(LayerId::new(layer_id), TrackId::new(track_id), color);
    let _ = app.emit("state-changed", ());
    Ok(())
}

#[tauri::command]
#[specta::specta]
pub fn set_waypoint_symbol(
    layer_id: u64,
    waypoint_id: u64,
    symbol: Option<String>,
    state: State<SharedState>,
    app: AppHandle,
) -> Result<(), String> {
    use crate::domain::{LayerId, WaypointId};
    let mut app_state = lock_app_state(state.inner())?;
    app_state
        .apply_set_waypoint_symbol(LayerId::new(layer_id), WaypointId::new(waypoint_id), symbol)
        .map_err(|e| e.to_string())?;
    let _ = app.emit("state-changed", ());
    Ok(())
}

/// Set or clear a waypoint's colour. `None` restores the default rather than
/// setting a colour that happens to look like it.
#[tauri::command]
#[specta::specta]
pub fn set_waypoint_color(
    layer_id: u64,
    waypoint_id: u64,
    color: Option<[u8; 4]>,
    state: State<SharedState>,
    app: AppHandle,
) -> Result<(), String> {
    use crate::domain::{LayerId, WaypointId};
    let mut app_state = lock_app_state(state.inner())?;
    app_state
        .apply_set_waypoint_color(LayerId::new(layer_id), WaypointId::new(waypoint_id), color)
        .map_err(|e| e.to_string())?;
    let _ = app.emit("state-changed", ());
    Ok(())
}

/// Show or hide every track at once — the triage operator's bulk control.
#[tauri::command]
#[specta::specta]
pub fn set_all_tracks_visible(
    visible: bool,
    state: State<SharedState>,
    app: AppHandle,
) -> Result<(), String> {
    lock_app_state(state.inner())?.set_all_tracks_visible(visible);
    let _ = app.emit("state-changed", ());
    Ok(())
}

/// Leave one track visible and hide every other one.
#[tauri::command]
#[specta::specta]
pub fn show_only_track(
    layer_id: u64,
    track_id: u64,
    state: State<SharedState>,
    app: AppHandle,
) -> Result<(), String> {
    use crate::domain::{LayerId, TrackId};
    lock_app_state(state.inner())?.show_only_track(LayerId::new(layer_id), TrackId::new(track_id));
    let _ = app.emit("state-changed", ());
    Ok(())
}

/// Abandon a drawing in progress: reverse its commands without leaving them
/// in the redo stack.
///
/// The track is named as well as counted. The undo stack is bounded, so a
/// drawing longer than the stack has already lost the command that created
/// the track, and counting alone left an empty one behind.
#[tauri::command]
#[specta::specta]
pub fn cancel_drawing(
    layer_id: u64,
    track_id: u64,
    command_count: u32,
    state: State<SharedState>,
    app: AppHandle,
) -> Result<(), String> {
    use crate::domain::{LayerId, TrackId};
    lock_app_state(state.inner())?.cancel_drawing_of(
        LayerId::new(layer_id),
        TrackId::new(track_id),
        command_count as usize,
    );
    let _ = app.emit("state-changed", ());
    Ok(())
}

#[tauri::command]
#[specta::specta]
pub fn toggle_track_visible(
    layer_id: u64,
    track_id: u64,
    state: State<SharedState>,
    app: AppHandle,
) -> Result<(), String> {
    use crate::domain::{LayerId, TrackId};
    let mut app_state = lock_app_state(state.inner())?;
    app_state.toggle_track_visible(LayerId::new(layer_id), TrackId::new(track_id));
    let _ = app.emit("state-changed", ());
    Ok(())
}

/// Show or hide every waypoint at once — the Waypoints tab's bulk control.
#[tauri::command]
#[specta::specta]
pub fn set_all_waypoints_visible(
    visible: bool,
    state: State<SharedState>,
    app: AppHandle,
) -> Result<(), String> {
    lock_app_state(state.inner())?.set_all_waypoints_visible(visible);
    let _ = app.emit("state-changed", ());
    Ok(())
}

/// Leave one waypoint visible and hide every other one.
#[tauri::command]
#[specta::specta]
pub fn show_only_waypoint(
    layer_id: u64,
    waypoint_id: u64,
    state: State<SharedState>,
    app: AppHandle,
) -> Result<(), String> {
    use crate::domain::{LayerId, WaypointId};
    lock_app_state(state.inner())?
        .show_only_waypoint(LayerId::new(layer_id), WaypointId::new(waypoint_id));
    let _ = app.emit("state-changed", ());
    Ok(())
}

#[tauri::command]
#[specta::specta]
pub fn toggle_waypoint_visible(
    layer_id: u64,
    waypoint_id: u64,
    state: State<SharedState>,
    app: AppHandle,
) -> Result<(), String> {
    use crate::domain::{LayerId, WaypointId};
    let mut app_state = lock_app_state(state.inner())?;
    app_state.toggle_waypoint_visible(LayerId::new(layer_id), WaypointId::new(waypoint_id));
    let _ = app.emit("state-changed", ());
    Ok(())
}

// ── Track point and track mutations ───────────────────────────────────────────

#[tauri::command]
#[specta::specta]
pub fn move_track_point(
    state: State<SharedState>,
    app: AppHandle,
    layer_id: u64,
    track_id: u64,
    segment_id: u64,
    point_id: u64,
    position: [f64; 2],
) -> Result<(), String> {
    use crate::domain::{LayerId, TrackId, TrackPointId, TrackSegmentId};
    let mut app_state = lock_app_state(state.inner())?;
    app_state
        .apply_move_track_point(
            LayerId::new(layer_id),
            TrackId::new(track_id),
            TrackSegmentId::new(segment_id),
            TrackPointId::new(point_id),
            position[0],
            position[1],
        )
        .map_err(|e| format!("{e}"))?;
    let _ = app.emit("state-changed", ());
    Ok(())
}

#[tauri::command]
#[specta::specta]
pub fn delete_track_point(
    state: State<SharedState>,
    app: AppHandle,
    layer_id: u64,
    track_id: u64,
    segment_id: u64,
    point_id: u64,
) -> Result<(), String> {
    use crate::domain::{LayerId, TrackId, TrackPointId, TrackSegmentId};
    let mut app_state = lock_app_state(state.inner())?;
    app_state
        .apply_delete_track_point(
            LayerId::new(layer_id),
            TrackId::new(track_id),
            TrackSegmentId::new(segment_id),
            TrackPointId::new(point_id),
        )
        .map_err(|e| format!("{e}"))?;
    let _ = app.emit("state-changed", ());
    Ok(())
}

#[tauri::command]
#[specta::specta]
pub fn insert_track_point(
    state: State<SharedState>,
    app: AppHandle,
    layer_id: u64,
    track_id: u64,
    segment_id: u64,
    index: usize,
    position: [f64; 2],
) -> Result<(), String> {
    use crate::domain::{LayerId, TrackId, TrackSegmentId};
    let mut app_state = lock_app_state(state.inner())?;
    app_state
        .apply_insert_track_point(
            LayerId::new(layer_id),
            TrackId::new(track_id),
            TrackSegmentId::new(segment_id),
            index,
            position[0],
            position[1],
        )
        .map_err(|e| format!("{e}"))?;
    let _ = app.emit("state-changed", ());
    Ok(())
}

#[tauri::command]
#[specta::specta]
pub fn split_segment(
    state: State<SharedState>,
    app: AppHandle,
    layer_id: u64,
    track_id: u64,
    segment_id: u64,
    point_id: u64,
) -> Result<(), String> {
    use crate::domain::{LayerId, TrackId, TrackPointId, TrackSegmentId};
    let mut app_state = lock_app_state(state.inner())?;
    app_state
        .apply_split_segment(
            LayerId::new(layer_id),
            TrackId::new(track_id),
            TrackSegmentId::new(segment_id),
            TrackPointId::new(point_id),
        )
        .map_err(|e| format!("{e}"))?;
    let _ = app.emit("state-changed", ());
    Ok(())
}

#[tauri::command]
#[specta::specta]
pub fn join_segments(
    state: State<SharedState>,
    app: AppHandle,
    layer_id: u64,
    track_id: u64,
    segment_id_a: u64,
    segment_id_b: u64,
) -> Result<(), String> {
    use crate::domain::{LayerId, TrackId, TrackSegmentId};
    let mut app_state = lock_app_state(state.inner())?;
    app_state
        .apply_join_segments(
            LayerId::new(layer_id),
            TrackId::new(track_id),
            TrackSegmentId::new(segment_id_a),
            TrackSegmentId::new(segment_id_b),
        )
        .map_err(|e| format!("{e}"))?;
    let _ = app.emit("state-changed", ());
    Ok(())
}

#[tauri::command]
#[specta::specta]
pub fn delete_track(
    state: State<SharedState>,
    app: AppHandle,
    layer_id: u64,
    track_id: u64,
) -> Result<(), String> {
    use crate::domain::{LayerId, TrackId};
    let mut app_state = lock_app_state(state.inner())?;
    app_state
        .apply_delete_track(LayerId::new(layer_id), TrackId::new(track_id))
        .map_err(|e| format!("{e}"))?;
    let _ = app.emit("state-changed", ());
    Ok(())
}

// ── Waypoint mutations ────────────────────────────────────────────────────────

#[tauri::command]
#[specta::specta]
pub fn delete_waypoint(
    state: State<SharedState>,
    app: AppHandle,
    layer_id: u64,
    waypoint_id: u64,
) -> Result<(), String> {
    use crate::domain::{LayerId, WaypointId};
    let mut app_state = lock_app_state(state.inner())?;
    app_state
        .apply_delete_waypoint(LayerId::new(layer_id), WaypointId::new(waypoint_id))
        .map_err(|e| format!("{e}"))?;
    let _ = app.emit("state-changed", ());
    Ok(())
}

#[tauri::command]
#[specta::specta]
pub fn rename_waypoint(
    state: State<SharedState>,
    app: AppHandle,
    layer_id: u64,
    waypoint_id: u64,
    new_name: String,
) -> Result<(), String> {
    use crate::domain::{LayerId, WaypointId};
    let mut app_state = lock_app_state(state.inner())?;
    app_state
        .apply_rename_waypoint(
            LayerId::new(layer_id),
            WaypointId::new(waypoint_id),
            new_name,
        )
        .map_err(|e| format!("{e}"))?;
    let _ = app.emit("state-changed", ());
    Ok(())
}

#[tauri::command]
#[specta::specta]
pub fn move_waypoint(
    state: State<SharedState>,
    app: AppHandle,
    layer_id: u64,
    waypoint_id: u64,
    position: [f64; 2],
) -> Result<(), String> {
    use crate::domain::{LayerId, WaypointId};
    let mut app_state = lock_app_state(state.inner())?;
    app_state
        .apply_move_waypoint(
            LayerId::new(layer_id),
            WaypointId::new(waypoint_id),
            position[0],
            position[1],
        )
        .map_err(|e| format!("{e}"))?;
    let _ = app.emit("state-changed", ());
    Ok(())
}

#[tauri::command]
#[specta::specta]
pub fn add_waypoint(
    state: State<SharedState>,
    app: AppHandle,
    layer_id: u64,
    lat: f64,
    lon: f64,
    name: String,
) -> Result<(), String> {
    use crate::domain::LayerId;
    let mut app_state = lock_app_state(state.inner())?;
    app_state
        .apply_add_waypoint(LayerId::new(layer_id), lat, lon, name)
        .map_err(|e| format!("{e}"))?;
    let _ = app.emit("state-changed", ());
    Ok(())
}

// ── Track simplification ──────────────────────────────────────────────────────

#[tauri::command]
#[specta::specta]
pub fn simplify_track(
    state: State<SharedState>,
    app: AppHandle,
    layer_id: u64,
    track_id: u64,
    tolerance: f64,
) -> Result<(), String> {
    use crate::domain::{LayerId, TrackId};
    let mut app_state = lock_app_state(state.inner())?;
    app_state
        .apply_simplify_track(LayerId::new(layer_id), TrackId::new(track_id), tolerance)
        .map_err(|e| format!("{e}"))?;
    let _ = app.emit("state-changed", ());
    Ok(())
}

/// CJ-4: sort every segment's points by timestamp (untimed first, stable).
/// One undoable step; a no-op when the track is already ordered.
#[tauri::command]
#[specta::specta]
pub fn sort_track_points(
    state: State<SharedState>,
    app: AppHandle,
    layer_id: u64,
    track_id: u64,
) -> Result<(), String> {
    use crate::domain::{LayerId, TrackId};
    let mut app_state = lock_app_state(state.inner())?;
    app_state
        .apply_sort_track_points(LayerId::new(layer_id), TrackId::new(track_id))
        .map_err(|e| format!("{e}"))?;
    let _ = app.emit("state-changed", ());
    Ok(())
}

/// Lat/lon bounding box for extent crops — the current map viewport.
#[derive(serde::Deserialize, specta::Type)]
pub struct ExtentDto {
    pub min_lat: f64,
    pub min_lon: f64,
    pub max_lat: f64,
    pub max_lon: f64,
}

/// CJ-4: crop the track to a lat/lon extent (the current map view). Returns
/// how many points were removed; refuses to remove every point.
#[tauri::command]
#[specta::specta]
pub fn crop_track_to_extent(
    state: State<SharedState>,
    app: AppHandle,
    layer_id: u64,
    track_id: u64,
    extent: ExtentDto,
) -> Result<u32, String> {
    use crate::domain::{LayerId, TrackId};
    let mut app_state = lock_app_state(state.inner())?;
    let removed = app_state
        .apply_crop_track_to_extent(
            LayerId::new(layer_id),
            TrackId::new(track_id),
            extent.min_lat,
            extent.min_lon,
            extent.max_lat,
            extent.max_lon,
        )
        .map_err(|e| format!("{e}"))?;
    let _ = app.emit("state-changed", ());
    Ok(removed as u32)
}

/// Trim a track at one of its points, keeping that point. `before` cuts what
/// came earlier, otherwise what came later. Returns removed-point count.
#[tauri::command]
#[specta::specta]
pub fn trim_track_at_point(
    state: State<SharedState>,
    app: AppHandle,
    layer_id: u64,
    track_id: u64,
    point_id: u64,
    before: bool,
) -> Result<u32, String> {
    use crate::domain::{LayerId, TrackId, TrackPointId};
    let mut app_state = lock_app_state(state.inner())?;
    let removed = app_state
        .apply_trim_track_at_point(
            LayerId::new(layer_id),
            TrackId::new(track_id),
            TrackPointId::new(point_id),
            before,
        )
        .map_err(|e| format!("{e}"))?;
    let _ = app.emit("state-changed", ());
    Ok(removed as u32)
}

/// CJ-4: crop the track to a time range (ISO-8601 UTC bounds, either side
/// optional). Untimed points are always kept. Returns removed-point count.
#[tauri::command]
#[specta::specta]
pub fn crop_track_to_time(
    state: State<SharedState>,
    app: AppHandle,
    layer_id: u64,
    track_id: u64,
    from: Option<String>,
    to: Option<String>,
) -> Result<u32, String> {
    use crate::domain::{LayerId, TrackId};
    let parse = |value: Option<String>,
                 bound: &str|
     -> Result<Option<chrono::DateTime<chrono::Utc>>, String> {
        value
            .map(|raw| {
                raw.parse::<chrono::DateTime<chrono::Utc>>()
                    .map_err(|e| format!("invalid {bound} timestamp {raw:?}: {e}"))
            })
            .transpose()
    };
    let from = parse(from, "from")?;
    let to = parse(to, "to")?;
    let mut app_state = lock_app_state(state.inner())?;
    let removed = app_state
        .apply_crop_track_to_time(LayerId::new(layer_id), TrackId::new(track_id), from, to)
        .map_err(|e| format!("{e}"))?;
    let _ = app.emit("state-changed", ());
    Ok(removed as u32)
}

// ── Track style ───────────────────────────────────────────────────────────────

#[tauri::command]
#[specta::specta]
pub fn set_track_line_width(
    state: State<SharedState>,
    app: AppHandle,
    layer_id: u64,
    track_id: u64,
    width: f32,
) -> Result<(), String> {
    use crate::domain::{LayerId, TrackId};
    let mut app_state = lock_app_state(state.inner())?;
    app_state.set_track_line_width(LayerId::new(layer_id), TrackId::new(track_id), width);
    let _ = app.emit("state-changed", ());
    Ok(())
}

// ── Read endpoints ────────────────────────────────────────────────────────────

#[derive(serde::Serialize, specta::Type)]
pub struct PointDetailDto {
    pub id: u64,
    pub lat: f64,
    pub lon: f64,
    pub elevation: Option<f32>,
    pub timestamp: Option<String>,
}

#[derive(serde::Serialize, specta::Type)]
pub struct SegmentDetailDto {
    pub id: u64,
    pub points: Vec<PointDetailDto>,
}

#[derive(serde::Serialize, specta::Type)]
pub struct TrackDetailDto {
    pub id: u64,
    pub name: String,
    pub segments: Vec<SegmentDetailDto>,
}

#[tauri::command]
#[specta::specta]
pub fn get_track_detail(
    state: State<SharedState>,
    layer_id: u64,
    track_id: u64,
) -> Result<TrackDetailDto, String> {
    use crate::domain::{LayerId, TrackId};
    let app_state = lock_app_state(state.inner())?;
    let lid = LayerId::new(layer_id);
    let tid = TrackId::new(track_id);

    let layer = app_state
        .track_layers()
        .iter()
        .find(|l| l.id() == lid)
        .ok_or_else(|| format!("track layer {layer_id} not found"))?;

    let track = layer
        .tracks()
        .iter()
        .find(|t| t.id() == tid)
        .ok_or_else(|| format!("track {track_id} not found in layer {layer_id}"))?;

    Ok(track_detail_dto(track))
}

/// Map one track to the detail shape the Inspector renders.
pub fn track_detail_dto(track: &crate::domain::Track) -> TrackDetailDto {
    TrackDetailDto {
        id: track.id().value(),
        name: track.name().to_owned(),
        segments: track
            .segments()
            .iter()
            .map(|seg| SegmentDetailDto {
                id: seg.id().value(),
                points: seg
                    .points()
                    .iter()
                    .map(|pt| PointDetailDto {
                        id: pt.id().value(),
                        lat: pt.latitude(),
                        lon: pt.longitude(),
                        elevation: pt.elevation().map(|e| e as f32),
                        timestamp: pt.timestamp().map(|ts| ts.to_rfc3339()),
                    })
                    .collect(),
            })
            .collect(),
    }
}

#[derive(serde::Serialize, specta::Type)]
pub struct WaypointDto {
    pub id: u64,
    pub name: String,
    pub lat: f64,
    pub lon: f64,
    pub symbol: Option<String>,
    pub visible: bool,
    /// RGBA, or absent for "whatever the map draws waypoints with". Absent is
    /// not a colour: changing the default later moves every uncoloured
    /// waypoint with it.
    pub color: Option<[u8; 4]>,
    /// The note beside the mark: what a crew is actually sent to.
    pub description: Option<String>,
}

#[tauri::command]
#[specta::specta]
pub fn get_waypoints(state: State<SharedState>, layer_id: u64) -> Result<Vec<WaypointDto>, String> {
    use crate::domain::LayerId;
    let app_state = lock_app_state(state.inner())?;
    let lid = LayerId::new(layer_id);

    let layer = app_state
        .project_waypoint_layers()
        .iter()
        .find(|l| l.id() == lid)
        .ok_or_else(|| format!("waypoint layer {layer_id} not found"))?;

    Ok(waypoint_dtos(layer.waypoints()))
}

/// Map waypoints to the shape the Waypoints tab renders.
pub fn waypoint_dtos(waypoints: &[crate::domain::Waypoint]) -> Vec<WaypointDto> {
    waypoints
        .iter()
        .map(|w| WaypointDto {
            id: w.id().value(),
            name: w.name().to_owned(),
            lat: w.latitude(),
            lon: w.longitude(),
            symbol: w.symbol().map(str::to_owned),
            description: w.description().map(str::to_owned),
            visible: w.visible(),
            color: w.color(),
        })
        .collect()
}

#[derive(serde::Serialize, specta::Type)]
pub struct SimplifiedSegmentDto {
    pub id: u64,
    pub original_count: usize,
    pub simplified_count: usize,
    pub kept_points: Vec<PointDetailDto>,
}

#[derive(serde::Serialize, specta::Type)]
pub struct SimplifiedPreviewDto {
    pub original_count: usize,
    pub simplified_count: usize,
    pub segments: Vec<SimplifiedSegmentDto>,
}

#[tauri::command]
#[specta::specta]
pub fn get_simplified_preview(
    state: State<SharedState>,
    layer_id: u64,
    track_id: u64,
    tolerance: f64,
) -> Result<SimplifiedPreviewDto, String> {
    use crate::domain::{LayerId, TrackId, simplify_track_points};
    let app_state = lock_app_state(state.inner())?;
    let lid = LayerId::new(layer_id);
    let tid = TrackId::new(track_id);

    let layer = app_state
        .track_layers()
        .iter()
        .find(|l| l.id() == lid)
        .ok_or_else(|| format!("track layer {layer_id} not found"))?;

    let track = layer
        .tracks()
        .iter()
        .find(|t| t.id() == tid)
        .ok_or_else(|| format!("track {track_id} not found in layer {layer_id}"))?;

    let mut total_original = 0usize;
    let mut total_simplified = 0usize;

    let segments = track
        .segments()
        .iter()
        .map(|seg| {
            let pts = seg.points();
            let kept_indices = simplify_track_points(pts, tolerance);
            let original_count = pts.len();
            let simplified_count = kept_indices.len();
            total_original += original_count;
            total_simplified += simplified_count;

            let kept_points = kept_indices
                .iter()
                .map(|&i| PointDetailDto {
                    id: pts[i].id().value(),
                    lat: pts[i].latitude(),
                    lon: pts[i].longitude(),
                    elevation: pts[i].elevation().map(|e| e as f32),
                    timestamp: pts[i].timestamp().map(|ts| ts.to_rfc3339()),
                })
                .collect();

            SimplifiedSegmentDto {
                id: seg.id().value(),
                original_count,
                simplified_count,
                kept_points,
            }
        })
        .collect();

    Ok(SimplifiedPreviewDto {
        original_count: total_original,
        simplified_count: total_simplified,
        segments,
    })
}

// ── Open-in-finder ────────────────────────────────────────────────────────────

#[tauri::command]
#[specta::specta]
pub fn reveal_bundle(state: State<SharedState>) -> Result<(), String> {
    lock_app_state(state.inner())?.reveal_active_bundle();
    Ok(())
}

// ── Track creation ────────────────────────────────────────────────────────────

#[tauri::command]
#[specta::specta]
pub fn create_empty_track(
    state: State<SharedState>,
    app: AppHandle,
    layer_id: u64,
    name: String,
) -> Result<u64, String> {
    use crate::domain::LayerId;
    let mut app_state = lock_app_state(state.inner())?;
    let track_id = app_state
        .apply_create_empty_track(LayerId::new(layer_id), name)
        .map_err(|e| format!("{e}"))?;
    let _ = app.emit("state-changed", ());
    Ok(track_id.value())
}

#[cfg(test)]
mod tests {
    use super::{
        PointDetailDto, SegmentDetailDto, TrackDetailDto, app_state_dto, build_tracks_geojson,
        list_track_summaries,
    };
    use crate::domain::{Track, TrackId, TrackPoint, TrackPointId, TrackSegment, TrackSegmentId};

    /// Importing a folder is how a day's recordings arrive — the archive is a
    /// directory of per-date subfolders — and the command answered with an
    /// English sentence that the Tracks tab put straight into a toast. A
    /// Russian crew read "Imported 12 tracks and 3 waypoints from 4 files".
    ///
    /// The same rule the bundle progress got in `progress-in-the-crews-
    /// language`: the backend sends what happened, the interface says it.
    #[test]
    fn a_folder_import_reports_counts_rather_than_an_english_sentence() {
        let report = crate::application::import::DirectoryImportReport {
            imported_files: 4,
            imported_tracks: 12,
            imported_waypoints: 3,
            skipped: vec![(
                std::path::PathBuf::from("/searches/2026-09-21/broken.plt"),
                "no valid points".to_owned(),
            )],
        };

        let dto = super::ImportReportDto::from(report);

        assert_eq!(dto.files, 4);
        assert_eq!(dto.tracks, 12);
        assert_eq!(dto.waypoints, 3);
        // The names, not the paths: a toast has no room for a directory tree,
        // and the file name is what the operator recognises.
        assert_eq!(dto.skipped, vec!["broken.plt".to_owned()]);
    }

    /// A folder where everything imported says so with an empty list, not with
    /// an absent field the interface has to guess about.
    #[test]
    fn a_clean_folder_import_reports_nothing_skipped() {
        let dto = super::ImportReportDto::from(crate::application::import::DirectoryImportReport {
            imported_files: 2,
            imported_tracks: 2,
            imported_waypoints: 0,
            skipped: Vec::new(),
        });
        assert!(dto.skipped.is_empty());
    }

    #[test]
    fn test_get_track_detail() {
        let track_id = TrackId::new(10);
        let segment_id = TrackSegmentId::new(5);

        let mut track = Track::new(track_id, "My Track");
        let mut segment = TrackSegment::new(segment_id);
        segment.add_point(TrackPoint::new(TrackPointId::new(1), 55.0, 37.0));
        segment.add_point(TrackPoint::new(TrackPointId::new(2), 55.1, 37.1));
        track.add_segment(segment);

        let dto = TrackDetailDto {
            id: track.id().value(),
            name: track.name().to_owned(),
            segments: track
                .segments()
                .iter()
                .map(|s| SegmentDetailDto {
                    id: s.id().value(),
                    points: s
                        .points()
                        .iter()
                        .map(|p| PointDetailDto {
                            id: p.id().value(),
                            lat: p.latitude(),
                            lon: p.longitude(),
                            elevation: p.elevation().map(|e| e as f32),
                            timestamp: p.timestamp().map(|ts| ts.to_rfc3339()),
                        })
                        .collect(),
                })
                .collect(),
        };

        assert_eq!(dto.id, 10);
        assert_eq!(dto.name, "My Track");
        assert_eq!(dto.segments.len(), 1);
        assert_eq!(dto.segments[0].id, 5);
        assert_eq!(dto.segments[0].points.len(), 2);
        assert_eq!(dto.segments[0].points[0].lat, 55.0);
        assert_eq!(dto.segments[0].points[0].lon, 37.0);
        assert_eq!(dto.segments[0].points[1].lat, 55.1);
        assert!(dto.segments[0].points[0].elevation.is_none());
        assert!(dto.segments[0].points[0].timestamp.is_none());
    }

    fn layer_with(tracks: Vec<Track>) -> crate::domain::TrackLayer {
        let mut layer = crate::domain::TrackLayer::new(crate::domain::LayerId::new(7), "Tracks");
        for track in tracks {
            layer.add_track(track);
        }
        layer
    }

    fn segment(id: u64, points: &[(f64, f64)]) -> TrackSegment {
        let mut seg = TrackSegment::new(TrackSegmentId::new(id));
        for (i, (lat, lon)) in points.iter().enumerate() {
            seg.add_point(TrackPoint::new(
                TrackPointId::new(id * 100 + i as u64),
                *lat,
                *lon,
            ));
        }
        seg
    }

    fn track_with(id: u64, name: &str, segments: Vec<TrackSegment>) -> Track {
        let mut track = Track::new(TrackId::new(id), name);
        for seg in segments {
            track.add_segment(seg);
        }
        track
    }

    /// A recorded track is one Feature with one MultiLineString part per
    /// segment. Flattening the segments into a single LineString drew a
    /// straight line across the map between the end of one segment and the
    /// start of the next.
    #[test]
    fn tracks_geojson_emits_one_part_per_segment() {
        let layer = layer_with(vec![track_with(
            2,
            "Two days",
            vec![
                segment(1, &[(55.0, 37.0), (55.1, 37.1)]),
                segment(2, &[(56.0, 38.0), (56.1, 38.1), (56.2, 38.2)]),
            ],
        )]);

        let geojson = build_tracks_geojson(std::slice::from_ref(&layer));

        let features = geojson["features"].as_array().expect("features array");
        assert_eq!(features.len(), 1);
        let geometry = &features[0]["geometry"];
        assert_eq!(geometry["type"], "MultiLineString");
        let parts = geometry["coordinates"].as_array().expect("parts");
        assert_eq!(parts.len(), 2, "one part per segment");
        assert_eq!(parts[0].as_array().expect("part 0").len(), 2);
        assert_eq!(parts[1].as_array().expect("part 1").len(), 3);
        assert_eq!(parts[0][0], serde_json::json!([37.0, 55.0]));
        assert_eq!(features[0]["properties"]["name"], "Two days");
        assert_eq!(features[0]["properties"]["track_id"], 2);
        assert_eq!(features[0]["properties"]["layer_id"], 7);
    }

    /// Splitting a segment adds a part, which is how a split becomes visible
    /// on the map at all.
    #[test]
    fn tracks_geojson_part_count_follows_segment_count() {
        let one = layer_with(vec![track_with(
            1,
            "Whole",
            vec![segment(1, &[(55.0, 37.0), (55.1, 37.1), (55.2, 37.2)])],
        )]);
        let split = layer_with(vec![track_with(
            1,
            "Split",
            vec![
                segment(1, &[(55.0, 37.0), (55.1, 37.1)]),
                segment(2, &[(55.1, 37.1), (55.2, 37.2)]),
            ],
        )]);

        let before = build_tracks_geojson(std::slice::from_ref(&one));
        let after = build_tracks_geojson(std::slice::from_ref(&split));

        assert_eq!(
            before["features"][0]["geometry"]["coordinates"]
                .as_array()
                .expect("parts")
                .len(),
            1
        );
        assert_eq!(
            after["features"][0]["geometry"]["coordinates"]
                .as_array()
                .expect("parts")
                .len(),
            2
        );
    }

    /// A one-point segment cannot be drawn as a line; it must not collapse the
    /// rest of the track or produce a degenerate part.
    #[test]
    fn tracks_geojson_skips_segments_shorter_than_two_points() {
        let layer = layer_with(vec![track_with(
            3,
            "Stray point",
            vec![
                segment(1, &[(55.0, 37.0)]),
                segment(2, &[(56.0, 38.0), (56.1, 38.1)]),
            ],
        )]);

        let geojson = build_tracks_geojson(std::slice::from_ref(&layer));

        let parts = geojson["features"][0]["geometry"]["coordinates"]
            .as_array()
            .expect("parts");
        assert_eq!(parts.len(), 1);
        assert_eq!(parts[0].as_array().expect("part 0").len(), 2);
    }

    /// A track with nothing drawable is omitted entirely rather than emitted
    /// as an empty geometry.
    #[test]
    fn tracks_geojson_omits_tracks_without_drawable_parts() {
        let layer = layer_with(vec![
            track_with(4, "Empty", vec![]),
            track_with(5, "Single point", vec![segment(1, &[(55.0, 37.0)])]),
            track_with(
                6,
                "Drawable",
                vec![segment(2, &[(55.0, 37.0), (55.1, 37.1)])],
            ),
        ]);

        let geojson = build_tracks_geojson(std::slice::from_ref(&layer));

        let features = geojson["features"].as_array().expect("features array");
        assert_eq!(features.len(), 1);
        assert_eq!(features[0]["properties"]["name"], "Drawable");
    }

    /// The Tracks tab built its rows from the map's GeoJSON, which drops a
    /// track with nothing drawable — correctly, for the map. The list
    /// inherited that: a track of one point, or of none, was in the project,
    /// counted towards its size, exported with it, and could not be seen,
    /// renamed or deleted, because it had no row.
    ///
    /// The listing is its own thing now, and it lists every track.
    #[test]
    fn the_track_listing_includes_tracks_the_map_cannot_draw() {
        let layer = layer_with(vec![
            track_with(4, "Empty", vec![]),
            track_with(5, "Single point", vec![segment(1, &[(55.0, 37.0)])]),
            track_with(
                6,
                "Drawable",
                vec![segment(2, &[(55.0, 37.0), (55.1, 37.1)])],
            ),
        ]);

        let rows = list_track_summaries(std::slice::from_ref(&layer));

        assert_eq!(
            rows.iter().map(|r| r.name.as_str()).collect::<Vec<_>>(),
            vec!["Empty", "Single point", "Drawable"],
            "every track in the project has a row, drawable or not"
        );
        assert_eq!(rows[0].point_count, 0);
        assert_eq!(rows[1].point_count, 1);
    }

    /// The rows carry what a row shows and nothing else. They used to arrive
    /// as the map's GeoJSON: every coordinate of every track, serialized,
    /// sent over IPC and parsed, so that a list of names could be drawn. A
    /// day's folder of recordings is hundreds of thousands of points.
    /// `get_app_state` is fetched on every `state-changed`, and during a
    /// bundle download that fires once per file. It used to carry the whole
    /// LizaAlert catalogue with it — thirteen thousand rows of slug and name,
    /// roughly a megabyte of JSON — for one consumer that only needed it to
    /// seed a store already seeded from `localStorage` and from the first
    /// `projects-chunk`. The catalogue is not application state.
    #[test]
    fn the_application_state_does_not_carry_the_catalogue() {
        use crate::application::{AppState, LizaProjectSummary};

        let mut state = AppState::new();
        for i in 0..500 {
            state.push_project_summary_for_test(LizaProjectSummary {
                slug: format!("2026-09-{:02}_search-{i}", (i % 28) + 1),
                name: format!("Search {i}"),
                url: format!("https://example.invalid/search-{i}/"),
            });
        }

        let dto = app_state_dto(&state);
        let json = serde_json::to_string(&dto).expect("serialize app state");

        assert!(
            !json.contains("search-499"),
            "the catalogue must not ride along in the application state"
        );
        assert!(
            json.len() < 4_000,
            "application state stayed small; it was {} bytes",
            json.len()
        );
    }

    #[test]
    fn the_track_listing_carries_no_geometry() {
        let layer = layer_with(vec![track_with(
            7,
            "Long one",
            vec![segment(1, &[(55.0, 37.0), (55.1, 37.1), (55.2, 37.2)])],
        )]);

        let rows = list_track_summaries(std::slice::from_ref(&layer));
        let json = serde_json::to_string(&rows).expect("serialize rows");

        assert!(!json.contains("coordinates"), "rows must carry no geometry");
        assert!(!json.contains("55.1"), "nor any point of one: {json}");
        assert_eq!(rows[0].point_count, 3, "only the count of them");
    }
}
