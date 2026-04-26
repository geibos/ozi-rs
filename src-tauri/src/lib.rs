mod application;
mod commands;
mod domain;
mod infrastructure;

use commands::SharedState;
use std::sync::{Arc, Mutex};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    let state: SharedState = Arc::new(Mutex::new(application::AppState::new_with_session_path(
        infrastructure::persistence::default_app_session_path(),
    )));

    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_shell::init())
        .manage(state)
        .invoke_handler(tauri::generate_handler![
            commands::get_app_state,
            commands::get_tracks_geojson,
            commands::load_projects,
            commands::load_project,
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
            commands::create_empty_track,
            commands::tiles::get_sqlite_tile,
            commands::tiles::get_ozi_tile,
            commands::tiles::get_ozi_tile_projected,
            commands::tiles::get_ozi_metadata,
        ])
        .run(tauri::generate_context!())
        .expect("error while running ozi-rs");
}
