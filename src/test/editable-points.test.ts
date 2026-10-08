import { describe, expect, it } from "vitest";

import { editablePointsInView } from "../lib/editable-points";

const segments = [
  {
    id: 1,
    points: [
      { id: 1, lat: 0.1, lon: 0.1 },
      { id: 2, lat: 0.5, lon: 0.5 },
    ],
  },
  { id: 2, points: [{ id: 3, lat: 0.2, lon: 0.2 }] },
];
const view = { west: 0, south: 0, east: 0.3, north: 0.3 };

describe("handles in edit mode", () => {
  it("go on the points in view, with their place in the segment", () => {
    const { points, tooMany } = editablePointsInView(segments, view);
    expect(tooMany).toBeNull();
    expect(points.map((p) => [p.segment.id, p.point.id, p.index])).toEqual([
      [1, 1, 0],
      [2, 3, 0],
    ]);
  });

  it("go on none, and say how many, when too many are in view", () => {
    expect(editablePointsInView(segments, view, 1)).toEqual({
      points: [],
      tooMany: 2,
    });
  });
});
