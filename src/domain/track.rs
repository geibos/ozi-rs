#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[serde(transparent)]
pub struct TrackId(u64);

impl TrackId {
    pub const fn new(value: u64) -> Self {
        Self(value)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[serde(transparent)]
pub struct TrackSegmentId(u64);

impl TrackSegmentId {
    pub const fn new(value: u64) -> Self {
        Self(value)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[serde(transparent)]
pub struct TrackPointId(u64);

impl TrackPointId {
    pub const fn new(value: u64) -> Self {
        Self(value)
    }
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct TrackPoint {
    id: TrackPointId,
    latitude: f64,
    longitude: f64,
}

impl TrackPoint {
    pub const fn new(id: TrackPointId, latitude: f64, longitude: f64) -> Self {
        Self {
            id,
            latitude,
            longitude,
        }
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
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Track {
    id: TrackId,
    name: String,
    segments: Vec<TrackSegment>,
}

impl Track {
    pub fn new(id: TrackId, name: impl Into<String>) -> Self {
        Self {
            id,
            name: name.into(),
            segments: Vec::new(),
        }
    }

    pub const fn id(&self) -> TrackId {
        self.id
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn segments(&self) -> &[TrackSegment] {
        &self.segments
    }

    pub fn add_segment(&mut self, segment: TrackSegment) {
        self.segments.push(segment);
    }
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
}
