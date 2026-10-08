/**
 * Which of a track's points edit mode puts a handle on.
 *
 * A handle is a DOM marker, and a phone records a point a second: an
 * 18 000-point track took 7.5 s to enter edit mode and 16.5 s to pan once on
 * the stand (2026-10-08). OziExplorer was "slow" on the same track; this was
 * worse. Handles go on the points in view, and only while there are few
 * enough of them to be told apart — past that, the operator zooms in, which
 * is what moving one point needs anyway.
 */

export const MAX_EDIT_HANDLES = 1000;

export interface Bounds {
  west: number;
  south: number;
  east: number;
  north: number;
}

interface PointLike {
  id: number;
  lat: number;
  lon: number;
}

interface SegmentLike<P extends PointLike> {
  id: number;
  points: P[];
}

export interface EditablePoint<S, P> {
  segment: S;
  point: P;
  /** The point's position in its segment, as inserting after it needs. */
  index: number;
}

/**
 * The points in `bounds`, or `null` with how many there are when they are
 * more than `max`.
 */
export function editablePointsInView<
  P extends PointLike,
  S extends SegmentLike<P>,
>(
  segments: S[],
  bounds: Bounds,
  max: number = MAX_EDIT_HANDLES,
): { points: EditablePoint<S, P>[]; tooMany: number | null } {
  const points: EditablePoint<S, P>[] = [];
  let count = 0;
  for (const segment of segments) {
    segment.points.forEach((point, index) => {
      if (
        point.lat >= bounds.south &&
        point.lat <= bounds.north &&
        point.lon >= bounds.west &&
        point.lon <= bounds.east
      ) {
        count += 1;
        if (count <= max) points.push({ segment, point, index });
      }
    });
  }
  return count > max
    ? { points: [], tooMany: count }
    : { points, tooMany: null };
}
