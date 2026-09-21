/**
 * The elevation of a track against distance along it.
 *
 * `PointDetailDto` has carried `elevation` since the first import path was
 * written; the inspector showed a card promising the chart "in a follow-up
 * change". For a crew reading someone else's recording, what the ground does
 * along the route is the difference between a twenty-minute leg and an hour.
 *
 * Gain and loss are deliberately not computed here. Summing every rise in a
 * GPS track adds up its own noise, and the threshold that fixes that is a
 * decision about the data rather than about the chart — it sits in
 * `docs/backlog.md` beside the moving-time threshold, which is the same
 * question. The range, which needs no threshold, is reported instead.
 */
import { distanceKm, type LatLon } from "./geo";

interface PointLike extends LatLon {
  elevation: number | null;
}

interface SegmentLike {
  points: PointLike[];
}

export interface ElevationSample {
  /** Distance from the track's first point, in kilometres. */
  km: number;
  metres: number;
}

export interface ElevationProfile {
  samples: ElevationSample[];
  minMetres: number;
  maxMetres: number;
  /** Distance to the last point, elevation or not. */
  totalKm: number;
}

/**
 * @returns `null` when fewer than two points carry an elevation — one sample
 *   is a reading, not a profile, and the card says so instead of drawing a
 *   line through nothing.
 */
export function elevationProfile(
  segments: SegmentLike[],
): ElevationProfile | null {
  const samples: ElevationSample[] = [];
  let km = 0;
  let previous: LatLon | null = null;

  for (const segment of segments) {
    for (const point of segment.points) {
      // Distance accumulates over every point: a gap in the elevation is a
      // gap in the data, not in the route.
      if (previous) km += distanceKm(previous, point);
      previous = point;
      if (point.elevation !== null) {
        samples.push({ km, metres: point.elevation });
      }
    }
  }

  if (samples.length < 2) return null;
  const metres = samples.map((s) => s.metres);
  return {
    samples,
    minMetres: Math.min(...metres),
    maxMetres: Math.max(...metres),
    totalKm: km,
  };
}

/**
 * An SVG path through the profile, in a box `width` × `height` with y growing
 * downwards, so the highest point sits at y = 0.
 */
export function profilePath(
  profile: ElevationProfile,
  width: number,
  height: number,
): string {
  const spanKm = profile.samples[profile.samples.length - 1].km;
  const spanMetres = profile.maxMetres - profile.minMetres;
  const round = (n: number) => Math.round(n * 100) / 100;

  const points = profile.samples.map((sample) => {
    const x = spanKm === 0 ? 0 : (sample.km / spanKm) * width;
    // A flat track has no range to scale against, so it runs down the middle
    // rather than through a division by zero.
    const y =
      spanMetres === 0
        ? height / 2
        : height - ((sample.metres - profile.minMetres) / spanMetres) * height;
    return `${round(x)},${round(y)}`;
  });

  return `M${points[0]} ${points
    .slice(1)
    .map((p) => `L${p}`)
    .join(" ")}`;
}
