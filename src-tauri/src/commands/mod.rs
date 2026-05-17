pub mod tiles;

use crate::application::{
    ActiveMapKind, AppState, DiagnosticLevel, LizaProjectSummary, OpenMapRequest,
};
use crate::infrastructure::lizaalert;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::thread;
use tauri::{AppHandle, Emitter, State};

pub type SharedState = Arc<Mutex<AppState>>;

fn lock_app_state<'a>(
    state: &'a SharedState,
) -> Result<std::sync::MutexGuard<'a, AppState>, String> {
    state
        .lock()
        .map_err(|e| format!("State lock poisoned: {}", e))
}

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
    pub downloading_maps: Vec<String>,
    pub projects: Vec<LizaProjectSummaryDto>,
    pub current_project: Option<LizaProjectDto>,
    pub active_map: Option<ActiveMapDto>,
    pub diagnostics: Vec<DiagnosticDto>,
    pub track_layers: Vec<LayerSummaryDto>,
    pub waypoint_layers: Vec<LayerSummaryDto>,
    pub track_layer_count: usize,
    pub waypoint_layer_count: usize,
}

#[derive(serde::Serialize, Clone)]
pub struct LayerSummaryDto {
    pub id: u64,
    pub name: String,
}

#[derive(serde::Serialize, Clone)]
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

fn to_project_summary_dtos(projects: &[LizaProjectSummary]) -> Vec<LizaProjectSummaryDto> {
    projects
        .iter()
        .map(|project| LizaProjectSummaryDto {
            slug: project.slug.clone(),
            name: project.name.clone(),
        })
        .collect()
}

// Events
#[derive(serde::Serialize, Clone)]
struct DownloadProgressPayload {
    package_name: String,
    downloaded_bytes: u64,
    total_bytes: Option<u64>,
}

#[derive(serde::Serialize, Clone)]
struct BundleProgressPayload {
    message: String,
    phase: &'static str,
    completed: Option<u64>,
    total: Option<u64>,
    downloaded_bytes: Option<u64>,
    total_bytes: Option<u64>,
}

// ── State snapshot ────────────────────────────────────────────────────────────

