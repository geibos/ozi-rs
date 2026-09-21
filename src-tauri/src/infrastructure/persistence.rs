use crate::domain::Project;
use std::fmt;
use std::path::{Path, PathBuf};

#[derive(Debug)]
pub enum PersistenceError {
    Io(std::io::Error),
    Json(serde_json::Error),
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct PersistedAppSession {
    pub last_project_path: Option<PathBuf>,
    pub active_map: Option<PersistedActiveMap>,
    /// Where downloaded bundles live. Absent in files written before the
    /// field existed, which is why it defaults instead of failing the read —
    /// a session that will not parse loses the restored project too.
    #[serde(default)]
    pub bundles_root: Option<PathBuf>,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct PersistedActiveMap {
    pub kind: String,
    pub project_name: String,
    pub package_name: String,
    pub remote_url: String,
    pub local_path: PathBuf,
    pub center_lat: f64,
    pub center_lon: f64,
    pub base_zoom: u8,
}

impl fmt::Display for PersistenceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(error) => write!(f, "file error: {error}"),
            Self::Json(error) => write!(f, "format error: {error}"),
        }
    }
}

impl std::error::Error for PersistenceError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io(error) => Some(error),
            Self::Json(error) => Some(error),
        }
    }
}

/// Atomic write-and-rename: contents go to a sibling `<name>.tmp` file,
/// are fsynced, and the temp file is renamed over the target. A failed
/// save therefore never truncates or corrupts an existing file at `path`
/// (same pattern as `download_to_path_async` in `infrastructure/lizaalert.rs`).
fn write_atomic(path: &Path, contents: &str) -> Result<(), PersistenceError> {
    fn write_and_rename(tmp_path: &Path, target: &Path, contents: &str) -> std::io::Result<()> {
        use std::io::Write;
        let mut file = std::fs::File::create(tmp_path)?;
        file.write_all(contents.as_bytes())?;
        file.sync_all()?;
        drop(file);
        std::fs::rename(tmp_path, target)
    }

    let mut tmp_name = path.as_os_str().to_owned();
    tmp_name.push(".tmp");
    let tmp_path = PathBuf::from(tmp_name);

    if let Err(error) = write_and_rename(&tmp_path, path, contents) {
        // Best-effort cleanup of the temp file; the write error is what matters.
        let _ = std::fs::remove_file(&tmp_path);
        return Err(PersistenceError::Io(error));
    }
    Ok(())
}

pub fn save_project(project: &Project, path: &Path) -> Result<(), PersistenceError> {
    let json = serde_json::to_string_pretty(project).map_err(PersistenceError::Json)?;
    write_atomic(path, &json)
}

pub fn load_project(path: &Path) -> Result<Project, PersistenceError> {
    let json = std::fs::read_to_string(path).map_err(PersistenceError::Io)?;
    let mut project: Project = serde_json::from_str(&json).map_err(PersistenceError::Json)?;
    // Normalize legacy projects to satisfy the default-layers invariant
    // declared by the `layers` capability. Existing layers are preserved
    // and only missing kinds get a default appended (in memory only —
    // the file on disk is not rewritten by load).
    project.ensure_default_layers();
    // Older builds could write two layers sharing one id. Every operation
    // addresses a layer by id, so the second of a colliding pair is
    // unreachable — renames, imports and deletes silently land on the first.
    // Repair on load (in memory; the file is rewritten only when the user
    // saves) and report it so the change is not silent.
    for (name, old_id, new_id) in project.deduplicate_layer_ids() {
        tracing::warn!(
            "layer {name:?} reused id {old_id}; renumbered to {new_id} so it can be addressed"
        );
    }
    Ok(project)
}

pub fn save_app_session(
    session: &PersistedAppSession,
    path: &Path,
) -> Result<(), PersistenceError> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(PersistenceError::Io)?;
    }
    let json = serde_json::to_string_pretty(session).map_err(PersistenceError::Json)?;
    write_atomic(path, &json)
}

pub fn load_app_session(path: &Path) -> Result<Option<PersistedAppSession>, PersistenceError> {
    match std::fs::read_to_string(path) {
        Ok(json) => serde_json::from_str(&json)
            .map(Some)
            .map_err(PersistenceError::Json),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(PersistenceError::Io(error)),
    }
}

