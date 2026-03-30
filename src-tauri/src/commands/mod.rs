pub mod tiles;

pub use tiles::{get_ozi_metadata, get_ozi_tile, get_sqlite_tile};

use crate::application::{
    ActiveMapKind, AppState, DiagnosticLevel, OpenMapRequest,
};
use crate::infrastructure::lizaalert;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::thread;
use tauri::{AppHandle, Emitter, State};

pub type SharedState = Arc<Mutex<AppState>>;

// ── Serializable DTOs ────────────────────────────────────────────────────────

#[derive(serde::Serialize, Clone)]
pub struct DiagnosticDto {
    pub level: &'static str,
    pub message: String,
}

#[derive(serde::Serialize)]
pub struct AppStateDto {
    pub project_name: String,
    pub project_saved: bool,
    pub status: String,
    pub busy: bool,
    pub projects: Vec<LizaProjectSummaryDto>,
    pub current_project: Option<LizaProjectDto>,
    pub active_map: Option<ActiveMapDto>,
    pub diagnostics: Vec<DiagnosticDto>,
    pub track_layer_count: usize,
    pub waypoint_layer_count: usize,
}

#[derive(serde::Serialize)]
pub struct LizaProjectSummaryDto {
    pub slug: String,
    pub name: String,
}

#[derive(serde::Serialize)]
pub struct LizaProjectDto {
    pub name: String,
    pub center_lat: f64,
    pub center_lon: f64,
    pub maps: Vec<LizaMapPackageDto>,
}

#[derive(serde::Serialize)]
pub struct LizaMapPackageDto {
    pub name: String,
    pub base_zoom: u8,
    pub downloaded: bool,
}

#[derive(serde::Serialize)]
pub struct ActiveMapDto {
    pub kind: &'static str,
    pub project_name: String,
    pub package_name: String,
    pub local_path: String,
    pub center_lat: f64,
    pub center_lon: f64,
    pub base_zoom: u8,
}

// Events
#[derive(serde::Serialize, Clone)]
struct DownloadProgressPayload {
    downloaded_bytes: u64,
    total_bytes: Option<u64>,
}

#[derive(serde::Serialize, Clone)]
struct DiagnosticPayload {
    level: &'static str,
    message: String,
}

// ── State snapshot ────────────────────────────────────────────────────────────

#[tauri::command]
pub fn get_app_state(state: State<SharedState>) -> AppStateDto {
    let s = state.lock().unwrap();

    let projects = s
        .lizaalert_projects()
        .iter()
        .map(|p| LizaProjectSummaryDto {
            slug: p.slug.clone(),
            name: p.name.clone(),
        })
        .collect();

    let current_project = s.current_project().map(|p| LizaProjectDto {
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
                DiagnosticLevel::Error => "error",
            },
            message: d.message().to_owned(),
        })
        .collect();

    AppStateDto {
        project_name: s.project_name().to_owned(),
        project_saved: s.project_file_path().is_some(),
        status: s.lizaalert_status().to_owned(),
        busy: s.lizaalert_busy(),
        projects,
        current_project,
        active_map,
        diagnostics,
        track_layer_count: s.track_layer_count(),
        waypoint_layer_count: s.waypoint_layer_count(),
    }
}

// ── Track GeoJSON ─────────────────────────────────────────────────────────────

#[tauri::command]
pub fn get_tracks_geojson(state: State<SharedState>) -> serde_json::Value {
    let s = state.lock().unwrap();
    let mut features = Vec::new();

    for layer in s.track_layers() {
        // LayerId has .value(); TrackId is #[serde(transparent)] so it serializes as u64
        let layer_id_val = layer.id().value();
        for track in layer.tracks() {
            let style = track.style();
            let [r, g, b, a] = style.color;
            let color = format!(
                "rgba({r},{g},{b},{:.3})",
                a as f64 / 255.0 * style.opacity as f64
            );

            let coords: Vec<serde_json::Value> = track
                .segments()
                .iter()
                .flat_map(|seg| seg.points())
                .map(|pt| serde_json::json!([pt.longitude(), pt.latitude()]))
                .collect();

            features.push(serde_json::json!({
                "type": "Feature",
                "properties": {
                    "layer_id": layer_id_val,
                    "track_id": track.id(),
                    "name": track.name(),
                    "color": color,
                    "line_width": style.line_width,
                    "visible": style.visible,
                },
                "geometry": {
                    "type": "LineString",
                    "coordinates": coords,
                }
            }));
        }
    }

    serde_json::json!({
        "type": "FeatureCollection",
        "features": features,
    })
}

