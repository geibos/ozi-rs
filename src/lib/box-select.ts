/**
 * Choosing a track's points with a box drawn on the map.
 *
 * OziExplorer's Selection Control: «нарисовать бокс, чтобы выбрать точки на
 * активном треке», then delete them. The owner used it on every track of a
 * real search (2026-10-08) — for a cluster of outliers, for the walk back to
 * HQ — because picking points 572 to 586 out of a list "is already
 * inconvenient". The box is drawn on the screen, so the test is in screen
 * pixels: what the operator sees inside the rectangle is what is chosen.
 */
import { writable, get } from "svelte/store";
import {
  addWaypointMode,
  drawingModeActive,
  editModeActive,
  setMeasuring,
  setProjection,
  setRing,
} from "./stores";

export interface ScreenPoint {
  x: number;
  y: number;
}

export interface ScreenBox {
  left: number;
  top: number;
  right: number;
  bottom: number;
}

/** The box between two corners, whichever way it was dragged. */
export function boxBetween(a: ScreenPoint, b: ScreenPoint): ScreenBox {
  return {
    left: Math.min(a.x, b.x),
    top: Math.min(a.y, b.y),
    right: Math.max(a.x, b.x),
    bottom: Math.max(a.y, b.y),
  };
}

/** Ids of the points whose screen position falls inside the box. */
export function pointsInBox(
  points: ReadonlyArray<{ id: number; lat: number; lon: number }>,
  project: (lon: number, lat: number) => ScreenPoint,
  box: ScreenBox,
): number[] {
  const inside: number[] = [];
  for (const p of points) {
    const { x, y } = project(p.lon, p.lat);
    if (x >= box.left && x <= box.right && y >= box.top && y <= box.bottom) {
      inside.push(p.id);
    }
  }
  return inside;
}

/**
 * The selection after another box: it replaces what was chosen, or with
 * Shift adds to it — two boxes for an outlier cluster that is not a
 * rectangle.
 */
export function combineSelection(
  previous: ReadonlySet<number>,
  boxed: readonly number[],
  additive: boolean,
): Set<number> {
  const next = new Set(additive ? previous : []);
  for (const id of boxed) next.add(id);
  return next;
}

/** The tool is on: dragging on the map draws a box instead of panning. */
export const boxSelectActive = writable(false);

/** The chosen points, and the track they belong to. */
export const boxSelection = writable<{
  layerId: bigint;
  trackId: bigint;
  ids: Set<number>;
} | null>(null);

/**
 * Turn the tool on or off. On, it takes the drag the other tools take, so
 * they go off; off, the selection goes with it.
 */
export function setBoxSelect(active: boolean): void {
  boxSelection.set(null);
  boxSelectActive.set(active);
  if (!active) return;
  setMeasuring(false);
  setRing(false);
  setProjection(false);
  if (get(editModeActive)) editModeActive.set(false);
  if (get(drawingModeActive)) drawingModeActive.set(false);
  if (get(addWaypointMode)) addWaypointMode.set(false);
}
