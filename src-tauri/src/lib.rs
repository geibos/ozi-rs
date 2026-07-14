mod application;
mod commands;
mod domain;
mod infrastructure;

use commands::{DownloadRegistry, SharedDownloads, SharedState};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use tauri::Manager;

/// Resolves the session-file and bundles-root locations through Tauri's
/// path resolver (portable across macOS, Windows and Linux) and injects
/// them into `AppState`, so no layer below this derives paths from
/// environment variables.
fn build_app_state(app: &tauri::AppHandle) -> application::AppState {
    let preferred_session = match app.path().app_data_dir() {
        Ok(dir) => Some(dir.join("session.json")),
        Err(error) => {
            tracing::warn!("app data dir unavailable, session may not persist: {error}");
            None
        }
    };
    let session_path = infrastructure::persistence::resolve_session_path(
        preferred_session,
        legacy_session_path(app),
    );

    let bundles_root = match app.path().document_dir() {
        Ok(documents) => documents.join("LizaAlert Maps"),
        Err(error) => {
            tracing::warn!("documents dir unavailable, using relative bundles dir: {error}");
            PathBuf::from("bundles")
        }
    };

    application::AppState::new_with_paths(session_path, bundles_root)
}

/// Releases before the path-resolver switch stored the session under a
/// literal `ozi-rs` directory in `~/Library/Application Support` (macOS was
/// the only working target). That file keeps being used until a session
/// exists at the new `app_data_dir()` location, so upgrading users do not
/// lose their session.
#[cfg(target_os = "macos")]
fn legacy_session_path(app: &tauri::AppHandle) -> Option<PathBuf> {
    let home = app.path().home_dir().ok()?;
    Some(
        home.join("Library")
            .join("Application Support")
            .join("ozi-rs")
            .join("session.json"),
    )
}

#[cfg(not(target_os = "macos"))]
fn legacy_session_path(_app: &tauri::AppHandle) -> Option<PathBuf> {
    None
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    let downloads: SharedDownloads = Arc::new(DownloadRegistry::default());

    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_shell::init())
        .setup(|app| {
            let state: SharedState = Arc::new(Mutex::new(build_app_state(app.handle())));
            app.manage(state);
            Ok(())
        })
        .manage(downloads)
        .invoke_handler(tauri::generate_handler![
            commands::get_app_state,
            commands::get_tracks_geojson,
            commands::load_projects,
            commands::load_project,
            commands::cancel_download,
            commands::open_selected_map,
            commands::open_local_bundle,
            commands::set_bundles_root,
            commands::save_project,
            commands::load_project_file,
            commands::import_gpx,
            commands::import_plt,
            commands::export_gpx,
            commands::get_track_export_default_path,
            commands::undo,
            commands::redo,
            commands::rename_track,
            commands::set_track_color,
            commands::toggle_track_visible,
            commands::toggle_waypoint_visible,
            commands::move_track_point,
            commands::delete_track_point,
            commands::insert_track_point,
            commands::split_segment,
            commands::join_segments,
            commands::delete_track,
            commands::add_waypoint,
            commands::move_waypoint,
            commands::delete_waypoint,
            commands::rename_waypoint,
            commands::set_waypoint_symbol,
            commands::simplify_track,
            commands::set_track_line_width,
            commands::get_track_detail,
            commands::get_waypoints,
            commands::get_simplified_preview,
            commands::reveal_bundle,
            commands::export_track_plt,
            commands::export_wpt_waypoints,
            commands::get_wpt_export_default_path,
            commands::create_empty_track,
            commands::tiles::get_sqlite_tile,
            commands::tiles::get_ozi_tile,
            commands::tiles::get_ozi_tile_projected,
            commands::tiles::get_ozi_metadata,
        ])
        .run(tauri::generate_context!())
        .expect("error while running ozi-rs");
}
