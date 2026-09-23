import { describe, expect, it } from "vitest";
import {
  destinationPoint,
  distanceKm,
  formatMeasuredDistance,
  pathLengthKm,
  ringAround,
  polygonAreaSqKm,
  formatMeasuredArea,
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

/**
 * OziExplorer's measuring tool is "Distance & Area", and the area is the half
 * a coordinator writes down: a sector is handed over as "прочесать 2.4 км²",
 * and that number decides how many people it takes.
 */
describe("polygonAreaSqKm", () => {
  it("has nothing to measure until three points enclose something", () => {
    expect(polygonAreaSqKm([])).toBe(0);
    expect(polygonAreaSqKm([{ lat: 59.95, lon: 31.6 }])).toBe(0);
    expect(
      polygonAreaSqKm([
        { lat: 59.95, lon: 31.6 },
        { lat: 59.96, lon: 31.6 },
      ]),
    ).toBe(0);
  });

  it("measures a square sector at the latitude these searches happen", () => {
    // One kilometre north-south, one east-west, at 60° north — where the
    // longitude degree is half the length of the latitude degree, which is
    // exactly what a flat shoelace on raw degrees would get wrong.
    const kmPerDegLat = (Math.PI * 6371) / 180;
    const dLat = 1 / kmPerDegLat;
    const dLon = dLat / Math.cos((60 * Math.PI) / 180);
    const square = [
      { lat: 60, lon: 31 },
      { lat: 60, lon: 31 + dLon },
      { lat: 60 + dLat, lon: 31 + dLon },
      { lat: 60 + dLat, lon: 31 },
    ];
    expect(polygonAreaSqKm(square)).toBeCloseTo(1, 2);
  });

  it("does not care which way round the sector was clicked", () => {
    const square = [
      { lat: 60, lon: 31 },
      { lat: 60, lon: 31.02 },
      { lat: 60.01, lon: 31.02 },
      { lat: 60.01, lon: 31 },
    ];
    expect(polygonAreaSqKm(square)).toBeCloseTo(
      polygonAreaSqKm([...square].reverse()),
      6,
    );
  });

  it("closes the shape whether or not the crew clicked back to the start", () => {
    const open = [
      { lat: 60, lon: 31 },
      { lat: 60, lon: 31.02 },
      { lat: 60.01, lon: 31.02 },
    ];
    const closed = [...open, open[0]];
    expect(polygonAreaSqKm(closed)).toBeCloseTo(polygonAreaSqKm(open), 6);
  });
});

describe("formatMeasuredArea", () => {
  it("says a sector in the unit a coordinator would say it in", () => {
    // Hectares in the middle: "40 га" is a sentence; "0.4 км²" is a
    // conversion.
    expect(formatMeasuredArea(0.4, "ru")).toBe("40 га");
    expect(formatMeasuredArea(2.43, "ru")).toBe("2.43 км²");
    expect(formatMeasuredArea(24.3, "ru")).toBe("24.3 км²");
    expect(formatMeasuredArea(0.005, "ru")).toBe("5000 м²");
  });

  it("says nothing when nothing is enclosed", () => {
    expect(formatMeasuredArea(0, "ru")).toBe("");
  });

  it("answers in English too", () => {
    expect(formatMeasuredArea(0.4, "en")).toBe("40 ha");
    expect(formatMeasuredArea(2.43, "en")).toBe("2.43 km²");
  });
});
