/**
 * Framing the camera on what a project contains.
 *
 * The camera used to be fitted to the data only after an import. Opening a
 * saved project left it wherever it happened to be, which for a crew
 * reopening yesterday's search is an empty screen — indistinguishable from a
 * project that failed to load.
 *
 * The maths was inline in `MapView`, which needs a MapLibre instance to mount
 * and therefore never ran under test.
 */
import type { LatLon } from "./geo";

export interface Bounds {
  west: number;
  south: number;
  east: number;
  north: number;
}

interface GeometryLike {
  geometry?: { coordinates?: unknown } | null;
}

/**
 * Every position in a feature collection, at whatever nesting depth: a
 * waypoint is a `Position`, a track segment a `Position[]`, a multi-segment
 * track a `Position[][]`.
 */
export function geojsonPositions(features: GeometryLike[]): LatLon[] {
  const out: LatLon[] = [];
  const visit = (coords: unknown): void => {
    if (
      Array.isArray(coords) &&
      typeof coords[0] === "number" &&
      typeof coords[1] === "number"
    ) {
      out.push({ lon: coords[0], lat: coords[1] });
      return;
    }
    if (Array.isArray(coords)) for (const child of coords) visit(child);
  };
  for (const feature of features) visit(feature.geometry?.coordinates);
  return out;
}

/**
 * @returns `null` when there is nothing usable to frame. A coordinate that is
 *   not a finite number is skipped rather than turned into a `NaN` box — one
 *   track the backend could not build should not send the camera to the
 *   middle of the ocean.
 */
export function boundsOf(points: LatLon[]): Bounds | null {
  let west = Infinity;
  let south = Infinity;
  let east = -Infinity;
  let north = -Infinity;
  let any = false;

  for (const { lon, lat } of points) {
    if (!Number.isFinite(lon) || !Number.isFinite(lat)) continue;
    any = true;
    if (lon < west) west = lon;
    if (lon > east) east = lon;
    if (lat < south) south = lat;
    if (lat > north) north = lat;
  }

  return any ? { west, south, east, north } : null;
}

/**
 * Whether the box is too small to fit a camera to — one point, or a few
 * metres across. `fitBounds` answers those with its maximum zoom, which puts
 * the crew inside a building.
 */
export function isDegenerate(bounds: Bounds, epsilon = 1e-4): boolean {
  return (
    bounds.east - bounds.west < epsilon && bounds.north - bounds.south < epsilon
  );
}

/** The pair of corners MapLibre's `fitBounds` takes. */
export function toLngLatBounds(
  bounds: Bounds,
): [[number, number], [number, number]] {
  return [
    [bounds.west, bounds.south],
    [bounds.east, bounds.north],
  ];
}

export function centreOf(bounds: Bounds): [number, number] {
  return [(bounds.west + bounds.east) / 2, (bounds.south + bounds.north) / 2];
}
