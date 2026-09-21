//! A deterministic project, and the wire snapshots taken from it.
//!
//! The frontend's component tests need data that looks like a real search:
//! Cyrillic track names, a track recorded in two sittings, a waypoint with a
//! symbol and one hidden, a catalogue where some bundles are on disk. Writing
//! that by hand in the tests produced mocks that drifted from what the backend
//! actually sends — which is how the Tracks tab came to filter for a geometry
//! type the backend had stopped emitting.
//!
//! These fixtures are built by the same mappers the commands use, so a change
//! to a DTO shows up as a diff here (and `fixtures_are_up_to_date` fails until
//! it is regenerated with `just fixtures`).

use crate::application::{
    ActiveMapKind, ActiveMapSelection, AppState, BundleEntry, LizaMapPackage, LizaProject,
    LizaProjectSummary, MapCenter,
};
use crate::domain::{
    LayerId, Track, TrackId, TrackPoint, TrackPointId, TrackSegment, TrackSegmentId, Waypoint,
    WaypointId, WaypointLayer,
};
use std::collections::HashSet;
use std::path::{Path, PathBuf};

/// Layer the fixture's tracks live in — the default "Tracks" layer.
pub const TRACK_LAYER_ID: u64 = 1;
/// Layer the fixture's waypoints live in — the default "Waypoints" layer.
pub const WAYPOINT_LAYER_ID: u64 = 1;
/// The two-segment track, the one worth opening in the inspector.
pub const DETAILED_TRACK_ID: u64 = 1;

fn ts(day: u32, hour: u32, minute: u32) -> chrono::DateTime<chrono::Utc> {
    use chrono::TimeZone;
    chrono::Utc
        .with_ymd_and_hms(2026, 7, day, hour, minute, 0)
        .single()
        .expect("a real instant")
}

fn segment(
    id: u64,
    first_point_id: u64,
    points: &[(f64, f64)],
    start: (u32, u32, u32),
) -> TrackSegment {
    let mut segment = TrackSegment::new(TrackSegmentId::new(id));
    for (index, (lat, lon)) in points.iter().enumerate() {
        segment.add_point(
            TrackPoint::new(TrackPointId::new(first_point_id + index as u64), *lat, *lon)
                .with_elevation(30.0 + index as f64)
                .with_timestamp(ts(start.0, start.1, start.2 + index as u32)),
        );
    }
    segment
}

