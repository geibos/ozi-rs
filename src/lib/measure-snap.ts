/**
 * The tape catching hold of a mark.
 *
 * "How far from the headquarters to the drop-off point" is the commonest
 * measurement a coordinator makes, and OziExplorer answers it with a dialog
 * listing waypoint pairs. The ruler here could already measure it — by clicking
 * as close to each mark as the hand manages, which is close enough for a sketch
 * and not for a task: fifteen pixels at a two-kilometre view is eighty metres,
 * and the number goes out over the radio.
 *
 * So the tape catches. A click that lands near a mark takes that mark's exact
 * position and its name, and the readout says which marks it measured between —
 * which is also what makes the catching visible, since nothing else on screen
 * would show that the click moved.
 */
import type { LatLon } from "./geo";

export interface SnapCandidate extends LatLon {
  name: string;
}

export interface SnapResult extends LatLon {
  /** The mark this point was taken from, or `null` for a bare click. */
  name: string | null;
}

/** How near a mark a click has to land, in screen pixels. */
export const SNAP_RADIUS_PX = 14;

/**
 * Take the click, or the mark it landed on.
 *
 * The distance is measured on screen rather than on the ground: a radius in
 * metres would catch nothing when zoomed out and everything when zoomed in,
 * and what the operator is aiming at is the marker under the cursor.
 */
export function snapToWaypoint(
  click: LatLon,
  candidates: readonly SnapCandidate[],
  project: (at: LatLon) => { x: number; y: number },
  radiusPx: number = SNAP_RADIUS_PX,
): SnapResult {
  const at = project(click);
  let best: SnapCandidate | null = null;
  let bestDistance = radiusPx;

  for (const candidate of candidates) {
    const there = project(candidate);
    const distance = Math.hypot(there.x - at.x, there.y - at.y);
    // Strictly nearer, so the first of two marks at the same place wins and
    // the answer does not depend on the order they were drawn in.
    if (distance < bestDistance) {
      best = candidate;
      bestDistance = distance;
    }
  }

  if (best === null) return { ...click, name: null };
  return { lat: best.lat, lon: best.lon, name: best.name };
}

/**
 * What the measurement is between, when it is between marks.
 *
 * Answers `null` unless both ends caught a mark: "ШТАБ → куда-то" is not worth
 * the line it takes, and a measurement along a route has no two ends to name.
 */
export function measuredBetween(points: readonly SnapResult[]): string | null {
  if (points.length !== 2) return null;
  const [from, to] = points;
  if (from.name === null || to.name === null) return null;
  return `${from.name} → ${to.name}`;
}
