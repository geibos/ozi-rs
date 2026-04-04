#![allow(dead_code)]

use crate::domain::{
    Track, TrackPoint, TrackPointId, TrackSegment, TrackSegmentId, Waypoint, WaypointId,
};
use std::fmt;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProjectLayerError {
    TrackLayerNotPresent(LayerId),
    WaypointLayerUnavailable(LayerId),
    WaypointNotFound(LayerId, WaypointId),
    MissingTrack {
        layer_id: u64,
        track_id: u64,
    },
    MissingTrackSegment {
        layer_id: u64,
        track_id: u64,
        segment_id: u64,
    },
    MissingTrackPoint {
        layer_id: u64,
        track_id: u64,
        segment_id: u64,
        point_id: u64,
    },
}

impl fmt::Display for ProjectLayerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::TrackLayerNotPresent(layer_id) => {
                write!(f, "missing track layer with id {}", layer_id.value())
            }
            Self::WaypointLayerUnavailable(layer_id) => {
                write!(f, "missing waypoint layer with id {}", layer_id.value())
            }
            Self::WaypointNotFound(layer_id, waypoint_id) => write!(
                f,
                "missing waypoint with id {} in layer {}",
                waypoint_id.value(),
                layer_id.value()
            ),
            Self::MissingTrack { layer_id, track_id } => {
                write!(
                    f,
                    "missing track with id {} in layer {}",
                    track_id, layer_id
                )
            }
            Self::MissingTrackSegment {
                layer_id,
                track_id,
                segment_id,
            } => write!(
                f,
                "missing track segment with id {} in track {} in layer {}",
                segment_id, track_id, layer_id
            ),
            Self::MissingTrackPoint {
                layer_id,
                track_id,
                segment_id,
                point_id,
            } => write!(
                f,
                "missing track point with id {} in segment {} in track {} in layer {}",
                point_id, segment_id, track_id, layer_id
            ),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[serde(transparent)]
pub struct ProjectId(u64);

