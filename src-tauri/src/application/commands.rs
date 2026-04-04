#![allow(dead_code)]

use crate::domain::{
    LayerId, MapLayer, Project, ProjectLayerError, Track, TrackId, TrackLayer, TrackPoint,
    TrackPointId, TrackSegmentId, Waypoint, WaypointId, WaypointLayer,
};
use std::path::PathBuf;

const MAX_STACK_DEPTH: usize = 100;

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
    MoveTrackPoint {
        layer_id: LayerId,
        track_id: TrackId,
        segment_id: TrackSegmentId,
        point_id: TrackPointId,
        lat: f64,
        lon: f64,
        old_lat: f64,
        old_lon: f64,
    },
    DeleteTrackPoint {
        layer_id: LayerId,
        track_id: TrackId,
        segment_id: TrackSegmentId,
        point_id: TrackPointId,
        removed_index: usize,
        removed_point: TrackPoint,
    },
    InsertTrackPoint {
        layer_id: LayerId,
        track_id: TrackId,
        segment_id: TrackSegmentId,
        index: usize,
        point: TrackPoint,
    },
    RenameTrack {
        layer_id: LayerId,
        track_id: TrackId,
        old_name: String,
        new_name: String,
    },
    RemoveMapLayer {
        layer: MapLayer,
    },
    RemoveTrackLayer {
        layer: TrackLayer,
    },
    RemoveWaypointLayer {
        layer: WaypointLayer,
    },
    RemoveTrack {
        layer_id: LayerId,
        track: Track,
    },
    RemoveWaypoint {
        layer_id: LayerId,
        waypoint: Waypoint,
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

    pub fn move_track_point(
        layer_id: LayerId,
        track_id: TrackId,
        segment_id: TrackSegmentId,
        point_id: TrackPointId,
        lat: f64,
        lon: f64,
        old_lat: f64,
        old_lon: f64,
    ) -> Self {
        Self::MoveTrackPoint {
            layer_id,
            track_id,
            segment_id,
            point_id,
            lat,
            lon,
            old_lat,
            old_lon,
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

    pub fn delete_track_point(
        layer_id: LayerId,
        track_id: TrackId,
        segment_id: TrackSegmentId,
        point_id: TrackPointId,
    ) -> Self {
        Self::DeleteTrackPoint {
            layer_id,
            track_id,
            segment_id,
            point_id,
            removed_index: 0,
            removed_point: TrackPoint::new(TrackPointId::new(0), 0.0, 0.0),
        }
    }

    pub fn insert_track_point(
        layer_id: LayerId,
        track_id: TrackId,
        segment_id: TrackSegmentId,
        index: usize,
        point: TrackPoint,
    ) -> Self {
        Self::InsertTrackPoint {
            layer_id,
            track_id,
            segment_id,
            index,
            point,
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
            Self::MoveTrackPoint {
                layer_id,
                track_id,
                segment_id,
                point_id,
                lat,
                lon,
                ..
            } => {
                project.move_point_in_layer(
                    layer_id.value(),
                    track_id.value(),
                    segment_id.value(),
                    point_id.value(),
                    *lat,
                    *lon,
                )?;
                Ok(())
            }
            Self::DeleteTrackPoint {
                layer_id,
                track_id,
                segment_id,
                point_id,
                ..
            } => {
                project.remove_point_from_layer(
                    layer_id.value(),
                    track_id.value(),
                    segment_id.value(),
                    point_id.value(),
                )?;
                Ok(())
            }
            Self::InsertTrackPoint {
                layer_id,
                track_id,
                segment_id,
                index,
                point,
            } => {
                project.insert_point_in_layer(
                    layer_id.value(),
                    track_id.value(),
                    segment_id.value(),
                    *index,
                    point.clone(),
                )?;
                Ok(())
            }
            Self::RenameTrack {
                layer_id,
                track_id,
                new_name,
                ..
            } => {
                if let Ok(track) = project.track_mut(layer_id.value(), track_id.value()) {
                    track.set_name(new_name.clone());
                }
                Ok(())
            }
            Self::RemoveMapLayer { layer } => {
                project.remove_map_layer(layer.id());
                Ok(())
            }
            Self::RemoveTrackLayer { layer } => {
                project.remove_track_layer(layer.id());
                Ok(())
            }
            Self::RemoveWaypointLayer { layer } => {
                project.remove_waypoint_layer(layer.id());
                Ok(())
            }
            Self::RemoveTrack { layer_id, track } => {
                project.remove_track_from_layer(*layer_id, track.id())?;
                Ok(())
            }
            Self::RemoveWaypoint { layer_id, waypoint } => {
                project.remove_waypoint_from_layer(*layer_id, waypoint.id())?;
                Ok(())
            }
        }
    }

    pub fn reverse(&self, project: &Project) -> ProjectCommand {
        match self {
            Self::AddMapLayer { id, name } => Self::RemoveMapLayer {
                layer: MapLayer::new(*id, name.clone()),
            },
            Self::AddMapLayerWithSource {
                id,
                name,
                source_path,
            } => Self::RemoveMapLayer {
                layer: MapLayer::with_source_path(*id, name.clone(), Some(source_path.clone())),
            },
            Self::AddTrackLayer { id, name } => Self::RemoveTrackLayer {
                layer: TrackLayer::new(*id, name.clone()),
            },
            Self::AddWaypointLayer { id, name } => Self::RemoveWaypointLayer {
                layer: WaypointLayer::new(*id, name.clone()),
            },
            Self::AddTrack { layer_id, track } => Self::RemoveTrack {
                layer_id: *layer_id,
                track: track.clone(),
            },
            Self::AddWaypoint { layer_id, waypoint } => Self::RemoveWaypoint {
                layer_id: *layer_id,
                waypoint: waypoint.clone(),
            },
            Self::MoveWaypoint {
                layer_id,
                waypoint_id,
                latitude,
                longitude,
            } => {
                let (previous_latitude, previous_longitude) = project
                    .waypoint_layers()
                    .iter()
                    .find(|layer| layer.id() == *layer_id)
                    .and_then(|layer| {
                        layer
                            .waypoints()
                            .iter()
                            .find(|waypoint| waypoint.id() == *waypoint_id)
                            .map(|waypoint| (waypoint.latitude(), waypoint.longitude()))
                    })
                    .unwrap_or((*latitude, *longitude));

                Self::MoveWaypoint {
                    layer_id: *layer_id,
                    waypoint_id: *waypoint_id,
                    latitude: previous_latitude,
                    longitude: previous_longitude,
                }
            }
            Self::MoveTrackPoint {
                layer_id,
                track_id,
                segment_id,
                point_id,
                lat,
                lon,
                old_lat,
                old_lon,
            } => Self::MoveTrackPoint {
                layer_id: *layer_id,
                track_id: *track_id,
                segment_id: *segment_id,
                point_id: *point_id,
                lat: *old_lat,
                lon: *old_lon,
                old_lat: *lat,
                old_lon: *lon,
            },
            Self::DeleteTrackPoint {
                layer_id,
                track_id,
                segment_id,
                point_id,
                removed_index,
                removed_point,
            } => {
                let found = project
                    .track_layers()
                    .iter()
                    .find(|layer| layer.id() == *layer_id)
                    .and_then(|layer| {
                        layer
                            .tracks()
                            .iter()
                            .find(|track| track.id() == *track_id)
                            .and_then(|track| {
                                track
                                    .segments()
                                    .iter()
                                    .find(|segment| segment.id() == *segment_id)
                                    .and_then(|segment| {
                                        segment
                                            .points()
                                            .iter()
                                            .enumerate()
                                            .find(|(_, point)| point.id() == *point_id)
                                            .map(|(index, point)| (index, point.clone()))
                                    })
                            })
                    });

                let (index, point) = found
                    .unwrap_or((*removed_index, removed_point.clone()));

                Self::InsertTrackPoint {
                    layer_id: *layer_id,
                    track_id: *track_id,
                    segment_id: *segment_id,
                    index,
                    point,
                }
            }
            Self::InsertTrackPoint {
                layer_id,
                track_id,
                segment_id,
                point,
                ..
            } => Self::DeleteTrackPoint {
                layer_id: *layer_id,
                track_id: *track_id,
                segment_id: *segment_id,
                point_id: point.id(),
                removed_index: 0,
                removed_point: point.clone(),
            },
            Self::RenameTrack {
                layer_id,
                track_id,
                old_name,
                new_name,
            } => {
                let previous_name = project
                    .track_layers()
                    .iter()
                    .find(|layer| layer.id() == *layer_id)
                    .and_then(|layer| {
                        layer
                            .tracks()
                            .iter()
                            .find(|track| track.id() == *track_id)
                            .map(|track| track.name().to_owned())
                    })
                    .unwrap_or_else(|| old_name.clone());

                Self::RenameTrack {
                    layer_id: *layer_id,
                    track_id: *track_id,
                    old_name: new_name.clone(),
                    new_name: previous_name,
                }
            }
            Self::RemoveMapLayer { layer } => {
                if let Some(source_path) = layer.source_path() {
                    Self::AddMapLayerWithSource {
                        id: layer.id(),
                        name: layer.name().to_owned(),
                        source_path: source_path.to_path_buf(),
                    }
                } else {
                    Self::AddMapLayer {
                        id: layer.id(),
                        name: layer.name().to_owned(),
                    }
                }
            }
            Self::RemoveTrackLayer { layer } => Self::AddTrackLayer {
                id: layer.id(),
                name: layer.name().to_owned(),
            },
            Self::RemoveWaypointLayer { layer } => Self::AddWaypointLayer {
                id: layer.id(),
                name: layer.name().to_owned(),
            },
            Self::RemoveTrack { layer_id, track } => Self::AddTrack {
                layer_id: *layer_id,
                track: track.clone(),
            },
            Self::RemoveWaypoint { layer_id, waypoint } => Self::AddWaypoint {
                layer_id: *layer_id,
                waypoint: waypoint.clone(),
            },
        }
    }

    fn targets_same_entity(&self, other: &ProjectCommand) -> bool {
        match (self, other) {
            (Self::AddMapLayer { id: left_id, .. }, Self::AddMapLayer { id: right_id, .. }) => {
                left_id == right_id
            }
            (
                Self::AddMapLayerWithSource { id: left_id, .. },
                Self::AddMapLayerWithSource { id: right_id, .. },
            ) => left_id == right_id,
            (Self::AddTrackLayer { id: left_id, .. }, Self::AddTrackLayer { id: right_id, .. }) => {
                left_id == right_id
            }
            (
                Self::AddWaypointLayer { id: left_id, .. },
                Self::AddWaypointLayer { id: right_id, .. },
            ) => left_id == right_id,
            (
                Self::AddTrack {
                    layer_id: left_layer_id,
                    track: left_track,
                },
                Self::AddTrack {
                    layer_id: right_layer_id,
                    track: right_track,
                },
            ) => left_layer_id == right_layer_id && left_track.id() == right_track.id(),
            (
                Self::AddWaypoint {
                    layer_id: left_layer_id,
                    waypoint: left_waypoint,
                },
                Self::AddWaypoint {
                    layer_id: right_layer_id,
                    waypoint: right_waypoint,
                },
            ) => left_layer_id == right_layer_id && left_waypoint.id() == right_waypoint.id(),
            (
                Self::MoveWaypoint {
                    layer_id: left_layer_id,
                    waypoint_id: left_waypoint_id,
                    ..
                },
                Self::MoveWaypoint {
                    layer_id: right_layer_id,
                    waypoint_id: right_waypoint_id,
                    ..
                },
            ) => left_layer_id == right_layer_id && left_waypoint_id == right_waypoint_id,
            (
                Self::MoveTrackPoint {
                    layer_id: left_layer_id,
                    track_id: left_track_id,
                    segment_id: left_segment_id,
                    point_id: left_point_id,
                    ..
                },
                Self::MoveTrackPoint {
                    layer_id: right_layer_id,
                    track_id: right_track_id,
                    segment_id: right_segment_id,
                    point_id: right_point_id,
                    ..
                },
            ) => {
                left_layer_id == right_layer_id
                    && left_track_id == right_track_id
                    && left_segment_id == right_segment_id
                    && left_point_id == right_point_id
            }
            (
                Self::RenameTrack {
                    layer_id: left_layer_id,
                    track_id: left_track_id,
                    ..
                },
                Self::RenameTrack {
                    layer_id: right_layer_id,
                    track_id: right_track_id,
                    ..
                },
            ) => left_layer_id == right_layer_id && left_track_id == right_track_id,
            _ => false,
        }
    }
}

#[derive(Debug, Clone)]
struct CommandDelta {
    forward: ProjectCommand,
    reverse: ProjectCommand,
}

#[derive(Debug, Clone, Default)]
pub struct CommandStack {
    undo_history: Vec<CommandDelta>,
    redo_history: Vec<CommandDelta>,
}

impl CommandStack {
    pub fn apply(
        &mut self,
        project: &mut Project,
        command: &ProjectCommand,
    ) -> Result<(), CommandError> {
        self.apply_or_merge(command.clone(), project)
    }

    pub fn apply_or_merge(
        &mut self,
        command: ProjectCommand,
        project: &mut Project,
    ) -> Result<(), CommandError> {
        self.redo_history.clear();

        if let Some(last_delta) = self.undo_history.last_mut()
            && last_delta.forward.targets_same_entity(&command)
        {
            command.apply(project)?;
            last_delta.forward = command;
            return Ok(());
        }

        let reverse = command.reverse(project);
        command.apply(project)?;
        self.undo_history.push(CommandDelta {
            forward: command,
            reverse,
        });
        if self.undo_history.len() > MAX_STACK_DEPTH {
            self.undo_history.remove(0);
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
        let Some(delta) = self.undo_history.pop() else {
            return false;
        };

        if delta.reverse.apply(project).is_err() {
            self.undo_history.push(delta);
            return false;
        }

        self.redo_history.push(delta);

        true
    }

    pub fn redo(&mut self, project: &mut Project) -> bool {
        let Some(delta) = self.redo_history.pop() else {
            return false;
        };

        if delta.forward.apply(project).is_err() {
            self.redo_history.push(delta);
            return false;
        }

        self.undo_history.push(delta);

        true
    }
}

#[cfg(test)]
mod tests {
    use super::{CommandError, CommandStack, ProjectCommand};
    use crate::domain::{
        LayerId, Project, ProjectLayerError, Track, TrackId, TrackLayer, TrackPoint,
        TrackPointId, TrackSegment, TrackSegmentId, Waypoint, WaypointId, WaypointLayer,
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
            CommandError::ProjectLayer(ProjectLayerError::WaypointLayerUnavailable(LayerId::new(
                30
            )))
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

    #[test]
    fn undo_and_redo_move_waypoint_round_trip_coordinates() {
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

        assert_eq!(project.waypoint_layers()[0].waypoints()[0].latitude(), 54.1);
        assert!(history.undo(&mut project));
        assert_eq!(project.waypoint_layers()[0].waypoints()[0].latitude(), 53.9);
        assert!(history.redo(&mut project));
        assert_eq!(project.waypoint_layers()[0].waypoints()[0].latitude(), 54.1);
    }

    #[test]
    fn apply_or_merge_coalesces_sequential_waypoint_moves_into_single_undo_step() {
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
            .apply_or_merge(
                ProjectCommand::move_waypoint(layer_id, waypoint_id, 54.0, 27.7),
                &mut project,
            )
            .unwrap();
        history
            .apply_or_merge(
                ProjectCommand::move_waypoint(layer_id, waypoint_id, 54.2, 27.9),
                &mut project,
            )
            .unwrap();

        assert_eq!(history.undo_history.len(), 3);
        assert!(history.undo(&mut project));
        let waypoint = &project.waypoint_layers()[0].waypoints()[0];
        assert_eq!(waypoint.latitude(), 53.9);
        assert_eq!(waypoint.longitude(), 27.5667);
    }

    #[test]
    fn move_track_point_apply_changes_coordinates() {
        let mut project = Project::untitled();
        let mut history = CommandStack::default();
        let layer_id = LayerId::new(20);
        let track_id = TrackId::new(1);
        let segment_id = TrackSegmentId::new(2);
        let point_id = TrackPointId::new(3);

        history
            .apply(&mut project, &ProjectCommand::add_track_layer(layer_id, "Tracks"))
            .unwrap();

        let mut track = Track::new(track_id, "Morning route");
        let mut segment = TrackSegment::new(segment_id);
        segment.add_point(TrackPoint::new(point_id, 53.9, 27.5667));
        track.add_segment(segment);

        history
            .apply(&mut project, &ProjectCommand::add_track(layer_id, track))
            .unwrap();

        history
            .apply(
                &mut project,
                &ProjectCommand::move_track_point(
                    layer_id, track_id, segment_id, point_id, 54.1, 27.8, 53.9, 27.5667,
                ),
            )
            .unwrap();

        let point = &project.track_layers()[0].tracks()[0].segments()[0].points()[0];
        assert_eq!(point.latitude(), 54.1);
        assert_eq!(point.longitude(), 27.8);
    }

    #[test]
    fn move_track_point_undo_restores_coordinates() {
        let mut project = Project::untitled();
        let mut history = CommandStack::default();
        let layer_id = LayerId::new(20);
        let track_id = TrackId::new(1);
        let segment_id = TrackSegmentId::new(2);
        let point_id = TrackPointId::new(3);

        history
            .apply(&mut project, &ProjectCommand::add_track_layer(layer_id, "Tracks"))
            .unwrap();

        let mut track = Track::new(track_id, "Morning route");
        let mut segment = TrackSegment::new(segment_id);
        segment.add_point(TrackPoint::new(point_id, 53.9, 27.5667));
        track.add_segment(segment);

        history
            .apply(&mut project, &ProjectCommand::add_track(layer_id, track))
            .unwrap();

        history
            .apply(
                &mut project,
                &ProjectCommand::move_track_point(
                    layer_id, track_id, segment_id, point_id, 54.1, 27.8, 53.9, 27.5667,
                ),
            )
            .unwrap();

        assert!(history.undo(&mut project));
        let point = &project.track_layers()[0].tracks()[0].segments()[0].points()[0];
        assert_eq!(point.latitude(), 53.9);
        assert_eq!(point.longitude(), 27.5667);
    }

    #[test]
    fn coalesce_move_track_point_merges_sequential_moves() {
        let mut project = Project::untitled();
        let mut history = CommandStack::default();
        let layer_id = LayerId::new(20);
        let track_id = TrackId::new(1);
        let segment_id = TrackSegmentId::new(2);
        let point_id = TrackPointId::new(3);

        history
            .apply(&mut project, &ProjectCommand::add_track_layer(layer_id, "Tracks"))
            .unwrap();

        let mut track = Track::new(track_id, "Morning route");
        let mut segment = TrackSegment::new(segment_id);
        segment.add_point(TrackPoint::new(point_id, 53.9, 27.5667));
        track.add_segment(segment);

        history
            .apply(&mut project, &ProjectCommand::add_track(layer_id, track))
            .unwrap();

        history
            .apply_or_merge(
                ProjectCommand::move_track_point(
                    layer_id, track_id, segment_id, point_id, 54.0, 27.7, 53.9, 27.5667,
                ),
                &mut project,
            )
            .unwrap();

        history
            .apply_or_merge(
                ProjectCommand::move_track_point(
                    layer_id, track_id, segment_id, point_id, 54.2, 27.9, 53.9, 27.5667,
                ),
                &mut project,
            )
            .unwrap();

        assert_eq!(history.undo_history.len(), 3);
        assert!(history.undo(&mut project));
        let point = &project.track_layers()[0].tracks()[0].segments()[0].points()[0];
        assert_eq!(point.latitude(), 53.9);
        assert_eq!(point.longitude(), 27.5667);
    }

    #[test]
    fn delete_track_point_apply_removes_point() {
        let mut project = Project::untitled();
        let mut history = CommandStack::default();
        let layer_id = LayerId::new(20);
        let track_id = TrackId::new(1);
        let segment_id = TrackSegmentId::new(2);
        let point_id = TrackPointId::new(3);

        history
            .apply(&mut project, &ProjectCommand::add_track_layer(layer_id, "Tracks"))
            .unwrap();

        let mut track = Track::new(track_id, "Morning route");
        let mut segment = TrackSegment::new(segment_id);
        segment.add_point(TrackPoint::new(point_id, 53.9, 27.5667));
        segment.add_point(TrackPoint::new(TrackPointId::new(4), 54.0, 27.7));
        track.add_segment(segment);

        history
            .apply(&mut project, &ProjectCommand::add_track(layer_id, track))
            .unwrap();

        history
            .apply(
                &mut project,
                &ProjectCommand::delete_track_point(layer_id, track_id, segment_id, point_id),
            )
            .unwrap();

        let points = project.track_layers()[0].tracks()[0].segments()[0].points();
        assert_eq!(points.len(), 1);
        assert_eq!(points[0].id(), TrackPointId::new(4));
    }

    #[test]
    fn delete_track_point_undo_restores_at_same_index() {
        let mut project = Project::untitled();
        let mut history = CommandStack::default();
        let layer_id = LayerId::new(20);
        let track_id = TrackId::new(1);
        let segment_id = TrackSegmentId::new(2);
        let point_id = TrackPointId::new(3);

        history
            .apply(&mut project, &ProjectCommand::add_track_layer(layer_id, "Tracks"))
            .unwrap();

        let mut track = Track::new(track_id, "Morning route");
        let mut segment = TrackSegment::new(segment_id);
        segment.add_point(TrackPoint::new(point_id, 53.9, 27.5667));
        segment.add_point(TrackPoint::new(TrackPointId::new(4), 54.0, 27.7));
        track.add_segment(segment);

        history
            .apply(&mut project, &ProjectCommand::add_track(layer_id, track))
            .unwrap();

        history
            .apply(
                &mut project,
                &ProjectCommand::delete_track_point(layer_id, track_id, segment_id, point_id),
            )
            .unwrap();

        assert!(history.undo(&mut project));

        let points = project.track_layers()[0].tracks()[0].segments()[0].points();
        assert_eq!(points.len(), 2);
        assert_eq!(points[0].id(), point_id);
        assert_eq!(points[1].id(), TrackPointId::new(4));
    }

    #[test]
    fn delete_track_point_missing_point_returns_error() {
        let mut project = Project::untitled();
        let mut history = CommandStack::default();
        let layer_id = LayerId::new(20);
        let track_id = TrackId::new(1);
        let segment_id = TrackSegmentId::new(2);

        history
            .apply(&mut project, &ProjectCommand::add_track_layer(layer_id, "Tracks"))
            .unwrap();

        let mut track = Track::new(track_id, "Morning route");
        let mut segment = TrackSegment::new(segment_id);
        segment.add_point(TrackPoint::new(TrackPointId::new(4), 54.0, 27.7));
        track.add_segment(segment);

        history
            .apply(&mut project, &ProjectCommand::add_track(layer_id, track))
            .unwrap();

        let error = history
            .apply(
                &mut project,
                &ProjectCommand::delete_track_point(
                    layer_id,
                    track_id,
                    segment_id,
                    TrackPointId::new(99),
                ),
            )
            .unwrap_err();

        assert_eq!(
            error,
            CommandError::ProjectLayer(ProjectLayerError::MissingTrackPoint {
                layer_id: layer_id.value(),
                track_id: track_id.value(),
                segment_id: segment_id.value(),
                point_id: 99,
            })
        );
    }

    #[test]
    fn insert_track_point_apply_adds_point_at_index() {
        let mut project = Project::untitled();
        let mut history = CommandStack::default();
        let layer_id = LayerId::new(20);
        let track_id = TrackId::new(1);
        let segment_id = TrackSegmentId::new(2);

        history
            .apply(&mut project, &ProjectCommand::add_track_layer(layer_id, "Tracks"))
            .unwrap();

        let mut track = Track::new(track_id, "Morning route");
        let mut segment = TrackSegment::new(segment_id);
        segment.add_point(TrackPoint::new(TrackPointId::new(3), 53.9, 27.5667));
        segment.add_point(TrackPoint::new(TrackPointId::new(4), 54.0, 27.7));
        track.add_segment(segment);

        history
            .apply(&mut project, &ProjectCommand::add_track(layer_id, track))
            .unwrap();

        history
            .apply(
                &mut project,
                &ProjectCommand::insert_track_point(
                    layer_id,
                    track_id,
                    segment_id,
                    1,
                    TrackPoint::new(TrackPointId::new(20), 53.95, 27.63),
                ),
            )
            .unwrap();

        let points = project.track_layers()[0].tracks()[0].segments()[0].points();
        assert_eq!(points.len(), 3);
        assert_eq!(points[1].id(), TrackPointId::new(20));
    }

    #[test]
    fn insert_track_point_undo_removes_inserted_point() {
        let mut project = Project::untitled();
        let mut history = CommandStack::default();
        let layer_id = LayerId::new(20);
        let track_id = TrackId::new(1);
        let segment_id = TrackSegmentId::new(2);

        history
            .apply(&mut project, &ProjectCommand::add_track_layer(layer_id, "Tracks"))
            .unwrap();

        let mut track = Track::new(track_id, "Morning route");
        let mut segment = TrackSegment::new(segment_id);
        segment.add_point(TrackPoint::new(TrackPointId::new(3), 53.9, 27.5667));
        segment.add_point(TrackPoint::new(TrackPointId::new(4), 54.0, 27.7));
        track.add_segment(segment);

        history
            .apply(&mut project, &ProjectCommand::add_track(layer_id, track))
            .unwrap();

        history
            .apply(
                &mut project,
                &ProjectCommand::insert_track_point(
                    layer_id,
                    track_id,
                    segment_id,
                    1,
                    TrackPoint::new(TrackPointId::new(20), 53.95, 27.63),
                ),
            )
            .unwrap();

        assert!(history.undo(&mut project));
        let points = project.track_layers()[0].tracks()[0].segments()[0].points();
        assert_eq!(points.len(), 2);
        assert_eq!(points[0].id(), TrackPointId::new(3));
        assert_eq!(points[1].id(), TrackPointId::new(4));
    }

    #[test]
    fn insert_track_point_out_of_bounds_returns_error() {
        let mut project = Project::untitled();
        let mut history = CommandStack::default();
        let layer_id = LayerId::new(20);
        let track_id = TrackId::new(1);
        let segment_id = TrackSegmentId::new(2);

        history
            .apply(&mut project, &ProjectCommand::add_track_layer(layer_id, "Tracks"))
            .unwrap();

        let mut track = Track::new(track_id, "Morning route");
        let mut segment = TrackSegment::new(segment_id);
        segment.add_point(TrackPoint::new(TrackPointId::new(3), 53.9, 27.5667));
        track.add_segment(segment);

        history
            .apply(&mut project, &ProjectCommand::add_track(layer_id, track))
            .unwrap();

        let error = history
            .apply(
                &mut project,
                &ProjectCommand::insert_track_point(
                    layer_id,
                    track_id,
                    segment_id,
                    10,
                    TrackPoint::new(TrackPointId::new(20), 53.95, 27.63),
                ),
            )
            .unwrap_err();

        assert_eq!(
            error,
            CommandError::ProjectLayer(ProjectLayerError::MissingTrackPoint {
                layer_id: layer_id.value(),
                track_id: track_id.value(),
                segment_id: segment_id.value(),
                point_id: 10,
            })
        );
    }

    #[test]
    fn command_stack_drops_oldest_entries_when_max_depth_exceeded() {
        let mut project = Project::untitled();
        let mut history = CommandStack::default();

        for id in 1..=101 {
            history
                .apply(
                    &mut project,
                    &ProjectCommand::add_map_layer(LayerId::new(id), format!("Layer {id}")),
                )
                .unwrap();
        }

        assert_eq!(history.undo_history.len(), 100);
        for _ in 0..100 {
            assert!(history.undo(&mut project));
        }
        assert!(!history.undo(&mut project));
        assert_eq!(project.map_layers().len(), 1);
        assert_eq!(project.map_layers()[0].id(), LayerId::new(1));
    }

    #[test]
    fn undo_restores_state_for_all_existing_command_variants() {
        fn assert_round_trip(mut project: Project, command: ProjectCommand) {
            let before = project.clone();
            let mut history = CommandStack::default();

            history.apply(&mut project, &command).unwrap();
            assert!(history.undo(&mut project));
            assert_eq!(project, before);
        }

        assert_round_trip(
            Project::untitled(),
            ProjectCommand::add_map_layer(LayerId::new(10), "Map"),
        );

        assert_round_trip(
            Project::untitled(),
            ProjectCommand::add_map_layer_with_source(LayerId::new(11), "Map", "demo.sqlitedb"),
        );

        assert_round_trip(
            Project::untitled(),
            ProjectCommand::add_track_layer(LayerId::new(20), "Tracks"),
        );

        assert_round_trip(
            Project::untitled(),
            ProjectCommand::add_waypoint_layer(LayerId::new(30), "Waypoints"),
        );

        let mut add_track_project = Project::untitled();
        add_track_project.add_track_layer(TrackLayer::new(LayerId::new(20), "Tracks"));
        assert_round_trip(
            add_track_project,
            ProjectCommand::add_track(LayerId::new(20), Track::new(TrackId::new(1), "Morning")),
        );

        let mut add_waypoint_project = Project::untitled();
        add_waypoint_project.add_waypoint_layer(WaypointLayer::new(LayerId::new(30), "Waypoints"));
        assert_round_trip(
            add_waypoint_project,
            ProjectCommand::add_waypoint(
                LayerId::new(30),
                Waypoint::new(WaypointId::new(4), "Camp", 53.9, 27.5667),
            ),
        );

        let mut move_waypoint_project = Project::untitled();
        move_waypoint_project.add_waypoint_layer(WaypointLayer::new(LayerId::new(30), "Waypoints"));
        move_waypoint_project
            .add_waypoint_to_layer(
                LayerId::new(30),
                Waypoint::new(WaypointId::new(4), "Camp", 53.9, 27.5667),
            )
            .unwrap();
        assert_round_trip(
            move_waypoint_project,
            ProjectCommand::move_waypoint(LayerId::new(30), WaypointId::new(4), 54.1, 27.8),
        );

        let mut rename_track_project = Project::untitled();
        rename_track_project.add_track_layer(TrackLayer::new(LayerId::new(20), "Tracks"));
        rename_track_project
            .add_track_to_layer(
                LayerId::new(20),
                Track::new(TrackId::new(1), "Morning route"),
            )
            .unwrap();
        assert_round_trip(
            rename_track_project,
            ProjectCommand::rename_track(
                LayerId::new(20),
                TrackId::new(1),
                "Morning route",
                "Renamed route",
            ),
        );
    }
}
