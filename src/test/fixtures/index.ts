/**
 * The wire snapshots the Rust core writes, typed against the generated
 * bindings.
 *
 * Hand-written mocks drift: the Tracks tab kept filtering for a geometry type
 * the backend had stopped emitting and every test still passed. These come
 * from the same mappers the commands use (`src-tauri/src/fixtures.rs`), and
 * `cargo test fixtures_are_up_to_date` fails when a DTO change has not been
 * regenerated here with `just fixtures`.
 */
import appStateJson from "./app-state.json";
import coldStartJson from "./app-state-cold.json";
import tracksGeojsonJson from "./tracks-geojson.json";
import tracksListJson from "./tracks-list.json";
import catalogueJson from "./catalogue.json";
import trackDetailJson from "./track-detail.json";
import waypointsJson from "./waypoints.json";

import type { LizaProjectSummaryDto } from "$lib/types";
import type {
  AppStateDto,
  TrackDetailDto,
  TrackSummaryDto,
  WaypointDto,
} from "$lib/bindings";

/** A project shaped like one search: two tracks, one of them hidden. */
export const appStateFixture = appStateJson as AppStateDto;

/**
 * The catalogue, as the `projects-chunk` stream delivers it.
 *
 * Its own fixture because it is its own stream: it left the state snapshot on
 * 2026-09-21, and the stand's project list went empty until this replaced it.
 */
export const catalogueFixture = catalogueJson as LizaProjectSummaryDto[];

/** The app's first screen: the catalogue is loaded, nothing is open. */
export const coldStartFixture = coldStartJson as unknown as AppStateDto;

/** The same project's tracks, as the map layer receives them. */
export const tracksGeojsonFixture =
  tracksGeojsonJson as unknown as GeoJSON.FeatureCollection;

/**
 * The same project's tracks, as the Tracks tab receives them.
 *
 * Three rows against the geometry's two: the map cannot draw a track of one
 * point, and for a while the list inherited that omission.
 */
export const tracksListFixture = tracksListJson as unknown as TrackSummaryDto[];

/** The two-segment track, as the inspector receives it. */
export const trackDetailFixture = trackDetailJson as unknown as TrackDetailDto;

/** The project's waypoints, as the Waypoints tab receives them. */
export const waypointsFixture = waypointsJson as unknown as WaypointDto[];

/** Ids the fixtures use, so tests do not hard-code them twice. */
export const FIXTURE_TRACK_LAYER_ID = 1n;
export const FIXTURE_WAYPOINT_LAYER_ID = 1n;
export const FIXTURE_DETAILED_TRACK_ID = 1n;