/// A project shaped like one search: two tracks in one layer (one of them
/// recorded across two sittings), three waypoints, one of them hidden.
pub fn sample_app_state() -> AppState {
    let mut state = AppState::new();
    let layer_id = LayerId::new(TRACK_LAYER_ID);

    // The track an operator opens: two segments, a real gap between them.
    let mut veter = Track::new(TrackId::new(DETAILED_TRACK_ID), "20260708_Veter2");
    veter.add_segment(segment(
        1,
        1,
        &[
            (59.95243, 31.59681),
            (59.95335, 31.60164),
            (59.95384, 31.60305),
        ],
        (8, 9, 0),
    ));
    veter.add_segment(segment(
        2,
        10,
        &[(59.94455, 31.65927), (59.94659, 31.67108)],
        (8, 14, 0),
    ));
    veter.style_mut().color = [220, 38, 38, 255];
    veter.style_mut().line_width = 3.0;

    // A second track, hidden, with a name that does not follow the convention.
    let mut lisa = Track::new(TrackId::new(2), "лиса15 вечер");
    lisa.add_segment(segment(
        3,
        20,
        &[(59.95196, 31.61564), (59.94976, 31.70290)],
        (9, 11, 0),
    ));
    lisa.style_mut().color = [37, 99, 235, 255];
    lisa.style_mut().visible = false;

    state
        .project_mut()
        .add_track_to_layer(layer_id, veter)
        .expect("track layer 1 exists in a fresh project");
    // A track the map cannot draw: one point, no line. It is in the project,
    // counts towards it and exports with it, and until 2026-09-21 the Tracks
    // tab had no row for it, because the rows came from the map's features.
    let mut stray = Track::new(TrackId::new(3), "точка отсечки");
    stray.add_segment(segment(4, 30, &[(59.95000, 31.62000)], (10, 2, 0)));
    stray.style_mut().color = [22, 163, 74, 255];

    state
        .project_mut()
        .add_track_to_layer(layer_id, lisa)
        .expect("track layer 1 exists in a fresh project");
    state
        .project_mut()
        .add_track_to_layer(layer_id, stray)
        .expect("track layer 1 exists in a fresh project");

    // A second track layer, the shape an import leaves behind.
    state
        .project_mut()
        .add_track_layer(crate::domain::TrackLayer::new(
            LayerId::new(2),
            "20260709_Veter3.gpx",
        ));

    let waypoint_layer = LayerId::new(WAYPOINT_LAYER_ID);
    for (id, name, lat, lon, symbol, visible) in [
        (1u64, "ШТАБ", 59.95243, 31.59681, Some("flag"), true),
        (2, "ЗАБРОС", 59.951938, 31.596359, None, true),
        (3, "Проход у полю", 59.94683, 31.69150, Some("camp"), false),
    ] {
        let mut waypoint = Waypoint::new(WaypointId::new(id), name, lat, lon);
        if let Some(symbol) = symbol {
            waypoint.set_symbol(Some(symbol.to_owned()));
        }
        waypoint.set_visible(visible);
        state
            .project_mut()
            .add_waypoint_to_layer(waypoint_layer, waypoint)
            .expect("waypoint layer 1 exists in a fresh project");
    }
    state.project_mut().add_waypoint_layer(WaypointLayer::new(
        LayerId::new(2),
        "Waypoints_20260709.gpx",
    ));

    state.set_fixture_catalogue(
        vec![
            LizaProjectSummary {
                slug: "2026-07-08_Lavrovo".to_owned(),
                name: "2026 07 08 Lavrovo".to_owned(),
                url: "https://maps.lizaalert.ru/maps/2026-07-08_Lavrovo/".to_owned(),
            },
            LizaProjectSummary {
                slug: "2026-07-14_Sagra".to_owned(),
                name: "2026 07 14 Sagra".to_owned(),
                url: "https://maps.lizaalert.ru/maps/2026-07-14_Sagra/".to_owned(),
            },
        ],
        Some(LizaProject {
            summary: LizaProjectSummary {
                slug: "2026-07-08_Lavrovo".to_owned(),
                name: "2026 07 08 Lavrovo".to_owned(),
                url: "https://maps.lizaalert.ru/maps/2026-07-08_Lavrovo/".to_owned(),
            },
            center: MapCenter {
                lat: 59.95243,
                lon: 31.59681,
            },
            maps: vec![
                LizaMapPackage {
                    name: "2026-07-08_Lavrovo_Topo_EEKO_z16.sqlitedb".to_owned(),
                    file_name: "2026-07-08_Lavrovo_Topo_EEKO_z16.sqlitedb".to_owned(),
                    url: "https://maps.lizaalert.ru/maps/2026-07-08_Lavrovo/8-Android&iOS/2026-07-08_Lavrovo_Topo_EEKO_z16.sqlitedb".to_owned(),
                    base_zoom: 16,
                    local_path: Some(PathBuf::from(
                        "/Users/operator/Documents/LizaAlert Maps/2026-07-08_Lavrovo/8-Android&iOS/2026-07-08_Lavrovo_Topo_EEKO_z16.sqlitedb",
                    )),
                    size_bytes: Some(16_672_522),
                },
                LizaMapPackage {
                    name: "2026-07-08_Lavrovo_Satell_z17.sqlitedb".to_owned(),
                    file_name: "2026-07-08_Lavrovo_Satell_z17.sqlitedb".to_owned(),
                    url: "https://maps.lizaalert.ru/maps/2026-07-08_Lavrovo/8-Android&iOS/2026-07-08_Lavrovo_Satell_z17.sqlitedb".to_owned(),
                    base_zoom: 17,
                    local_path: None,
                    size_bytes: Some(194_093_875),
                },
            ],
            contents: vec![
                BundleEntry {
                    name: "2-Coordinates.txt".to_owned(),
                    is_dir: false,
                    size_bytes: Some(165),
                },
                BundleEntry {
                    name: "6-Ozi(Win&Android)_Satell.zip".to_owned(),
                    is_dir: false,
                    size_bytes: Some(56_623_104),
                },
                BundleEntry {
                    name: "8-Android&iOS".to_owned(),
                    is_dir: true,
                    size_bytes: None,
                },
                BundleEntry {
                    name: "9-Map_4_print".to_owned(),
                    is_dir: true,
                    size_bytes: None,
                },
                BundleEntry {
                    name: "10-Tracks".to_owned(),
                    is_dir: true,
                    size_bytes: None,
                },
            ],
        }),
        // Tiles rather than an OZI raster: an OZF2 map's calibration can only
        // be read from the file itself, so a fixture that claimed one would
        // force the stand to invent metadata. The bundle case needs none.
        Some(ActiveMapSelection {
            kind: ActiveMapKind::SqliteTiles,
            project_name: "2026 07 08 Lavrovo".to_owned(),
            package_name: "2026-07-08_Lavrovo_Topo_EEKO_z16.sqlitedb".to_owned(),
            remote_url: "https://maps.lizaalert.ru/maps/2026-07-08_Lavrovo/8-Android&iOS/2026-07-08_Lavrovo_Topo_EEKO_z16.sqlitedb".to_owned(),
            local_path: PathBuf::from(
                "/Users/operator/Documents/LizaAlert Maps/2026-07-08_Lavrovo/8-Android&iOS/2026-07-08_Lavrovo_Topo_EEKO_z16.sqlitedb",
            ),
            center: MapCenter {
                lat: 59.95243,
                lon: 31.59681,
            },
            base_zoom: 16,
        }),
        "Loaded project: 2026 07 08 Lavrovo",
    );

    state
}