// ── LizaAlert project loading ─────────────────────────────────────────────────

#[tauri::command]
pub fn load_projects(state: State<SharedState>, app: AppHandle) {
    let started = state.lock().unwrap().begin_load_projects();
    if started.is_none() {
        return;
    }

    let state_arc = Arc::clone(&state);
    thread::spawn(move || {
        let result = lizaalert::fetch_project_summaries();
        state_arc.lock().unwrap().apply_projects_loaded(result);
        let _ = app.emit("state-changed", ());
    });
}

#[tauri::command]
pub fn load_project(slug: String, state: State<SharedState>, app: AppHandle) {
    let data = state.lock().unwrap().begin_load_project(&slug);
    let Some((summary, bundles_root)) = data else {
        return;
    };

    let state_arc = Arc::clone(&state);
    thread::spawn(move || {
        let result = lizaalert::open_project(summary, &bundles_root, |progress| {
            state_arc
                .lock()
                .unwrap()
                .apply_progress(progress.message.clone());
            let _ = app.emit("state-changed", ());
        });
        state_arc.lock().unwrap().apply_project_loaded(result);
        let _ = app.emit("state-changed", ());
    });
}

#[tauri::command]
pub fn open_selected_map(map_name: String, state: State<SharedState>, app: AppHandle) {
    let request = state.lock().unwrap().begin_open_map(&map_name);
    let Some(request) = request else {
        return;
    };

    match request {
        OpenMapRequest::Local(selection) => {
            // OZI raster — open synchronously
            if selection.kind == ActiveMapKind::OziRaster {
                let path = selection.local_path.clone();
                match state.lock().unwrap().open_local_ozi_map(path) {
                    Ok(()) => {}
                    Err(e) => state
                        .lock()
                        .unwrap()
                        .report_runtime_error(e.to_string()),
                }
            } else {
                state.lock().unwrap().open_local_map_selection(selection);
            }
            let _ = app.emit("state-changed", ());
        }
        OpenMapRequest::Download(selection) => {
            let state_arc = Arc::clone(&state);
            thread::spawn(move || {
                let result = lizaalert::download_map(selection, |progress| {
                    let _ = app.emit(
                        "download-progress",
                        DownloadProgressPayload {
                            downloaded_bytes: progress.downloaded_bytes,
                            total_bytes: progress.total_bytes,
                        },
                    );
                });
                state_arc.lock().unwrap().apply_map_downloaded(result);
                let _ = app.emit("state-changed", ());
            });
        }
    }
}

#[tauri::command]
pub fn open_local_bundle(dir: String, state: State<SharedState>, app: AppHandle) {
    let data = state
        .lock()
        .unwrap()
        .begin_open_local_bundle(PathBuf::from(&dir));
    let Some(dir_path) = data else {
        return;
    };

    let state_arc = Arc::clone(&state);
    thread::spawn(move || {
        let result = lizaalert::open_bundle_directory(&dir_path, |progress| {
            state_arc
                .lock()
                .unwrap()
                .apply_progress(progress.message.clone());
            let _ = app.emit("state-changed", ());
        });
        state_arc.lock().unwrap().apply_project_loaded(result);
        let _ = app.emit("state-changed", ());
    });
}

// ── Settings ──────────────────────────────────────────────────────────────────

