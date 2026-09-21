/**
 * Row model for the Library Tracks tab, derived from the tracks GeoJSON.
 *
 * Track geometry is one `MultiLineString` per track (one part per segment);
 * older data and the drawing preview still produce `LineString`. Both count as
 * a track — filtering on a single geometry type once emptied the whole rail
 * while the map kept drawing every track.
 */
import { filterByName } from "./name-filter";

export interface TrackFeature {
  layerId: bigint;
  trackId: bigint;
  name: string;
  color: string;
  lineWidth: number;
  visible: boolean;
  distanceKm: number;
  durationSeconds: number | null;
  pointCount: number;
}

const LINE_GEOMETRIES = new Set(["LineString", "MultiLineString"]);

/** Sort by (layer, track) so rows belonging to one layer stay together. */
function sortKey(t: TrackFeature): string {
  return `${t.layerId.toString().padStart(20, "0")}:${t.trackId
    .toString()
    .padStart(20, "0")}`;
}

export function trackFeaturesFromGeojson(
  geojson: GeoJSON.FeatureCollection,
): TrackFeature[] {
  const rows = geojson.features
    .filter((f) => f.geometry && LINE_GEOMETRIES.has(f.geometry.type))
    .map((f) => {
      const props = f.properties ?? {};
      const rawDuration = props.duration_seconds as number | null | undefined;
      return {
        layerId: BigInt(props.layer_id as number),
        trackId: BigInt(props.track_id as number),
        name: props.name as string,
        color: props.color as string,
        lineWidth: Number(props.line_width ?? 3),
        visible: props.visible as boolean,
        distanceKm: Number(props.distance_km ?? 0),
        durationSeconds:
          rawDuration === null || rawDuration === undefined
            ? null
            : Number(rawDuration),
        pointCount: Number(props.point_count ?? 0),
      } satisfies TrackFeature;
    });
  rows.sort((a, b) => sortKey(a).localeCompare(sortKey(b)));
  return rows;
}

/** Narrow the rows to those whose name contains `query` (see `filterByName`). */
export function filterTrackFeatures<T extends { name: string }>(
  rows: T[],
  query: string,
): T[] {
  return filterByName(rows, query, (row) => row.name);
}