/// Chooses where the app session file lives. Both candidate locations are
/// resolved by the caller (via Tauri's platform path resolver), so this
/// module never derives paths from environment variables.
///
/// An existing `preferred` file always wins. Otherwise an existing `legacy`
/// file (the pre-path-resolver macOS location) keeps being used, so
/// upgrading users do not lose their session. When neither file exists yet,
/// the `preferred` location is chosen for new sessions.
pub fn resolve_session_path(
    preferred: Option<PathBuf>,
    legacy: Option<PathBuf>,
) -> Option<PathBuf> {
    let existing_legacy = legacy.filter(|path| path.exists());
    match preferred {
        Some(path) if path.exists() => Some(path),
        Some(path) => Some(existing_legacy.unwrap_or(path)),
        None => existing_legacy,
    }
}

#[cfg(test)]
mod tests {
    use super::{load_project, resolve_session_path, save_project};
    use crate::domain::{LayerId, Project, TrackLayer, Waypoint, WaypointId, WaypointLayer};
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_path(suffix: &str) -> std::path::PathBuf {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time")
            .as_nanos();
        std::env::temp_dir()
            .join(format!(
                "ozi-rs-persistence-{}-{unique}.ozp",
                std::process::id()
            ))
            .with_extension(suffix)
    }

    #[test]
    fn project_survives_save_and_load_round_trip() {
        let mut project = Project::untitled();
        let layer_id = LayerId::new(30);
        project.add_waypoint_layer(WaypointLayer::new(layer_id, "Waypoints"));
        // Everything an operator sets on a waypoint, not a bare one: a round
        // trip over defaults proves only that defaults survive.
        let mut waypoint = Waypoint::new(WaypointId::new(1), "ШТАБ", 55.75, 37.61);
        waypoint.set_symbol(Some("flag".to_owned()));
        waypoint.set_color(Some([37, 99, 235, 255]));
        waypoint.set_visible(false);
        project.add_waypoint_to_layer(layer_id, waypoint).unwrap();

        let path = temp_path("ozp");
        save_project(&project, &path).expect("save");
        let loaded = load_project(&path).expect("load");

        assert_eq!(loaded, project);
    }

    /// A project written before waypoints could carry a colour.
    ///
    /// This is the case that costs a crew their work if it is wrong, and it
    /// rests on one `#[serde(default)]` that nothing else exercises — a field
    /// added without it turns every older `.ozp` into a load error.
    ///
    /// The file is this build's own output with the `color` key deleted,
    /// rather than JSON written by hand: hand-written, it tests my idea of the
    /// format, and the first attempt failed on a field I had not known was
    /// there.
    #[test]
    fn a_project_saved_before_waypoint_colours_still_loads() {
        let mut project = Project::untitled();
        let layer_id = LayerId::new(1);
        project.add_waypoint_layer(WaypointLayer::new(layer_id, "Waypoints"));
        let mut waypoint = Waypoint::new(WaypointId::new(1), "ШТАБ", 55.75, 37.61);
        waypoint.set_symbol(Some("flag".to_owned()));
        project.add_waypoint_to_layer(layer_id, waypoint).unwrap();

        let path = temp_path("ozp");
        save_project(&project, &path).expect("save");

        let written = std::fs::read_to_string(&path).expect("read");
        assert!(
            written.contains("\"color\""),
            "this build writes the field, so removing it is a meaningful older file"
        );
        let older: serde_json::Value = {
            let mut value: serde_json::Value = serde_json::from_str(&written).expect("parse");
            let waypoints = value["waypoint_layers"][0]["waypoints"]
                .as_array_mut()
                .expect("the waypoints array");
            for w in waypoints {
                w.as_object_mut()
                    .expect("a waypoint object")
                    .remove("color");
            }
            value
        };
        std::fs::write(
            &path,
            serde_json::to_string_pretty(&older).expect("serialize"),
        )
        .expect("write the older file");

        let loaded = load_project(&path).expect("an older project still loads");
        let waypoint = &loaded.waypoint_layers()[0].waypoints()[0];
        assert_eq!(waypoint.name(), "ШТАБ");
        assert_eq!(waypoint.symbol(), Some("flag"));
        assert_eq!(
            waypoint.color(),
            None,
            "no colour, which is what the file meant — not a default colour"
        );
        assert!(waypoint.visible(), "and visible, as it was");
    }

