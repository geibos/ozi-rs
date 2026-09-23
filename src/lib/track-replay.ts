/**
 * Where a crew was at a given moment.
 *
 * A recording is a list of places with times on them, and the question a
 * headquarters asks of it is the other way round: "where were they at half past
 * two". Answering that from the points table means scrolling a few thousand
 * rows looking for a timestamp, which is why OziExplorer has Track Replay and
 * why a coordinator who has used it misses it here.
 *
 * The arithmetic is here and the control is elsewhere: what is worth testing is
 * the moment between two points, the gap where the navigator was off, and the
 * ends — not a slider.
 */
import type { LatLon } from "./geo";

interface TimedPoint extends LatLon {
  /** ISO 8601, as the backend sends it, or `null` for a point with no time. */
  timestamp: string | null;
}

interface SegmentLike {
  points: TimedPoint[];
}

export interface ReplayPoint extends LatLon {
  /** Epoch milliseconds of the moment asked for. */
  atMs: number;
  /**
   * True when the moment falls in a gap longer than `GAP_MS` — the navigator
   * was off, or in a pocket, and the straight line between the two points
   * either side of it is a guess rather than a route.
   */
  inGap: boolean;
}

export interface ReplayRange {
  fromMs: number;
  toMs: number;
}

/**
 * How long a silence has to be before the line across it stops being a route.
 *
 * Five minutes: a navigator logging every second or every ten is inside it,
 * and a crew that turned the thing off for a smoke is not. The figure is a
 * judgement rather than a measurement, and it is the only one here.
 */
export const GAP_MS = 5 * 60 * 1000;

/** Every timed point of the track, in time order, across all its segments. */
function timeline(
  segments: readonly SegmentLike[],
): { at: number; point: LatLon }[] {
  const out: { at: number; point: LatLon }[] = [];
  for (const segment of segments) {
    for (const point of segment.points) {
      if (point.timestamp === null) continue;
      const at = Date.parse(point.timestamp);
      if (!Number.isFinite(at)) continue;
      out.push({ at, point: { lat: point.lat, lon: point.lon } });
    }
  }
  // Sorted, because a recording is not always in order — which is exactly why
  // "sort points by time" exists as an edit.
  out.sort((a, b) => a.at - b.at);
  return out;
}

/**
 * The first and last moment the track knows about.
 *
 * `null` when fewer than two points carry a time: one moment is a reading, and
 * there is nothing to play back.
 */
export function replayRange(
  segments: readonly SegmentLike[],
): ReplayRange | null {
  const points = timeline(segments);
  if (points.length < 2) return null;
  return { fromMs: points[0].at, toMs: points[points.length - 1].at };
}

/**
 * Where the crew was at `atMs`.
 *
 * Between two points the position is interpolated, because a crew walking at
 * four kilometres an hour is somewhere in the middle of a minute's gap and the
 * nearest point is up to two hundred metres away from them.
 *
 * Outside the recording, the nearest end: before they started they were at the
 * start, and a slider that answers nothing at its own extremes is a slider
 * nobody trusts.
 */
export function positionAt(
  segments: readonly SegmentLike[],
  atMs: number,
): ReplayPoint | null {
  const points = timeline(segments);
  if (points.length === 0) return null;

  if (atMs <= points[0].at) {
    return { ...points[0].point, atMs: points[0].at, inGap: false };
  }
  const last = points[points.length - 1];
  if (atMs >= last.at) {
    return { ...last.point, atMs: last.at, inGap: false };
  }

  // The last point at or before the moment. Linear rather than a binary
  // search: a track is a few thousand points and this runs on a slider drag,
  // which is a comparison per point and not worth the cleverness.
  let index = 0;
  while (index + 1 < points.length && points[index + 1].at <= atMs) index += 1;

  const before = points[index];
  const after = points[index + 1];
  const span = after.at - before.at;
  const through = span === 0 ? 0 : (atMs - before.at) / span;

  return {
    lat: before.point.lat + (after.point.lat - before.point.lat) * through,
    lon: before.point.lon + (after.point.lon - before.point.lon) * through,
    atMs,
    inGap: span > GAP_MS,
  };
}

/** The moment as a coordinator reads it off a radio log: `14:30:05`. */
export function formatMoment(atMs: number): string {
  const at = new Date(atMs);
  const two = (n: number) => String(n).padStart(2, "0");
  return `${two(at.getHours())}:${two(at.getMinutes())}:${two(at.getSeconds())}`;
}
