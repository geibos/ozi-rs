/**
 * Track-points data and formatting helpers.
 *
 * Extracted from the legacy `TrackPointsPanel.svelte` floating panel as
 * part of the `redesign-library-sidebar` change. The track-points UI is
 * being relocated into the Inspector pane delivered by the follow-up
 * `redesign-inspector-pane` change. To keep that change small, this module
 * parks the points-table data layer (fetcher, formatters, pagination
 * helper) in a stable, importable location — pure TypeScript, no Svelte
 * runes, no store subscriptions, no DOM.
 *
 * The intended consumer is the Inspector pane; the legacy
 * `TrackPointsPanel.svelte` floating card was deleted in the same change
 * that introduced this module.
 */

import type { PointDetail, SegmentDetail, TrackDetail } from "./types";
import { getTrackDetail } from "./api";

/**
 * Maximum number of points rendered up-front per segment. Larger segments
 * paginate behind a "Show N more" affordance. The value matches the
 * historical limit `TrackPointsPanel.svelte` used so the Inspector pane
 * does not surprise the user with a different perceived performance
 * profile.
 */
export const TRACK_POINTS_INITIAL_LIMIT = 1000;

/**
 * Re-export of the typed API wrapper. The Inspector pane (and any future
 * consumer) MUST go through this rather than re-deriving the IPC call so
 * that downstream refactors of the command name happen in one place.
 */
export async function loadTrackDetail(
  layerId: bigint,
  trackId: bigint,
): Promise<TrackDetail> {
  return getTrackDetail(layerId, trackId);
}

/**
 * Format a point coordinate pair as `<lat>, <lon>` with five decimal
 * places — roughly 1 m precision at the equator. Matches the legacy
 * panel's display verbatim.
 */
export function formatCoordinates(point: PointDetail): string {
  return `${point.lat.toFixed(5)}, ${point.lon.toFixed(5)}`;
}

/**
 * Format an elevation value as `<m>m` with one decimal place. Returns
 * `null` if the point has no elevation field.
 */
export function formatElevation(point: PointDetail): string | null {
  if (point.elevation === undefined || point.elevation === null) {
    return null;
  }
  return `${point.elevation.toFixed(1)}m`;
}

/**
 * Returns the backend-provided timestamp as-is, or `null` if the point
 * lacks one. The legacy panel rendered the timestamp only when present
 * (no "No timestamp" placeholder); callers preserve that contract.
 */
export function formatTimestamp(point: PointDetail): string | null {
  if (!point.timestamp) return null;
  return point.timestamp;
}

/**
 * Result of paginating a single segment's points list. `displayed` are
 * the points the UI renders this turn; `remaining` is the count behind
 * the "Show N more" button (0 when the segment is fully shown or
 * `expanded` is true).
 */
export interface PaginatedSegment {
  segmentId: number;
  displayed: PointDetail[];
  remaining: number;
}

/**
 * Slice a segment's points to the initial limit unless `expanded` is
 * true. Pure function over plain data — no side effects, safe to call
 * from any rendering layer.
 */
export function paginateSegment(
  segment: SegmentDetail,
  expanded: boolean,
  limit: number = TRACK_POINTS_INITIAL_LIMIT,
): PaginatedSegment {
  if (expanded || segment.points.length <= limit) {
    return {
      segmentId: segment.id,
      displayed: segment.points,
      remaining: 0,
    };
  }
  return {
    segmentId: segment.id,
    displayed: segment.points.slice(0, limit),
    remaining: segment.points.length - limit,
  };
}

/**
 * Compute the per-segment displayed slice for an entire track detail.
 * `expandedSegmentIds` is a set of segment IDs whose "Show N more" has
 * already been clicked.
 */
export function paginateTrackDetail(
  detail: TrackDetail,
  expandedSegmentIds: ReadonlySet<number>,
  limit: number = TRACK_POINTS_INITIAL_LIMIT,
): PaginatedSegment[] {
  return detail.segments.map((segment) =>
    paginateSegment(segment, expandedSegmentIds.has(segment.id), limit),
  );
}

// ── Inspector-compat helpers ────────────────────────────────────────────────
// The Track Inspector (redesign-inspector-pane) was implemented in parallel
// with this module and anticipated slightly different naming + return shape.
// Rather than churn the Inspector's call sites, the canonical helpers above
// get thin aliases that match the Inspector's expected API.

export const formatPointCoords = formatCoordinates;
export const formatPointTimestamp = formatTimestamp;

/**
 * One-line header for a track segment row in the Inspector's segments table.
 */
export function segmentHeader(segment: SegmentDetail): string {
  return `Segment ${segment.id} · ${segment.points.length} pts`;
}

export interface InspectorPagedSegment {
  /** Points that should currently be rendered in the segment row. */
  visible: PointDetail[];
  /** How many additional points are clipped behind a "Show N more" affordance. */
  hiddenCount: number;
}

/**
 * Adapter around `paginateSegment` returning the field names the Inspector's
 * table template expects (`visible` / `hiddenCount`).
 */
export function pageSegmentPoints(
  segment: SegmentDetail,
  expanded: boolean,
): InspectorPagedSegment {
  const paged = paginateSegment(segment, expanded);
  return { visible: paged.displayed, hiddenCount: paged.remaining };
}