    #[test]
    fn saved_file_is_valid_json() {
        let project = Project::untitled();
        let path = temp_path("ozp");
        save_project(&project, &path).expect("save");

        let contents = std::fs::read_to_string(&path).expect("read");
        let parsed: serde_json::Value = serde_json::from_str(&contents).expect("valid json");
        assert_eq!(parsed["name"], "Untitled Project");
    }

    #[test]
    fn load_project_fails_gracefully_on_missing_file() {
        let result = load_project(std::path::Path::new("/nonexistent/path/project.ozp"));

        assert!(result.is_err());
    }

    #[test]
    fn empty_project_round_trips_all_layer_types() {
        // `Project::untitled()` already contains one default track layer and
        // one default waypoint layer (default-layers invariant). Adding extra
        // layers with non-conflicting IDs verifies they survive the round
        // trip alongside the defaults.
        let mut project = Project::untitled();
        project.add_track_layer(TrackLayer::new(LayerId::new(20), "Extra Tracks"));
        project.add_waypoint_layer(WaypointLayer::new(LayerId::new(30), "Extra Waypoints"));

        let path = temp_path("ozp");
        save_project(&project, &path).expect("save");
        let loaded = load_project(&path).expect("load");

        assert_eq!(loaded.track_layers().len(), 2);
        assert_eq!(loaded.waypoint_layers().len(), 2);
        assert!(
            loaded
                .track_layers()
                .iter()
                .any(|l| l.id() == LayerId::new(20) && l.name() == "Extra Tracks")
        );
        assert!(
            loaded
                .waypoint_layers()
                .iter()
                .any(|l| l.id() == LayerId::new(30) && l.name() == "Extra Waypoints")
        );
    }

    fn write_raw_ozp(path: &std::path::Path, json: &str) {
        std::fs::write(path, json).expect("write raw ozp");
    }

    /// Unique per-test directory under the system temp dir.
    fn temp_dir(label: &str) -> std::path::PathBuf {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time")
            .as_nanos();
        let dir = std::env::temp_dir().join(format!(
            "ozi-rs-persistence-{label}-{}-{unique}",
            std::process::id()
        ));
        std::fs::create_dir_all(&dir).expect("create test dir");
        dir
    }

    fn tmp_sibling(path: &std::path::Path) -> std::path::PathBuf {
        let mut name = path.as_os_str().to_owned();
        name.push(".tmp");
        std::path::PathBuf::from(name)
    }

    #[cfg(unix)]
    #[test]
    fn failed_save_keeps_existing_file_intact() {
        use std::os::unix::fs::PermissionsExt;

        // Failure simulation: the parent directory is made read-only
        // (0o555), so the sibling `.tmp` file cannot be created. A
        // truncate-in-place implementation is NOT affected by this —
        // overwriting an existing file needs no directory write
        // permission — so it would silently replace the original.
        let dir = temp_dir("readonly-parent");
        let path = dir.join("project.ozp");

        let mut original = Project::untitled();
        let layer_id = LayerId::new(30);
        original.add_waypoint_layer(WaypointLayer::new(layer_id, "Waypoints"));
        original
            .add_waypoint_to_layer(
                layer_id,
                Waypoint::new(WaypointId::new(1), "Campsite", 55.75, 37.61),
            )
            .unwrap();
        save_project(&original, &path).expect("initial save");

        let different = Project::untitled();
        std::fs::set_permissions(&dir, std::fs::Permissions::from_mode(0o555))
            .expect("make dir read-only");
        let result = save_project(&different, &path);
        std::fs::set_permissions(&dir, std::fs::Permissions::from_mode(0o755))
            .expect("restore dir permissions");

        assert!(
            result.is_err(),
            "save into a read-only directory must fail instead of \
             truncating the target in place"
        );
        let on_disk = load_project(&path).expect("original must still load");
        assert_eq!(on_disk, original, "failed save must not touch the target");
        assert!(
            !tmp_sibling(&path).exists(),
            "no stale temp file after failure"
        );
    }

