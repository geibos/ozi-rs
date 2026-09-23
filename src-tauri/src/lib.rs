mod application;
mod commands;
mod domain;
// Test-only: the fixture generator is compiled by `cargo test` (and so by
// `just fixtures`), never into the shipped binary.
#[cfg(test)]
mod fixtures;
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

/// The single registry of IPC commands, shared by the runtime invoke handler
/// and the TypeScript bindings export (`src/lib/bindings.ts`). Adding a
/// command here is the ONLY registration step; the generated bindings make
/// frontend drift a compile error instead of a runtime surprise.
fn specta_builder() -> tauri_specta::Builder {
    tauri_specta::Builder::<tauri::Wry>::new().commands(tauri_specta::collect_commands![
        commands::get_app_state,
        commands::get_tracks_geojson,
        commands::list_tracks,
        commands::load_projects,
        commands::load_project,
        commands::preview_project,
        commands::cancel_download,
        commands::cancel_project_listing,
        commands::open_selected_map,
        commands::open_local_bundle,
        commands::set_bundles_root,
        commands::save_project,
        commands::new_project,
        commands::load_project_file,
        commands::import_gpx,
        commands::import_plt,
        commands::import_wpt,
        commands::import_tracks_directory,
        commands::export_gpx,
        commands::export_all_tracks_gpx,
        commands::get_track_export_default_path,
        commands::undo,
        commands::redo,
        commands::rename_track,
        commands::create_track_layer,
        commands::create_waypoint_layer,
        commands::rename_track_layer,
        commands::rename_waypoint_layer,
        commands::delete_track_layer,
        commands::delete_waypoint_layer,
        commands::set_track_color,
        commands::cancel_drawing,
        commands::toggle_track_visible,
        commands::set_all_tracks_visible,
        commands::show_only_track,
        commands::set_all_waypoints_visible,
        commands::show_only_waypoint,
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
        commands::set_waypoint_color,
        commands::simplify_track,
        commands::sort_track_points,
        commands::crop_track_to_extent,
        commands::trim_track_at_point,
        commands::crop_track_to_time,
        commands::set_track_line_width,
        commands::get_track_detail,
        commands::get_waypoints,
        commands::get_simplified_preview,
        commands::reveal_bundle,
        commands::export_track_plt,
        commands::export_wpt_waypoints,
        commands::export_gpx_waypoints,
        commands::get_waypoints_export_default_path,
        commands::get_wpt_export_default_path,
        commands::create_empty_track,
        commands::tiles::get_ozi_metadata,
    ])
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
        .invoke_handler({
            // Tile commands return raw bytes (`tauri::ipc::Response`), which
            // specta cannot type — they stay on a plain handler, consumed by
            // the MapLibre protocol layer rather than typed app code.
            let typed = specta_builder().invoke_handler();
            let tiles: fn(tauri::ipc::Invoke<tauri::Wry>) -> bool = tauri::generate_handler![
                commands::tiles::get_sqlite_tile,
                commands::tiles::get_ozi_tile,
                commands::tiles::get_ozi_tile_projected,
            ];
            move |invoke: tauri::ipc::Invoke<tauri::Wry>| match invoke.message.command() {
                "get_sqlite_tile" | "get_ozi_tile" | "get_ozi_tile_projected" => tiles(invoke),
                _ => typed(invoke),
            }
        })
        .run(tauri::generate_context!())
        .expect("error while running ozi-rs");
}

#[cfg(test)]
mod bindings_tests {
    use super::specta_builder;

    /// Regenerates `src/lib/bindings.ts` and fails when the committed file
    /// was stale — the same role a codegen-diff CI check would play, but it
    /// runs with plain `cargo test`. On failure the file HAS been rewritten:
    /// review the diff and commit it.
    #[test]
    fn typescript_bindings_are_up_to_date() {
        let bindings_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("workspace root")
            .join("src/lib/bindings.ts");
        let before = std::fs::read_to_string(&bindings_path).unwrap_or_default();

        specta_builder()
            .export(
                specta_typescript::Typescript::default()
                    // The generated file carries unused event plumbing and
                    // `any`-typed glue when no events are registered; exempt
                    // it from the strict project tsconfig and eslint (its
                    // exported types stay checked at every use site).
                    .header("/* eslint-disable */\n// @ts-nocheck")
                    // Wire format is serde_json: u64 IDs travel as JSON
                    // numbers. Frontend stores use bigint by convention and
                    // convert explicitly at the api.ts boundary.
                    .bigint(specta_typescript::BigIntExportBehavior::Number),
                &bindings_path,
            )
            .expect("failed to export typescript bindings");

        let after = std::fs::read_to_string(&bindings_path).expect("bindings written");
        assert_eq!(
            before, after,
            "src/lib/bindings.ts was stale; it has just been regenerated — \
             review the diff and commit the updated file",
        );
    }
}
