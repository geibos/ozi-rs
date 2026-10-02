/**
 * Time on the move, and the climb — the two track statistics that need a
 * threshold.
 *
 * Both answer what GPS says when nothing is happening. A standing navigator's
 * position wanders by five to twenty metres, so judged fix by fix a rest stop
 * is slow walking; GPS elevation wanders by several metres, so summed point by
 * point the climb is mostly noise. The owner's thresholds (2026-10-01): a stop
 * is less than 25 m moved in two minutes, a rise or fall counts from 5 m. Both
 * are settings (`settings.ts`).
 */
import { distanceKm } from "./geo";

export interface MotionSettings {
  /** Moving less than this within `stopWindowS` is standing still. */
  stopDistanceM: number;
  stopWindowS: number;
  /** A rise or a fall counts once it reaches this. */
  climbThresholdM: number;
}

export const DEFAULT_MOTION_SETTINGS: MotionSettings = {
  stopDistanceM: 25,
  stopWindowS: 120,
  climbThresholdM: 5,
};

interface PointLike {
  lat: number;
  lon: number;
  elevation: number | null;
  timestamp: string | null;
}

interface SegmentLike {
  points: PointLike[];
}

interface TimedPoint {
  lat: number;
  lon: number;
  t: number;
}

function metresBetween(a: TimedPoint, b: TimedPoint): number {
  return distanceKm(a, b) * 1000;
}

/**
 * Seconds the crew was moving, or `null` when no point carries a time.
 *
 * Each interval between two fixes is judged by a window of at least
 * `stopWindowS` around it: moving if the crew covered `stopDistanceM` across
 * the window. The window runs forward from the interval and, near the end of
 * a segment where there is no more future, backward — scaling the distance
 * down for a short tail would let the last few seconds of a standing fix's
 * wander count as walking. Only a segment shorter than the window is judged
 * on its own length.
 *
 * Segments are counted separately: the time between two of them is a gap in
 * the recording, not a walk.
 */
export function movingSeconds(
  segments: SegmentLike[],
  settings: MotionSettings,
): number | null {
  let anyTimed = false;
  let total = 0;
  const window = settings.stopWindowS;

  for (const segment of segments) {
    const points: TimedPoint[] = [];
    for (const p of segment.points) {
      if (p.timestamp === null) continue;
      const ms = Date.parse(p.timestamp);
      if (Number.isNaN(ms)) continue;
      points.push({ lat: p.lat, lon: p.lon, t: ms / 1000 });
    }
    if (points.length > 0) anyTimed = true;
    const n = points.length;
    let ahead = 1;
    for (let i = 0; i < n - 1; i += 1) {
      const step = points[i + 1].t - points[i].t;
      if (step <= 0) continue;
      if (ahead < i + 1) ahead = i + 1;
      while (ahead < n - 1 && points[ahead].t - points[i].t < window) {
        ahead += 1;
      }
      let from = i;
      while (from > 0 && points[ahead].t - points[from].t < window) {
        from -= 1;
      }
      const span = points[ahead].t - points[from].t;
      const needed = settings.stopDistanceM * Math.min(1, span / window);
      if (metresBetween(points[from], points[ahead]) >= needed) total += step;
    }
  }

  return anyTimed ? total : null;
}

/**
 * Metres climbed and descended, or `null` when no point carries a height.
 *
 * Counted between turning points, and a top or a bottom is a turning point
 * only once the elevation has come back from it by `climbThresholdM`. That
 * keeps noise inside the threshold out entirely, and — unlike counting each
 * rise once it passes the threshold from the last counted height — does not
 * drop the last few metres below every summit.
 */
export function ascentDescent(
  segments: SegmentLike[],
  settings: MotionSettings,
): { ascentM: number; descentM: number } | null {
  const threshold = settings.climbThresholdM;
  let anyHeight = false;
  let ascentM = 0;
  let descentM = 0;

  for (const segment of segments) {
    const heights = segment.points
      .map((p) => p.elevation)
      .filter((e): e is number => e !== null && Number.isFinite(e));
    if (heights.length === 0) continue;
    anyHeight = true;

    let anchor = heights[0];
    let extreme = heights[0];
    let direction: -1 | 0 | 1 = 0;
    for (const h of heights.slice(1)) {
      if (direction === 0) {
        if (h - anchor >= threshold) {
          direction = 1;
          extreme = h;
        } else if (anchor - h >= threshold) {
          direction = -1;
          extreme = h;
        }
      } else if (direction === 1) {
        if (h > extreme) extreme = h;
        else if (extreme - h >= threshold) {
          ascentM += extreme - anchor;
          anchor = extreme;
          direction = -1;
          extreme = h;
        }
      } else {
        if (h < extreme) extreme = h;
        else if (h - extreme >= threshold) {
          descentM += anchor - extreme;
          anchor = extreme;
          direction = 1;
          extreme = h;
        }
      }
    }
    if (direction === 1) ascentM += extreme - anchor;
    if (direction === -1) descentM += anchor - extreme;
  }

  if (!anyHeight) return null;
  return { ascentM: Math.round(ascentM), descentM: Math.round(descentM) };
}