    #[test]
    fn successful_save_leaves_no_tmp_sibling() {
        let dir = temp_dir("no-tmp-sibling");
        let path = dir.join("project.ozp");

        save_project(&Project::untitled(), &path).expect("save");

        assert!(path.exists());
        assert!(
            !tmp_sibling(&path).exists(),
            "temp file must be renamed away on success"
        );
    }

    #[test]
    fn failed_rename_cleans_up_tmp_sibling() {
        // A directory at the target path makes the final rename fail
        // deterministically; the temp file must not be left behind.
        let dir = temp_dir("rename-blocked");
        let path = dir.join("blocked.ozp");
        std::fs::create_dir_all(&path).expect("create blocking dir");

        let result = save_project(&Project::untitled(), &path);

        assert!(result.is_err(), "saving over a directory must fail");
        assert!(
            !tmp_sibling(&path).exists(),
            "no stale temp file after failure"
        );
    }

    fn touch(path: &std::path::Path) {
        let parent = path.parent().expect("session file has a parent dir");
        std::fs::create_dir_all(parent).expect("create parent dir");
        std::fs::write(path, "{}").expect("write session placeholder");
    }

    #[test]
    fn resolve_session_path_prefers_existing_preferred_file() {
        let dir = temp_dir("resolve-preferred-wins");
        let preferred = dir.join("new").join("session.json");
        let legacy = dir.join("legacy").join("session.json");
        touch(&preferred);
        touch(&legacy);

        let resolved = resolve_session_path(Some(preferred.clone()), Some(legacy));

        assert_eq!(resolved, Some(preferred));
    }

    #[test]
    fn resolve_session_path_falls_back_to_existing_legacy_file() {
        let dir = temp_dir("resolve-legacy-fallback");
        let preferred = dir.join("new").join("session.json");
        let legacy = dir.join("legacy").join("session.json");
        touch(&legacy);

        let resolved = resolve_session_path(Some(preferred), Some(legacy.clone()));

        assert_eq!(resolved, Some(legacy));
    }

    #[test]
    fn resolve_session_path_uses_preferred_when_neither_file_exists() {
        let dir = temp_dir("resolve-fresh-install");
        let preferred = dir.join("new").join("session.json");
        let legacy = dir.join("legacy").join("session.json");

        let resolved = resolve_session_path(Some(preferred.clone()), Some(legacy));

        assert_eq!(resolved, Some(preferred));
    }

    #[test]
    fn resolve_session_path_without_preferred_uses_existing_legacy() {
        let dir = temp_dir("resolve-no-preferred");
        let legacy = dir.join("legacy").join("session.json");
        touch(&legacy);

        let resolved = resolve_session_path(None, Some(legacy.clone()));

        assert_eq!(resolved, Some(legacy));
    }

    #[test]
    fn resolve_session_path_returns_none_when_nothing_is_available() {
        let dir = temp_dir("resolve-nothing");
        let legacy = dir.join("legacy").join("session.json");

        assert_eq!(resolve_session_path(None, Some(legacy)), None);
        assert_eq!(resolve_session_path(None, None), None);
    }

    #[test]
    fn load_legacy_ozp_with_no_layers_appends_defaults() {
        // Legacy file format: no track or waypoint layers at all.
        let raw = r#"{
            "id": 1,
            "name": "Legacy Project",
            "map_layers": [],
            "track_layers": [],
            "waypoint_layers": []
        }"#;

        let path = temp_path("ozp");
        write_raw_ozp(&path, raw);

        let loaded = load_project(&path).expect("load");

