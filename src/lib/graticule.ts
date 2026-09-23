/**
 * The coordinate grid drawn over the map.
 *
 * A search is cut into sectors and the sectors are given out over the radio,
 * as coordinates. Reading one off a map with no grid means clicking a place
 * and reading the status bar; writing one down means the same in reverse. Every
 * paper sheet a headquarters has ever used carries a grid, and OziExplorer
 * draws one, and this did not.
 *
 * The whole of it is here rather than in the map component: which lines to
 * draw at this zoom, where they go, and what each is called are decisions
 * about numbers, and they are the part worth testing. The component turns the
 * answer into a layer.
 */
import type { LatLon } from "./geo";

/**
 * The steps a grid is allowed to use, in degrees, coarsest first.
 *
 * Degrees, then minutes, then seconds — the ladder a map reader expects, so a
 * line always falls on a round number they can say out loud. A step of 0.15°
 * would fit the screen better and be useless on the radio.
 */
export const GRID_STEPS_DEGREES = [
  10,
  5,
  2,
  1,
  30 / 60,
  20 / 60,
  10 / 60,
  5 / 60,
  2 / 60,
  1 / 60,
  30 / 3600,
  20 / 3600,
  10 / 3600,
  5 / 3600,
  2 / 3600,
  1 / 3600,
] as const;

export interface GridBounds {
  south: number;
  west: number;
  north: number;
  east: number;
}

export interface GridLine {
  /** `lat` for a parallel, `lon` for a meridian. */
  kind: "lat" | "lon";
  /** The degree value the line stands at. */
  degrees: number;
  /** The line itself, in order. */
  points: LatLon[];
  /** What to write beside it. */
  label: string;
}

/**
 * The coarsest step that still puts at least `wanted` lines across the view.
 *
 * Coarsest rather than closest: a grid is read, not measured, and too many
 * lines cost more than too few. Longitude is chosen against the view's own
 * width, so a strip of the far north — where a degree of longitude is a few
 * kilometres — does not end up with ten times the lines of a strip at the
 * equator.
 */
export function chooseStep(span: number, wanted = 4): number {
  if (!Number.isFinite(span) || span <= 0) {
    return GRID_STEPS_DEGREES[GRID_STEPS_DEGREES.length - 1];
  }
  for (const step of GRID_STEPS_DEGREES) {
    if (span / step >= wanted) return step;
  }
  return GRID_STEPS_DEGREES[GRID_STEPS_DEGREES.length - 1];
}

/**
 * Longitude back into −180…180.
 *
 * A view that runs past the antimeridian produces line values of 180.5 or 181,
 * and `181°E` is not a coordinate anybody can read out over a radio. Found by
 * a reviewer, 2026-09-23.
 */
export function normaliseLon(lon: number): number {
  return ((((lon + 180) % 360) + 360) % 360) - 180;
}

/** Degrees as a map reader writes them: `59°56'` or `59°56'15"`. */
export function formatDegrees(value: number, kind: "lat" | "lon"): string {
  const shown = kind === "lon" ? normaliseLon(value) : value;
  // The antimeridian belongs to neither hemisphere and is written bare; the
  // equator likewise. `180°W` is not wrong, exactly, but nobody says it.
  const onTheLine = shown === 0 || Math.abs(shown) === 180;
  const hemisphere = onTheLine
    ? ""
    : kind === "lat"
      ? shown < 0
        ? "S"
        : "N"
      : shown < 0
        ? "W"
        : "E";
  const magnitude = Math.abs(shown);
  // Rounded to whole seconds first, so 59.999999 does not print as 59°59'60".
  const totalSeconds = Math.round(magnitude * 3600);
  const degrees = Math.floor(totalSeconds / 3600);
  const minutes = Math.floor((totalSeconds % 3600) / 60);
  const seconds = totalSeconds % 60;

  if (minutes === 0 && seconds === 0) return `${degrees}°${hemisphere}`;
  if (seconds === 0) {
    return `${degrees}°${String(minutes).padStart(2, "0")}'${hemisphere}`;
  }
  return (
    `${degrees}°${String(minutes).padStart(2, "0")}'` +
    `${String(seconds).padStart(2, "0")}"${hemisphere}`
  );
}

/**
 * The lines to draw over this view.
 *
 * Each line is drawn across the whole of the view and a little beyond, so it
 * reaches the edges of a rotated or tilted map without a gap.
 */
export function graticule(bounds: GridBounds, wanted = 4): GridLine[] {
  const latSpan = bounds.north - bounds.south;
  // A view straddling the antimeridian arrives with west greater than east —
  // MapLibre reports 179 … −179 for it. Unwrapping the eastern edge past 180
  // keeps the lines marching the way the view runs; the labels normalise back,
  // so a meridian drawn at 180.5 is named 179°30'W. Before this the whole grid
  // came back empty there. Found by a reviewer, 2026-09-23.
  const unwrappedEast =
    bounds.east < bounds.west ? bounds.east + 360 : bounds.east;
  const lonSpan = unwrappedEast - bounds.west;
  if (!(latSpan > 0) || !(lonSpan > 0)) return [];
  bounds = { ...bounds, east: unwrappedEast };

  const latStep = chooseStep(latSpan, wanted);
  const lonStep = chooseStep(lonSpan, wanted);

  // A margin so the lines run past the corners rather than stopping at them.
  const latPad = latSpan * 0.1;
  const lonPad = lonSpan * 0.1;
  const south = Math.max(-90, bounds.south - latPad);
  const north = Math.min(90, bounds.north + latPad);
  const west = bounds.west - lonPad;
  const east = bounds.east + lonPad;

  const lines: GridLine[] = [];

  // Parallels. Snapped to the step so they land on round numbers.
  for (
    let lat = Math.ceil(south / latStep) * latStep;
    lat <= north + 1e-9;
    lat += latStep
  ) {
    if (lat < -90 || lat > 90) continue;
    const value = roundToStep(lat, latStep);
    lines.push({
      kind: "lat",
      degrees: value,
      label: formatDegrees(value, "lat"),
      points: [
        { lat: value, lon: west },
        { lat: value, lon: east },
      ],
    });
  }

  // Meridians.
  for (
    let lon = Math.ceil(west / lonStep) * lonStep;
    lon <= east + 1e-9;
    lon += lonStep
  ) {
    const value = roundToStep(lon, lonStep);
    lines.push({
      kind: "lon",
      degrees: value,
      label: formatDegrees(value, "lon"),
      points: [
        { lat: south, lon: value },
        { lat: north, lon: value },
      ],
    });
  }

  return lines;
}

/**
 * Accumulated addition drifts — after forty steps of 1/3600 the value is no
 * longer the round number the label claims. Snapping each one back to its own
 * step is what keeps the line and its name the same place.
 */
function roundToStep(value: number, step: number): number {
  return Math.round(value / step) * step;
}

/** The lines as a GeoJSON collection, which is what the map layer takes. */
export function graticuleGeoJson(lines: GridLine[]): GeoJSON.FeatureCollection {
  return {
    type: "FeatureCollection",
    features: lines.map((line) => ({
      type: "Feature",
      properties: { kind: line.kind, label: line.label },
      geometry: {
        type: "LineString",
        coordinates: line.points.map((p) => [p.lon, p.lat]),
      },
    })),
  };
}
