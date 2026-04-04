use crate::domain::{
    LayerId, MapLayer, Project, ProjectLayerError, Track, TrackId, TrackLayer, Waypoint,
    WaypointId, WaypointLayer,
};
use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq)]
pub enum CommandError {
    ProjectLayer(ProjectLayerError),
}

impl From<ProjectLayerError> for CommandError {
    fn from(value: ProjectLayerError) -> Self {
        Self::ProjectLayer(value)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ProjectCommand {
    AddMapLayer {
        id: LayerId,
        name: String,
    },
    AddMapLayerWithSource {
        id: LayerId,
        name: String,
        source_path: PathBuf,
    },
    AddTrackLayer {
        id: LayerId,
        name: String,
    },
    AddWaypointLayer {
        id: LayerId,
        name: String,
    },
    AddTrack {
        layer_id: LayerId,
        track: Track,
    },
    AddWaypoint {
        layer_id: LayerId,
        waypoint: Waypoint,
    },
    MoveWaypoint {
        layer_id: LayerId,
        waypoint_id: WaypointId,
        latitude: f64,
        longitude: f64,
    },
    RenameTrack {
        layer_id: LayerId,
        track_id: TrackId,
        old_name: String,
        new_name: String,
    },
}

impl ProjectCommand {
    pub fn add_map_layer(id: LayerId, name: impl Into<String>) -> Self {
        Self::AddMapLayer {
            id,
            name: name.into(),
        }
    }

    pub fn add_track_layer(id: LayerId, name: impl Into<String>) -> Self {
        Self::AddTrackLayer {
            id,
            name: name.into(),
        }
    }

    pub fn add_map_layer_with_source(
        id: LayerId,
        name: impl Into<String>,
        source_path: impl Into<PathBuf>,
    ) -> Self {
        Self::AddMapLayerWithSource {
            id,
            name: name.into(),
            source_path: source_path.into(),
        }
    }

    pub fn add_waypoint_layer(id: LayerId, name: impl Into<String>) -> Self {
        Self::AddWaypointLayer {
            id,
            name: name.into(),
        }
    }

    pub fn add_track(layer_id: LayerId, track: Track) -> Self {
        Self::AddTrack { layer_id, track }
    }

    pub fn add_waypoint(layer_id: LayerId, waypoint: Waypoint) -> Self {
        Self::AddWaypoint { layer_id, waypoint }
    }

    pub fn move_waypoint(
        layer_id: LayerId,
        waypoint_id: WaypointId,
        latitude: f64,
        longitude: f64,
    ) -> Self {
        Self::MoveWaypoint {
            layer_id,
            waypoint_id,
            latitude,
            longitude,
        }
    }

    pub fn rename_track(
        layer_id: LayerId,
        track_id: TrackId,
        old_name: impl Into<String>,
        new_name: impl Into<String>,
    ) -> Self {
        Self::RenameTrack {
            layer_id,
            track_id,
            old_name: old_name.into(),
            new_name: new_name.into(),
        }
    }

    pub fn apply(&self, project: &mut Project) -> Result<(), CommandError> {
        match self {
            Self::AddMapLayer { id, name } => {
                project.add_map_layer(MapLayer::new(*id, name.clone()));
                Ok(())
            }
            Self::AddMapLayerWithSource {
                id,
                name,
                source_path,
            } => {
                project.add_map_layer(MapLayer::with_source_path(
                    *id,
                    name.clone(),
                    Some(source_path.clone()),
                ));
                Ok(())
            }
            Self::AddTrackLayer { id, name } => {
                project.add_track_layer(TrackLayer::new(*id, name.clone()));
                Ok(())
            }
            Self::AddWaypointLayer { id, name } => {
                project.add_waypoint_layer(WaypointLayer::new(*id, name.clone()));
                Ok(())
            }
            Self::AddTrack { layer_id, track } => {
                project.add_track_to_layer(*layer_id, track.clone())?;
                Ok(())
            }
            Self::AddWaypoint { layer_id, waypoint } => {
                project.add_waypoint_to_layer(*layer_id, waypoint.clone())?;
                Ok(())
            }
            Self::MoveWaypoint {
                layer_id,
                waypoint_id,
                latitude,
                longitude,
            } => {
                project.move_waypoint_in_layer(*layer_id, *waypoint_id, *latitude, *longitude)?;
                Ok(())
            }
            Self::RenameTrack {
                layer_id,
                track_id,
                new_name,
                ..
            } => {
                if let Some(track) = project.track_mut(*layer_id, *track_id) {
                    track.set_name(new_name.clone());
                }
                Ok(())
            }
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct CommandStack {
    undo_history: Vec<Project>,
    redo_history: Vec<Project>,
}

impl CommandStack {
    pub fn apply(
        &mut self,
        project: &mut Project,
        command: &ProjectCommand,
    ) -> Result<(), CommandError> {
        self.undo_history.push(project.clone());
        self.redo_history.clear();
        if let Err(error) = command.apply(project) {
            self.undo_history.pop();
            return Err(error);
        }

        Ok(())
    }

    pub fn can_undo(&self) -> bool {
        !self.undo_history.is_empty()
    }

    pub fn can_redo(&self) -> bool {
        !self.redo_history.is_empty()
    }

    pub fn undo(&mut self, project: &mut Project) -> bool {
        let Some(previous_project) = self.undo_history.pop() else {
            return false;
        };

        self.redo_history.push(project.clone());
        *project = previous_project;

        true
    }

    pub fn redo(&mut self, project: &mut Project) -> bool {
        let Some(next_project) = self.redo_history.pop() else {
            return false;
        };

        self.undo_history.push(project.clone());
        *project = next_project;

        true
    }
}

#[cfg(test)]
mod tests {
    use super::{CommandError, CommandStack, ProjectCommand};
    use crate::domain::{
        LayerId, Project, ProjectLayerError, Track, TrackId, Waypoint, WaypointId,
    };
    use std::path::Path;

    #[test]
    fn applying_add_track_command_updates_matching_layer() {
        let mut project = Project::untitled();
        let mut history = CommandStack::default();
        let layer_id = LayerId::new(20);
        history
            .apply(
                &mut project,
                &ProjectCommand::add_track_layer(layer_id, "Recorded tracks"),
            )
            .unwrap();

        history
            .apply(
                &mut project,
                &ProjectCommand::add_track(layer_id, Track::new(TrackId::new(1), "Morning route")),
            )
            .unwrap();

        assert_eq!(project.track_layers()[0].tracks().len(), 1);
    }

    #[test]
    fn applying_add_waypoint_command_requires_existing_layer() {
        let mut project = Project::untitled();
        let mut history = CommandStack::default();

        let error = history
            .apply(
                &mut project,
                &ProjectCommand::add_waypoint(
                    LayerId::new(30),
                    Waypoint::new(WaypointId::new(4), "Camp", 53.9, 27.5667),
                ),
            )
            .unwrap_err();

        assert_eq!(
            error,
            CommandError::ProjectLayer(ProjectLayerError::WaypointLayerUnavailable(LayerId::new(30)))
        );
        assert!(!history.can_undo());
    }

    #[test]
    fn applying_add_map_layer_with_source_stores_local_path() {
        let mut project = Project::untitled();
        let mut history = CommandStack::default();

        history
            .apply(
                &mut project,
                &ProjectCommand::add_map_layer_with_source(
                    LayerId::new(10),
                    "Cached map",
                    ".tmp/lizaalert-maps/demo/map.sqlitedb",
                ),
            )
            .unwrap();

        assert_eq!(project.map_layers().len(), 1);
        assert_eq!(
            project.map_layers()[0].source_path(),
            Some(Path::new(".tmp/lizaalert-maps/demo/map.sqlitedb"))
        );
    }

    #[test]
    fn applying_move_waypoint_command_updates_coordinates() {
        let mut project = Project::untitled();
        let mut history = CommandStack::default();
        let layer_id = LayerId::new(30);
        let waypoint_id = WaypointId::new(4);

        history
            .apply(
                &mut project,
                &ProjectCommand::add_waypoint_layer(layer_id, "Waypoints"),
            )
            .unwrap();
        history
            .apply(
                &mut project,
                &ProjectCommand::add_waypoint(
                    layer_id,
                    Waypoint::new(waypoint_id, "Camp", 53.9, 27.5667),
                ),
            )
            .unwrap();

        history
            .apply(
                &mut project,
                &ProjectCommand::move_waypoint(layer_id, waypoint_id, 54.1, 27.8),
            )
            .unwrap();

        let waypoint = &project.waypoint_layers()[0].waypoints()[0];
        assert_eq!(waypoint.latitude(), 54.1);
        assert_eq!(waypoint.longitude(), 27.8);
    }

    #[test]
    fn applying_move_waypoint_command_requires_existing_waypoint() {
        let mut project = Project::untitled();
        let mut history = CommandStack::default();
        let layer_id = LayerId::new(30);
        history
            .apply(
                &mut project,
                &ProjectCommand::add_waypoint_layer(layer_id, "Waypoints"),
            )
            .unwrap();

        let error = history
            .apply(
                &mut project,
                &ProjectCommand::move_waypoint(layer_id, WaypointId::new(99), 54.1, 27.8),
            )
            .unwrap_err();

        assert_eq!(
            error,
            CommandError::ProjectLayer(ProjectLayerError::WaypointNotFound(
                layer_id,
                WaypointId::new(99)
            ))
        );
    }
}
