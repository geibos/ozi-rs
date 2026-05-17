#![allow(dead_code)]

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[serde(transparent)]
pub struct WaypointId(u64);

impl WaypointId {
    pub const fn new(value: u64) -> Self {
        Self(value)
    }

    pub const fn value(&self) -> u64 {
        self.0
    }
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Waypoint {
    id: WaypointId,
    name: String,
    symbol: Option<String>,
    latitude: f64,
    longitude: f64,
    /// Whether this waypoint is rendered on the map. Defaults to `true`.
    ///
    /// Marked with `#[serde(default = "default_true")]` so legacy `.ozp`
    /// files without the field deserialize as visible.
    #[serde(default = "default_true")]
    visible: bool,
}

fn default_true() -> bool {
    true
}

impl Waypoint {
    pub fn new(id: WaypointId, name: impl Into<String>, latitude: f64, longitude: f64) -> Self {
        Self {
            id,
            name: name.into(),
            symbol: None,
            latitude,
            longitude,
            visible: true,
        }
    }

    pub const fn id(&self) -> WaypointId {
        self.id
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn symbol(&self) -> Option<&str> {
        self.symbol.as_deref()
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

    pub fn set_name(&mut self, name: String) -> String {
        std::mem::replace(&mut self.name, name)
    }

    pub fn set_symbol(&mut self, symbol: Option<String>) -> Option<String> {
        std::mem::replace(&mut self.symbol, symbol)
    }

    pub const fn visible(&self) -> bool {
        self.visible
    }

    /// Flip the visibility flag and return the new value.
    ///
    /// Visibility is a non-undoable style mutation; mirrors `TrackStyle.visible`.
    pub fn toggle_visible(&mut self) -> bool {
        self.visible = !self.visible;
        self.visible
    }

    pub fn set_visible(&mut self, visible: bool) {
        self.visible = visible;
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

    #[test]
    fn waypoint_default_visible_is_true_and_toggle_flips() {
        let mut waypoint = Waypoint::new(WaypointId::new(8), "Camp", 53.9, 27.5667);

        assert!(waypoint.visible(), "new waypoints SHALL default to visible");
        assert!(!waypoint.toggle_visible());
        assert!(!waypoint.visible());
        assert!(waypoint.toggle_visible());
        assert!(waypoint.visible());
    }

    #[test]
    fn waypoint_serde_round_trip_preserves_visible_field() {
        let mut waypoint = Waypoint::new(WaypointId::new(8), "Camp", 53.9, 27.5667);
        waypoint.set_visible(false);

        let json = serde_json::to_string(&waypoint).expect("serialize");
        assert!(
            json.contains("\"visible\":false"),
            "serialized JSON must include visible flag: {json}"
        );

        let restored: Waypoint = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(restored, waypoint);
        assert!(!restored.visible());
    }

    #[test]
    fn waypoint_legacy_json_without_visible_field_deserializes_as_visible() {
        // Pre-change `.ozp` files have no `visible` field — they must
        // deserialize as visible.
        let legacy = r#"{
            "id": 8,
            "name": "Camp",
            "symbol": null,
            "latitude": 53.9,
            "longitude": 27.5667
        }"#;

        let waypoint: Waypoint = serde_json::from_str(legacy).expect("legacy deserialize");
        assert!(
            waypoint.visible(),
            "missing `visible` field SHALL deserialize as `true` for backward compatibility"
        );
        assert_eq!(waypoint.name(), "Camp");
    }

    #[test]
    fn waypoint_setters_return_previous_values_for_undo() {
        let mut waypoint = Waypoint::new(WaypointId::new(8), "Camp", 53.9, 27.5667);

        assert_eq!(waypoint.symbol(), None);
        assert_eq!(waypoint.set_name("Base camp".to_owned()), "Camp");
        assert_eq!(waypoint.name(), "Base camp");
        assert_eq!(waypoint.set_symbol(Some("Flag".to_owned())), None);
        assert_eq!(waypoint.symbol(), Some("Flag"));
        assert_eq!(waypoint.set_symbol(None), Some("Flag".to_owned()));
        assert_eq!(waypoint.symbol(), None);
    }
}