        assert_eq!(loaded.track_layers().len(), 1);
        assert_eq!(loaded.track_layers()[0].name(), "Tracks");
        assert_eq!(loaded.waypoint_layers().len(), 1);
        assert_eq!(loaded.waypoint_layers()[0].name(), "Waypoints");
    }

    /// The smallest project an early build could have written, with a track
    /// and a waypoint in it.
    ///
    /// The whole format rests on `#[serde(default)]` in the right places: a
    /// field added to any of these structs without one turns every project a
    /// crew has ever saved into a load error, and the only place that shows up
    /// is a load. The existing legacy test covers the project shell with no
    /// layers; this covers what is inside them.
    ///
    /// If this fails after a field is added, the field wants a default — or,
    /// if it truly cannot have one, the format wants a version and a migration,
    /// which it does not have (tracked as CJ-8).
    #[test]
    fn a_project_from_an_early_build_still_loads_with_its_contents() {
        let raw = r#"{
            "id": 1,
            "name": "Ранний поиск",
            "map_layers": [],
            "track_layers": [
                {
                    "id": 1,
                    "name": "Tracks",
                    "tracks": [
                        {
                            "id": 1,
                            "name": "20260708_Ветер",
                            "segments": [
                                {
                                    "id": 1,
                                    "points": [
                                        { "id": 1, "latitude": 59.95, "longitude": 31.59 },
                                        { "id": 2, "latitude": 59.96, "longitude": 31.60 }
                                    ]
                                }
                            ]
                        }
                    ]
                }
            ],
            "waypoint_layers": [
                {
                    "id": 1,
                    "name": "Waypoints",
                    "waypoints": [
                        { "id": 1, "name": "ШТАБ", "latitude": 59.95, "longitude": 31.59 }
                    ]
                }
            ]
        }"#;

        let path = temp_path("ozp");
        write_raw_ozp(&path, raw);

        let loaded = load_project(&path).expect("a project from an early build still loads");

        let track = &loaded.track_layers()[0].tracks()[0];
        assert_eq!(track.name(), "20260708_Ветер");
        assert_eq!(track.segments()[0].points().len(), 2);
        assert!(
            track.style().visible,
            "a track with no style recorded is visible, which is what its absence meant"
        );

        let waypoint = &loaded.waypoint_layers()[0].waypoints()[0];
        assert_eq!(waypoint.name(), "ШТАБ");
        assert_eq!(waypoint.symbol(), None);
        assert_eq!(waypoint.color(), None);
        assert!(waypoint.visible());
    }

    #[test]
    fn load_ozp_with_existing_layers_does_not_append_defaults() {
        // A project with one track layer and two waypoint layers (none of
        // them at id=1) must not gain any extra "default" layers on load.
        let raw = r#"{
            "id": 1,
            "name": "Already-Normalized",
            "map_layers": [],
            "track_layers": [
                {"id": 11, "name": "Recorded", "tracks": []}
            ],
            "waypoint_layers": [
                {"id": 21, "name": "Camps", "waypoints": []},
                {"id": 22, "name": "Hazards", "waypoints": []}
            ]
        }"#;

        let path = temp_path("ozp");
        write_raw_ozp(&path, raw);

        let loaded = load_project(&path).expect("load");

        assert_eq!(loaded.track_layers().len(), 1);
        assert_eq!(loaded.track_layers()[0].name(), "Recorded");
        assert_eq!(loaded.waypoint_layers().len(), 2);
        assert!(loaded.waypoint_layers().iter().any(|l| l.name() == "Camps"));
        assert!(
            loaded
                .waypoint_layers()
                .iter()
                .any(|l| l.name() == "Hazards")
        );
    }

    #[test]
    fn legacy_load_then_save_persists_normalized_layers() {
        // Regression: a legacy file (no layers) is loaded → in-memory state
        // gains defaults via the invariant → re-saved file now contains the
        // defaults explicitly → a fresh load returns the same project.
        let legacy_raw = r#"{
            "id": 1,
            "name": "Legacy Project",
            "map_layers": [],
            "track_layers": [],
            "waypoint_layers": []
        }"#;
        let legacy_path = temp_path("legacy.ozp");
        write_raw_ozp(&legacy_path, legacy_raw);

        let loaded_once = load_project(&legacy_path).expect("load legacy");

        let normalized_path = temp_path("normalized.ozp");
        save_project(&loaded_once, &normalized_path).expect("save normalized");

        let loaded_twice = load_project(&normalized_path).expect("re-load normalized");

        assert_eq!(loaded_once, loaded_twice);
        assert_eq!(loaded_twice.track_layers().len(), 1);
        assert_eq!(loaded_twice.waypoint_layers().len(), 1);
    }
}
