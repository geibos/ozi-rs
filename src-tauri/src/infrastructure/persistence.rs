use crate::domain::Project;
use std::fmt;
use std::path::Path;

#[derive(Debug)]
pub enum PersistenceError {
    Io(std::io::Error),
    Json(serde_json::Error),
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

pub fn save_project(project: &Project, path: &Path) -> Result<(), PersistenceError> {
    let json = serde_json::to_string_pretty(project).map_err(PersistenceError::Json)?;
    std::fs::write(path, json).map_err(PersistenceError::Io)
}

pub fn load_project(path: &Path) -> Result<Project, PersistenceError> {
    let json = std::fs::read_to_string(path).map_err(PersistenceError::Io)?;
    serde_json::from_str(&json).map_err(PersistenceError::Json)
}

#[cfg(test)]
mod tests {
    use super::{load_project, save_project};
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
        project
            .add_waypoint_to_layer(
                layer_id,
                Waypoint::new(WaypointId::new(1), "Campsite", 55.75, 37.61),
            )
            .unwrap();

        let path = temp_path("ozp");
        save_project(&project, &path).expect("save");
        let loaded = load_project(&path).expect("load");

        assert_eq!(loaded, project);
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
        let mut project = Project::untitled();
        project.add_track_layer(TrackLayer::new(LayerId::new(1), "Tracks"));
        project.add_waypoint_layer(WaypointLayer::new(LayerId::new(2), "Waypoints"));

        let path = temp_path("ozp");
        save_project(&project, &path).expect("save");
        let loaded = load_project(&path).expect("load");

        assert_eq!(loaded.track_layers().len(), 1);
        assert_eq!(loaded.waypoint_layers().len(), 1);
    }
}
