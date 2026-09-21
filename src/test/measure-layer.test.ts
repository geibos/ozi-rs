import { describe, expect, it, vi } from "vitest";
import type { Map as MapLibreMap } from "maplibre-gl";
import { updateMeasureLayer } from "../lib/maplibre/measure-layer";

/**
 * What the tape hands MapLibre.
 *
 * The rendered pixels are not checked here and were not checked by hand
 * either: MapLibre does not set `preserveDrawingBuffer`, so reading the canvas
 * back after a frame is presented returns an empty buffer — a zero there means
 * nothing either way. What is deterministic is the data, so that is what this
 * pins.
 */
function fakeMap(): {
  map: MapLibreMap;
  data: () => GeoJSON.FeatureCollection;
} {
  let last: GeoJSON.FeatureCollection = {
    type: "FeatureCollection",
    features: [],
  };
  const map = {
    getSource: () => ({
      setData: (d: GeoJSON.FeatureCollection) => (last = d),
    }),
    getLayer: vi.fn(),
    moveLayer: vi.fn(),
  } as unknown as MapLibreMap;
  return { map, data: () => last };
}

describe("the tape the measuring tool draws", () => {
  it("marks every point that was clicked", () => {
    const { map, data } = fakeMap();
    updateMeasureLayer(map, [
      { lat: 59.95, lon: 31.59 },
      { lat: 59.96, lon: 31.6 },
      { lat: 59.97, lon: 31.61 },
    ]);

    const points = data().features.filter((f) => f.geometry.type === "Point");
    expect(points).toHaveLength(3);
    // GeoJSON is lon, lat — the order a map gets wrong once and only once.
    expect((points[0].geometry as GeoJSON.Point).coordinates).toEqual([
      31.59, 59.95,
    ]);
  });

  it("joins them with one line, in the order they were clicked", () => {
    const { map, data } = fakeMap();
    updateMeasureLayer(map, [
      { lat: 59.95, lon: 31.59 },
      { lat: 59.96, lon: 31.6 },
    ]);

    const lines = data().features.filter(
      (f) => f.geometry.type === "LineString",
    );
    expect(lines).toHaveLength(1);
    expect((lines[0].geometry as GeoJSON.LineString).coordinates).toEqual([
      [31.59, 59.95],
      [31.6, 59.96],
    ]);
  });

  /** One point is a place, not a line. */
  it("draws no line for a single point", () => {
    const { map, data } = fakeMap();
    updateMeasureLayer(map, [{ lat: 59.95, lon: 31.59 }]);

    expect(data().features).toHaveLength(1);
    expect(data().features[0].geometry.type).toBe("Point");
  });

  it("clears to nothing when the tool is put away", () => {
    const { map, data } = fakeMap();
    updateMeasureLayer(map, [{ lat: 1, lon: 1 }]);
    updateMeasureLayer(map, []);

    expect(data().features).toEqual([]);
  });
});