impl ProjectId {
    pub const fn new(value: u64) -> Self {
        Self(value)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[serde(transparent)]
pub struct LayerId(u64);

impl LayerId {
    pub const fn new(value: u64) -> Self {
        Self(value)
    }

    pub const fn value(&self) -> u64 {
        self.0
    }
}

pub trait LayerIdLike {
    fn into_u64(self) -> u64;
}

impl LayerIdLike for u64 {
    fn into_u64(self) -> u64 {
        self
    }
}

impl LayerIdLike for LayerId {
    fn into_u64(self) -> u64 {
        self.value()
    }
}

pub trait WaypointIdLike {
    fn into_u64(self) -> u64;
}

impl WaypointIdLike for u64 {
    fn into_u64(self) -> u64 {
        self
    }
}

impl WaypointIdLike for WaypointId {
    fn into_u64(self) -> u64 {
        self.value()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct MapLayer {
    id: LayerId,
    name: String,
    source_path: Option<PathBuf>,
}

impl MapLayer {
    pub fn new(id: LayerId, name: impl Into<String>) -> Self {
        Self::with_source_path(id, name, None::<PathBuf>)
    }

    pub fn with_source_path(
        id: LayerId,
        name: impl Into<String>,
        source_path: impl Into<Option<PathBuf>>,
    ) -> Self {
        Self {
            id,
            name: name.into(),
            source_path: source_path.into(),
        }
    }

    pub const fn id(&self) -> LayerId {
        self.id
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn source_path(&self) -> Option<&Path> {
        self.source_path.as_deref()
    }
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct TrackLayer {
    id: LayerId,
    name: String,
    tracks: Vec<Track>,
}

impl TrackLayer {
    pub fn new(id: LayerId, name: impl Into<String>) -> Self {
        Self {
            id,
            name: name.into(),
            tracks: Vec::new(),
        }
    }

    pub const fn id(&self) -> LayerId {
        self.id
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn tracks(&self) -> &[Track] {
        &self.tracks
    }

    pub fn add_track(&mut self, track: Track) {
        self.tracks.push(track);
    }

    pub fn remove_track(
        &mut self,
        track_id: crate::domain::TrackId,
    ) -> Result<(usize, Track), ProjectLayerError> {
        let Some(index) = self.tracks.iter().position(|track| track.id() == track_id) else {
            return Err(ProjectLayerError::MissingTrack {
                layer_id: self.id.value(),
                track_id: track_id.value(),
            });
        };

        Ok((index, self.tracks.remove(index)))
    }

    pub fn track_mut(&mut self, track_id: crate::domain::TrackId) -> Option<&mut Track> {
        self.tracks.iter_mut().find(|t| t.id() == track_id)
    }

    pub fn set_track_visible(&mut self, track_id: crate::domain::TrackId, visible: bool) {
        if let Some(track) = self.tracks.iter_mut().find(|t| t.id() == track_id) {
            track.style_mut().visible = visible;
        }
    }

    pub fn create_empty_track(&mut self, track_id: crate::domain::TrackId, name: String) {
        let segment = crate::domain::TrackSegment::new(crate::domain::TrackSegmentId::new(1));
        let mut track = crate::domain::Track::new(track_id, name);
        track.add_segment(segment);
        self.tracks.push(track);
    }
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct WaypointLayer {
    id: LayerId,
    name: String,
    waypoints: Vec<Waypoint>,
}

impl WaypointLayer {
    pub fn new(id: LayerId, name: impl Into<String>) -> Self {
        Self {
            id,
            name: name.into(),
            waypoints: Vec::new(),
        }
    }

    pub const fn id(&self) -> LayerId {
        self.id
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn waypoints(&self) -> &[Waypoint] {
        &self.waypoints
    }

    pub fn add_waypoint(&mut self, waypoint: Waypoint) {
        self.waypoints.push(waypoint);
    }

    pub fn remove_waypoint(
        &mut self,
        waypoint_id: u64,
    ) -> Result<(usize, Waypoint), ProjectLayerError> {
        let Some(index) = self
            .waypoints
            .iter()
            .position(|waypoint| waypoint.id().value() == waypoint_id)
        else {
            return Err(ProjectLayerError::WaypointNotFound(
                self.id,
                WaypointId::new(waypoint_id),
            ));
        };

        Ok((index, self.waypoints.remove(index)))
    }

    pub fn rename_waypoint(
        &mut self,
        waypoint_id: u64,
        new_name: String,
    ) -> Result<String, ProjectLayerError> {
        let Some(waypoint) = self
            .waypoints
            .iter_mut()
            .find(|waypoint| waypoint.id().value() == waypoint_id)
        else {
            return Err(ProjectLayerError::WaypointNotFound(
                self.id,
                WaypointId::new(waypoint_id),
            ));
        };

        Ok(waypoint.set_name(new_name))
    }

    pub fn set_waypoint_symbol(
        &mut self,
        waypoint_id: u64,
        symbol: Option<String>,
    ) -> Result<Option<String>, ProjectLayerError> {
        let Some(waypoint) = self
            .waypoints
            .iter_mut()
            .find(|waypoint| waypoint.id().value() == waypoint_id)
        else {
            return Err(ProjectLayerError::WaypointNotFound(
                self.id,
                WaypointId::new(waypoint_id),
            ));
        };

        Ok(waypoint.set_symbol(symbol))
    }

    pub fn move_waypoint(
        &mut self,
        waypoint_id: WaypointId,
        latitude: f64,
        longitude: f64,
    ) -> bool {
        let Some(waypoint) = self
            .waypoints
            .iter_mut()
            .find(|waypoint| waypoint.id() == waypoint_id)
        else {
            return false;
        };

        waypoint.move_to(latitude, longitude);
        true
    }
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Project {
    id: ProjectId,
    name: String,
    map_layers: Vec<MapLayer>,
    track_layers: Vec<TrackLayer>,
    waypoint_layers: Vec<WaypointLayer>,
}

impl Project {
    pub fn untitled() -> Self {
        Self {
            id: ProjectId::new(1),
            name: "Untitled Project".to_owned(),
            map_layers: Vec::new(),
            track_layers: Vec::new(),
            waypoint_layers: Vec::new(),
        }
    }

    pub const fn id(&self) -> ProjectId {
        self.id
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn map_layers(&self) -> &[MapLayer] {
        &self.map_layers
    }

    pub fn track_layers(&self) -> &[TrackLayer] {
        &self.track_layers
    }

    pub fn waypoint_layers(&self) -> &[WaypointLayer] {
        &self.waypoint_layers
    }

    pub fn add_map_layer(&mut self, layer: MapLayer) {
        self.map_layers.push(layer);
    }

    pub fn remove_map_layer(&mut self, layer_id: LayerId) -> bool {
        let Some(index) = self
            .map_layers
            .iter()
            .position(|layer| layer.id() == layer_id)
        else {
            return false;
        };

        self.map_layers.remove(index);
        true
    }

    pub fn add_track_layer(&mut self, layer: TrackLayer) {
        self.track_layers.push(layer);
    }

    pub fn remove_track_layer(&mut self, layer_id: LayerId) -> bool {
        let Some(index) = self
            .track_layers
            .iter()
            .position(|layer| layer.id() == layer_id)
        else {
            return false;
        };

        self.track_layers.remove(index);
        true
    }

    pub fn add_waypoint_layer(&mut self, layer: WaypointLayer) {
        self.waypoint_layers.push(layer);
    }

    pub fn remove_waypoint_layer(&mut self, layer_id: LayerId) -> bool {
        let Some(index) = self
            .waypoint_layers
            .iter()
            .position(|layer| layer.id() == layer_id)
        else {
            return false;
        };

        self.waypoint_layers.remove(index);
        true
    }

    pub fn track_layer_mut(&mut self, layer_id: u64) -> Result<&mut TrackLayer, ProjectLayerError> {
        self.track_layers
            .iter_mut()
            .find(|layer| layer.id().value() == layer_id)
            .ok_or(ProjectLayerError::TrackLayerNotPresent(LayerId::new(
                layer_id,
            )))
    }

    pub fn track_mut(
        &mut self,
        layer_id: u64,
        track_id: u64,
    ) -> Result<&mut Track, ProjectLayerError> {
        let layer = self.track_layer_mut(layer_id)?;
        layer
            .track_mut(crate::domain::TrackId::new(track_id))
            .ok_or(ProjectLayerError::MissingTrack { layer_id, track_id })
    }

    pub fn track_segment_mut(
        &mut self,
        layer_id: u64,
        track_id: u64,
        segment_id: u64,
    ) -> Result<&mut TrackSegment, ProjectLayerError> {
        let track = self.track_mut(layer_id, track_id)?;
        track.segment_mut(TrackSegmentId::new(segment_id)).ok_or(
            ProjectLayerError::MissingTrackSegment {
                layer_id,
                track_id,
                segment_id,
            },
        )
    }

    pub fn track_point_mut(
        &mut self,
        layer_id: u64,
        track_id: u64,
        segment_id: u64,
        point_id: u64,
    ) -> Result<&mut TrackPoint, ProjectLayerError> {
        let segment = self.track_segment_mut(layer_id, track_id, segment_id)?;
        segment
            .point_mut(TrackPointId::new(point_id))
            .ok_or(ProjectLayerError::MissingTrackPoint {
                layer_id,
                track_id,
                segment_id,
                point_id,
            })
    }

    pub fn set_track_visible_in_layer(
        &mut self,
        layer_id: LayerId,
        track_id: crate::domain::TrackId,
        visible: bool,
    ) {
        if let Some(layer) = self.track_layers.iter_mut().find(|l| l.id() == layer_id) {
            layer.set_track_visible(track_id, visible);
        }
    }

    pub fn add_track_to_layer(
        &mut self,
        layer_id: LayerId,
        track: Track,
    ) -> Result<(), ProjectLayerError> {
        self.track_layer_mut(layer_id.value())?.add_track(track);
        Ok(())
    }

    pub fn remove_track_from_layer(
        &mut self,
        layer_id: LayerId,
        track_id: crate::domain::TrackId,
    ) -> Result<(usize, Track), ProjectLayerError> {
        let layer = self.track_layer_mut(layer_id.value())?;
        layer.remove_track(track_id)
    }

    pub fn add_waypoint_to_layer(
        &mut self,
        layer_id: LayerId,
        waypoint: Waypoint,
    ) -> Result<(), ProjectLayerError> {
        let Some(layer) = self
            .waypoint_layers
            .iter_mut()
            .find(|layer| layer.id() == layer_id)
        else {
            return Err(ProjectLayerError::WaypointLayerUnavailable(layer_id));
        };

        layer.add_waypoint(waypoint);
        Ok(())
    }

    pub fn remove_waypoint_from_layer<L: LayerIdLike, W: WaypointIdLike>(
        &mut self,
        layer_id: L,
        waypoint_id: W,
    ) -> Result<(usize, Waypoint), ProjectLayerError> {
        let layer_id = layer_id.into_u64();
        let waypoint_id = waypoint_id.into_u64();
        let layer = self.waypoint_layer_mut(layer_id)?;
        layer.remove_waypoint(waypoint_id)
    }

    pub fn rename_waypoint_in_layer<L: LayerIdLike, W: WaypointIdLike>(
        &mut self,
        layer_id: L,
        waypoint_id: W,
        new_name: String,
    ) -> Result<String, ProjectLayerError> {
        let layer_id = layer_id.into_u64();
        let waypoint_id = waypoint_id.into_u64();
        let layer = self.waypoint_layer_mut(layer_id)?;
        layer.rename_waypoint(waypoint_id, new_name)
    }

    pub fn set_waypoint_symbol_in_layer<L: LayerIdLike, W: WaypointIdLike>(
        &mut self,
        layer_id: L,
        waypoint_id: W,
        symbol: Option<String>,
    ) -> Result<Option<String>, ProjectLayerError> {
        let layer_id = layer_id.into_u64();
        let waypoint_id = waypoint_id.into_u64();
        let layer = self.waypoint_layer_mut(layer_id)?;
        layer.set_waypoint_symbol(waypoint_id, symbol)
    }

    pub fn move_waypoint_in_layer<L: LayerIdLike, W: WaypointIdLike>(
        &mut self,
        layer_id: L,
        waypoint_id: W,
        latitude: f64,
        longitude: f64,
    ) -> Result<(), ProjectLayerError> {
        let layer_id = layer_id.into_u64();
        let waypoint_id = waypoint_id.into_u64();
        let layer = self.waypoint_layer_mut(layer_id)?;

        if layer.move_waypoint(WaypointId::new(waypoint_id), latitude, longitude) {
            return Ok(());
        }

        Err(ProjectLayerError::WaypointNotFound(
            LayerId::new(layer_id),
            WaypointId::new(waypoint_id),
        ))
    }

    fn waypoint_layer_mut(
        &mut self,
        layer_id: u64,
    ) -> Result<&mut WaypointLayer, ProjectLayerError> {
        self.waypoint_layers
            .iter_mut()
            .find(|layer| layer.id().value() == layer_id)
            .ok_or(ProjectLayerError::WaypointLayerUnavailable(LayerId::new(
                layer_id,
            )))
    }

    pub fn move_point_in_layer(
        &mut self,
        layer_id: u64,
        track_id: u64,
        segment_id: u64,
        point_id: u64,
        lat: f64,
        lon: f64,
    ) -> Result<(f64, f64), ProjectLayerError> {
        let segment = self.track_segment_mut(layer_id, track_id, segment_id)?;
        segment
            .move_point(point_id, lat, lon)
            .map_err(|_| ProjectLayerError::MissingTrackPoint {
                layer_id,
                track_id,
                segment_id,
                point_id,
            })
    }

    pub fn remove_point_from_layer(
        &mut self,
        layer_id: u64,
        track_id: u64,
        segment_id: u64,
        point_id: u64,
    ) -> Result<(usize, TrackPoint), ProjectLayerError> {
        let segment = self.track_segment_mut(layer_id, track_id, segment_id)?;
        segment
            .remove_point(point_id)
            .map_err(|_| ProjectLayerError::MissingTrackPoint {
                layer_id,
                track_id,
                segment_id,
                point_id,
            })
    }

    pub fn insert_point_in_layer(
        &mut self,
        layer_id: u64,
        track_id: u64,
        segment_id: u64,
        index: usize,
        point: TrackPoint,
    ) -> Result<(), ProjectLayerError> {
        let segment = self.track_segment_mut(layer_id, track_id, segment_id)?;
        segment
            .insert_point_at(index, point)
            .map_err(|_| ProjectLayerError::MissingTrackPoint {
                layer_id,
                track_id,
                segment_id,
                point_id: index as u64,
            })
    }

    pub fn split_segment_in_layer(
        &mut self,
        layer_id: u64,
        track_id: u64,
        segment_id: u64,
        point_id: u64,
    ) -> Result<TrackSegment, ProjectLayerError> {
        let new_id = {
            let track = self.track_mut(layer_id, track_id)?;
            let max_id = track
                .segments()
                .iter()
                .map(|s| s.id().value())
                .max()
                .unwrap_or(0);
            TrackSegmentId::new(max_id + 1)
        };
        let segment = self.track_segment_mut(layer_id, track_id, segment_id)?;
        segment
            .split_at_point(point_id, new_id)
            .map_err(|_| ProjectLayerError::MissingTrackPoint {
                layer_id,
                track_id,
                segment_id,
                point_id,
            })
    }

    pub fn remove_segment_from_layer(
        &mut self,
        layer_id: u64,
        track_id: u64,
        segment_id: u64,
    ) -> Result<(usize, TrackSegment), ProjectLayerError> {
        let track = self.track_mut(layer_id, track_id)?;
        track
            .remove_segment(segment_id)
            .map_err(|_| ProjectLayerError::MissingTrackSegment {
                layer_id,
                track_id,
                segment_id,
            })
    }

    pub fn join_segments_in_layer(
        &mut self,
        layer_id: u64,
        track_id: u64,
        seg_id_a: u64,
        seg_id_b: u64,
    ) -> Result<TrackSegment, ProjectLayerError> {
        let track = self.track_mut(layer_id, track_id)?;
        track.join_segments(seg_id_a, seg_id_b).map_err(|_| {
            ProjectLayerError::MissingTrackSegment {
                layer_id,
                track_id,
                segment_id: seg_id_b,
            }
        })
    }
}

impl Default for Project {
    fn default() -> Self {
        Self::untitled()
    }
}

#[cfg(test)]
mod tests {
    use super::{LayerId, MapLayer, Project, ProjectLayerError, TrackLayer, WaypointLayer};
    use crate::domain::{
        Track, TrackId, TrackPoint, TrackPointId, TrackSegment, TrackSegmentId, Waypoint,
        WaypointId,
    };
    use std::path::{Path, PathBuf};

    #[test]
    fn untitled_project_starts_with_independent_empty_layer_collections() {
        let project = Project::untitled();

        assert_eq!(project.name(), "Untitled Project");
        assert!(project.map_layers().is_empty());
        assert!(project.track_layers().is_empty());
        assert!(project.waypoint_layers().is_empty());
    }

    #[test]
    fn adding_layers_keeps_each_collection_separate() {
        let mut project = Project::untitled();

        project.add_map_layer(MapLayer::new(LayerId::new(10), "Base map"));
        project.add_track_layer(TrackLayer::new(LayerId::new(20), "Recorded tracks"));
        project.add_waypoint_layer(WaypointLayer::new(LayerId::new(30), "Waypoints"));

        assert_eq!(project.map_layers().len(), 1);
        assert_eq!(project.track_layers().len(), 1);
        assert_eq!(project.waypoint_layers().len(), 1);
        assert_eq!(project.map_layers()[0].name(), "Base map");
        assert!(project.map_layers()[0].source_path().is_none());
        assert_eq!(project.track_layers()[0].name(), "Recorded tracks");
        assert_eq!(project.waypoint_layers()[0].name(), "Waypoints");
    }

    #[test]
    fn map_layer_can_store_source_path_metadata() {
        let layer = MapLayer::with_source_path(
            LayerId::new(10),
            "Cached map",
            Some(PathBuf::from(".tmp/lizaalert-maps/demo/map.sqlitedb")),
        );

        assert_eq!(layer.name(), "Cached map");
        assert_eq!(
            layer.source_path(),
            Some(Path::new(".tmp/lizaalert-maps/demo/map.sqlitedb"))
        );
    }

    #[test]
    fn track_layers_store_tracks_with_explicit_segments_and_points() {
        let mut track_layer = TrackLayer::new(LayerId::new(20), "Recorded tracks");
        let mut track = Track::new(TrackId::new(1), "Morning route");
        let mut segment = TrackSegment::new(TrackSegmentId::new(2));
        segment.add_point(TrackPoint::new(TrackPointId::new(3), 55.75, 37.61));
        track.add_segment(segment);

        track_layer.add_track(track);

        assert_eq!(track_layer.tracks().len(), 1);
        assert_eq!(track_layer.tracks()[0].segments().len(), 1);
        assert_eq!(track_layer.tracks()[0].segments()[0].points().len(), 1);
    }

    #[test]
    fn waypoint_layers_store_independent_waypoint_collections() {
        let mut waypoint_layer = WaypointLayer::new(LayerId::new(30), "Waypoints");
        waypoint_layer.add_waypoint(Waypoint::new(WaypointId::new(4), "Camp", 53.9, 27.5667));

        assert_eq!(waypoint_layer.waypoints().len(), 1);
        assert_eq!(waypoint_layer.waypoints()[0].name(), "Camp");
    }

    #[test]
    fn project_adds_track_to_matching_track_layer() {
        let mut project = Project::untitled();
        let layer_id = LayerId::new(20);
        project.add_track_layer(TrackLayer::new(layer_id, "Recorded tracks"));
        let track = Track::new(TrackId::new(1), "Morning route");

        project.add_track_to_layer(layer_id, track).unwrap();

        assert_eq!(project.track_layers()[0].tracks().len(), 1);
        assert_eq!(
            project.track_layers()[0].tracks()[0].name(),
            "Morning route"
        );
    }

    #[test]
    fn project_reports_missing_track_layer_when_adding_track() {
        let mut project = Project::untitled();
        let track = Track::new(TrackId::new(1), "Morning route");

        let error = project
            .add_track_to_layer(LayerId::new(99), track)
            .unwrap_err();

        assert_eq!(
            error,
            ProjectLayerError::TrackLayerNotPresent(LayerId::new(99))
        );
    }

    #[test]
    fn track_layer_remove_track_returns_index_and_track_for_undo() {
        let mut track_layer = TrackLayer::new(LayerId::new(20), "Recorded tracks");
        track_layer.add_track(Track::new(TrackId::new(1), "First"));
        track_layer.add_track(Track::new(TrackId::new(2), "Second"));
        let expected = track_layer.tracks()[0].clone();

        let (index, track) = track_layer.remove_track(TrackId::new(1)).unwrap();

        assert_eq!(index, 0);
        assert_eq!(track, expected);
        assert_eq!(track_layer.tracks().len(), 1);
        assert_eq!(track_layer.tracks()[0].id(), TrackId::new(2));
    }

    #[test]
    fn track_layer_remove_track_reports_missing_track() {
        let mut track_layer = TrackLayer::new(LayerId::new(20), "Recorded tracks");

        let error = track_layer.remove_track(TrackId::new(99)).unwrap_err();

        assert_eq!(
            error,
            ProjectLayerError::MissingTrack {
                layer_id: 20,
                track_id: 99,
            }
        );
    }

    #[test]
    fn project_removes_track_from_matching_track_layer() {
        let mut project = Project::untitled();
        let layer_id = LayerId::new(20);
        project.add_track_layer(TrackLayer::new(layer_id, "Recorded tracks"));
        project
            .add_track_to_layer(layer_id, Track::new(TrackId::new(1), "Morning route"))
            .unwrap();

        let (index, track) = project
            .remove_track_from_layer(layer_id, TrackId::new(1))
            .unwrap();

        assert_eq!(index, 0);
        assert_eq!(track.id(), TrackId::new(1));
        assert!(project.track_layers()[0].tracks().is_empty());
    }

    #[test]
    fn project_reports_missing_track_when_removing_unknown_track() {
        let mut project = Project::untitled();
        let layer_id = LayerId::new(20);
        project.add_track_layer(TrackLayer::new(layer_id, "Recorded tracks"));

        let error = project
            .remove_track_from_layer(layer_id, TrackId::new(99))
            .unwrap_err();

        assert_eq!(
            error,
            ProjectLayerError::MissingTrack {
                layer_id: 20,
                track_id: 99,
            }
        );
    }

    #[test]
    fn project_adds_waypoint_to_matching_waypoint_layer() {
        let mut project = Project::untitled();
        let layer_id = LayerId::new(30);
        project.add_waypoint_layer(WaypointLayer::new(layer_id, "Waypoints"));
        let waypoint = Waypoint::new(WaypointId::new(4), "Camp", 53.9, 27.5667);

        project.add_waypoint_to_layer(layer_id, waypoint).unwrap();

        assert_eq!(project.waypoint_layers()[0].waypoints().len(), 1);
        assert_eq!(project.waypoint_layers()[0].waypoints()[0].name(), "Camp");
    }

    #[test]
    fn project_moves_waypoint_in_matching_waypoint_layer() {
        let mut project = Project::untitled();
        let layer_id = LayerId::new(30);
        let waypoint_id = WaypointId::new(4);
        project.add_waypoint_layer(WaypointLayer::new(layer_id, "Waypoints"));
        project
            .add_waypoint_to_layer(layer_id, Waypoint::new(waypoint_id, "Camp", 53.9, 27.5667))
            .unwrap();

        project
            .move_waypoint_in_layer(layer_id.value(), waypoint_id.value(), 54.1, 27.8)
            .unwrap();

        assert_eq!(project.waypoint_layers()[0].waypoints()[0].latitude(), 54.1);
        assert_eq!(
            project.waypoint_layers()[0].waypoints()[0].longitude(),
            27.8
        );
    }

    #[test]
    fn project_reports_missing_waypoint_when_moving_unknown_waypoint() {
        let mut project = Project::untitled();
        let layer_id = LayerId::new(30);
        project.add_waypoint_layer(WaypointLayer::new(layer_id, "Waypoints"));

        let error = project
            .move_waypoint_in_layer(layer_id.value(), 99, 54.1, 27.8)
            .unwrap_err();

        assert_eq!(
            error,
            ProjectLayerError::WaypointNotFound(layer_id, WaypointId::new(99))
        );
    }

    #[test]
    fn project_renames_and_symbols_waypoints_in_matching_layer() {
        let mut project = Project::untitled();
        let layer_id = LayerId::new(30);
        let waypoint_id = WaypointId::new(4);
        project.add_waypoint_layer(WaypointLayer::new(layer_id, "Waypoints"));
        project
            .add_waypoint_to_layer(layer_id, Waypoint::new(waypoint_id, "Camp", 53.9, 27.5667))
            .unwrap();

        assert_eq!(
            project
                .rename_waypoint_in_layer(
                    layer_id.value(),
                    waypoint_id.value(),
                    "Base camp".to_owned()
                )
                .unwrap(),
            "Camp"
        );
        assert_eq!(
            project
                .set_waypoint_symbol_in_layer(
                    layer_id.value(),
                    waypoint_id.value(),
                    Some("Flag".to_owned())
                )
                .unwrap(),
            None
        );

        let waypoint = &project.waypoint_layers()[0].waypoints()[0];
        assert_eq!(waypoint.name(), "Base camp");
        assert_eq!(waypoint.symbol(), Some("Flag"));
    }

    #[test]
    fn project_reports_missing_waypoint_when_renaming_or_removing() {
        let mut project = Project::untitled();
        let layer_id = LayerId::new(30);
        project.add_waypoint_layer(WaypointLayer::new(layer_id, "Waypoints"));

        let rename_error = project
            .rename_waypoint_in_layer(layer_id.value(), 99, "Base camp".to_owned())
            .unwrap_err();
        assert_eq!(
            rename_error,
            ProjectLayerError::WaypointNotFound(layer_id, WaypointId::new(99))
        );

        let remove_error = project
            .remove_waypoint_from_layer(layer_id.value(), 99)
            .unwrap_err();
        assert_eq!(
            remove_error,
            ProjectLayerError::WaypointNotFound(layer_id, WaypointId::new(99))
        );
    }

    #[test]
    fn waypoint_layer_remove_waypoint_returns_index_and_waypoint_for_undo() {
        let mut waypoint_layer = WaypointLayer::new(LayerId::new(30), "Waypoints");
        waypoint_layer.add_waypoint(Waypoint::new(WaypointId::new(4), "Camp", 53.9, 27.5667));
        waypoint_layer.add_waypoint(Waypoint::new(WaypointId::new(5), "Cache", 53.8, 27.7));
        let expected = waypoint_layer.waypoints()[0].clone();

        let (index, waypoint) = waypoint_layer.remove_waypoint(4).unwrap();

        assert_eq!(index, 0);
        assert_eq!(waypoint, expected);
        assert_eq!(waypoint_layer.waypoints().len(), 1);
        assert_eq!(waypoint_layer.waypoints()[0].id(), WaypointId::new(5));
    }

    #[test]
    fn waypoint_layer_remove_waypoint_reports_missing_waypoint() {
        let mut waypoint_layer = WaypointLayer::new(LayerId::new(30), "Waypoints");

        let error = waypoint_layer.remove_waypoint(99).unwrap_err();

        assert_eq!(
            error,
            ProjectLayerError::WaypointNotFound(LayerId::new(30), WaypointId::new(99))
        );
    }

    #[test]
    fn project_layer_error_formats_new_track_variants() {
        assert_eq!(
            ProjectLayerError::MissingTrack {
                layer_id: 2,
                track_id: 7,
            }
            .to_string(),
            "missing track with id 7 in layer 2"
        );
        assert_eq!(
            ProjectLayerError::MissingTrackSegment {
                layer_id: 2,
                track_id: 7,
                segment_id: 9,
            }
            .to_string(),
            "missing track segment with id 9 in track 7 in layer 2"
        );
        assert_eq!(
            ProjectLayerError::MissingTrackPoint {
                layer_id: 2,
                track_id: 7,
                segment_id: 9,
                point_id: 11,
            }
            .to_string(),
            "missing track point with id 11 in segment 9 in track 7 in layer 2"
        );
    }

    #[test]
    fn project_reports_missing_nested_track_entities() {
        let mut project = Project::untitled();
        let layer_id = 20;
        project.add_track_layer(TrackLayer::new(LayerId::new(layer_id), "Recorded tracks"));

        let track_error = project.track_mut(layer_id, 7).unwrap_err();
        assert_eq!(
            track_error,
            ProjectLayerError::MissingTrack {
                layer_id,
                track_id: 7,
            }
        );

        project
            .track_layer_mut(layer_id)
            .unwrap()
            .add_track(Track::new(TrackId::new(7), "Morning route"));

        let segment_error = project.track_segment_mut(layer_id, 7, 9).unwrap_err();
        assert_eq!(
            segment_error,
            ProjectLayerError::MissingTrackSegment {
                layer_id,
                track_id: 7,
                segment_id: 9,
            }
        );

        let mut track = Track::new(TrackId::new(7), "Morning route");
        track.add_segment(TrackSegment::new(TrackSegmentId::new(9)));
        project.track_layer_mut(layer_id).unwrap().tracks.clear();
        project.track_layer_mut(layer_id).unwrap().add_track(track);

        let point_error = project.track_point_mut(layer_id, 7, 9, 11).unwrap_err();
        assert_eq!(
            point_error,
            ProjectLayerError::MissingTrackPoint {
                layer_id,
                track_id: 7,
                segment_id: 9,
                point_id: 11,
            }
        );
    }

    #[test]
    fn track_layer_create_empty_track_adds_track_with_one_segment() {
        let layer_id = LayerId::new(1);
        let mut layer = TrackLayer::new(layer_id, "Tracks");

        layer.create_empty_track(TrackId::new(1), "New Track".to_string());

        assert_eq!(layer.tracks().len(), 1);
        assert_eq!(layer.tracks()[0].segments().len(), 1);
        assert_eq!(layer.tracks()[0].segments()[0].points().len(), 0);
    }

    #[test]
    fn track_layer_create_empty_track_uses_explicit_track_id() {
        let layer_id = LayerId::new(1);
        let mut layer = TrackLayer::new(layer_id, "Tracks");

        layer.create_empty_track(TrackId::new(42), "My Track".to_string());

        assert_eq!(layer.tracks()[0].id(), TrackId::new(42));
        assert_eq!(layer.tracks()[0].name(), "My Track");
    }
}
