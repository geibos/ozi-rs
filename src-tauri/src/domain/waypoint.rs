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
    /// RGBA, or `None` for "whatever the map draws waypoints with".
    ///
    /// `None` is not a colour: a waypoint that has never been coloured follows
    /// the default, so changing that default later moves every uncoloured
    /// waypoint with it. A `.ozp` written before this field existed loads with
    /// every waypoint uncoloured — being an `Option` is what does that, since
    /// serde reads a missing `Option` field as `None`; the attribute is
    /// belt-and-braces. A non-`Option` field added here would need the
    /// attribute for the same effect.
    #[serde(default)]
    color: Option<[u8; 4]>,
    /// What the mark means, in the crew's words.
    ///
    /// A mark called «улика» is the place; the note beside it — «красная
    /// куртка, 200 м от просеки» — is what a crew is sent to. OziExplorer
    /// carries it in field 11 of a `.wpt` and GPX in `<desc>`, and this
    /// application dropped it in both directions: the place survived the
    /// exchange between headquarters and the reason for it did not.
    ///
    /// `None` for a mark with nothing to say, which is not the same as an
    /// empty note somebody typed and cleared. Every project saved before this
    /// field existed reads as `None`.
    #[serde(default)]
    description: Option<String>,
    /// Files that belong to this mark — a photograph of the find, a scan, a
    /// voice note taken at the spot.
    ///
    /// Paths, not bytes. A `.ozp` is JSON that two headquarters send each
    /// other, and a photograph inside it turns a readable text file into a
    /// megabyte of base64 that no editor opens and no diff shows. The files
    /// live beside the project, which is how everything else here works — a
    /// `.map` beside its picture, a bundle in its folder — and sending the
    /// work means sending the folder, which is what a crew already does.
    ///
    /// Relative to the project file when the file sits beside it, absolute
    /// otherwise; the path is kept as the operator's own picker gave it, and
    /// a path that no longer resolves is reported when it is opened rather
    /// than silently dropped.
    ///
    /// OziExplorer carries one such link per waypoint. This carries a list,
    /// because a find is photographed from three sides.
    #[serde(default)]
    attachments: Vec<String>,
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
            color: None,
            description: None,
            attachments: Vec::new(),
        }
    }

    pub const fn id(&self) -> WaypointId {
        self.id
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn description(&self) -> Option<&str> {
        self.description.as_deref()
    }

    /// Set the note, answering with what it was. An empty note is `None`:
    /// clearing the field means the mark has nothing to say, not that it says
    /// nothing.
    pub fn set_description(&mut self, description: Option<String>) -> Option<String> {
        let normalised = description.and_then(|d| {
            let trimmed = d.trim();
            if trimmed.is_empty() {
                None
            } else {
                Some(trimmed.to_owned())
            }
        });
        std::mem::replace(&mut self.description, normalised)
    }

    pub fn attachments(&self) -> &[String] {
        &self.attachments
    }

    /// Replace the whole list, answering what it was.
    ///
    /// Whole rather than add/remove: the undo delta needs the previous list
    /// either way, and one command that says "these are the files now" cannot
    /// get out of step with itself the way a pair can.
    ///
    /// Blank entries are dropped and duplicates are not added twice: the same
    /// photograph attached twice is a mistake, not a decision.
    pub fn set_attachments(&mut self, attachments: Vec<String>) -> Vec<String> {
        let mut cleaned: Vec<String> = Vec::with_capacity(attachments.len());
        for path in attachments {
            let trimmed = path.trim();
            if trimmed.is_empty() || cleaned.iter().any(|kept| kept == trimmed) {
                continue;
            }
            cleaned.push(trimmed.to_owned());
        }
        std::mem::replace(&mut self.attachments, cleaned)
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

    pub const fn color(&self) -> Option<[u8; 4]> {
        self.color
    }

    /// Set or clear the colour, returning the previous one for the undo delta.
    pub fn set_color(&mut self, color: Option<[u8; 4]>) -> Option<[u8; 4]> {
        std::mem::replace(&mut self.color, color)
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
    fn waypoint_attachments_start_empty_and_round_trip() {
        let mut waypoint = Waypoint::new(WaypointId::new(8), "улика", 53.9, 27.5);
        assert!(waypoint.attachments().is_empty());

        let previous = waypoint.set_attachments(vec![
            "находки/куртка-1.jpg".to_owned(),
            "находки/куртка-2.jpg".to_owned(),
        ]);
        assert!(
            previous.is_empty(),
            "the previous list is what undo restores"
        );
        assert_eq!(waypoint.attachments().len(), 2);

        let json = serde_json::to_string(&waypoint).expect("serialize");
        let restored: Waypoint = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(restored.attachments(), waypoint.attachments());
    }

    #[test]
    fn waypoint_attachments_drop_blanks_and_repeats() {
        let mut waypoint = Waypoint::new(WaypointId::new(8), "улика", 53.9, 27.5);
        waypoint.set_attachments(vec![
            "  находки/куртка.jpg  ".to_owned(),
            "".to_owned(),
            "   ".to_owned(),
            "находки/куртка.jpg".to_owned(),
        ]);
        // The same photograph attached twice is a mistake, not a decision.
        assert_eq!(waypoint.attachments(), ["находки/куртка.jpg"]);
    }

    #[test]
    fn waypoint_without_attachments_field_loads_with_none() {
        // Every project saved before this field existed.
        let legacy = r#"{
            "id": 8,
            "name": "улика",
            "symbol": null,
            "latitude": 53.9,
            "longitude": 27.5
        }"#;
        let waypoint: Waypoint = serde_json::from_str(legacy).expect("legacy");
        assert!(waypoint.attachments().is_empty());
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
