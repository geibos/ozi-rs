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
}