/// The app's first screen: the catalogue is loaded, nothing is open.
///
/// The cold-start route is what a crew sees when they launch the app, and it
/// had never been looked at — the other fixture carries an active map, so the
/// workspace always won the redirect.
pub fn cold_start_app_state() -> AppState {
    let mut state = AppState::new();
    state.set_fixture_catalogue(
        vec![
            LizaProjectSummary {
                slug: "2026-07-08_Lavrovo".to_owned(),
                name: "2026 07 08 Lavrovo".to_owned(),
                url: "https://maps.lizaalert.ru/maps/2026-07-08_Lavrovo/".to_owned(),
            },
            LizaProjectSummary {
                slug: "2026-07-14_Sagra".to_owned(),
                name: "2026 07 14 Sagra".to_owned(),
                url: "https://maps.lizaalert.ru/maps/2026-07-14_Sagra/".to_owned(),
            },
            LizaProjectSummary {
                slug: "2026-09-20_Schuvalovo".to_owned(),
                name: "2026 09 20 Schuvalovo".to_owned(),
                url: "https://maps.lizaalert.ru/maps/2026-09-20_Schuvalovo/".to_owned(),
            },
        ],
        None,
        None,
        "Load projects from maps.lizaalert.ru",
    );
    state
}

/// The bundles the fixture's catalogue reports as already downloaded.
fn cached_slugs() -> HashSet<String> {
    HashSet::from(["2026-07-08_Lavrovo".to_owned()])
}

