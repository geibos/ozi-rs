import { describe, expect, it } from "vitest";
import { distanceKm, formatMeasuredDistance, pathLengthKm } from "../lib/geo";

/**
 * The same haversine the Rust side uses for track statistics, with the same
 * earth radius: a measured leg and a track's length must be the same number
 * for the same two points, or the tool argues with the list beside it.
 */
describe("distance on the globe", () => {
  it("measures a known separation", () => {
    // One degree of latitude is ~111.19 km on a sphere of radius 6371.
    expect(distanceKm({ lat: 0, lon: 0 }, { lat: 1, lon: 0 })).toBeCloseTo(
      111.19,
      1,
    );
  });

  it("is zero for a point measured against itself", () => {
    const p = { lat: 59.95243, lon: 31.59681 };
    expect(distanceKm(p, p)).toBe(0);
  });

  it("does not care which way round the two points are given", () => {
    const a = { lat: 59.95243, lon: 31.59681 };
    const b = { lat: 59.94455, lon: 31.65927 };
    expect(distanceKm(a, b)).toBeCloseTo(distanceKm(b, a), 12);
  });

  it("adds a path up leg by leg", () => {
    const a = { lat: 0, lon: 0 };
    const b = { lat: 1, lon: 0 };
    const c = { lat: 2, lon: 0 };
    expect(pathLengthKm([a, b, c])).toBeCloseTo(distanceKm(a, b) * 2, 6);
  });

  it("is zero for fewer than two points", () => {
    expect(pathLengthKm([])).toBe(0);
    expect(pathLengthKm([{ lat: 1, lon: 1 }])).toBe(0);
  });
});

/**
 * A crew measuring the width of a clearing wants "180 м", not "0.2 км": the
 * difference between 40 and 140 metres is the difference between two sides of
 * a road.
 */
describe("a measured distance a person reads", () => {
  it("is metres under a kilometre", () => {
    expect(formatMeasuredDistance(0.18, "ru")).toBe("180 м");
    expect(formatMeasuredDistance(0.18, "en")).toBe("180 m");
    expect(formatMeasuredDistance(0, "ru")).toBe("0 м");
  });

  it("keeps two decimals in the first kilometres and one beyond ten", () => {
    expect(formatMeasuredDistance(1.234, "ru")).toBe("1.23 км");
    expect(formatMeasuredDistance(12.34, "ru")).toBe("12.3 км");
    expect(formatMeasuredDistance(12.34, "en")).toBe("12.3 km");
  });
});
