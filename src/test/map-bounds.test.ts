import { describe, expect, it } from "vitest";
import { boundsOf, geojsonPositions, isDegenerate } from "../lib/map-bounds";

/**
 * The camera only ever framed the data after an import. Opening a saved
 * project left it wherever it was — which, for a crew reopening yesterday's
 * search on a different map, is an empty screen that looks exactly like a
 * project that failed to load.
 *
 * The maths lived inline in `MapView`, which needs a MapLibre instance to
 * mount and so was never under test. It is here now.
 */
describe("geojsonPositions", () => {
  it("walks every depth of coordinates a track can produce", () => {
    const features = [
      { geometry: { coordinates: [31.6, 59.95] } },
      {
        geometry: {
          coordinates: [
            [31.7, 59.96],
            [31.8, 59.97],
          ],
        },
      },
      {
        geometry: {
          coordinates: [
            [
              [31.9, 59.98],
              [32.0, 59.99],
            ],
          ],
        },
      },
    ];
    expect(geojsonPositions(features)).toHaveLength(5);
  });

  it("ignores a feature with no geometry rather than throwing", () => {
    expect(
      geojsonPositions([
        { geometry: null },
        { geometry: { coordinates: [1, 2] } },
      ]),
    ).toEqual([{ lon: 1, lat: 2 }]);
  });
});

describe("boundsOf", () => {
  it("is null for nothing to frame", () => {
    expect(boundsOf([])).toBeNull();
  });

  it("spans every point it is given", () => {
    expect(
      boundsOf([
        { lon: 31.6, lat: 59.95 },
        { lon: 30.2, lat: 60.1 },
        { lon: 32.0, lat: 59.8 },
      ]),
    ).toEqual({ west: 30.2, south: 59.8, east: 32.0, north: 60.1 });
  });

  it("survives a single point", () => {
    expect(boundsOf([{ lon: 31.6, lat: 59.95 }])).toEqual({
      west: 31.6,
      south: 59.95,
      east: 31.6,
      north: 59.95,
    });
  });

  it("skips a coordinate that is not a number", () => {
    // A track whose geometry the backend could not build should not turn the
    // whole frame into NaN and leave the crew looking at the ocean.
    expect(
      boundsOf([
        { lon: 31.6, lat: 59.95 },
        { lon: Number.NaN, lat: 60.0 },
      ]),
    ).toEqual({ west: 31.6, south: 59.95, east: 31.6, north: 59.95 });
  });

  it("is null when nothing it was given is usable", () => {
    expect(boundsOf([{ lon: Number.NaN, lat: Number.NaN }])).toBeNull();
  });
});

describe("isDegenerate", () => {
  it("is true for one point, where there is no extent to fit", () => {
    expect(
      isDegenerate({ west: 31.6, south: 59.95, east: 31.6, north: 59.95 }),
    ).toBe(true);
  });

  it("is true for an extent too small to mean anything", () => {
    // A metre across. `fitBounds` would answer with the maximum zoom.
    expect(
      isDegenerate({
        west: 31.6,
        south: 59.95,
        east: 31.600005,
        north: 59.950005,
      }),
    ).toBe(true);
  });

  it("is false for a real search area", () => {
    expect(
      isDegenerate({ west: 31.5, south: 59.9, east: 31.7, north: 60.0 }),
    ).toBe(false);
  });
});
