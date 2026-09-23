import type { Map as MapLibreMap } from "maplibre-gl";

const MEASURE_SOURCE = "measure";
const MEASURE_LINE = "measure-line";
const MEASURE_POINTS = "measure-points";

/**
 * What the on-map tools draw: the tape's clicked points and the line through
 * them, and the radius ring.
 *
 * Not decoration. A crew clicking on a map at night has to see where the
 * points went — whether the click registered at all, whether it landed on the
 * road or beside it, and what shape the thing they are measuring has. A
 * running total with nothing under it is a number they cannot check.
 *
 * Drawn in a colour that is not a track colour and not a waypoint colour, so
 * the tape never reads as something in the project.
 */
const TAPE_COLOR = "#f97316";

export function initMeasureLayer(map: MapLibreMap): void {
  map.addSource(MEASURE_SOURCE, {
    type: "geojson",
    data: { type: "FeatureCollection", features: [] },
  });

  map.addLayer({
    id: MEASURE_LINE,
    type: "line",
    source: MEASURE_SOURCE,
    filter: ["==", ["geometry-type"], "LineString"],
    layout: { "line-join": "round", "line-cap": "round" },
    paint: {
      "line-color": TAPE_COLOR,
      "line-width": 2,
      // Dashed, so it reads as a measurement rather than as a recorded track.
      "line-dasharray": [2, 1.5],
    },
  });

  map.addLayer({
    id: MEASURE_POINTS,
    type: "circle",
    source: MEASURE_SOURCE,
    filter: ["==", ["geometry-type"], "Point"],
    paint: {
      "circle-radius": 4,
      "circle-color": TAPE_COLOR,
      "circle-stroke-width": 1.5,
      "circle-stroke-color": "#ffffff",
    },
  });
}

/** Replace what the tape shows. An empty list clears it. */
export function updateMeasureLayer(
  map: MapLibreMap,
  points: { lat: number; lon: number }[],
  ring: { lat: number; lon: number }[] = [],
  /**
   * Draw the tape as a closed shape.
   *
   * The area tool reports what the points enclose, and an outline left open
   * reads as a path while the number beside it claims an area. The closing
   * edge is the difference between "I measured this sector" and "I measured
   * this walk".
   */
  closed = false,
): void {
  const source = map.getSource(MEASURE_SOURCE);
  if (!source || !("setData" in source)) return;

  const features: GeoJSON.Feature[] = points.map((p) => ({
    type: "Feature",
    properties: {},
    geometry: { type: "Point", coordinates: [p.lon, p.lat] },
  }));

  // One point is a place, not a line: MapLibre would draw nothing for a
  // one-coordinate LineString, but emitting one is still a lie about the data.
  if (points.length >= 2) {
    features.push({
      type: "Feature",
      properties: {},
      geometry: {
        type: "LineString",
        coordinates: (closed && points.length >= 3
          ? [...points, points[0]]
          : points
        ).map((p) => [p.lon, p.lat]),
      },
    });
  }

  if (ring.length >= 2) {
    features.push({
      type: "Feature",
      properties: {},
      geometry: {
        type: "LineString",
        coordinates: ring.map((p) => [p.lon, p.lat]),
      },
    });
  }

  (source as { setData: (data: GeoJSON.FeatureCollection) => void }).setData({
    type: "FeatureCollection",
    features,
  });
}

/** Keep the tape above the track and waypoint layers it is measuring across. */
export function raiseMeasureLayer(map: MapLibreMap): void {
  for (const id of [MEASURE_LINE, MEASURE_POINTS]) {
    if (map.getLayer(id)) map.moveLayer(id);
  }
}
