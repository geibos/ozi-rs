import { describe, expect, it } from "vitest";
import {
  destinationPoint,
  distanceKm,
  formatMeasuredDistance,
  pathLengthKm,
  ringAround,
} from "../lib/geo";

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

/**
 * A ring drawn as a flat circle in screen pixels is wrong everywhere except
 * the equator, and worse the further north the search is. At 60° — which is
 * where these searches happen — a "500 m" circle drawn flat is half a
 * kilometre north-south and a kilometre east-west, and the crew standing in it
 * is looking in the wrong place.
 */
describe("a ring of a given radius", () => {
  const centre = { lat: 59.95, lon: 31.59 };

  it("puts every point the radius away from the centre, at 60° north", () => {
    for (const point of ringAround(centre, 0.5)) {
      expect(distanceKm(centre, point)).toBeCloseTo(0.5, 6);
    }
  });

  it("holds at a radius where a flat circle would be badly wrong", () => {
    for (const point of ringAround(centre, 25)) {
      expect(distanceKm(centre, point)).toBeCloseTo(25, 4);
    }
  });

  it("closes on itself", () => {
    const ring = ringAround(centre, 1, 8);
    expect(ring).toHaveLength(9);
    expect(ring[8]).toEqual(ring[0]);
  });

  it("is nothing at all for a radius of zero", () => {
    expect(ringAround(centre, 0)).toEqual([]);
    expect(ringAround(centre, -1)).toEqual([]);
  });

  it("goes due north when asked to", () => {
    const north = destinationPoint(centre, 0, 1);
    expect(north.lat).toBeGreaterThan(centre.lat);
    expect(north.lon).toBeCloseTo(centre.lon, 9);
  });

  /** A ring near the antimeridian must not come out as a band around the world. */
  it("keeps longitudes inside −180..180", () => {
    for (const point of ringAround({ lat: 65, lon: 179.98 }, 5)) {
      expect(point.lon).toBeGreaterThanOrEqual(-180);
      expect(point.lon).toBeLessThanOrEqual(180);
    }
  });
});