#[tauri::command]
pub fn set_bundles_root(path: String, state: State<SharedState>, app: AppHandle) {
    state
        .lock()
        .unwrap()
        .set_bundles_root(PathBuf::from(path));
    let _ = app.emit("state-changed", ());
}

// ── Project persistence ───────────────────────────────────────────────────────

#[tauri::command]
pub fn save_project(path: String, state: State<SharedState>, app: AppHandle) {
    state
        .lock()
        .unwrap()
        .save_project_to(PathBuf::from(path));
    let _ = app.emit("state-changed", ());
}

#[tauri::command]
pub fn load_project_file(path: String, state: State<SharedState>, app: AppHandle) {
    state
        .lock()
        .unwrap()
        .load_project_from(PathBuf::from(path));
    let _ = app.emit("state-changed", ());
}

// ── Import / export ───────────────────────────────────────────────────────────

#[tauri::command]
pub fn import_gpx(path: String, state: State<SharedState>, app: AppHandle) -> Result<String, String> {
    let result = state
        .lock()
        .unwrap()
        .import_gpx_file(PathBuf::from(path))
        .map_err(|e| e.to_string())?;
    let _ = app.emit("state-changed", ());
    Ok(format!(
        "Imported {} tracks in {} layers",
        result.imported_tracks(), result.imported_track_layers()
    ))
}

#[tauri::command]
pub fn import_plt(path: String, state: State<SharedState>, app: AppHandle) -> Result<String, String> {
    let result = state
        .lock()
        .unwrap()
        .import_plt_file(PathBuf::from(path))
        .map_err(|e| e.to_string())?;
    let _ = app.emit("state-changed", ());
    Ok(format!(
        "Imported {} tracks in {} layers",
        result.imported_tracks(), result.imported_track_layers()
    ))
}

#[tauri::command]
pub fn export_gpx(
    layer_id: u64,
    path: String,
    state: State<SharedState>,
    app: AppHandle,
) {
    use crate::domain::LayerId;
    state
        .lock()
        .unwrap()
        .export_layer_to_gpx(LayerId::new(layer_id), PathBuf::from(path));
    let _ = app.emit("state-changed", ());
}

// ── Undo / redo ───────────────────────────────────────────────────────────────

#[tauri::command]
pub fn undo(state: State<SharedState>, app: AppHandle) {
    state.lock().unwrap().undo();
    let _ = app.emit("state-changed", ());
}

#[tauri::command]
pub fn redo(state: State<SharedState>, app: AppHandle) {
    state.lock().unwrap().redo();
    let _ = app.emit("state-changed", ());
}

// ── Track mutations ───────────────────────────────────────────────────────────

#[tauri::command]
pub fn rename_track(
    layer_id: u64,
    track_id: u64,
    new_name: String,
    state: State<SharedState>,
    app: AppHandle,
) {
    use crate::domain::{LayerId, TrackId};
    state
        .lock()
        .unwrap()
        .rename_track(LayerId::new(layer_id), TrackId::new(track_id), new_name);
    let _ = app.emit("state-changed", ());
}

#[tauri::command]
pub fn set_track_color(
    layer_id: u64,
    track_id: u64,
    color: [u8; 4],
    state: State<SharedState>,
    app: AppHandle,
) {
    use crate::domain::{LayerId, TrackId};
    state
        .lock()
        .unwrap()
        .set_track_color(LayerId::new(layer_id), TrackId::new(track_id), color);
    let _ = app.emit("state-changed", ());
}

#[tauri::command]
pub fn toggle_track_visible(
    layer_id: u64,
    track_id: u64,
    state: State<SharedState>,
    app: AppHandle,
) {
    use crate::domain::{LayerId, TrackId};
    state
        .lock()
        .unwrap()
        .toggle_track_visible(LayerId::new(layer_id), TrackId::new(track_id));
    let _ = app.emit("state-changed", ());
}

// ── Open-in-finder ────────────────────────────────────────────────────────────

#[tauri::command]
pub fn reveal_bundle(state: State<SharedState>) {
    state.lock().unwrap().reveal_active_bundle();
}