/// Write every fixture under `dir`, returning the files written.
pub fn write_fixtures(dir: &Path) -> std::io::Result<Vec<PathBuf>> {
    std::fs::create_dir_all(dir)?;
    let state = sample_app_state();

    let app_state = crate::commands::app_state_dto(&state);
    let tracks_geojson = crate::commands::build_tracks_geojson(state.track_layers());
    let tracks_list = crate::commands::list_track_summaries(state.track_layers());
    let track = state.track_layers()[0]
        .tracks()
        .iter()
        .find(|t| t.id().value() == DETAILED_TRACK_ID)
        .expect("the detailed track");
    let track_detail = crate::commands::track_detail_dto(track);
    let waypoints = crate::commands::waypoint_dtos(state.project_waypoint_layers()[0].waypoints());

    let cold_state = cold_start_app_state();
    let cold = crate::commands::app_state_dto(&cold_state);
    // The catalogue is its own stream now, not part of the state snapshot, so
    // it needs a fixture of its own — otherwise the stand shows an empty
    // project list, which is exactly what happened when it left the snapshot.
    let catalogue =
        crate::commands::to_project_summary_dtos(cold_state.fixture_catalogue(), &cached_slugs());

    let files: Vec<(&str, serde_json::Value)> = vec![
        (
            "app-state-cold.json",
            serde_json::to_value(&cold).expect("serialize cold-start state"),
        ),
        (
            "app-state.json",
            serde_json::to_value(&app_state).expect("serialize app state"),
        ),
        // `.json`, not `.geojson`: the frontend bundler only parses the
        // former as data, and a fixture the tests cannot import is no fixture.
        ("tracks-geojson.json", tracks_geojson),
        // The rows the Tracks tab reads. Written separately because they are
        // not the map's features: the map omits a track it cannot draw, and a
        // list that inherited that omission was how a one-point track ended up
        // with no row at all.
        (
            "catalogue.json",
            serde_json::to_value(&catalogue).expect("serialize catalogue chunk"),
        ),
        (
            "tracks-list.json",
            serde_json::to_value(&tracks_list).expect("serialize track rows"),
        ),
        (
            "track-detail.json",
            serde_json::to_value(&track_detail).expect("serialize track detail"),
        ),
        (
            "waypoints.json",
            serde_json::to_value(&waypoints).expect("serialize waypoints"),
        ),
    ];

    let mut written = Vec::new();
    for (name, value) in files {
        let path = dir.join(name);
        let mut json = serde_json::to_string_pretty(&value).expect("pretty json");
        json.push('\n');
        std::fs::write(&path, json)?;
        written.push(path);
    }

    Ok(written)
}

/// Where the committed fixtures live, relative to the workspace root.
pub fn fixtures_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("workspace root")
        .join("src/test/fixtures")
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Regenerates the fixtures and fails when the committed ones were stale,
    /// the same role the bindings test plays: a DTO change that the frontend
    /// mocks have not seen is a bug waiting in a component test.
    #[test]
    fn fixtures_are_up_to_date() {
        let dir = fixtures_dir();
        let before: Vec<(PathBuf, String)> = std::fs::read_dir(&dir)
            .map(|entries| {
                entries
                    .filter_map(|e| e.ok())
                    .map(|e| {
                        let path = e.path();
                        let contents = std::fs::read_to_string(&path).unwrap_or_default();
                        (path, contents)
                    })
                    .collect()
            })
            .unwrap_or_default();

        let written = write_fixtures(&dir).expect("write fixtures");

        for path in &written {
            let after = std::fs::read_to_string(path).expect("fixture written");
            let previous = before
                .iter()
                .find(|(p, _)| p == path)
                .map(|(_, c)| c.clone())
                .unwrap_or_default();
            // Deliberately not `assert_eq!`: the payloads are whole JSON
            // documents and printing both of them buries the one line that
            // matters — which file to look at.
            assert!(
                previous == after,
                "{} was stale; it has just been regenerated — review the diff and commit it",
                path.display()
            );
        }
    }

    /// The fixture has to look like a real search, or the screens rendered
    /// against it prove nothing.
    #[test]
    fn the_sample_project_covers_the_states_a_screen_has_to_handle() {
        let state = sample_app_state();
        let dto = crate::commands::app_state_dto(&state);

        assert!(
            dto.tracks.iter().any(|t| !t.visible),
            "a hidden track, so the rail's dimmed row is exercised"
        );
        assert!(
            dto.tracks.iter().any(|t| t.name.contains('л')),
            "a Cyrillic name, which is what the crews type"
        );
        assert!(
            dto.track_layers.len() > 1 && dto.waypoint_layers.len() > 1,
            "more than one layer of each kind, so the selectors have something to select"
        );
        let project = dto.current_project.as_ref().expect("a current project");
        assert!(
            project.maps.iter().any(|m| m.downloaded) && project.maps.iter().any(|m| !m.downloaded),
            "one map on disk and one not, so both badges are exercised"
        );
        assert!(
            project.maps.iter().all(|m| m.size_bytes.is_some()),
            "sizes, so the row that states them is exercised"
        );
        // The catalogue no longer rides in the application state, so there is
        // nothing to assert about it here. Cached bundles are exercised
        // through `cached_slugs` on the chunk path instead.
        assert!(
            !cached_slugs().is_empty(),
            "a bundle already on disk, for the chunk path to mark"
        );
    }
}
