// STUB — replaced by `redesign-library-sidebar` at merge time. Do NOT extend this stub.
//
// This file is created by the `redesign-inspector-pane` agent to keep its
// branch self-compiling. The `redesign-library-sidebar` agent ships the
// full pure-TypeScript extraction of `TrackPointsPanel.svelte`'s helpers
// (data load, coordinate / timestamp formatting, optional pagination /
// per-segment speeds). At merge time the parent worktree takes that file
// wholesale; this stub is silently overwritten.
//
// Inspector pane (the consumer) imports only the helpers below. The
// signatures are a superset of what is strictly needed so that any other
// caller already wired up in the library agent's branch keeps compiling
// after the merge.

import { getTrackDetail } from "./api";
import type { PointDetail, SegmentDetail, TrackDetail } from "./types";

/**
 * Wrapper around the IPC `getTrackDetail` call. Keeps the call site free of
 * the `invoke` import and matches what the legacy `TrackPointsPanel.svelte`
 * did inline.
 */
export async function loadTrackDetail(
  layerId: bigint,
  trackId: bigint,
): Promise<TrackDetail> {
  return getTrackDetail(layerId, trackId);
}

/**
 * Format a `(lat, lon[, elevation])` triplet for the points-table cell.
 * Matches the legacy panel's inline `.toFixed(5)` + optional elevation
 * suffix.
 */
export function formatPointCoords(point: PointDetail): string {
  const base = `${point.lat.toFixed(5)}, ${point.lon.toFixed(5)}`;
  if (point.elevation === undefined || point.elevation === null) return base;
  return `${base}, ele=${point.elevation.toFixed(1)}m`;
}

/**
 * Return the point's backend timestamp verbatim when present. Returns null
 * when the point has no timestamp — the caller decides whether to render a
 * second line at all (the legacy panel hides the line entirely on null).
 */
export function formatPointTimestamp(point: PointDetail): string | null {
  return point.timestamp ?? null;
}

/**
 * Total point count for a `TrackDetail` (sum across all segments). Used by
 * the Inspector stats card when the track summary's `point_count` is not
 * trusted by the segments view (e.g. after an inline edit).
 */
export function totalPointCount(detail: TrackDetail | null): number {
  if (!detail) return 0;
  return detail.segments.reduce((acc, seg) => acc + seg.points.length, 0);
}

/**
 * Segment-level "summary" the table header renders ("Segment N (M points)").
 * Pulled out as a helper so the Inspector and any future read-only viewer
 * stay in sync.
 */
export function segmentHeader(segment: SegmentDetail): string {
  return `Segment ${segment.id} (${segment.points.length} points)`;
}

/**
 * Page a segment's points to a fixed window. The legacy panel rendered the
 * first 1000 points and exposed a "Show N more" affordance to expand; this
 * helper centralises the window so the Inspector matches that behaviour
 * byte-for-byte.
 */
export const POINTS_PAGE_SIZE = 1000;

export function pageSegmentPoints(
  segment: SegmentDetail,
  expanded: boolean,
): { visible: PointDetail[]; hiddenCount: number } {
  if (expanded || segment.points.length <= POINTS_PAGE_SIZE) {
    return { visible: segment.points, hiddenCount: 0 };
  }
  return {
    visible: segment.points.slice(0, POINTS_PAGE_SIZE),
    hiddenCount: segment.points.length - POINTS_PAGE_SIZE,
  };
}
