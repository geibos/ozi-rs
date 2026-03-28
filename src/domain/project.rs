use crate::domain::{Track, Waypoint, WaypointId};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProjectLayerError {
    MissingTrackLayer(LayerId),
    MissingWaypointLayer(LayerId),
    MissingWaypoint(LayerId, WaypointId),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ProjectId(u64);

impl ProjectId {
    pub const fn new(value: u64) -> Self {
        Self(value)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct LayerId(u64);

impl LayerId {
    pub const fn new(value: u64) -> Self {
        Self(value)
    }

    pub const fn value(&self) -> u64 {
        self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
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

#[derive(Debug, Clone, PartialEq)]
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
}

#[derive(Debug, Clone, PartialEq)]
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

#[derive(Debug, Clone, PartialEq)]
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

    pub fn add_track_layer(&mut self, layer: TrackLayer) {
        self.track_layers.push(layer);
    }

    pub fn add_waypoint_layer(&mut self, layer: WaypointLayer) {
        self.waypoint_layers.push(layer);
    }

    pub fn add_track_to_layer(
        &mut self,
        layer_id: LayerId,
        track: Track,
    ) -> Result<(), ProjectLayerError> {
        let Some(layer) = self
            .track_layers
            .iter_mut()
            .find(|layer| layer.id() == layer_id)
        else {
            return Err(ProjectLayerError::MissingTrackLayer(layer_id));
        };

        layer.add_track(track);
        Ok(())
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
            return Err(ProjectLayerError::MissingWaypointLayer(layer_id));
        };

        layer.add_waypoint(waypoint);
        Ok(())
    }

    pub fn move_waypoint_in_layer(
        &mut self,
        layer_id: LayerId,
        waypoint_id: WaypointId,
        latitude: f64,
        longitude: f64,
    ) -> Result<(), ProjectLayerError> {
        let Some(layer) = self
            .waypoint_layers
            .iter_mut()
            .find(|layer| layer.id() == layer_id)
        else {
            return Err(ProjectLayerError::MissingWaypointLayer(layer_id));
        };

        if layer.move_waypoint(waypoint_id, latitude, longitude) {
            return Ok(());
        }

        Err(ProjectLayerError::MissingWaypoint(layer_id, waypoint_id))
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
            ProjectLayerError::MissingTrackLayer(LayerId::new(99))
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
            .move_waypoint_in_layer(layer_id, waypoint_id, 54.1, 27.8)
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
            .move_waypoint_in_layer(layer_id, WaypointId::new(99), 54.1, 27.8)
            .unwrap_err();

        assert_eq!(
            error,
            ProjectLayerError::MissingWaypoint(layer_id, WaypointId::new(99))
        );
    }
}
