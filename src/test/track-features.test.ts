import { describe, expect, it } from "vitest";
import { trackFeaturesFromGeojson } from "../lib/track-features";

const properties = (over: Record<string, unknown> = {}) => ({
  layer_id: 2,
  track_id: 7,
  name: "20260709-ЛИСА10",
  color: "#ff0000",
  line_width: 3,
  visible: true,
  distance_km: 11.5,
  duration_seconds: 24000,
  point_count: 1193,
  ...over,
});

const feature = (
  geometry: GeoJSON.Geometry,
  over: Record<string, unknown> = {},
): GeoJSON.Feature =>
  ({ type: "Feature", geometry, properties: properties(over) }) as GeoJSON.Feature;

const collection = (features: GeoJSON.Feature[]): GeoJSON.FeatureCollection => ({
  type: "FeatureCollection",
  features,
});

const multi: GeoJSON.Geometry = {
  type: "MultiLineString",
  coordinates: [
    [
      [37.0, 55.0],
      [37.1, 55.1],
    ],
  ],
};

const single: GeoJSON.Geometry = {
  type: "LineString",
  coordinates: [
    [37.0, 55.0],
    [37.1, 55.1],
  ],
};

/**
 * The Tracks tab used to keep only `LineString` features. When track geometry
 * became one `MultiLineString` per track, that filter silently emptied the
 * list: the map still drew every track while the rail said "No tracks loaded".
 * Nothing caught it because no test walked this path.
 */
describe("track rows built from the tracks GeoJSON", () => {
  it("keeps MultiLineString tracks", () => {
    const rows = trackFeaturesFromGeojson(collection([feature(multi)]));
    expect(rows).toHaveLength(1);
    expect(rows[0].name).toBe("20260709-ЛИСА10");
    expect(rows[0].layerId).toBe(2n);
    expect(rows[0].trackId).toBe(7n);
    expect(rows[0].distanceKm).toBeCloseTo(11.5);
    expect(rows[0].pointCount).toBe(1193);
  });

  it("still keeps plain LineString tracks", () => {
    expect(trackFeaturesFromGeojson(collection([feature(single)]))).toHaveLength(1);
  });

  it("drops geometry that is not a line", () => {
    const point: GeoJSON.Geometry = { type: "Point", coordinates: [37, 55] };
    expect(trackFeaturesFromGeojson(collection([feature(point)]))).toHaveLength(0);
  });

  it("treats a missing duration as absent rather than zero", () => {
    const rows = trackFeaturesFromGeojson(
      collection([feature(multi, { duration_seconds: null })]),
    );
    expect(rows[0].durationSeconds).toBeNull();
  });

  it("orders rows by layer then track so one layer's rows cluster", () => {
    const rows = trackFeaturesFromGeojson(
      collection([
        feature(multi, { layer_id: 10, track_id: 1, name: "b" }),
        feature(multi, { layer_id: 2, track_id: 9, name: "a2" }),
        feature(multi, { layer_id: 2, track_id: 1, name: "a1" }),
      ]),
    );
    expect(rows.map((r) => r.name)).toEqual(["a1", "a2", "b"]);
  });
});
