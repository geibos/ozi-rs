import { describe, expect, it } from "vitest";
import {
  chooseStep,
  formatDegrees,
  graticule,
  graticuleGeoJson,
} from "../lib/graticule";

/**
 * A search is cut into sectors and the sectors go out over the radio as
 * coordinates. Every paper sheet carries a grid for that; this map had none,
 * so reading a coordinate off it meant clicking a place and reading the status
 * bar, and writing one down meant the same in reverse.
 */
describe("chooseStep", () => {
  it("takes whole degrees across a region", () => {
    // Twelve degrees of latitude: two-degree lines give six, and five-degree
    // lines only two — the rule is the coarsest step that still fills the view.
    expect(chooseStep(12)).toBe(2);
  });

  it("takes minutes across a search area", () => {
    // A fifth of a degree is about twenty kilometres — a day's search.
    expect(chooseStep(0.2)).toBeCloseTo(2 / 60, 10);
  });

  it("takes seconds when the operator is on top of a clearing", () => {
    // 0.002° is about 220 m of latitude: one-second lines are 31 m apart and
    // give seven of them; two-second lines give three, which is too few.
    expect(chooseStep(0.002)).toBeCloseTo(1 / 3600, 10);
  });

  it("never runs out of steps", () => {
    // Below the finest step there is nothing left to choose; the finest is the
    // answer rather than an empty grid.
    expect(chooseStep(0.0000001)).toBeCloseTo(1 / 3600, 10);
    expect(chooseStep(0)).toBeCloseTo(1 / 3600, 10);
    expect(chooseStep(Number.NaN)).toBeCloseTo(1 / 3600, 10);
  });

  it("gives at least as many lines as asked for", () => {
    for (const span of [0.003, 0.05, 0.4, 3, 40]) {
      const step = chooseStep(span, 4);
      expect(span / step, `span ${span}`).toBeGreaterThanOrEqual(4);
    }
  });
});

describe("formatDegrees", () => {
  it("writes a whole degree without minutes", () => {
    expect(formatDegrees(60, "lat")).toBe("60°N");
    expect(formatDegrees(30, "lon")).toBe("30°E");
  });

  it("writes minutes with a leading zero", () => {
    expect(formatDegrees(59 + 6 / 60, "lat")).toBe("59°06'N");
  });

  it("writes seconds when the grid is that fine", () => {
    expect(formatDegrees(59 + 56 / 60 + 15 / 3600, "lat")).toBe("59°56'15\"N");
  });

  it("names the southern and western hemispheres", () => {
    expect(formatDegrees(-33.5, "lat")).toBe("33°30'S");
    expect(formatDegrees(-18.25, "lon")).toBe("18°15'W");
  });

  it("does not print sixty seconds", () => {
    // 59.99999999° is a whole degree once rounded, and "59°59'60\"" is not a
    // coordinate anybody says.
    expect(formatDegrees(59.999999999, "lat")).toBe("60°N");
  });
});

describe("graticule", () => {
  const area = { south: 59.9, west: 30.2, north: 60.0, east: 30.4 };

  it("draws parallels and meridians on round numbers", () => {
    const lines = graticule(area);
    expect(lines.length).toBeGreaterThan(4);

    const step = chooseStep(0.1);
    for (const line of lines.filter((l) => l.kind === "lat")) {
      // Every line stands at a whole number of its step — that is what makes
      // it sayable.
      expect(
        Math.abs(line.degrees / step - Math.round(line.degrees / step)),
      ).toBeLessThan(1e-9);
    }
  });

  it("gives each line the name of the place it stands at", () => {
    const lines = graticule(area);
    const parallel = lines.find((l) => l.kind === "lat");
    expect(parallel!.label).toBe(formatDegrees(parallel!.degrees, "lat"));
  });

  it("runs the lines past the corners so the edges have no gap", () => {
    const lines = graticule(area);
    const parallel = lines.find((l) => l.kind === "lat")!;
    expect(parallel.points[0].lon).toBeLessThan(area.west);
    expect(parallel.points[1].lon).toBeGreaterThan(area.east);
  });

  it("keeps a parallel off the pole", () => {
    const lines = graticule({ south: 88, west: 0, north: 90, east: 40 });
    for (const line of lines.filter((l) => l.kind === "lat")) {
      expect(Math.abs(line.degrees)).toBeLessThanOrEqual(90);
    }
  });

  it("says nothing about a view with no extent", () => {
    expect(graticule({ south: 60, west: 30, north: 60, east: 30 })).toEqual([]);
  });

  it("keeps the label and the line at the same place after many steps", () => {
    // Accumulated addition drifts: forty steps of one second is where a line
    // and its name part company if nothing snaps them back.
    const fine = graticule({
      south: 59.9,
      west: 30.2,
      north: 59.911,
      east: 30.211,
    });
    for (const line of fine) {
      const written = formatDegrees(line.degrees, line.kind);
      expect(line.label).toBe(written);
    }
  });
});

describe("graticuleGeoJson", () => {
  it("writes lon before lat, as GeoJSON does", () => {
    const collection = graticuleGeoJson([
      {
        kind: "lat",
        degrees: 60,
        label: "60°N",
        points: [
          { lat: 60, lon: 30 },
          { lat: 60, lon: 31 },
        ],
      },
    ]);
    const geometry = collection.features[0].geometry as GeoJSON.LineString;
    expect(geometry.coordinates).toEqual([
      [30, 60],
      [31, 60],
    ]);
    expect(collection.features[0].properties).toEqual({
      kind: "lat",
      label: "60°N",
    });
  });
});