#[tauri::command]
pub fn get_app_state(state: State<SharedState>) -> Result<AppStateDto, String> {
    let s = lock_app_state(state.inner())?;

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

    let track_layers = s
        .track_layers()
        .iter()
        .map(|layer| LayerSummaryDto {
            id: layer.id().value(),
            name: layer.name().to_owned(),
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

    Ok(AppStateDto {
        project_name: s.project_name().to_owned(),
        project_saved: s.project_file_path().is_some(),
        status: s.lizaalert_status().to_owned(),
        busy: s.lizaalert_busy(),
        downloading_maps: s.downloading_maps().iter().cloned().collect(),
        projects,
        current_project,
        active_map,
        diagnostics,
        track_layers,
        waypoint_layers,
        track_layer_count: s.track_layer_count(),
        waypoint_layer_count: s.waypoint_layer_count(),
    })
}

// ── Track GeoJSON ─────────────────────────────────────────────────────────────

#[tauri::command]
pub fn get_tracks_geojson(state: State<SharedState>) -> Result<serde_json::Value, String> {
    let s = lock_app_state(state.inner())?;
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

            if coords.len() < 2 {
                continue;
            }

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

    Ok(serde_json::json!({
        "type": "FeatureCollection",
        "features": features,
    }))
}

// ── LizaAlert project loading ─────────────────────────────────────────────────

#[tauri::command]
pub fn load_projects(state: State<SharedState>, app: AppHandle) -> Result<(), String> {
    let Some(bundles_root) = lock_app_state(state.inner())?.begin_load_projects() else {
        return Ok(());
    };

    if let Ok(cached_projects) = lizaalert::load_project_summaries_cache(&bundles_root)
        && !cached_projects.is_empty()
    {
        if let Ok(mut s) = lock_app_state(state.inner()) {
            s.apply_projects_chunk(cached_projects.clone());
        }
        let _ = app.emit("projects-chunk", to_project_summary_dtos(&cached_projects));
    }

    let state_arc = Arc::clone(&state);
    thread::spawn(move || {
        let chunk_state = Arc::clone(&state_arc);
        let chunk_app = app.clone();
        let result = lizaalert::fetch_project_summaries_streaming(move |chunk| {
            let chunk_payload = to_project_summary_dtos(&chunk);
            if let Ok(mut s) = lock_app_state(&chunk_state) {
                s.apply_projects_chunk(chunk);
            }
            let _ = chunk_app.emit("projects-chunk", chunk_payload);
        });

        if let Ok(projects) = &result {
            let _ = lizaalert::save_project_summaries_cache(&bundles_root, projects);
        }

        if let Ok(mut s) = lock_app_state(&state_arc) {
            s.apply_projects_loaded(result);
        }
        let _ = app.emit("state-changed", ());
    });

    Ok(())
}

#[tauri::command]
pub fn load_project(slug: String, state: State<SharedState>, app: AppHandle) -> Result<(), String> {
    let data = lock_app_state(state.inner())?.begin_load_project(&slug);
    let Some((summary, bundles_root)) = data else {
        return Ok(());
    };
    let _ = app.emit("state-changed", ());

    let state_arc = Arc::clone(&state);
    thread::spawn(move || {
        let result = lizaalert::open_project(summary, &bundles_root, |progress| {
            if let Ok(mut s) = lock_app_state(&state_arc) {
                s.apply_progress(progress.message.clone());
            }
            let _ = app.emit(
                "bundle-progress",
                BundleProgressPayload {
                    message: progress.message,
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

    Ok(())
}

#[tauri::command]
pub fn open_selected_map(
    map_name: String,
    state: State<SharedState>,
    app: AppHandle,
) -> Result<(), String> {
    let request = lock_app_state(state.inner())?.begin_open_map(&map_name);
    let Some(request) = request else {
        return Ok(());
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
            let package_name = selection.package_name.clone();
            thread::spawn(move || {
                let pkg = package_name.clone();
                let result = lizaalert::download_map(selection, |progress| {
                    let _ = app.emit(
                        "download-progress",
                        DownloadProgressPayload {
                            package_name: pkg.clone(),
                            downloaded_bytes: progress.downloaded_bytes,
                            total_bytes: progress.total_bytes,
                        },
                    );
                });
                if let Ok(mut s) = lock_app_state(&state_arc) {
                    s.apply_map_downloaded(&package_name, result);
                }
                let _ = app.emit("state-changed", ());
            });
        }
    }

    Ok(())
}

#[tauri::command]
pub fn open_local_bundle(
    dir: String,
    state: State<SharedState>,
    app: AppHandle,
) -> Result<(), String> {
    let data = lock_app_state(state.inner())?.begin_open_local_bundle(PathBuf::from(&dir));
    let Some(dir_path) = data else {
        return Ok(());
    };
    let _ = app.emit("state-changed", ());

    let state_arc = Arc::clone(&state);
    thread::spawn(move || {
        let result = lizaalert::open_bundle_directory(&dir_path, |progress| {
            if let Ok(mut s) = lock_app_state(&state_arc) {
                s.apply_progress(progress.message.clone());
            }
            let _ = app.emit(
                "bundle-progress",
                BundleProgressPayload {
                    message: progress.message,
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

    Ok(())
}

// ── Settings ──────────────────────────────────────────────────────────────────

#[tauri::command]
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
pub fn save_project(path: String, state: State<SharedState>, app: AppHandle) -> Result<(), String> {
    lock_app_state(state.inner())?.save_project_to(PathBuf::from(path));
    let _ = app.emit("state-changed", ());
    Ok(())
}

#[tauri::command]
pub fn load_project_file(
    path: String,
    state: State<SharedState>,
    app: AppHandle,
) -> Result<(), String> {
    lock_app_state(state.inner())?.load_project_from(PathBuf::from(path));
    let _ = app.emit("state-changed", ());
    Ok(())
}

// ── Import / export ───────────────────────────────────────────────────────────

#[tauri::command]
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

#[tauri::command]
pub fn export_gpx(
    layer_id: u64,
    path: String,
    state: State<SharedState>,
    app: AppHandle,
) -> Result<(), String> {
    use crate::domain::LayerId;
    lock_app_state(state.inner())?.export_layer_to_gpx(LayerId::new(layer_id), PathBuf::from(path));
    let _ = app.emit("state-changed", ());
    Ok(())
}

#[tauri::command]
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
pub fn undo(state: State<SharedState>, app: AppHandle) -> Result<(), String> {
    lock_app_state(state.inner())?.undo();
    let _ = app.emit("state-changed", ());
    Ok(())
}

#[tauri::command]
pub fn redo(state: State<SharedState>, app: AppHandle) -> Result<(), String> {
    lock_app_state(state.inner())?.redo();
    let _ = app.emit("state-changed", ());
    Ok(())
}

// ── Track mutations ───────────────────────────────────────────────────────────

#[tauri::command]
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

#[tauri::command]
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

#[tauri::command]
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

// ── Track style ───────────────────────────────────────────────────────────────

#[tauri::command]
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

#[derive(serde::Serialize)]
pub struct PointDetailDto {
    pub id: u64,
    pub lat: f64,
    pub lon: f64,
    pub elevation: Option<f32>,
    pub timestamp: Option<String>,
}

#[derive(serde::Serialize)]
pub struct SegmentDetailDto {
    pub id: u64,
    pub points: Vec<PointDetailDto>,
}

#[derive(serde::Serialize)]
pub struct TrackDetailDto {
    pub id: u64,
    pub name: String,
    pub segments: Vec<SegmentDetailDto>,
}

#[tauri::command]
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

    let segments = track
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
                    timestamp: pt
                        .timestamp()
                        .map(|ts| ts.to_rfc3339()),
                })
                .collect(),
        })
        .collect();

    Ok(TrackDetailDto {
        id: track.id().value(),
        name: track.name().to_owned(),
        segments,
    })
}

#[derive(serde::Serialize)]
pub struct WaypointDto {
    pub id: u64,
    pub name: String,
    pub lat: f64,
    pub lon: f64,
    pub symbol: Option<String>,
    pub visible: bool,
}

#[tauri::command]
pub fn get_waypoints(
    state: State<SharedState>,
    layer_id: u64,
) -> Result<Vec<WaypointDto>, String> {
    use crate::domain::LayerId;
    let app_state = lock_app_state(state.inner())?;
    let lid = LayerId::new(layer_id);

    let layer = app_state
        .project_waypoint_layers()
        .iter()
        .find(|l| l.id() == lid)
        .ok_or_else(|| format!("waypoint layer {layer_id} not found"))?;

    let waypoints = layer
        .waypoints()
        .iter()
        .map(|w| WaypointDto {
            id: w.id().value(),
            name: w.name().to_owned(),
            lat: w.latitude(),
            lon: w.longitude(),
            symbol: w.symbol().map(str::to_owned),
            visible: w.visible(),
        })
        .collect();

    Ok(waypoints)
}

#[derive(serde::Serialize)]
pub struct SimplifiedSegmentDto {
    pub id: u64,
    pub original_count: usize,
    pub simplified_count: usize,
    pub kept_points: Vec<PointDetailDto>,
}

#[derive(serde::Serialize)]
pub struct SimplifiedPreviewDto {
    pub original_count: usize,
    pub simplified_count: usize,
    pub segments: Vec<SimplifiedSegmentDto>,
}

#[tauri::command]
pub fn get_simplified_preview(
    state: State<SharedState>,
    layer_id: u64,
    track_id: u64,
    tolerance: f64,
) -> Result<SimplifiedPreviewDto, String> {
    use crate::domain::{simplify_track_points, LayerId, TrackId};
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
pub fn reveal_bundle(state: State<SharedState>) -> Result<(), String> {
    lock_app_state(state.inner())?.reveal_active_bundle();
    Ok(())
}

// ── Track creation ────────────────────────────────────────────────────────────

#[tauri::command]
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
    use super::{PointDetailDto, SegmentDetailDto, TrackDetailDto};
    use crate::domain::{
        Track, TrackId, TrackPoint, TrackPointId, TrackSegment, TrackSegmentId,
    };

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

    #[test]
    fn get_tracks_geojson_skips_empty_tracks() {
        use crate::domain::{
            Track, TrackId, TrackPoint, TrackPointId, TrackSegment, TrackSegmentId,
        };

        // Empty track produces empty coords
        let empty = Track::new(TrackId::new(1), "Empty");
        let coords: Vec<serde_json::Value> = empty
            .segments()
            .iter()
            .flat_map(|seg| seg.points())
            .map(|pt| serde_json::json!([pt.longitude(), pt.latitude()]))
            .collect();
        assert!(coords.len() < 2);

        // Track with 2 points produces valid coords
        let mut valid = Track::new(TrackId::new(2), "Valid");
        let mut seg = TrackSegment::new(TrackSegmentId::new(1));
        seg.add_point(TrackPoint::new(TrackPointId::new(1), 55.0, 37.0));
        seg.add_point(TrackPoint::new(TrackPointId::new(2), 55.1, 37.1));
        valid.add_segment(seg);
        let coords2: Vec<serde_json::Value> = valid
            .segments()
            .iter()
            .flat_map(|seg| seg.points())
            .map(|pt| serde_json::json!([pt.longitude(), pt.latitude()]))
            .collect();
        assert!(coords2.len() >= 2);
    }
}
