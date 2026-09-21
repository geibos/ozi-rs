/**
 * Row model for the Library Tracks tab.
 *
 * The rows come from `list_tracks`, which returns one per track and no
 * geometry. They used to be derived from the map's GeoJSON, which cost every
 * coordinate of every track over IPC to draw a list of names, and which omits
 * a track with nothing drawable — right for the map, wrong for a list, because
 * a one-point track then had no row and could not be renamed or deleted.
 */
import { filterByName } from "./name-filter";
import type { TrackSummaryDto } from "./bindings";

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

/** Sort by (layer, track) so rows belonging to one layer stay together. */
function sortKey(t: TrackFeature): string {
  return `${t.layerId.toString().padStart(20, "0")}:${t.trackId
    .toString()
    .padStart(20, "0")}`;
}

/** The rows as `list_tracks` returns them, in the tab's order. */
export function trackFeaturesFromSummaries(
  summaries: TrackSummaryDto[],
): TrackFeature[] {
  const rows = summaries.map(
    (s) =>
      ({
        layerId: BigInt(s.layer_id),
        trackId: BigInt(s.track_id),
        name: s.name,
        color: s.color,
        lineWidth: s.line_width,
        visible: s.visible,
        distanceKm: s.distance_km,
        durationSeconds:
          s.duration_seconds === null ? null : Number(s.duration_seconds),
        pointCount: s.point_count,
      }) satisfies TrackFeature,
  );
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
