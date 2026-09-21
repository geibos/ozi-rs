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
    InvalidSegmentOperation {
        layer_id: u64,
        track_id: u64,
        segment_id: u64,
        reason: &'static str,
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
            Self::InvalidSegmentOperation {
                layer_id,
                track_id,
                segment_id,
                reason,
            } => write!(
                f,
                "invalid segment operation on segment {} in track {} in layer {}: {}",
                segment_id, track_id, layer_id, reason
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
    /// Reassign this layer's identifier. Only `Project::deduplicate_layer_ids`
    /// uses this, to repair a project file whose layers share an id.
    pub(crate) fn set_id(&mut self, id: LayerId) {
        self.id = id;
    }

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
    /// Reassign this layer's identifier. Only `Project::deduplicate_layer_ids`
    /// uses this, to repair a project file whose layers share an id.
    pub(crate) fn set_id(&mut self, id: LayerId) {
        self.id = id;
    }

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
        debug_assert!(
            !self.tracks.iter().any(|t| t.id() == track_id),
            "duplicate track id: {track_id:?}"
        );
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
    /// Reassign this layer's identifier. Only `Project::deduplicate_layer_ids`
    /// uses this, to repair a project file whose layers share an id.
    pub(crate) fn set_id(&mut self, id: LayerId) {
        self.id = id;
    }

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

    /// Set a waypoint's visibility flag. Returns the new value on success
    /// or `None` if the waypoint is missing. Non-undoable mutation.
    pub fn set_waypoint_visible(&mut self, waypoint_id: WaypointId, visible: bool) -> Option<bool> {
        let waypoint = self
            .waypoints
            .iter_mut()
            .find(|waypoint| waypoint.id() == waypoint_id)?;
        waypoint.set_visible(visible);
        Some(visible)
    }

    /// Flip a waypoint's visibility flag. Returns the new value or `None`
    /// if the waypoint is missing. Non-undoable mutation.
    pub fn toggle_waypoint_visible(&mut self, waypoint_id: WaypointId) -> Option<bool> {
        let waypoint = self
            .waypoints
            .iter_mut()
            .find(|waypoint| waypoint.id() == waypoint_id)?;
        Some(waypoint.toggle_visible())
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
        let mut project = Self {
            id: ProjectId::new(1),
            name: "Untitled Project".to_owned(),
            map_layers: Vec::new(),
            track_layers: Vec::new(),
            waypoint_layers: Vec::new(),
        };
        project.ensure_default_layers();
        project
    }

    /// Enforce the invariant declared by the `layers` capability: an open
    /// project always has at least one track layer and one waypoint layer.
    /// Idempotent — existing layers are not modified or reordered; only
    /// missing kinds get a single default layer appended.
    pub fn ensure_default_layers(&mut self) {
        if self.track_layers.is_empty() {
            self.track_layers
                .push(TrackLayer::new(LayerId::new(1), "Tracks"));
        }
        if self.waypoint_layers.is_empty() {
            self.waypoint_layers
                .push(WaypointLayer::new(LayerId::new(1), "Waypoints"));
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

    /// Give every layer an identifier that is unique among layers of its kind,
    /// returning the layers that had to be renumbered as `(name, old, new)`.
    ///
    /// Layers are addressed by id within their own kind — a track layer and a
    /// waypoint layer may both be id 1, and the default project relies on that.
    /// Two layers of the *same* kind sharing an id is the problem: the second
    /// is unreachable, so renames, imports and deletions all land on the first.
    /// Older project files carry such duplicates, so a loaded project is
    /// repaired rather than trusted. New ids continue past the highest in use
    /// anywhere, matching how imports allocate them.
    pub fn deduplicate_layer_ids(&mut self) -> Vec<(String, u64, u64)> {
        let mut next: u64 = self
            .map_layers
            .iter()
            .map(|l| l.id().value())
            .chain(self.track_layers.iter().map(|l| l.id().value()))
            .chain(self.waypoint_layers.iter().map(|l| l.id().value()))
            .max()
            .unwrap_or(0);

        let mut renumbered = Vec::new();
        macro_rules! repair_kind {
            ($layers:expr) => {{
                let mut seen: std::collections::HashSet<u64> = std::collections::HashSet::new();
                for layer in $layers.iter_mut() {
                    let old = layer.id().value();
                    if seen.insert(old) {
                        continue;
                    }
                    next += 1;
                    layer.set_id(LayerId::new(next));
                    seen.insert(next);
                    renumbered.push((layer.name().to_owned(), old, next));
                }
            }};
        }
        repair_kind!(self.map_layers);
        repair_kind!(self.track_layers);
        repair_kind!(self.waypoint_layers);
        renumbered
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

    /// Set the visibility of every track in every track layer.
    ///
    /// Triage on a search is done one track at a time, so hiding the other
    /// twenty-five has to be one operation rather than twenty-five.
    pub fn set_all_tracks_visible(&mut self, visible: bool) {
        for layer in self.track_layers.iter_mut() {
            let ids: Vec<crate::domain::TrackId> =
                layer.tracks().iter().map(|track| track.id()).collect();
            for id in ids {
                layer.set_track_visible(id, visible);
            }
        }
    }

    /// Make one track visible and hide every other track in the project.
    ///
    /// Returns false when the track does not exist, leaving visibility as it
    /// was rather than hiding everything.
    pub fn show_only_track(&mut self, layer_id: LayerId, track_id: crate::domain::TrackId) -> bool {
        let exists = self.track_layers.iter().any(|layer| {
            layer.id() == layer_id && layer.tracks().iter().any(|t| t.id() == track_id)
        });
        if !exists {
            return false;
        }
        self.set_all_tracks_visible(false);
        self.set_track_visible_in_layer(layer_id, track_id, true);
        true
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

    /// Flip the `visible` flag on a waypoint. Non-undoable mutation,
    /// returns the new value or `None` if the layer/waypoint is missing.
    pub fn toggle_waypoint_visible_in_layer(
        &mut self,
        layer_id: LayerId,
        waypoint_id: WaypointId,
    ) -> Option<bool> {
        let layer = self
            .waypoint_layers
            .iter_mut()
            .find(|layer| layer.id() == layer_id)?;
        layer.toggle_waypoint_visible(waypoint_id)
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
        track
            .join_segments(seg_id_a, seg_id_b)
            .map_err(|err| match err {
                ProjectLayerError::InvalidSegmentOperation {
                    segment_id, reason, ..
                } => ProjectLayerError::InvalidSegmentOperation {
                    layer_id,
                    track_id,
                    segment_id,
                    reason,
                },
                _ => ProjectLayerError::MissingTrackSegment {
                    layer_id,
                    track_id,
                    segment_id: seg_id_b,
                },
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
    /// Ids start past 1: `Project::untitled()` already carries a default
    /// "Tracks" layer with id 1, and a second layer claiming that id would be
    /// unreachable — every lookup finds the first match.
    fn project_with_two_layers() -> Project {
        let mut first = TrackLayer::new(LayerId::new(10), "A");
        first.add_track(Track::new(TrackId::new(1), "a1"));
        first.add_track(Track::new(TrackId::new(2), "a2"));
        let mut second = TrackLayer::new(LayerId::new(20), "B");
        second.add_track(Track::new(TrackId::new(1), "b1"));
        let mut project = Project::untitled();
        project.add_track_layer(first);
        project.add_track_layer(second);
        project
    }

    fn visibility(project: &Project) -> Vec<bool> {
        project
            .track_layers()
            .iter()
            .flat_map(|l| l.tracks().iter().map(|t| t.style().visible))
            .collect()
    }

    #[test]
    fn set_all_tracks_visible_covers_every_layer() {
        let mut project = project_with_two_layers();
        project.set_all_tracks_visible(false);
        assert!(visibility(&project).iter().all(|v| !v));
        project.set_all_tracks_visible(true);
        assert!(visibility(&project).iter().all(|v| *v));
    }

    /// Isolating is the triage move: one track on the basemap, the rest gone.
    #[test]
    fn show_only_track_leaves_exactly_one_visible() {
        let mut project = project_with_two_layers();
        assert!(project.show_only_track(LayerId::new(20), TrackId::new(1)));

        let visible: Vec<&str> = project
            .track_layers()
            .iter()
            .flat_map(|l| l.tracks().iter())
            .filter(|t| t.style().visible)
            .map(|t| t.name())
            .collect();
        assert_eq!(visible, vec!["b1"]);
    }

    #[test]
    fn show_only_track_shows_a_track_that_was_hidden() {
        let mut project = project_with_two_layers();
        project.set_all_tracks_visible(false);
        assert!(project.show_only_track(LayerId::new(10), TrackId::new(2)));
        let visible: Vec<&str> = project
            .track_layers()
            .iter()
            .flat_map(|l| l.tracks().iter())
            .filter(|t| t.style().visible)
            .map(|t| t.name())
            .collect();
        assert_eq!(visible, vec!["a2"]);
    }

    /// A stale row must not blank the map.
    #[test]
    fn show_only_track_is_a_no_op_for_a_missing_track() {
        let mut project = project_with_two_layers();
        assert!(!project.show_only_track(LayerId::new(10), TrackId::new(99)));
        assert!(visibility(&project).iter().all(|v| *v));
    }

    /// A project file written by an older build carries two track layers that
    /// both claim id 1. Everything addresses a layer by id, so the second one
    /// is unreachable: renames, imports and deletes all land on the first.
    #[test]
    fn deduplicate_layer_ids_renumbers_collisions_and_reports_them() {
        let mut project = Project {
            id: super::ProjectId::new(1),
            name: "Test".to_owned(),
            map_layers: vec![MapLayer::new(LayerId::new(2), "Map")],
            track_layers: vec![
                TrackLayer::new(LayerId::new(1), "Tracks"),
                TrackLayer::new(LayerId::new(1), "Tracks"),
            ],
            waypoint_layers: vec![WaypointLayer::new(LayerId::new(1), "Waypoints")],
        };

        let renumbered = project.deduplicate_layer_ids();

        let mut track_ids: Vec<u64> = project
            .track_layers()
            .iter()
            .map(|l| l.id().value())
            .collect();
        let track_count = track_ids.len();
        track_ids.sort_unstable();
        track_ids.dedup();
        assert_eq!(
            track_ids.len(),
            track_count,
            "track layers must not share an id with each other"
        );

        assert_eq!(
            renumbered.len(),
            1,
            "only the duplicate Tracks layer; a waypoint layer may share id 1 with a track layer"
        );
        assert_eq!(renumbered[0].1, 1);
        assert!(renumbered[0].2 > 2);
        assert_eq!(
            project.waypoint_layers()[0].id().value(),
            1,
            "a waypoint layer keeps id 1 even though a track layer also uses it"
        );
    }

    #[test]
    fn deduplicate_layer_ids_leaves_a_healthy_project_untouched() {
        let mut project = Project {
            id: super::ProjectId::new(1),
            name: "Test".to_owned(),
            map_layers: vec![MapLayer::new(LayerId::new(3), "Map")],
            track_layers: vec![TrackLayer::new(LayerId::new(1), "Tracks")],
            waypoint_layers: vec![WaypointLayer::new(LayerId::new(2), "Waypoints")],
        };

        assert!(project.deduplicate_layer_ids().is_empty());
        assert_eq!(project.track_layers()[0].id().value(), 1);
        assert_eq!(project.waypoint_layers()[0].id().value(), 2);
        assert_eq!(project.map_layers()[0].id().value(), 3);
    }

    /// Track contents must survive the repair — only the identifier changes.
    #[test]
    fn deduplicate_layer_ids_keeps_layer_contents() {
        let mut first = TrackLayer::new(LayerId::new(1), "Tracks");
        first.add_track(Track::new(TrackId::new(1), "kept"));
        let mut second = TrackLayer::new(LayerId::new(1), "Imported");
        second.add_track(Track::new(TrackId::new(2), "also kept"));
        let mut project = Project {
            id: super::ProjectId::new(1),
            name: "Test".to_owned(),
            map_layers: Vec::new(),
            track_layers: vec![first, second],
            waypoint_layers: Vec::new(),
        };

        project.deduplicate_layer_ids();

        assert_eq!(project.track_layers()[0].tracks()[0].name(), "kept");
        assert_eq!(project.track_layers()[1].tracks()[0].name(), "also kept");
        assert_ne!(
            project.track_layers()[0].id(),
            project.track_layers()[1].id()
        );
    }

    use super::{LayerId, MapLayer, Project, ProjectLayerError, TrackLayer, WaypointLayer};
    use crate::domain::{
        Track, TrackId, TrackPoint, TrackPointId, TrackSegment, TrackSegmentId, Waypoint,
        WaypointId,
    };
    use std::path::{Path, PathBuf};

    #[test]
    fn untitled_project_satisfies_default_layers_invariant() {
        let project = Project::untitled();

        assert_eq!(project.name(), "Untitled Project");
        assert!(project.map_layers().is_empty());
        assert_eq!(project.track_layers().len(), 1);
        assert_eq!(project.track_layers()[0].name(), "Tracks");
        assert_eq!(project.waypoint_layers().len(), 1);
        assert_eq!(project.waypoint_layers()[0].name(), "Waypoints");
    }

    #[test]
    fn default_project_satisfies_default_layers_invariant() {
        let project = Project::default();

        assert_eq!(project.track_layers().len(), 1);
        assert_eq!(project.track_layers()[0].name(), "Tracks");
        assert_eq!(project.waypoint_layers().len(), 1);
        assert_eq!(project.waypoint_layers()[0].name(), "Waypoints");
    }

    #[test]
    fn ensure_default_layers_is_idempotent_and_appends_only_missing_kinds() {
        let mut project = Project {
            id: super::ProjectId::new(1),
            name: "Test".to_owned(),
            map_layers: Vec::new(),
            track_layers: vec![TrackLayer::new(LayerId::new(42), "Recorded")],
            waypoint_layers: Vec::new(),
        };

        project.ensure_default_layers();

        // Existing track layer kept untouched, default waypoint layer appended.
        assert_eq!(project.track_layers().len(), 1);
        assert_eq!(project.track_layers()[0].id(), LayerId::new(42));
        assert_eq!(project.track_layers()[0].name(), "Recorded");
        assert_eq!(project.waypoint_layers().len(), 1);
        assert_eq!(project.waypoint_layers()[0].name(), "Waypoints");

        // Running again does not duplicate.
        project.ensure_default_layers();
        assert_eq!(project.track_layers().len(), 1);
        assert_eq!(project.waypoint_layers().len(), 1);
    }

    #[test]
    fn adding_layers_keeps_each_collection_separate() {
        let mut project = Project::untitled();

        project.add_map_layer(MapLayer::new(LayerId::new(10), "Base map"));
        project.add_track_layer(TrackLayer::new(LayerId::new(20), "Recorded tracks"));
        project.add_waypoint_layer(WaypointLayer::new(LayerId::new(30), "Extra Waypoints"));

        // Tracks and waypoint collections each carry their default layer
        // plus the explicitly-added one.
        assert_eq!(project.map_layers().len(), 1);
        assert_eq!(project.track_layers().len(), 2);
        assert_eq!(project.waypoint_layers().len(), 2);
        assert_eq!(project.map_layers()[0].name(), "Base map");
        assert!(project.map_layers()[0].source_path().is_none());
        assert!(
            project
                .track_layers()
                .iter()
                .any(|l| l.id() == LayerId::new(20) && l.name() == "Recorded tracks")
        );
        assert!(
            project
                .waypoint_layers()
                .iter()
                .any(|l| l.id() == LayerId::new(30) && l.name() == "Extra Waypoints")
        );
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

        let layer = project
            .track_layers()
            .iter()
            .find(|l| l.id() == layer_id)
            .expect("added layer present");
        assert_eq!(layer.tracks().len(), 1);
        assert_eq!(layer.tracks()[0].name(), "Morning route");
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
        project.add_waypoint_layer(WaypointLayer::new(layer_id, "Extra Waypoints"));
        let waypoint = Waypoint::new(WaypointId::new(4), "Camp", 53.9, 27.5667);

        project.add_waypoint_to_layer(layer_id, waypoint).unwrap();

        let layer = project
            .waypoint_layers()
            .iter()
            .find(|l| l.id() == layer_id)
            .expect("added layer present");
        assert_eq!(layer.waypoints().len(), 1);
        assert_eq!(layer.waypoints()[0].name(), "Camp");
    }

    #[test]
    fn project_moves_waypoint_in_matching_waypoint_layer() {
        let mut project = Project::untitled();
        let layer_id = LayerId::new(30);
        let waypoint_id = WaypointId::new(4);
        project.add_waypoint_layer(WaypointLayer::new(layer_id, "Extra Waypoints"));
        project
            .add_waypoint_to_layer(layer_id, Waypoint::new(waypoint_id, "Camp", 53.9, 27.5667))
            .unwrap();

        project
            .move_waypoint_in_layer(layer_id.value(), waypoint_id.value(), 54.1, 27.8)
            .unwrap();

        let layer = project
            .waypoint_layers()
            .iter()
            .find(|l| l.id() == layer_id)
            .expect("added layer present");
        assert_eq!(layer.waypoints()[0].latitude(), 54.1);
        assert_eq!(layer.waypoints()[0].longitude(), 27.8);
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
        project.add_waypoint_layer(WaypointLayer::new(layer_id, "Extra Waypoints"));
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

        let layer = project
            .waypoint_layers()
            .iter()
            .find(|l| l.id() == layer_id)
            .expect("added layer present");
        let waypoint = &layer.waypoints()[0];
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
