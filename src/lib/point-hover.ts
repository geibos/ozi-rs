/**
 * Which point of the selected track the cursor is over.
 *
 * "Удобно, когда можно навести на точку на карте и посмотреть, что это за
 * точка, и потом найти её в списке" — the owner, 2026-10-08, checking whether
 * a group really walked a triangle into the marsh. The point under the cursor
 * is named with its number, time and leg, and a click selects it in the
 * points table.
 *
 * A phone's track has 18 000 points and the cursor moves sixty times a
 * second, so the points are first cut to those inside a small box around the
 * cursor in degrees — a comparison per point — and only those are projected.
 */

export interface LngLatBox {
  west: number;
  south: number;
  east: number;
  north: number;
}

/**
 * The nearest of `points` to `cursor` within `radiusPx`, or `null`.
 * `box` is the cursor's neighbourhood in degrees, which no point outside of
 * can be within the radius.
 */
export function nearestPointWithin<P extends { lat: number; lon: number }>(
  points: readonly P[],
  box: LngLatBox,
  project: (lon: number, lat: number) => { x: number; y: number },
  cursor: { x: number; y: number },
  radiusPx: number,
): P | null {
  let best: P | null = null;
  let bestSq = radiusPx * radiusPx;
  for (const p of points) {
    if (
      p.lat < box.south ||
      p.lat > box.north ||
      p.lon < box.west ||
      p.lon > box.east
    ) {
      continue;
    }
    const { x, y } = project(p.lon, p.lat);
    const d = (x - cursor.x) ** 2 + (y - cursor.y) ** 2;
    if (d <= bestSq) {
      bestSq = d;
      best = p;
    }
  }
  return best;
}
