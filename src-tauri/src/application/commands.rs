#![allow(dead_code)]

use crate::domain::{
    LayerId, MapLayer, Project, ProjectLayerError, Track, TrackId, TrackLayer, TrackPoint,
    TrackPointId, TrackSegmentId, Waypoint, WaypointId, WaypointLayer, simplify_track_points,
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
    SplitSegment {
        layer_id: LayerId,
        track_id: TrackId,
        segment_id: TrackSegmentId,
        point_id: TrackPointId,
        new_segment_id: TrackSegmentId,
    },
    JoinSegments {
        layer_id: LayerId,
        track_id: TrackId,
        segment_id_a: TrackSegmentId,
        segment_id_b: TrackSegmentId,
    },
    /// CJ-4: set an explicit per-segment point order (sort-by-time is the
    /// main producer). Symmetric: its reverse is another ReorderTrackPoints
    /// carrying the pre-apply order, so undo/redo are exact inverses.
    ReorderTrackPoints {
        layer_id: LayerId,
        track_id: TrackId,
        order: Vec<(TrackSegmentId, Vec<TrackPointId>)>,
    },
    /// CJ-4: bulk point removal (crop by extent / time range). Reverse is
    /// RestoreTrackPoints with exact indices captured pre-apply.
    CropTrackPoints {
        layer_id: LayerId,
        track_id: TrackId,
        points: Vec<(TrackSegmentId, Vec<TrackPointId>)>,
    },
    DeleteTrack {
        layer_id: LayerId,
        track_id: TrackId,
    },
    DeleteWaypoint {
        layer_id: LayerId,
        waypoint_id: WaypointId,
    },
    RenameWaypoint {
        layer_id: LayerId,
        waypoint_id: WaypointId,
        old_name: String,
        new_name: String,
    },
    SetWaypointSymbol {
        layer_id: LayerId,
        waypoint_id: WaypointId,
        old_symbol: Option<String>,
        new_symbol: Option<String>,
    },
    SetWaypointColor {
        layer_id: LayerId,
        waypoint_id: WaypointId,
        old_color: Option<[u8; 4]>,
        new_color: Option<[u8; 4]>,
    },
    /// The note beside a mark: what a crew is actually sent to.
    SetWaypointDescription {
        layer_id: LayerId,
        waypoint_id: WaypointId,
        old_description: Option<String>,
        new_description: Option<String>,
    },
    SimplifyTrack {
        layer_id: LayerId,
        track_id: TrackId,
        tolerance_km: f64,
        removed: Vec<(TrackSegmentId, usize, TrackPoint)>,
    },
    RestoreTrackPoints {
        layer_id: LayerId,
        track_id: TrackId,
        points: Vec<(TrackSegmentId, usize, TrackPoint)>,
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
    /// Put a layer back exactly as it was, contents included.
    ///
    /// This is what a removal reverses to. Reversing it to `AddTrackLayer`
    /// rebuilt the layer from its id and name alone, so undoing the removal of
    /// a day's work handed back an empty layer.
    RestoreTrackLayer {
        layer: TrackLayer,
    },
    RestoreWaypointLayer {
        layer: WaypointLayer,
    },
    RenameTrackLayer {
        layer_id: LayerId,
        old_name: String,
        new_name: String,
    },
    RenameWaypointLayer {
        layer_id: LayerId,
        old_name: String,
        new_name: String,
    },
    RemoveTrack {
        layer_id: LayerId,
        track: Track,
    },
    RemoveWaypoint {
        layer_id: LayerId,
        waypoint: Waypoint,
    },
    CreateEmptyTrack {
        layer_id: LayerId,
        track_id: TrackId,
        name: String,
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

    pub fn reorder_track_points(
        layer_id: LayerId,
        track_id: TrackId,
        order: Vec<(u64, Vec<u64>)>,
    ) -> Self {
        Self::ReorderTrackPoints {
            layer_id,
            track_id,
            order: order
                .into_iter()
                .map(|(seg, ids)| {
                    (
                        TrackSegmentId::new(seg),
                        ids.into_iter().map(TrackPointId::new).collect(),
                    )
                })
                .collect(),
        }
    }

    pub fn crop_track_points(
        layer_id: LayerId,
        track_id: TrackId,
        points: Vec<(TrackSegmentId, Vec<TrackPointId>)>,
    ) -> Self {
        Self::CropTrackPoints {
            layer_id,
            track_id,
            points,
        }
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

    #[allow(clippy::too_many_arguments)]
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

    pub fn split_segment(
        layer_id: LayerId,
        track_id: TrackId,
        segment_id: TrackSegmentId,
        point_id: TrackPointId,
        new_segment_id: TrackSegmentId,
    ) -> Self {
        Self::SplitSegment {
            layer_id,
            track_id,
            segment_id,
            point_id,
            new_segment_id,
        }
    }

    pub fn join_segments(
        layer_id: LayerId,
        track_id: TrackId,
        segment_id_a: TrackSegmentId,
        segment_id_b: TrackSegmentId,
    ) -> Self {
        Self::JoinSegments {
            layer_id,
            track_id,
            segment_id_a,
            segment_id_b,
        }
    }

    pub fn delete_track(layer_id: LayerId, track_id: TrackId) -> Self {
        Self::DeleteTrack { layer_id, track_id }
    }

    pub fn delete_waypoint(layer_id: LayerId, waypoint_id: WaypointId) -> Self {
        Self::DeleteWaypoint {
            layer_id,
            waypoint_id,
        }
    }

    pub fn rename_waypoint(
        layer_id: LayerId,
        waypoint_id: WaypointId,
        old_name: impl Into<String>,
        new_name: impl Into<String>,
    ) -> Self {
        Self::RenameWaypoint {
            layer_id,
            waypoint_id,
            old_name: old_name.into(),
            new_name: new_name.into(),
        }
    }

    pub fn set_waypoint_symbol(
        layer_id: LayerId,
        waypoint_id: WaypointId,
        old_symbol: Option<String>,
        new_symbol: Option<String>,
    ) -> Self {
        Self::SetWaypointSymbol {
            layer_id,
            waypoint_id,
            old_symbol,
            new_symbol,
        }
    }

    pub fn set_waypoint_color(
        layer_id: LayerId,
        waypoint_id: WaypointId,
        old_color: Option<[u8; 4]>,
        new_color: Option<[u8; 4]>,
    ) -> Self {
        Self::SetWaypointColor {
            layer_id,
            waypoint_id,
            old_color,
            new_color,
        }
    }

    pub fn simplify_track(layer_id: LayerId, track_id: TrackId, tolerance_km: f64) -> Self {
        Self::SimplifyTrack {
            layer_id,
            track_id,
            tolerance_km,
            removed: Vec::new(),
        }
    }

    pub fn create_empty_track(
        layer_id: LayerId,
        track_id: TrackId,
        name: impl Into<String>,
    ) -> Self {
        Self::CreateEmptyTrack {
            layer_id,
            track_id,
            name: name.into(),
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
            Self::SplitSegment {
                layer_id,
                track_id,
                segment_id,
                point_id,
                new_segment_id,
            } => {
                let new_segment = {
                    let track = project.track_mut(layer_id.value(), track_id.value())?;
                    let segment = track.segment_mut(*segment_id).ok_or(
                        ProjectLayerError::MissingTrackSegment {
                            layer_id: layer_id.value(),
                            track_id: track_id.value(),
                            segment_id: segment_id.value(),
                        },
                    )?;

                    segment
                        .split_at_point(point_id.value(), *new_segment_id)
                        .map_err(|err| match err {
                            ProjectLayerError::InvalidSegmentOperation {
                                segment_id,
                                reason,
                                ..
                            } => ProjectLayerError::InvalidSegmentOperation {
                                layer_id: layer_id.value(),
                                track_id: track_id.value(),
                                segment_id,
                                reason,
                            },
                            _ => ProjectLayerError::MissingTrackPoint {
                                layer_id: layer_id.value(),
                                track_id: track_id.value(),
                                segment_id: segment_id.value(),
                                point_id: point_id.value(),
                            },
                        })?
                };

                let track = project.track_mut(layer_id.value(), track_id.value())?;
                let insert_index = track
                    .segments()
                    .iter()
                    .position(|segment| segment.id() == *segment_id)
                    .map_or(track.segments().len(), |idx| idx + 1);
                track.insert_segment_at(insert_index, new_segment);
                Ok(())
            }
            Self::JoinSegments {
                layer_id,
                track_id,
                segment_id_a,
                segment_id_b,
            } => {
                project.join_segments_in_layer(
                    layer_id.value(),
                    track_id.value(),
                    segment_id_a.value(),
                    segment_id_b.value(),
                )?;
                Ok(())
            }
            Self::ReorderTrackPoints {
                layer_id,
                track_id,
                order,
            } => {
                let track = project.track_mut(layer_id.value(), track_id.value())?;
                for (segment_id, ids) in order {
                    let segment = track.segment_mut(*segment_id).ok_or(
                        ProjectLayerError::MissingTrackSegment {
                            layer_id: layer_id.value(),
                            track_id: track_id.value(),
                            segment_id: segment_id.value(),
                        },
                    )?;
                    segment.reorder_points(ids).map_err(|mut err| {
                        if let ProjectLayerError::MissingTrackPoint {
                            layer_id: l,
                            track_id: tk,
                            ..
                        } = &mut err
                        {
                            *l = layer_id.value();
                            *tk = track_id.value();
                        }
                        err
                    })?;
                }
                Ok(())
            }
            Self::CropTrackPoints {
                layer_id,
                track_id,
                points,
            } => {
                let track = project.track_mut(layer_id.value(), track_id.value())?;
                let total: usize = track.segments().iter().map(|s| s.points().len()).sum();
                let removing: usize = points.iter().map(|(_, ids)| ids.len()).sum();
                if removing >= total {
                    return Err(CommandError::ProjectLayer(
                        ProjectLayerError::InvalidSegmentOperation {
                            layer_id: layer_id.value(),
                            track_id: track_id.value(),
                            segment_id: 0,
                            reason: "crop would remove every point of the track",
                        },
                    ));
                }
                for (segment_id, ids) in points {
                    let segment = track.segment_mut(*segment_id).ok_or(
                        ProjectLayerError::MissingTrackSegment {
                            layer_id: layer_id.value(),
                            track_id: track_id.value(),
                            segment_id: segment_id.value(),
                        },
                    )?;
                    for point_id in ids {
                        segment.remove_point(point_id.value()).map_err(|_| {
                            ProjectLayerError::MissingTrackPoint {
                                layer_id: layer_id.value(),
                                track_id: track_id.value(),
                                segment_id: segment_id.value(),
                                point_id: point_id.value(),
                            }
                        })?;
                    }
                }
                Ok(())
            }
            Self::DeleteTrack { layer_id, track_id } => {
                project.remove_track_from_layer(*layer_id, *track_id)?;
                Ok(())
            }
            Self::DeleteWaypoint {
                layer_id,
                waypoint_id,
            } => {
                project.remove_waypoint_from_layer(*layer_id, *waypoint_id)?;
                Ok(())
            }
            Self::RenameWaypoint {
                layer_id,
                waypoint_id,
                new_name,
                ..
            } => {
                project.rename_waypoint_in_layer(*layer_id, *waypoint_id, new_name.clone())?;
                Ok(())
            }
            Self::SetWaypointDescription {
                layer_id,
                waypoint_id,
                new_description,
                ..
            } => {
                project.set_waypoint_description_in_layer(
                    *layer_id,
                    *waypoint_id,
                    new_description.clone(),
                )?;
                Ok(())
            }
            Self::SetWaypointSymbol {
                layer_id,
                waypoint_id,
                new_symbol,
                ..
            } => {
                project.set_waypoint_symbol_in_layer(
                    *layer_id,
                    *waypoint_id,
                    new_symbol.clone(),
                )?;
                Ok(())
            }
            Self::SetWaypointColor {
                layer_id,
                waypoint_id,
                new_color,
                ..
            } => {
                project.set_waypoint_color_in_layer(*layer_id, *waypoint_id, *new_color)?;
                Ok(())
            }
            Self::SimplifyTrack {
                layer_id,
                track_id,
                tolerance_km,
                removed,
            } => {
                let removed_points = if removed.is_empty() {
                    collect_simplify_removed(project, *layer_id, *track_id, *tolerance_km)
                } else {
                    removed.clone()
                };

                for (segment_id, _, point) in removed_points.iter().rev() {
                    project.remove_point_from_layer(
                        layer_id.value(),
                        track_id.value(),
                        segment_id.value(),
                        point.id().value(),
                    )?;
                }

                Ok(())
            }
            Self::RestoreTrackPoints {
                layer_id,
                track_id,
                points,
            } => {
                let mut sorted = points.clone();
                sorted.sort_by_key(|(segment_id, index, _)| (segment_id.value(), *index));

                for (segment_id, index, point) in sorted {
                    project.insert_point_in_layer(
                        layer_id.value(),
                        track_id.value(),
                        segment_id.value(),
                        index,
                        point,
                    )?;
                }

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
            Self::RestoreTrackLayer { layer } => {
                project.add_track_layer(layer.clone());
                Ok(())
            }
            Self::RestoreWaypointLayer { layer } => {
                project.add_waypoint_layer(layer.clone());
                Ok(())
            }
            // A rename of a layer that is not there is an error, not a
            // success. Answering `Ok` let the command stack clear the redo
            // history and count a mutation for something that did not happen —
            // the same shape as the refused-command defect the September
            // review found, reached from a different direction. Found by an
            // outside reviewer, 2026-09-23.
            Self::RenameTrackLayer {
                layer_id, new_name, ..
            } => {
                project
                    .track_layer_mut(layer_id.value())?
                    .set_name(new_name.clone());
                Ok(())
            }
            Self::RenameWaypointLayer {
                layer_id, new_name, ..
            } => {
                project
                    .waypoint_layer_mut(layer_id.value())?
                    .set_name(new_name.clone());
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
            Self::CreateEmptyTrack {
                layer_id,
                track_id,
                name,
            } => {
                let layer = project.track_layer_mut(layer_id.value())?;
                layer.create_empty_track(*track_id, name.clone());
                Ok(())
            }
        }
    }

    pub fn reverse(&self, project: &Project) -> ProjectCommand {
        match self {
            Self::ReorderTrackPoints {
                layer_id,
                track_id,
                order,
            } => {
                // Symmetric inverse: capture the CURRENT order of exactly the
                // segments being reordered.
                let current = project
                    .track_layers()
                    .iter()
                    .find(|l| l.id() == *layer_id)
                    .and_then(|l| l.tracks().iter().find(|t| t.id() == *track_id))
                    .map(|track| {
                        order
                            .iter()
                            .filter_map(|(segment_id, _)| {
                                track
                                    .segments()
                                    .iter()
                                    .find(|s| s.id() == *segment_id)
                                    .map(|s| {
                                        (
                                            *segment_id,
                                            s.points().iter().map(|p| p.id()).collect::<Vec<_>>(),
                                        )
                                    })
                            })
                            .collect()
                    })
                    .unwrap_or_default();
                Self::ReorderTrackPoints {
                    layer_id: *layer_id,
                    track_id: *track_id,
                    order: current,
                }
            }
            Self::CropTrackPoints {
                layer_id,
                track_id,
                points,
            } => {
                // Capture each removed point with its exact index pre-apply.
                let restored = project
                    .track_layers()
                    .iter()
                    .find(|l| l.id() == *layer_id)
                    .and_then(|l| l.tracks().iter().find(|t| t.id() == *track_id))
                    .map(|track| {
                        points
                            .iter()
                            .flat_map(|(segment_id, ids)| {
                                track
                                    .segments()
                                    .iter()
                                    .find(|s| s.id() == *segment_id)
                                    .into_iter()
                                    .flat_map(move |segment| {
                                        segment
                                            .points()
                                            .iter()
                                            .enumerate()
                                            .filter(|(_, point)| ids.contains(&point.id()))
                                            .map(|(index, point)| {
                                                (*segment_id, index, point.clone())
                                            })
                                            .collect::<Vec<_>>()
                                    })
                            })
                            .collect()
                    })
                    .unwrap_or_default();
                Self::RestoreTrackPoints {
                    layer_id: *layer_id,
                    track_id: *track_id,
                    points: restored,
                }
            }
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

                let (index, point) = found.unwrap_or((*removed_index, removed_point.clone()));

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
            Self::SplitSegment {
                layer_id,
                track_id,
                segment_id,
                new_segment_id,
                ..
            } => Self::JoinSegments {
                layer_id: *layer_id,
                track_id: *track_id,
                segment_id_a: *segment_id,
                segment_id_b: *new_segment_id,
            },
            Self::JoinSegments {
                layer_id,
                track_id,
                segment_id_a,
                segment_id_b,
            } => {
                let split_point_id = project
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
                                    .find(|segment| segment.id() == *segment_id_a)
                                    .and_then(|segment| segment.points().last())
                            })
                    })
                    .map(|point| point.id())
                    .unwrap_or(TrackPointId::new(0));

                Self::SplitSegment {
                    layer_id: *layer_id,
                    track_id: *track_id,
                    segment_id: *segment_id_a,
                    point_id: split_point_id,
                    new_segment_id: *segment_id_b,
                }
            }
            Self::DeleteTrack { layer_id, track_id } => {
                let track = project
                    .track_layers()
                    .iter()
                    .find(|layer| layer.id() == *layer_id)
                    .and_then(|layer| layer.tracks().iter().find(|track| track.id() == *track_id))
                    .cloned()
                    .unwrap_or_else(|| Track::new(*track_id, ""));

                Self::AddTrack {
                    layer_id: *layer_id,
                    track,
                }
            }
            Self::DeleteWaypoint {
                layer_id,
                waypoint_id,
            } => {
                let waypoint = project
                    .waypoint_layers()
                    .iter()
                    .find(|layer| layer.id() == *layer_id)
                    .and_then(|layer| {
                        layer
                            .waypoints()
                            .iter()
                            .find(|waypoint| waypoint.id() == *waypoint_id)
                    })
                    .cloned()
                    .unwrap_or_else(|| Waypoint::new(*waypoint_id, "", 0.0, 0.0));

                Self::AddWaypoint {
                    layer_id: *layer_id,
                    waypoint,
                }
            }
            Self::RenameWaypoint {
                layer_id,
                waypoint_id,
                old_name,
                new_name,
            } => Self::RenameWaypoint {
                layer_id: *layer_id,
                waypoint_id: *waypoint_id,
                old_name: new_name.clone(),
                new_name: old_name.clone(),
            },
            Self::SetWaypointDescription {
                layer_id,
                waypoint_id,
                old_description,
                new_description,
            } => Self::SetWaypointDescription {
                layer_id: *layer_id,
                waypoint_id: *waypoint_id,
                old_description: new_description.clone(),
                new_description: old_description.clone(),
            },
            Self::SetWaypointSymbol {
                layer_id,
                waypoint_id,
                old_symbol,
                new_symbol,
            } => Self::SetWaypointSymbol {
                layer_id: *layer_id,
                waypoint_id: *waypoint_id,
                old_symbol: new_symbol.clone(),
                new_symbol: old_symbol.clone(),
            },
            Self::SetWaypointColor {
                layer_id,
                waypoint_id,
                old_color,
                new_color,
            } => Self::SetWaypointColor {
                layer_id: *layer_id,
                waypoint_id: *waypoint_id,
                old_color: *new_color,
                new_color: *old_color,
            },
            Self::SimplifyTrack {
                layer_id,
                track_id,
                tolerance_km,
                removed,
            } => {
                let points = if removed.is_empty() {
                    collect_simplify_removed(project, *layer_id, *track_id, *tolerance_km)
                } else {
                    removed.clone()
                };

                Self::RestoreTrackPoints {
                    layer_id: *layer_id,
                    track_id: *track_id,
                    points,
                }
            }
            Self::RestoreTrackPoints {
                layer_id,
                track_id,
                points,
            } => Self::SimplifyTrack {
                layer_id: *layer_id,
                track_id: *track_id,
                tolerance_km: 0.0,
                removed: points.clone(),
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
            // Not `AddTrackLayer`: that rebuilds the layer from its id and
            // name, so undoing the removal of a day's recordings handed back an
            // empty layer and lost the tracks. External review of the command
            // set, 2026-09-23.
            Self::RemoveTrackLayer { layer } => Self::RestoreTrackLayer {
                layer: layer.clone(),
            },
            Self::RemoveWaypointLayer { layer } => Self::RestoreWaypointLayer {
                layer: layer.clone(),
            },
            Self::RestoreTrackLayer { layer } => Self::RemoveTrackLayer {
                layer: layer.clone(),
            },
            Self::RestoreWaypointLayer { layer } => Self::RemoveWaypointLayer {
                layer: layer.clone(),
            },
            Self::RenameTrackLayer {
                layer_id,
                old_name,
                new_name,
            } => Self::RenameTrackLayer {
                layer_id: *layer_id,
                old_name: new_name.clone(),
                new_name: old_name.clone(),
            },
            Self::RenameWaypointLayer {
                layer_id,
                old_name,
                new_name,
            } => Self::RenameWaypointLayer {
                layer_id: *layer_id,
                old_name: new_name.clone(),
                new_name: old_name.clone(),
            },
            Self::RemoveTrack { layer_id, track } => Self::AddTrack {
                layer_id: *layer_id,
                track: track.clone(),
            },
            Self::RemoveWaypoint { layer_id, waypoint } => Self::AddWaypoint {
                layer_id: *layer_id,
                waypoint: waypoint.clone(),
            },
            Self::CreateEmptyTrack {
                layer_id,
                track_id,
                name,
            } => {
                let mut track = crate::domain::Track::new(*track_id, name.clone());
                track.add_segment(crate::domain::TrackSegment::new(
                    crate::domain::TrackSegmentId::new(1),
                ));
                Self::RemoveTrack {
                    layer_id: *layer_id,
                    track,
                }
            }
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

fn collect_simplify_removed(
    project: &Project,
    layer_id: LayerId,
    track_id: TrackId,
    tolerance_km: f64,
) -> Vec<(TrackSegmentId, usize, TrackPoint)> {
    project
        .track_layers()
        .iter()
        .find(|layer| layer.id() == layer_id)
        .and_then(|layer| layer.tracks().iter().find(|track| track.id() == track_id))
        .map(|track| {
            let mut removed = Vec::new();
            for segment in track.segments() {
                let keep = simplify_track_points(segment.points(), tolerance_km);
                let keep_set: std::collections::BTreeSet<usize> = keep.into_iter().collect();

                for (index, point) in segment.points().iter().enumerate() {
                    if !keep_set.contains(&index) {
                        removed.push((segment.id(), index, point.clone()));
                    }
                }
            }
            removed
        })
        .unwrap_or_default()
}

#[derive(Debug, Clone)]
struct CommandDelta {
    forward: ProjectCommand,
    reverse: ProjectCommand,
    /// The gesture this entry belongs to, when it belongs to one.
    ///
    /// Only entries of the same gesture coalesce. Before, merging asked one
    /// question — do these two commands touch the same entity — so dragging a
    /// point, letting go, looking at it and dragging it again produced a
    /// single undo step, and Ctrl+Z went back past a correction the operator
    /// had already accepted. A gesture is what the operator would call one
    /// action, and only the caller knows where one ends.
    /// External review, 2026-09-22.
    gesture: Option<GestureId>,
}

/// Identifies one continuous operator action, so the commands it produces
/// collapse into a single undo step.
///
/// Nothing merges today: every caller sends one command per gesture, so each
/// carries `None` and each lands as its own entry. The type exists for the
/// path that will send a command per pointer move while a drag is in flight —
/// that is the case coalescing was written for, and the one where merging by
/// entity alone would be right by accident.
pub type GestureId = u64;

#[derive(Debug, Clone, Default)]
pub struct CommandStack {
    undo_history: Vec<CommandDelta>,
    redo_history: Vec<CommandDelta>,
    /// Total successful project mutations routed through this stack (apply,
    /// merge, undo, redo). Structural dirty-tracking input: every undoable
    /// edit bumps it HERE, so call sites cannot forget to mark the project
    /// dirty (see `AppState::project_dirty`).
    mutation_count: u64,
}

impl CommandStack {
    /// Apply a command as its own undo step.
    pub fn apply(
        &mut self,
        project: &mut Project,
        command: &ProjectCommand,
    ) -> Result<(), CommandError> {
        self.apply_or_merge(command.clone(), None, project)
    }

    /// Apply a command, merging it into the previous undo step when both
    /// belong to the same gesture and touch the same entity.
    ///
    /// `gesture` of `None` never merges: an action the caller has not tied to
    /// a gesture is an action of its own, and the operator gets one Ctrl+Z
    /// for it. See [`GestureId`].
    pub fn apply_or_merge(
        &mut self,
        command: ProjectCommand,
        gesture: Option<GestureId>,
        project: &mut Project,
    ) -> Result<(), CommandError> {
        // The redo stack is cleared only once the command has actually landed.
        // Clearing first meant a refused operation destroyed the redo the
        // operator still had — the edit did not happen, and the history
        // changed anyway. External review, 2026-09-22.
        if let Some(last_delta) = self.undo_history.last_mut()
            && gesture.is_some()
            && last_delta.gesture == gesture
            && last_delta.forward.targets_same_entity(&command)
        {
            command.apply(project)?;
            self.redo_history.clear();
            last_delta.forward = command;
            self.mutation_count += 1;
            return Ok(());
        }

        let reverse = command.reverse(project);
        command.apply(project)?;
        self.redo_history.clear();
        self.mutation_count += 1;
        self.undo_history.push(CommandDelta {
            forward: command,
            reverse,
            gesture,
        });
        if self.undo_history.len() > MAX_STACK_DEPTH {
            self.undo_history.remove(0);
        }

        Ok(())
    }

    pub fn can_undo(&self) -> bool {
        !self.undo_history.is_empty()
    }

    /// Current depth of the undo stack. Exposed for tests that verify
    /// non-undoable mutations (style/visibility) do not push entries.
    pub fn undo_depth(&self) -> usize {
        self.undo_history.len()
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

        self.mutation_count += 1;
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

        self.mutation_count += 1;
        self.undo_history.push(delta);

        true
    }

    /// Undo the last `count` commands and forget them.
    ///
    /// Abandoning a drawing is not an edit the operator will want back: doing
    /// it with `undo` left the discarded track sitting in the redo stack, so
    /// a later redo resurrected a track that was deliberately thrown away,
    /// and the project stayed marked as changed although nothing had changed.
    ///
    /// Returns the number of commands actually discarded, which is less than
    /// `count` only if the stack was shorter or a reverse failed to apply.
    pub fn discard_last(&mut self, count: usize, project: &mut Project) -> usize {
        let mut discarded = 0;
        for _ in 0..count {
            let Some(delta) = self.undo_history.pop() else {
                break;
            };
            if delta.reverse.apply(project).is_err() {
                self.undo_history.push(delta);
                break;
            }
            discarded += 1;
        }

        // The discarded commands never happened as far as dirty tracking is
        // concerned, so their mutations come back off the counter instead of
        // adding more.
        self.mutation_count = self.mutation_count.saturating_sub(discarded as u64);
        discarded
    }

    /// See the `mutation_count` field docs; consumed by
    /// `AppState::project_dirty`.
    pub fn mutation_count(&self) -> u64 {
        self.mutation_count
    }
}

#[cfg(test)]
mod cj4_tests {
    use super::{CommandStack, ProjectCommand};
    use crate::domain::{
        LayerId, Project, Track, TrackId, TrackPoint, TrackPointId, TrackSegment, TrackSegmentId,
    };
    use chrono::TimeZone;

    fn ts(minute: u32) -> chrono::DateTime<chrono::Utc> {
        chrono::Utc
            .with_ymd_and_hms(2026, 7, 15, 10, minute, 0)
            .unwrap()
    }

    fn project_with_shuffled_track() -> Project {
        let mut project = Project::untitled();
        let mut bootstrap_layer = CommandStack::default();
        bootstrap_layer
            .apply(
                &mut project,
                &ProjectCommand::add_track_layer(LayerId::new(20), "Tracks"),
            )
            .expect("bootstrap layer");
        let mut track = Track::new(TrackId::new(1), "Shuffled");
        let mut seg = TrackSegment::new(TrackSegmentId::new(2));
        // Timestamps deliberately out of order; point 12 is untimed.
        seg.add_point(TrackPoint::new(TrackPointId::new(10), 55.0, 37.0).with_timestamp(ts(30)));
        seg.add_point(TrackPoint::new(TrackPointId::new(11), 55.1, 37.1).with_timestamp(ts(10)));
        seg.add_point(TrackPoint::new(TrackPointId::new(12), 55.2, 37.2));
        seg.add_point(TrackPoint::new(TrackPointId::new(13), 55.3, 37.3).with_timestamp(ts(20)));
        track.add_segment(seg);
        let mut bootstrap = CommandStack::default();
        bootstrap
            .apply(
                &mut project,
                &ProjectCommand::add_track(LayerId::new(20), track),
            )
            .expect("bootstrap track");
        project
    }

    fn shuffled_track(project: &Project) -> &Track {
        project
            .track_layers()
            .iter()
            .find(|l| l.id() == LayerId::new(20))
            .expect("layer 20")
            .tracks()
            .iter()
            .find(|t| t.id() == TrackId::new(1))
            .expect("track 1")
    }

    fn point_ids(project: &Project) -> Vec<u64> {
        shuffled_track(project).segments()[0]
            .points()
            .iter()
            .map(|p| p.id().value())
            .collect()
    }

    /// CJ-4 sort: reorder is a symmetric command — undo restores the exact
    /// previous order, redo re-applies. Untimed points sort first, stably.
    #[test]
    fn reorder_track_points_applies_and_undoes_exactly() {
        let mut project = project_with_shuffled_track();
        let mut history = CommandStack::default();

        let order = crate::domain::sorted_point_order_by_time(shuffled_track(&project).segments());
        assert_eq!(
            order,
            vec![(2_u64, vec![12_u64, 11, 13, 10])],
            "untimed first (stable), then ascending timestamps",
        );

        history
            .apply(
                &mut project,
                &ProjectCommand::reorder_track_points(LayerId::new(20), TrackId::new(1), order),
            )
            .expect("reorder applies");
        assert_eq!(point_ids(&project), vec![12, 11, 13, 10]);

        assert!(history.undo(&mut project));
        assert_eq!(
            point_ids(&project),
            vec![10, 11, 12, 13],
            "undo = old order"
        );
        assert!(history.redo(&mut project));
        assert_eq!(point_ids(&project), vec![12, 11, 13, 10]);
    }

    /// CJ-4 crop: removing a point set is undoable with exact positions.
    #[test]
    fn crop_track_points_removes_and_undo_restores_positions() {
        let mut project = project_with_shuffled_track();
        let mut history = CommandStack::default();

        history
            .apply(
                &mut project,
                &ProjectCommand::crop_track_points(
                    LayerId::new(20),
                    TrackId::new(1),
                    vec![(
                        TrackSegmentId::new(2),
                        vec![TrackPointId::new(11), TrackPointId::new(13)],
                    )],
                ),
            )
            .expect("crop applies");
        assert_eq!(point_ids(&project), vec![10, 12]);

        assert!(history.undo(&mut project));
        assert_eq!(
            point_ids(&project),
            vec![10, 11, 12, 13],
            "undo restores removed points at their exact indices",
        );
        assert!(history.redo(&mut project));
        assert_eq!(point_ids(&project), vec![10, 12]);
    }

    /// Cropping everything is a user error, not silent track destruction.
    #[test]
    fn crop_rejects_removing_every_point_of_a_track() {
        let mut project = project_with_shuffled_track();
        let mut history = CommandStack::default();

        let result = history.apply(
            &mut project,
            &ProjectCommand::crop_track_points(
                LayerId::new(20),
                TrackId::new(1),
                vec![(
                    TrackSegmentId::new(2),
                    vec![
                        TrackPointId::new(10),
                        TrackPointId::new(11),
                        TrackPointId::new(12),
                        TrackPointId::new(13),
                    ],
                )],
            ),
        );
        assert!(result.is_err(), "crop of every point must be rejected");
        assert_eq!(
            point_ids(&project),
            vec![10, 11, 12, 13],
            "project untouched"
        );
    }
}

#[cfg(test)]
mod tests {
    /// Неуспешная команда не должна трогать историю.
    ///
    /// Внешнее ревью 22.09: `redo_history.clear()` стоял до `apply`, поэтому
    /// отказ операции стирал возможность повтора, хотя правка не состоялась.
    /// Крыло нажимает «удалить точку», получает отказ — и молча теряет всё,
    /// что могло вернуть через «повторить».
    #[test]
    fn a_command_that_fails_leaves_redo_alone() {
        use crate::domain::{LayerId, Project, TrackId};

        let mut project = Project::default();
        let layer = LayerId::new(1);
        let mut history = CommandStack::default();
        history
            .apply(
                &mut project,
                &ProjectCommand::create_empty_track(layer, TrackId::new(1), "трек".to_owned()),
            )
            .expect("create");
        assert!(history.undo(&mut project), "undo");
        assert!(history.can_redo(), "после отмены повтор доступен");

        // Команда, которая обязана отказать: такого слоя нет.
        let failed = history.apply(
            &mut project,
            &ProjectCommand::create_empty_track(
                LayerId::new(999),
                TrackId::new(2),
                "нет".to_owned(),
            ),
        );
        assert!(failed.is_err(), "команда должна отказать");
        assert!(
            history.can_redo(),
            "отказ не должен уничтожать стек повтора"
        );
    }

    use super::{CommandError, CommandStack, ProjectCommand};
    use crate::domain::{
        LayerId, Project, ProjectLayerError, Track, TrackId, TrackLayer, TrackPoint, TrackPointId,
        TrackSegment, TrackSegmentId, Waypoint, WaypointId, WaypointLayer,
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

        assert_eq!(project.track_layers()[1].tracks().len(), 1);
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

        let waypoint = &project.waypoint_layers()[1].waypoints()[0];
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

        assert_eq!(project.waypoint_layers()[1].waypoints()[0].latitude(), 54.1);
        assert!(history.undo(&mut project));
        assert_eq!(project.waypoint_layers()[1].waypoints()[0].latitude(), 53.9);
        assert!(history.redo(&mut project));
        assert_eq!(project.waypoint_layers()[1].waypoints()[0].latitude(), 54.1);
    }

    /// Dragging a mark, letting go, looking at it and dragging it again is two
    /// actions. Merging on "same entity" alone made it one undo step, so a
    /// Ctrl+Z after the second drag went back past the first — a correction
    /// the operator had already accepted. External review, 2026-09-22.
    /// Removing a layer carries the whole layer in the command, and reversing
    /// it used to build `AddTrackLayer { id, name }` — an *empty* layer. A day
    /// of forty crews' recordings would have come back as a name. Nothing
    /// could reach the command from the interface, which is the only reason
    /// this is a fix and not an incident.
    #[test]
    fn a_removed_track_layer_comes_back_with_its_tracks() {
        let mut project = Project::untitled();
        let mut history = CommandStack::default();
        let layer_id = LayerId::new(40);

        history
            .apply(
                &mut project,
                &ProjectCommand::add_track_layer(layer_id, "Day three"),
            )
            .unwrap();
        for n in 0..3u64 {
            let mut track = Track::new(TrackId::new(n + 1), format!("ЛИСА{n}"));
            let mut segment = TrackSegment::new(TrackSegmentId::new(n + 1));
            segment.add_point(TrackPoint::new(TrackPointId::new(n + 1), 53.9, 27.5));
            track.add_segment(segment);
            history
                .apply(&mut project, &ProjectCommand::add_track(layer_id, track))
                .unwrap();
        }

        let live = project
            .track_layers()
            .iter()
            .find(|l| l.id() == layer_id)
            .expect("the layer")
            .clone();
        assert_eq!(live.tracks().len(), 3);

        history
            .apply(
                &mut project,
                &ProjectCommand::RemoveTrackLayer { layer: live },
            )
            .unwrap();
        assert!(
            !project.track_layers().iter().any(|l| l.id() == layer_id),
            "the layer SHALL be gone once removed"
        );

        assert!(history.undo(&mut project));
        let back = project
            .track_layers()
            .iter()
            .find(|l| l.id() == layer_id)
            .expect("the layer SHALL come back");
        assert_eq!(
            back.tracks().len(),
            3,
            "undoing a layer removal SHALL bring its tracks back, not a name"
        );
        assert_eq!(back.name(), "Day three");
        assert_eq!(back.tracks()[1].name(), "ЛИСА1");
    }

    /// A rename aimed at a layer that is not there used to answer `Ok`: the
    /// stack then cleared the redo history and counted a mutation for
    /// something that never happened. Found by an outside reviewer,
    /// 2026-09-23.
    #[test]
    fn renaming_a_layer_that_is_not_there_is_an_error_and_costs_no_redo() {
        let mut project = Project::untitled();
        let mut history = CommandStack::default();
        let layer_id = LayerId::new(77);

        history
            .apply(
                &mut project,
                &ProjectCommand::add_track_layer(layer_id, "Day one"),
            )
            .unwrap();
        assert!(history.undo(&mut project));
        assert!(history.can_redo());

        let refused = history.apply(
            &mut project,
            &ProjectCommand::RenameTrackLayer {
                layer_id: LayerId::new(9999),
                old_name: "nothing".to_owned(),
                new_name: "something".to_owned(),
            },
        );

        assert!(refused.is_err(), "a rename of a missing layer SHALL fail");
        assert!(
            history.can_redo(),
            "a refused rename SHALL NOT take the redo the operator still had"
        );
    }

    /// The import names a layer after the path it came from
    /// (`Imported tracks: /Volumes/…/day3.gpx`). Renaming it is the first
    /// thing anybody does, and it has to come back like any other edit.
    #[test]
    fn renaming_a_layer_is_undoable() {
        let mut project = Project::untitled();
        let mut history = CommandStack::default();
        let layer_id = LayerId::new(42);

        history
            .apply(
                &mut project,
                &ProjectCommand::add_track_layer(layer_id, "Imported tracks: /tmp/day3.gpx"),
            )
            .unwrap();
        history
            .apply(
                &mut project,
                &ProjectCommand::RenameTrackLayer {
                    layer_id,
                    old_name: "Imported tracks: /tmp/day3.gpx".to_owned(),
                    new_name: "День 3".to_owned(),
                },
            )
            .unwrap();

        let name_of = |p: &Project| {
            p.track_layers()
                .iter()
                .find(|l| l.id() == layer_id)
                .map(|l| l.name().to_owned())
                .expect("the layer")
        };
        assert_eq!(name_of(&project), "День 3");

        assert!(history.undo(&mut project));
        assert_eq!(name_of(&project), "Imported tracks: /tmp/day3.gpx");

        assert!(history.redo(&mut project));
        assert_eq!(name_of(&project), "День 3");
    }

    /// The same for the marks: a waypoint layer is where the ШТАБ lives.
    #[test]
    fn a_removed_waypoint_layer_comes_back_with_its_marks() {
        let mut project = Project::untitled();
        let mut history = CommandStack::default();
        let layer_id = LayerId::new(41);

        history
            .apply(
                &mut project,
                &ProjectCommand::add_waypoint_layer(layer_id, "Marks"),
            )
            .unwrap();
        history
            .apply(
                &mut project,
                &ProjectCommand::add_waypoint(
                    layer_id,
                    Waypoint::new(WaypointId::new(7), "ШТАБ", 53.9, 27.5),
                ),
            )
            .unwrap();

        let live = project
            .waypoint_layers()
            .iter()
            .find(|l| l.id() == layer_id)
            .expect("the layer")
            .clone();
        history
            .apply(
                &mut project,
                &ProjectCommand::RemoveWaypointLayer { layer: live },
            )
            .unwrap();
        assert!(history.undo(&mut project));

        let back = project
            .waypoint_layers()
            .iter()
            .find(|l| l.id() == layer_id)
            .expect("the layer SHALL come back");
        assert_eq!(back.waypoints().len(), 1);
        assert_eq!(back.waypoints()[0].name(), "ШТАБ");
    }

    #[test]
    fn two_drags_of_the_same_waypoint_are_two_undo_steps() {
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
        let before_moves = history.undo_history.len();

        history
            .apply_or_merge(
                ProjectCommand::move_waypoint(layer_id, waypoint_id, 54.0, 27.7),
                Some(1),
                &mut project,
            )
            .unwrap();
        history
            .apply_or_merge(
                ProjectCommand::move_waypoint(layer_id, waypoint_id, 54.2, 27.9),
                Some(2),
                &mut project,
            )
            .unwrap();

        assert_eq!(
            history.undo_history.len(),
            before_moves + 2,
            "a second gesture SHALL NOT fold into the first"
        );

        assert!(history.undo(&mut project));
        let waypoint = &project.waypoint_layers()[1].waypoints()[0];
        assert_eq!(
            (waypoint.latitude(), waypoint.longitude()),
            (54.0, 27.7),
            "one undo SHALL take back one drag"
        );
    }

    /// An action nobody tied to a gesture is an action of its own. `apply` —
    /// which is every path but a drag — must never fold two of them together.
    #[test]
    fn commands_outside_a_gesture_never_merge() {
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
        let before_moves = history.undo_history.len();

        history
            .apply(
                &mut project,
                &ProjectCommand::move_waypoint(layer_id, waypoint_id, 54.0, 27.7),
            )
            .unwrap();
        history
            .apply(
                &mut project,
                &ProjectCommand::move_waypoint(layer_id, waypoint_id, 54.2, 27.9),
            )
            .unwrap();

        assert_eq!(history.undo_history.len(), before_moves + 2);
        assert!(history.undo(&mut project));
        let waypoint = &project.waypoint_layers()[1].waypoints()[0];
        assert_eq!((waypoint.latitude(), waypoint.longitude()), (54.0, 27.7));
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
                Some(1),
                &mut project,
            )
            .unwrap();
        history
            .apply_or_merge(
                ProjectCommand::move_waypoint(layer_id, waypoint_id, 54.2, 27.9),
                Some(1),
                &mut project,
            )
            .unwrap();

        assert_eq!(history.undo_history.len(), 3);
        assert!(history.undo(&mut project));
        let waypoint = &project.waypoint_layers()[1].waypoints()[0];
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
            .apply(
                &mut project,
                &ProjectCommand::add_track_layer(layer_id, "Tracks"),
            )
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

        let point = &project.track_layers()[1].tracks()[0].segments()[0].points()[0];
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
            .apply(
                &mut project,
                &ProjectCommand::add_track_layer(layer_id, "Tracks"),
            )
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
        let point = &project.track_layers()[1].tracks()[0].segments()[0].points()[0];
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
            .apply(
                &mut project,
                &ProjectCommand::add_track_layer(layer_id, "Tracks"),
            )
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
                Some(7),
                &mut project,
            )
            .unwrap();

        history
            .apply_or_merge(
                ProjectCommand::move_track_point(
                    layer_id, track_id, segment_id, point_id, 54.2, 27.9, 53.9, 27.5667,
                ),
                Some(7),
                &mut project,
            )
            .unwrap();

        assert_eq!(history.undo_history.len(), 3);
        assert!(history.undo(&mut project));
        let point = &project.track_layers()[1].tracks()[0].segments()[0].points()[0];
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
            .apply(
                &mut project,
                &ProjectCommand::add_track_layer(layer_id, "Tracks"),
            )
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

        let points = project.track_layers()[1].tracks()[0].segments()[0].points();
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
            .apply(
                &mut project,
                &ProjectCommand::add_track_layer(layer_id, "Tracks"),
            )
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

        let points = project.track_layers()[1].tracks()[0].segments()[0].points();
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
            .apply(
                &mut project,
                &ProjectCommand::add_track_layer(layer_id, "Tracks"),
            )
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
            .apply(
                &mut project,
                &ProjectCommand::add_track_layer(layer_id, "Tracks"),
            )
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

        let points = project.track_layers()[1].tracks()[0].segments()[0].points();
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
            .apply(
                &mut project,
                &ProjectCommand::add_track_layer(layer_id, "Tracks"),
            )
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
        let points = project.track_layers()[1].tracks()[0].segments()[0].points();
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
            .apply(
                &mut project,
                &ProjectCommand::add_track_layer(layer_id, "Tracks"),
            )
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
    fn split_segment_apply_creates_two_segments() {
        let mut project = Project::untitled();
        let mut history = CommandStack::default();
        let layer_id = LayerId::new(20);
        let track_id = TrackId::new(1);
        let segment_id = TrackSegmentId::new(2);
        let new_segment_id = TrackSegmentId::new(99);

        history
            .apply(
                &mut project,
                &ProjectCommand::add_track_layer(layer_id, "Tracks"),
            )
            .unwrap();

        let mut track = Track::new(track_id, "Morning route");
        let mut segment = TrackSegment::new(segment_id);
        segment.add_point(TrackPoint::new(TrackPointId::new(10), 53.9, 27.5));
        segment.add_point(TrackPoint::new(TrackPointId::new(11), 54.0, 27.6));
        segment.add_point(TrackPoint::new(TrackPointId::new(12), 54.1, 27.7));
        track.add_segment(segment);

        history
            .apply(&mut project, &ProjectCommand::add_track(layer_id, track))
            .unwrap();

        history
            .apply(
                &mut project,
                &ProjectCommand::split_segment(
                    layer_id,
                    track_id,
                    segment_id,
                    TrackPointId::new(11),
                    new_segment_id,
                ),
            )
            .unwrap();

        let segments = project.track_layers()[1].tracks()[0].segments();
        assert_eq!(segments.len(), 2);
        assert_eq!(segments[0].id(), segment_id);
        assert_eq!(segments[1].id(), new_segment_id);
    }

    #[test]
    fn split_segment_undo_merges_back_to_one() {
        let mut project = Project::untitled();
        let mut history = CommandStack::default();
        let layer_id = LayerId::new(20);
        let track_id = TrackId::new(1);
        let segment_id = TrackSegmentId::new(2);

        history
            .apply(
                &mut project,
                &ProjectCommand::add_track_layer(layer_id, "Tracks"),
            )
            .unwrap();

        let mut track = Track::new(track_id, "Morning route");
        let mut segment = TrackSegment::new(segment_id);
        segment.add_point(TrackPoint::new(TrackPointId::new(10), 53.9, 27.5));
        segment.add_point(TrackPoint::new(TrackPointId::new(11), 54.0, 27.6));
        segment.add_point(TrackPoint::new(TrackPointId::new(12), 54.1, 27.7));
        track.add_segment(segment);

        history
            .apply(&mut project, &ProjectCommand::add_track(layer_id, track))
            .unwrap();

        history
            .apply(
                &mut project,
                &ProjectCommand::split_segment(
                    layer_id,
                    track_id,
                    segment_id,
                    TrackPointId::new(11),
                    TrackSegmentId::new(99),
                ),
            )
            .unwrap();

        assert!(history.undo(&mut project));
        let segments = project.track_layers()[1].tracks()[0].segments();
        assert_eq!(segments.len(), 1);
        let ids: Vec<u64> = segments[0]
            .points()
            .iter()
            .map(|p| p.id().value())
            .collect();
        assert_eq!(ids, vec![10, 11, 12]);
    }

    #[test]
    fn split_segment_apply_does_not_duplicate_points() {
        let mut project = Project::untitled();
        let mut history = CommandStack::default();
        let layer_id = LayerId::new(20);
        let track_id = TrackId::new(1);
        let segment_id = TrackSegmentId::new(2);

        history
            .apply(
                &mut project,
                &ProjectCommand::add_track_layer(layer_id, "Tracks"),
            )
            .unwrap();

        let mut track = Track::new(track_id, "Morning route");
        let mut segment = TrackSegment::new(segment_id);
        segment.add_point(TrackPoint::new(TrackPointId::new(10), 53.9, 27.5));
        segment.add_point(TrackPoint::new(TrackPointId::new(11), 54.0, 27.6));
        segment.add_point(TrackPoint::new(TrackPointId::new(12), 54.1, 27.7));
        track.add_segment(segment);

        history
            .apply(&mut project, &ProjectCommand::add_track(layer_id, track))
            .unwrap();

        history
            .apply(
                &mut project,
                &ProjectCommand::split_segment(
                    layer_id,
                    track_id,
                    segment_id,
                    TrackPointId::new(11),
                    TrackSegmentId::new(99),
                ),
            )
            .unwrap();

        let segments = project.track_layers()[1].tracks()[0].segments();
        assert_eq!(segments.len(), 2);
        let left_ids: Vec<u64> = segments[0]
            .points()
            .iter()
            .map(|p| p.id().value())
            .collect();
        let right_ids: Vec<u64> = segments[1]
            .points()
            .iter()
            .map(|p| p.id().value())
            .collect();
        assert_eq!(left_ids, vec![10, 11]);
        assert_eq!(right_ids, vec![12]);
    }

    #[test]
    fn split_undo_redo_undo_remains_stable() {
        let mut project = Project::untitled();
        let mut history = CommandStack::default();
        let layer_id = LayerId::new(20);
        let track_id = TrackId::new(1);
        let segment_id = TrackSegmentId::new(2);

        history
            .apply(
                &mut project,
                &ProjectCommand::add_track_layer(layer_id, "Tracks"),
            )
            .unwrap();

        let mut track = Track::new(track_id, "Morning route");
        let mut segment = TrackSegment::new(segment_id);
        segment.add_point(TrackPoint::new(TrackPointId::new(10), 53.9, 27.5));
        segment.add_point(TrackPoint::new(TrackPointId::new(11), 54.0, 27.6));
        segment.add_point(TrackPoint::new(TrackPointId::new(12), 54.1, 27.7));
        track.add_segment(segment);

        history
            .apply(&mut project, &ProjectCommand::add_track(layer_id, track))
            .unwrap();

        history
            .apply(
                &mut project,
                &ProjectCommand::split_segment(
                    layer_id,
                    track_id,
                    segment_id,
                    TrackPointId::new(11),
                    TrackSegmentId::new(99),
                ),
            )
            .unwrap();

        assert!(history.undo(&mut project));
        assert!(history.redo(&mut project));
        assert!(history.undo(&mut project));

        let segments = project.track_layers()[1].tracks()[0].segments();
        assert_eq!(segments.len(), 1);
        let ids: Vec<u64> = segments[0]
            .points()
            .iter()
            .map(|p| p.id().value())
            .collect();
        assert_eq!(ids, vec![10, 11, 12]);
    }

    #[test]
    fn split_segment_missing_point_returns_error() {
        let mut project = Project::untitled();
        let mut history = CommandStack::default();
        let layer_id = LayerId::new(20);
        let track_id = TrackId::new(1);
        let segment_id = TrackSegmentId::new(2);

        history
            .apply(
                &mut project,
                &ProjectCommand::add_track_layer(layer_id, "Tracks"),
            )
            .unwrap();

        let mut track = Track::new(track_id, "Morning route");
        let mut segment = TrackSegment::new(segment_id);
        segment.add_point(TrackPoint::new(TrackPointId::new(10), 53.9, 27.5));
        segment.add_point(TrackPoint::new(TrackPointId::new(11), 54.0, 27.6));
        track.add_segment(segment);

        history
            .apply(&mut project, &ProjectCommand::add_track(layer_id, track))
            .unwrap();

        let error = history
            .apply(
                &mut project,
                &ProjectCommand::split_segment(
                    layer_id,
                    track_id,
                    segment_id,
                    TrackPointId::new(99),
                    TrackSegmentId::new(100),
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
    fn join_segments_apply_combines_into_one() {
        let mut project = Project::untitled();
        let mut history = CommandStack::default();
        let layer_id = LayerId::new(20);
        let track_id = TrackId::new(1);

        history
            .apply(
                &mut project,
                &ProjectCommand::add_track_layer(layer_id, "Tracks"),
            )
            .unwrap();

        let mut track = Track::new(track_id, "Morning route");
        let mut segment_a = TrackSegment::new(TrackSegmentId::new(10));
        segment_a.add_point(TrackPoint::new(TrackPointId::new(1), 53.9, 27.5));
        segment_a.add_point(TrackPoint::new(TrackPointId::new(2), 54.0, 27.6));
        let mut segment_b = TrackSegment::new(TrackSegmentId::new(20));
        segment_b.add_point(TrackPoint::new(TrackPointId::new(3), 54.1, 27.7));
        segment_b.add_point(TrackPoint::new(TrackPointId::new(4), 54.2, 27.8));
        track.add_segment(segment_a);
        track.add_segment(segment_b);

        history
            .apply(&mut project, &ProjectCommand::add_track(layer_id, track))
            .unwrap();

        history
            .apply(
                &mut project,
                &ProjectCommand::join_segments(
                    layer_id,
                    track_id,
                    TrackSegmentId::new(10),
                    TrackSegmentId::new(20),
                ),
            )
            .unwrap();

        let segments = project.track_layers()[1].tracks()[0].segments();
        assert_eq!(segments.len(), 1);
        assert_eq!(segments[0].id(), TrackSegmentId::new(10));
        assert_eq!(segments[0].points().len(), 4);
    }

    #[test]
    fn join_segments_undo_restores_both() {
        let mut project = Project::untitled();
        let mut history = CommandStack::default();
        let layer_id = LayerId::new(20);
        let track_id = TrackId::new(1);

        history
            .apply(
                &mut project,
                &ProjectCommand::add_track_layer(layer_id, "Tracks"),
            )
            .unwrap();

        let mut track = Track::new(track_id, "Morning route");
        let mut segment_a = TrackSegment::new(TrackSegmentId::new(10));
        segment_a.add_point(TrackPoint::new(TrackPointId::new(1), 53.9, 27.5));
        segment_a.add_point(TrackPoint::new(TrackPointId::new(2), 54.0, 27.6));
        let mut segment_b = TrackSegment::new(TrackSegmentId::new(20));
        segment_b.add_point(TrackPoint::new(TrackPointId::new(3), 54.1, 27.7));
        segment_b.add_point(TrackPoint::new(TrackPointId::new(4), 54.2, 27.8));
        track.add_segment(segment_a);
        track.add_segment(segment_b);

        history
            .apply(&mut project, &ProjectCommand::add_track(layer_id, track))
            .unwrap();

        history
            .apply(
                &mut project,
                &ProjectCommand::join_segments(
                    layer_id,
                    track_id,
                    TrackSegmentId::new(10),
                    TrackSegmentId::new(20),
                ),
            )
            .unwrap();

        assert!(history.undo(&mut project));
        let segments = project.track_layers()[1].tracks()[0].segments();
        assert_eq!(segments.len(), 2);
        assert_eq!(segments[0].id(), TrackSegmentId::new(10));
        assert_eq!(segments[1].id(), TrackSegmentId::new(20));
        let a_ids: Vec<u64> = segments[0]
            .points()
            .iter()
            .map(|p| p.id().value())
            .collect();
        let b_ids: Vec<u64> = segments[1]
            .points()
            .iter()
            .map(|p| p.id().value())
            .collect();
        assert_eq!(a_ids, vec![1, 2]);
        assert_eq!(b_ids, vec![3, 4]);
    }

    #[test]
    fn join_segments_error_on_missing_segment() {
        let mut project = Project::untitled();
        let mut history = CommandStack::default();
        let layer_id = LayerId::new(20);
        let track_id = TrackId::new(1);

        history
            .apply(
                &mut project,
                &ProjectCommand::add_track_layer(layer_id, "Tracks"),
            )
            .unwrap();

        let mut track = Track::new(track_id, "Morning route");
        let mut segment_a = TrackSegment::new(TrackSegmentId::new(10));
        segment_a.add_point(TrackPoint::new(TrackPointId::new(1), 53.9, 27.5));
        segment_a.add_point(TrackPoint::new(TrackPointId::new(2), 54.0, 27.6));
        track.add_segment(segment_a);

        history
            .apply(&mut project, &ProjectCommand::add_track(layer_id, track))
            .unwrap();

        let error = history
            .apply(
                &mut project,
                &ProjectCommand::join_segments(
                    layer_id,
                    track_id,
                    TrackSegmentId::new(10),
                    TrackSegmentId::new(99),
                ),
            )
            .unwrap_err();

        assert_eq!(
            error,
            CommandError::ProjectLayer(ProjectLayerError::MissingTrackSegment {
                layer_id: layer_id.value(),
                track_id: track_id.value(),
                segment_id: 99,
            })
        );
    }

    #[test]
    fn delete_track_apply_removes_track() {
        let mut project = Project::untitled();
        let mut history = CommandStack::default();
        let layer_id = LayerId::new(20);
        let track_id = TrackId::new(1);

        history
            .apply(
                &mut project,
                &ProjectCommand::add_track_layer(layer_id, "Tracks"),
            )
            .unwrap();
        history
            .apply(
                &mut project,
                &ProjectCommand::add_track(layer_id, Track::new(track_id, "Morning route")),
            )
            .unwrap();

        history
            .apply(
                &mut project,
                &ProjectCommand::delete_track(layer_id, track_id),
            )
            .unwrap();

        assert!(project.track_layers()[1].tracks().is_empty());
    }

    #[test]
    fn delete_track_undo_restores_track() {
        let mut project = Project::untitled();
        let mut history = CommandStack::default();
        let layer_id = LayerId::new(20);
        let track_id = TrackId::new(1);

        history
            .apply(
                &mut project,
                &ProjectCommand::add_track_layer(layer_id, "Tracks"),
            )
            .unwrap();
        history
            .apply(
                &mut project,
                &ProjectCommand::add_track(layer_id, Track::new(track_id, "Morning route")),
            )
            .unwrap();

        history
            .apply(
                &mut project,
                &ProjectCommand::delete_track(layer_id, track_id),
            )
            .unwrap();

        assert!(history.undo(&mut project));
        assert_eq!(project.track_layers()[1].tracks().len(), 1);
        assert_eq!(project.track_layers()[1].tracks()[0].id(), track_id);
    }

    #[test]
    fn delete_track_missing_track_returns_error() {
        let mut project = Project::untitled();
        let mut history = CommandStack::default();
        let layer_id = LayerId::new(20);

        history
            .apply(
                &mut project,
                &ProjectCommand::add_track_layer(layer_id, "Tracks"),
            )
            .unwrap();

        let error = history
            .apply(
                &mut project,
                &ProjectCommand::delete_track(layer_id, TrackId::new(99)),
            )
            .unwrap_err();

        assert_eq!(
            error,
            CommandError::ProjectLayer(ProjectLayerError::MissingTrack {
                layer_id: layer_id.value(),
                track_id: 99,
            })
        );
    }

    #[test]
    fn delete_waypoint_apply_removes_waypoint() {
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
                &ProjectCommand::delete_waypoint(layer_id, waypoint_id),
            )
            .unwrap();

        assert!(project.waypoint_layers()[1].waypoints().is_empty());
    }

    #[test]
    fn delete_waypoint_undo_restores_with_all_fields() {
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
                &ProjectCommand::delete_waypoint(layer_id, waypoint_id),
            )
            .unwrap();

        assert!(history.undo(&mut project));
        assert_eq!(project.waypoint_layers()[1].waypoints().len(), 1);
        let restored = &project.waypoint_layers()[1].waypoints()[0];
        assert_eq!(restored.id(), waypoint_id);
        assert_eq!(restored.name(), "Camp");
        assert_eq!(restored.latitude(), 53.9);
        assert_eq!(restored.longitude(), 27.5667);
    }

    #[test]
    fn delete_waypoint_missing_waypoint_returns_error() {
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
                &ProjectCommand::delete_waypoint(layer_id, WaypointId::new(99)),
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
    fn rename_waypoint_apply_changes_name() {
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
                &ProjectCommand::rename_waypoint(layer_id, waypoint_id, "Camp", "Base camp"),
            )
            .unwrap();

        assert_eq!(
            project.waypoint_layers()[1].waypoints()[0].name(),
            "Base camp"
        );
    }

    #[test]
    fn rename_waypoint_undo_restores_old_name() {
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
                &ProjectCommand::rename_waypoint(layer_id, waypoint_id, "Camp", "Base camp"),
            )
            .unwrap();

        assert!(history.undo(&mut project));
        assert_eq!(project.waypoint_layers()[1].waypoints()[0].name(), "Camp");
    }

    #[test]
    fn simplify_track_apply_reduces_point_count() {
        let mut project = Project::untitled();
        let mut history = CommandStack::default();
        let layer_id = LayerId::new(20);
        let track_id = TrackId::new(1);
        let segment_id = TrackSegmentId::new(2);

        history
            .apply(
                &mut project,
                &ProjectCommand::add_track_layer(layer_id, "Tracks"),
            )
            .unwrap();

        let mut track = Track::new(track_id, "Morning route");
        let mut segment = TrackSegment::new(segment_id);
        segment.add_point(TrackPoint::new(TrackPointId::new(1), 55.0, 37.0));
        segment.add_point(TrackPoint::new(TrackPointId::new(2), 55.025, 37.025));
        segment.add_point(TrackPoint::new(TrackPointId::new(3), 55.05, 37.05));
        segment.add_point(TrackPoint::new(TrackPointId::new(4), 55.075, 37.075));
        segment.add_point(TrackPoint::new(TrackPointId::new(5), 55.1, 37.1));
        track.add_segment(segment);

        history
            .apply(&mut project, &ProjectCommand::add_track(layer_id, track))
            .unwrap();

        history
            .apply(
                &mut project,
                &ProjectCommand::simplify_track(layer_id, track_id, 0.001),
            )
            .unwrap();

        let points = project.track_layers()[1].tracks()[0].segments()[0].points();
        assert_eq!(points.len(), 2);
    }

    #[test]
    fn simplify_track_undo_restores_all_points() {
        let mut project = Project::untitled();
        let mut history = CommandStack::default();
        let layer_id = LayerId::new(20);
        let track_id = TrackId::new(1);
        let segment_id = TrackSegmentId::new(2);

        history
            .apply(
                &mut project,
                &ProjectCommand::add_track_layer(layer_id, "Tracks"),
            )
            .unwrap();

        let mut track = Track::new(track_id, "Morning route");
        let mut segment = TrackSegment::new(segment_id);
        segment.add_point(TrackPoint::new(TrackPointId::new(1), 55.0, 37.0));
        segment.add_point(TrackPoint::new(TrackPointId::new(2), 55.025, 37.025));
        segment.add_point(TrackPoint::new(TrackPointId::new(3), 55.05, 37.05));
        segment.add_point(TrackPoint::new(TrackPointId::new(4), 55.075, 37.075));
        segment.add_point(TrackPoint::new(TrackPointId::new(5), 55.1, 37.1));
        track.add_segment(segment);

        history
            .apply(&mut project, &ProjectCommand::add_track(layer_id, track))
            .unwrap();

        history
            .apply(
                &mut project,
                &ProjectCommand::simplify_track(layer_id, track_id, 0.001),
            )
            .unwrap();

        assert!(history.undo(&mut project));
        let points = project.track_layers()[1].tracks()[0].segments()[0].points();
        assert_eq!(points.len(), 5);
        assert_eq!(points[1].id(), TrackPointId::new(2));
        assert_eq!(points[3].id(), TrackPointId::new(4));
    }

    #[test]
    fn simplify_track_zero_tolerance_keeps_all() {
        let mut project = Project::untitled();
        let mut history = CommandStack::default();
        let layer_id = LayerId::new(20);
        let track_id = TrackId::new(1);
        let segment_id = TrackSegmentId::new(2);

        history
            .apply(
                &mut project,
                &ProjectCommand::add_track_layer(layer_id, "Tracks"),
            )
            .unwrap();

        let mut track = Track::new(track_id, "Morning route");
        let mut segment = TrackSegment::new(segment_id);
        segment.add_point(TrackPoint::new(TrackPointId::new(1), 55.0, 37.0));
        segment.add_point(TrackPoint::new(TrackPointId::new(2), 55.025, 37.025));
        segment.add_point(TrackPoint::new(TrackPointId::new(3), 55.05, 37.05));
        track.add_segment(segment);

        history
            .apply(&mut project, &ProjectCommand::add_track(layer_id, track))
            .unwrap();

        history
            .apply(
                &mut project,
                &ProjectCommand::simplify_track(layer_id, track_id, 0.0),
            )
            .unwrap();

        let points = project.track_layers()[1].tracks()[0].segments()[0].points();
        assert_eq!(points.len(), 3);
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

    #[test]
    fn create_empty_track_apply_creates_track_with_one_empty_segment() {
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
                &ProjectCommand::create_empty_track(layer_id, TrackId::new(1), "New Track"),
            )
            .unwrap();

        let tracks = project.track_layers()[1].tracks();
        assert_eq!(tracks.len(), 1);
        assert_eq!(tracks[0].id(), TrackId::new(1));
        assert_eq!(tracks[0].name(), "New Track");
        assert_eq!(tracks[0].segments().len(), 1);
        assert_eq!(tracks[0].segments()[0].points().len(), 0);
    }

    #[test]
    fn create_empty_track_undo_removes_created_track() {
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
                &ProjectCommand::create_empty_track(layer_id, TrackId::new(1), "New Track"),
            )
            .unwrap();

        assert_eq!(project.track_layers()[1].tracks().len(), 1);

        history.undo(&mut project);

        assert_eq!(project.track_layers()[1].tracks().len(), 0);
    }

    #[test]
    fn move_track_point_missing_layer_returns_error() {
        let mut project = Project::untitled();
        let mut history = CommandStack::default();
        let result = history.apply(
            &mut project,
            &ProjectCommand::move_track_point(
                LayerId::new(999),
                TrackId::new(1),
                TrackSegmentId::new(1),
                TrackPointId::new(1),
                55.0,
                37.0,
                55.0,
                37.0,
            ),
        );
        assert!(result.is_err(), "expected Err when layer does not exist");
    }

    #[test]
    fn rename_waypoint_missing_layer_returns_error() {
        let mut project = Project::untitled();
        let mut history = CommandStack::default();
        let result = history.apply(
            &mut project,
            &ProjectCommand::rename_waypoint(LayerId::new(999), WaypointId::new(1), "Old", "New"),
        );
        assert!(
            result.is_err(),
            "expected Err when waypoint layer does not exist"
        );
    }

    #[test]
    fn simplify_track_missing_track_returns_error() {
        let mut project = Project::untitled();
        let mut history = CommandStack::default();
        let cmd = ProjectCommand::SimplifyTrack {
            layer_id: LayerId::new(999),
            track_id: TrackId::new(1),
            tolerance_km: 0.001,
            removed: vec![(
                TrackSegmentId::new(1),
                0,
                TrackPoint::new(TrackPointId::new(1), 55.0, 37.0),
            )],
        };
        let result = history.apply(&mut project, &cmd);
        assert!(
            result.is_err(),
            "expected Err when layer does not exist and removed is non-empty"
        );
    }

    #[test]
    fn create_empty_track_missing_layer_returns_error() {
        let mut project = Project::untitled();
        let mut history = CommandStack::default();
        let result = history.apply(
            &mut project,
            &ProjectCommand::create_empty_track(LayerId::new(999), TrackId::new(1), "Ghost Track"),
        );
        assert!(
            result.is_err(),
            "expected Err when track layer does not exist"
        );
    }
}
