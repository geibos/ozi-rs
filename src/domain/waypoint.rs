#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct WaypointId(u64);

impl WaypointId {
    pub const fn new(value: u64) -> Self {
        Self(value)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Waypoint {
    id: WaypointId,
    name: String,
    latitude: f64,
    longitude: f64,
}

impl Waypoint {
    pub fn new(id: WaypointId, name: impl Into<String>, latitude: f64, longitude: f64) -> Self {
        Self {
            id,
            name: name.into(),
            latitude,
            longitude,
        }
    }

    pub const fn id(&self) -> WaypointId {
        self.id
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub const fn latitude(&self) -> f64 {
        self.latitude
    }

    pub const fn longitude(&self) -> f64 {
        self.longitude
    }

    pub fn move_to(&mut self, latitude: f64, longitude: f64) {
        self.latitude = latitude;
        self.longitude = longitude;
    }
}

#[cfg(test)]
mod tests {
    use super::{Waypoint, WaypointId};

    #[test]
    fn waypoint_preserves_name_and_coordinates() {
        let waypoint = Waypoint::new(WaypointId::new(8), "Camp", 53.9, 27.5667);

        assert_eq!(waypoint.id(), WaypointId::new(8));
        assert_eq!(waypoint.name(), "Camp");
        assert_eq!(waypoint.latitude(), 53.9);
        assert_eq!(waypoint.longitude(), 27.5667);
    }

    #[test]
    fn waypoint_move_updates_coordinates() {
        let mut waypoint = Waypoint::new(WaypointId::new(8), "Camp", 53.9, 27.5667);

        waypoint.move_to(54.1, 27.8);

        assert_eq!(waypoint.latitude(), 54.1);
        assert_eq!(waypoint.longitude(), 27.8);
    }
}
