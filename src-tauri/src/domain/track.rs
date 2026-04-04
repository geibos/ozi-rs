#![allow(dead_code)]

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[serde(transparent)]
pub struct TrackId(u64);

impl TrackId {
    pub const fn new(value: u64) -> Self {
        Self(value)
    }

    pub const fn value(&self) -> u64 {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[serde(transparent)]
pub struct TrackSegmentId(u64);

impl TrackSegmentId {
    pub const fn new(value: u64) -> Self {
        Self(value)
    }

    pub const fn value(&self) -> u64 {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[serde(transparent)]
pub struct TrackPointId(u64);

impl TrackPointId {
    pub const fn new(value: u64) -> Self {
        Self(value)
    }

    pub const fn value(&self) -> u64 {
        self.0
    }
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct TrackStyle {
    /// RGBA color bytes.
    pub color: [u8; 4],
    pub line_width: f32,
    pub visible: bool,
    /// Multiplier applied on top of color alpha (0.0–1.0).
    pub opacity: f32,
}

impl Default for TrackStyle {
    fn default() -> Self {
        Self {
            color: [255, 0, 0, 255],
            line_width: 2.0,
            visible: true,
            opacity: 1.0,
        }
    }
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct TrackPoint {
    id: TrackPointId,
    latitude: f64,
    longitude: f64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    elevation: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    timestamp: Option<chrono::DateTime<chrono::Utc>>,
}

impl TrackPoint {
    pub const fn new(id: TrackPointId, latitude: f64, longitude: f64) -> Self {
        Self {
            id,
            latitude,
            longitude,
            elevation: None,
            timestamp: None,
        }
    }

    pub fn with_elevation(mut self, elevation: f64) -> Self {
        self.elevation = Some(elevation);
        self
    }

    pub fn with_timestamp(mut self, timestamp: chrono::DateTime<chrono::Utc>) -> Self {
        self.timestamp = Some(timestamp);
        self
    }

    pub const fn id(&self) -> TrackPointId {
        self.id
    }

    pub const fn latitude(&self) -> f64 {
        self.latitude
    }

    pub const fn longitude(&self) -> f64 {
        self.longitude
    }

    pub const fn elevation(&self) -> Option<f64> {
        self.elevation
    }

    pub fn timestamp(&self) -> Option<chrono::DateTime<chrono::Utc>> {
        self.timestamp
    }
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct TrackSegment {
    id: TrackSegmentId,
    points: Vec<TrackPoint>,
}

impl TrackSegment {
    pub fn new(id: TrackSegmentId) -> Self {
        Self {
            id,
            points: Vec::new(),
        }
    }

    pub const fn id(&self) -> TrackSegmentId {
        self.id
    }

    pub fn points(&self) -> &[TrackPoint] {
        &self.points
    }

    pub fn add_point(&mut self, point: TrackPoint) {
        self.points.push(point);
    }

    pub fn point_mut(&mut self, point_id: TrackPointId) -> Option<&mut TrackPoint> {
        self.points.iter_mut().find(|point| point.id() == point_id)
    }

    pub fn points_mut(&mut self) -> &mut Vec<TrackPoint> {
        &mut self.points
    }

    /// Move a point to new coordinates. Returns old `(lat, lon)` for undo.
    pub fn move_point(
        &mut self,
        point_id: u64,
        lat: f64,
        lon: f64,
    ) -> Result<(f64, f64), crate::domain::ProjectLayerError> {
        let pid = TrackPointId::new(point_id);
        let point = self.points.iter_mut().find(|p| p.id() == pid).ok_or(
            crate::domain::ProjectLayerError::MissingTrackPoint {
                layer_id: 0,
                track_id: 0,
                segment_id: self.id.value(),
                point_id,
            },
        )?;
        let old = (point.latitude, point.longitude);
        point.latitude = lat;
        point.longitude = lon;
        Ok(old)
    }

    /// Remove a point by id. Returns `(index, point)` so undo can call `insert_point_at`.
    pub fn remove_point(
        &mut self,
        point_id: u64,
    ) -> Result<(usize, TrackPoint), crate::domain::ProjectLayerError> {
        let pid = TrackPointId::new(point_id);
        let index = self.points.iter().position(|p| p.id() == pid).ok_or(
            crate::domain::ProjectLayerError::MissingTrackPoint {
                layer_id: 0,
                track_id: 0,
                segment_id: self.id.value(),
                point_id,
            },
        )?;
        let removed = self.points.remove(index);
        Ok((index, removed))
    }

    /// Insert a point at a given index. Errors if `index > len`.
    pub fn insert_point_at(
        &mut self,
        index: usize,
        point: TrackPoint,
    ) -> Result<(), crate::domain::ProjectLayerError> {
        if index > self.points.len() {
            return Err(crate::domain::ProjectLayerError::MissingTrackPoint {
                layer_id: 0,
                track_id: 0,
                segment_id: self.id.value(),
                point_id: index as u64,
            });
        }
        self.points.insert(index, point);
        Ok(())
    }

    /// Split segment at the given point. Left segment retains original ID and points `0..=split`.
    /// Returns the new right segment (new ID) containing points `split..end` (split point shared).
    pub fn split_at_point(
        &mut self,
        point_id: u64,
        new_segment_id: TrackSegmentId,
    ) -> Result<TrackSegment, crate::domain::ProjectLayerError> {
        let pid = TrackPointId::new(point_id);
        let split_index = self.points.iter().position(|p| p.id() == pid).ok_or(
            crate::domain::ProjectLayerError::MissingTrackPoint {
                layer_id: 0,
                track_id: 0,
                segment_id: self.id.value(),
                point_id,
            },
        )?;

        let right_points = self.points[split_index..].to_vec();
        self.points.truncate(split_index + 1);

        let mut right = TrackSegment::new(new_segment_id);
        right.points = right_points;
        Ok(right)
    }
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Track {
    id: TrackId,
    name: String,
    #[serde(default)]
    style: TrackStyle,
    segments: Vec<TrackSegment>,
}

impl Track {
    pub fn new(id: TrackId, name: impl Into<String>) -> Self {
        Self {
            id,
            name: name.into(),
            style: TrackStyle::default(),
            segments: Vec::new(),
        }
    }

    pub const fn id(&self) -> TrackId {
        self.id
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn set_name(&mut self, name: impl Into<String>) {
        self.name = name.into();
    }

    pub fn style(&self) -> &TrackStyle {
        &self.style
    }

    pub fn style_mut(&mut self) -> &mut TrackStyle {
        &mut self.style
    }

    pub fn segments(&self) -> &[TrackSegment] {
        &self.segments
    }

    pub fn add_segment(&mut self, segment: TrackSegment) {
        self.segments.push(segment);
    }

    pub fn segment_mut(&mut self, segment_id: TrackSegmentId) -> Option<&mut TrackSegment> {
        self.segments
            .iter_mut()
            .find(|segment| segment.id() == segment_id)
    }

    /// Total distance across all segments in kilometres (Haversine).
    pub fn total_distance_km(&self) -> f64 {
        self.segments
            .iter()
            .flat_map(|seg| seg.points().windows(2))
            .map(|pair| {
                haversine_km(
                    pair[0].latitude(),
                    pair[0].longitude(),
                    pair[1].latitude(),
                    pair[1].longitude(),
                )
            })
            .sum()
    }

    /// Duration from first to last timestamped point across all segments.
    pub fn total_duration(&self) -> Option<chrono::Duration> {
        let first = self
            .segments
            .iter()
            .flat_map(|s| s.points())
            .filter_map(|p| p.timestamp())
            .next()?;
        let last = self
            .segments
            .iter()
            .flat_map(|s| s.points())
            .filter_map(|p| p.timestamp())
            .next_back()?;
        Some(last - first)
    }

    /// Total number of track points across all segments.
    pub fn point_count(&self) -> usize {
        self.segments.iter().map(|s| s.points().len()).sum()
    }

    /// Remove a segment by id. Returns `(index, segment)` so undo can reinsert it.
    pub fn remove_segment(
        &mut self,
        segment_id: u64,
    ) -> Result<(usize, TrackSegment), crate::domain::ProjectLayerError> {
        let sid = TrackSegmentId::new(segment_id);
        let index = self.segments.iter().position(|s| s.id() == sid).ok_or(
            crate::domain::ProjectLayerError::MissingTrackSegment {
                layer_id: 0,
                track_id: self.id.value(),
                segment_id,
            },
        )?;
        let removed = self.segments.remove(index);
        Ok((index, removed))
    }

    /// Insert a segment at a given index (for undo of `remove_segment`).
    pub fn insert_segment_at(&mut self, index: usize, segment: TrackSegment) {
        let clamped = index.min(self.segments.len());
        self.segments.insert(clamped, segment);
    }

    /// Join two adjacent segments: append all points from B into A, then remove B.
    /// B must immediately follow A in the segments list (`index_b == index_a + 1`).
    /// Returns the removed segment B so undo can split or reinsert it.
    pub fn join_segments(
        &mut self,
        seg_id_a: u64,
        seg_id_b: u64,
    ) -> Result<TrackSegment, crate::domain::ProjectLayerError> {
        let sid_a = TrackSegmentId::new(seg_id_a);
        let sid_b = TrackSegmentId::new(seg_id_b);

        let index_a = self.segments.iter().position(|s| s.id() == sid_a).ok_or(
            crate::domain::ProjectLayerError::MissingTrackSegment {
                layer_id: 0,
                track_id: self.id.value(),
                segment_id: seg_id_a,
            },
        )?;

        let index_b = self.segments.iter().position(|s| s.id() == sid_b).ok_or(
            crate::domain::ProjectLayerError::MissingTrackSegment {
                layer_id: 0,
                track_id: self.id.value(),
                segment_id: seg_id_b,
            },
        )?;

        if index_b != index_a + 1 {
            return Err(crate::domain::ProjectLayerError::MissingTrackSegment {
                layer_id: 0,
                track_id: self.id.value(),
                segment_id: seg_id_b,
            });
        }

        // Remove B first (higher index), then drain its points into A.
        let mut seg_b = self.segments.remove(index_b);
        let points_b: Vec<TrackPoint> = seg_b.points_mut().drain(..).collect();
        self.segments[index_a].points_mut().extend(points_b);

        Ok(seg_b)
    }
}

fn haversine_km(lat1: f64, lon1: f64, lat2: f64, lon2: f64) -> f64 {
    const R: f64 = 6_371.0;
    let dlat = (lat2 - lat1).to_radians();
    let dlon = (lon2 - lon1).to_radians();
    let a = (dlat / 2.0).sin().powi(2)
        + lat1.to_radians().cos() * lat2.to_radians().cos() * (dlon / 2.0).sin().powi(2);
    2.0 * R * a.sqrt().asin()
}

#[cfg(test)]
mod tests {
    use super::{Track, TrackId, TrackPoint, TrackPointId, TrackSegment, TrackSegmentId};

    #[test]
    fn track_segments_keep_inserted_points_in_order() {
        let mut segment = TrackSegment::new(TrackSegmentId::new(5));

        segment.add_point(TrackPoint::new(TrackPointId::new(10), 55.75, 37.61));
        segment.add_point(TrackPoint::new(TrackPointId::new(11), 55.76, 37.62));

        assert_eq!(segment.points().len(), 2);
        assert_eq!(segment.points()[0].id(), TrackPointId::new(10));
        assert_eq!(segment.points()[1].id(), TrackPointId::new(11));
    }

    #[test]
    fn tracks_keep_segments_as_explicit_collections() {
        let mut track = Track::new(TrackId::new(1), "Morning route");
        let mut segment = TrackSegment::new(TrackSegmentId::new(2));
        segment.add_point(TrackPoint::new(TrackPointId::new(3), 54.0, 27.0));

        track.add_segment(segment);

        assert_eq!(track.name(), "Morning route");
        assert_eq!(track.segments().len(), 1);
        assert_eq!(track.segments()[0].points().len(), 1);
    }

    #[test]
    fn track_point_builder_methods_set_optional_fields() {
        use chrono::TimeZone as _;

        let ts = chrono::Utc.with_ymd_and_hms(2024, 6, 1, 10, 0, 0).unwrap();
        let point = TrackPoint::new(TrackPointId::new(1), 55.0, 37.0)
            .with_elevation(200.0)
            .with_timestamp(ts);

        assert_eq!(point.elevation(), Some(200.0));
        assert_eq!(point.timestamp(), Some(ts));
    }

    #[test]
    fn default_track_style_is_visible_red() {
        use super::TrackStyle;

        let style = TrackStyle::default();

        assert!(style.visible);
        assert_eq!(style.color, [255, 0, 0, 255]);
    }

    fn make_segment_with_points() -> TrackSegment {
        let mut seg = TrackSegment::new(TrackSegmentId::new(1));
        seg.add_point(TrackPoint::new(TrackPointId::new(10), 55.0, 37.0));
        seg.add_point(TrackPoint::new(TrackPointId::new(11), 55.1, 37.1));
        seg.add_point(TrackPoint::new(TrackPointId::new(12), 55.2, 37.2));
        seg
    }

    #[test]
    fn move_point_returns_old_coords_on_success() {
        let mut seg = make_segment_with_points();

        let old = seg.move_point(11, 60.0, 40.0).unwrap();

        assert_eq!(old, (55.1, 37.1));
        assert_eq!(seg.points()[1].latitude(), 60.0);
        assert_eq!(seg.points()[1].longitude(), 40.0);
    }

    #[test]
    fn move_point_errors_on_missing_point_id() {
        let mut seg = make_segment_with_points();

        let err = seg.move_point(99, 0.0, 0.0).unwrap_err();

        assert!(matches!(
            err,
            crate::domain::ProjectLayerError::MissingTrackPoint { point_id: 99, .. }
        ));
    }

    #[test]
    fn remove_point_returns_index_and_point_on_success() {
        let mut seg = make_segment_with_points();

        let (index, point) = seg.remove_point(11).unwrap();

        assert_eq!(index, 1);
        assert_eq!(point.id(), TrackPointId::new(11));
        assert_eq!(seg.points().len(), 2);
    }

    #[test]
    fn remove_point_errors_on_missing_point_id() {
        let mut seg = make_segment_with_points();

        let err = seg.remove_point(99).unwrap_err();

        assert!(matches!(
            err,
            crate::domain::ProjectLayerError::MissingTrackPoint { point_id: 99, .. }
        ));
    }

    #[test]
    fn insert_point_at_places_point_at_correct_index() {
        let mut seg = make_segment_with_points();
        let new_point = TrackPoint::new(TrackPointId::new(20), 56.0, 38.0);

        seg.insert_point_at(1, new_point).unwrap();

        assert_eq!(seg.points().len(), 4);
        assert_eq!(seg.points()[1].id(), TrackPointId::new(20));
    }

    #[test]
    fn insert_point_at_errors_when_index_exceeds_length() {
        let mut seg = make_segment_with_points();
        let new_point = TrackPoint::new(TrackPointId::new(20), 56.0, 38.0);

        let err = seg.insert_point_at(100, new_point).unwrap_err();

        assert!(matches!(
            err,
            crate::domain::ProjectLayerError::MissingTrackPoint { .. }
        ));
    }

    #[test]
    fn split_at_point_left_keeps_id_and_points_up_to_split() {
        let mut seg = make_segment_with_points();
        let original_id = seg.id();

        let right = seg.split_at_point(11, TrackSegmentId::new(99)).unwrap();

        assert_eq!(seg.id(), original_id);
        assert_eq!(seg.points().len(), 2);
        assert_eq!(seg.points()[0].id(), TrackPointId::new(10));
        assert_eq!(seg.points()[1].id(), TrackPointId::new(11));
        assert_eq!(right.id(), TrackSegmentId::new(99));
        assert_eq!(right.points().len(), 2);
        assert_eq!(right.points()[0].id(), TrackPointId::new(11));
        assert_eq!(right.points()[1].id(), TrackPointId::new(12));
    }

    #[test]
    fn split_at_point_errors_on_missing_point_id() {
        let mut seg = make_segment_with_points();

        let err = seg.split_at_point(99, TrackSegmentId::new(2)).unwrap_err();

        assert!(matches!(
            err,
            crate::domain::ProjectLayerError::MissingTrackPoint { point_id: 99, .. }
        ));
    }

    fn make_track_with_two_segments() -> Track {
        let mut track = Track::new(TrackId::new(1), "Route");
        let mut seg_a = TrackSegment::new(TrackSegmentId::new(10));
        seg_a.add_point(TrackPoint::new(TrackPointId::new(1), 55.0, 37.0));
        seg_a.add_point(TrackPoint::new(TrackPointId::new(2), 55.1, 37.1));
        let mut seg_b = TrackSegment::new(TrackSegmentId::new(20));
        seg_b.add_point(TrackPoint::new(TrackPointId::new(3), 55.2, 37.2));
        seg_b.add_point(TrackPoint::new(TrackPointId::new(4), 55.3, 37.3));
        track.add_segment(seg_a);
        track.add_segment(seg_b);
        track
    }

    #[test]
    fn remove_segment_returns_correct_index_and_segment() {
        let mut track = make_track_with_two_segments();

        let (index, removed) = track.remove_segment(10).unwrap();

        assert_eq!(index, 0);
        assert_eq!(removed.id(), TrackSegmentId::new(10));
        assert_eq!(track.segments().len(), 1);
        assert_eq!(track.segments()[0].id(), TrackSegmentId::new(20));
    }

    #[test]
    fn remove_segment_errors_on_missing_segment_id() {
        let mut track = make_track_with_two_segments();

        let err = track.remove_segment(99).unwrap_err();

        assert!(matches!(
            err,
            crate::domain::ProjectLayerError::MissingTrackSegment { segment_id: 99, .. }
        ));
    }

    #[test]
    fn join_segments_appends_b_points_into_a_and_removes_b() {
        let mut track = make_track_with_two_segments();

        let removed_b = track.join_segments(10, 20).unwrap();

        assert_eq!(track.segments().len(), 1);
        assert_eq!(track.segments()[0].id(), TrackSegmentId::new(10));
        assert_eq!(track.segments()[0].points().len(), 4);
        assert_eq!(track.segments()[0].points()[2].id(), TrackPointId::new(3));
        assert_eq!(track.segments()[0].points()[3].id(), TrackPointId::new(4));
        assert_eq!(removed_b.id(), TrackSegmentId::new(20));
    }

    #[test]
    fn join_segments_errors_on_non_adjacent_segments() {
        let mut track = Track::new(TrackId::new(1), "Route");
        track.add_segment(TrackSegment::new(TrackSegmentId::new(10)));
        track.add_segment(TrackSegment::new(TrackSegmentId::new(20)));
        track.add_segment(TrackSegment::new(TrackSegmentId::new(30)));

        let err = track.join_segments(10, 30).unwrap_err();

        assert!(matches!(
            err,
            crate::domain::ProjectLayerError::MissingTrackSegment { segment_id: 30, .. }
        ));
    }
}
