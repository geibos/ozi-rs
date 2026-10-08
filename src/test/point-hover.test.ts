import { describe, expect, it } from "vitest";

import { nearestPointWithin } from "../lib/point-hover";

/** A degree is a hundred pixels; north is up. */
const project = (lon: number, lat: number) => ({ x: lon * 100, y: -lat * 100 });
const everywhere = { west: -180, south: -90, east: 180, north: 90 };

describe("the point under the cursor", () => {
  const points = [
    { id: 1, lat: 0.1, lon: 0.1 },
    { id: 2, lat: 0.1, lon: 0.15 },
    { id: 3, lat: 0.5, lon: 0.5 },
  ];

  it("is the nearest within the radius", () => {
    // Cursor at (14, -10): 4 px from point 1, 1 px from point 2.
    expect(
      nearestPointWithin(points, everywhere, project, { x: 14, y: -10 }, 10)
        ?.id,
    ).toBe(2);
  });

  it("is none when nothing is close enough", () => {
    expect(
      nearestPointWithin(points, everywhere, project, { x: 30, y: -30 }, 10),
    ).toBeNull();
  });

  it("does not project points outside the cursor's box", () => {
    let projected = 0;
    const counting = (lon: number, lat: number) => {
      projected += 1;
      return project(lon, lat);
    };
    const box = { west: 0.05, south: 0.05, east: 0.2, north: 0.2 };
    nearestPointWithin(points, box, counting, { x: 10, y: -10 }, 10);
    expect(projected).toBe(2);
  });
});
