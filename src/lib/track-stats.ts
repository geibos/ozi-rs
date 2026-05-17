// Track statistics formatting helpers for the Tracks panel row UI.
//
// Mirrors the semantics required by the `track-display` capability:
// distance is shown to one decimal place in kilometres, duration is shown as
// `<h>h <m>m` (or `<m>m` when under an hour) and only when timestamps are
// available, and the point count is shown with a `pts` suffix. The bullet
// separator joins only the segments that are present, so the duration
// separator is dropped when `duration_seconds` is null/undefined.

export function formatDistanceKm(distanceKm: number): string {
  return `${distanceKm.toFixed(1)} km`;
}

export function formatDurationSeconds(durationSeconds: number): string {
  const totalSeconds = Math.max(0, Math.trunc(durationSeconds));
  const hours = Math.floor(totalSeconds / 3600);
  const minutes = Math.floor((totalSeconds % 3600) / 60);
  if (hours > 0) {
    return `${hours}h ${minutes}m`;
  }
  return `${minutes}m`;
}

export function formatPointCount(pointCount: number): string {
  return `${pointCount} pts`;
}

/**
 * Build the compact track-stats label rendered next to the track name.
 *
 * Examples:
 *   `12.3 km · 1h 24m · 156 pts` when timestamps are present
 *   `12.3 km · 156 pts`          when `durationSeconds` is null/undefined
 */
export function formatTrackStats(
  distanceKm: number,
  durationSeconds: number | null | undefined,
  pointCount: number
): string {
  const segments = [formatDistanceKm(distanceKm)];
  if (durationSeconds !== null && durationSeconds !== undefined) {
    segments.push(formatDurationSeconds(durationSeconds));
  }
  segments.push(formatPointCount(pointCount));
  return segments.join(" · ");
}
