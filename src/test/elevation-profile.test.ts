import { describe, expect, it } from "vitest";
import { elevationProfile, profilePath } from "../lib/elevation-profile";

/**
 * The inspector carried a card that said "elevation chart — coming in a
 * follow-up change" while `PointDetailDto` had been carrying `elevation` all
 * along. A crew reading a recording of someone else's route wants to know what
 * the ground does: a climb out of a river valley is the difference between a
 * twenty-minute leg and an hour.
 *
 * Gain and loss are deliberately absent. Summing every rise in a GPS track
 * inflates the total by its own noise, and the threshold that fixes that is a
 * decision about the data, not about the chart — it is in `docs/backlog.md`
 * next to the moving-time threshold, which is the same question.
 */
const seg = (points: [number, number, number | null][]) => ({
  points: points.map(([lat, lon, elevation], i) => ({
    id: i + 1,
    lat,
    lon,
    elevation,
    timestamp: null,
  })),
});

describe("elevationProfile", () => {
  it("is null when nothing carries an elevation", () => {
    expect(
      elevationProfile([
        seg([
          [59.95, 31.59, null],
          [59.96, 31.6, null],
        ]),
      ]),
    ).toBeNull();
  });

  it("is null for a single sample — one point is not a profile", () => {
    expect(
      elevationProfile([
        seg([
          [59.95, 31.59, 120],
          [59.96, 31.6, null],
        ]),
      ]),
    ).toBeNull();
  });

  it("places each sample at its distance along the whole track", () => {
    const profile = elevationProfile([
      seg([
        [59.95, 31.59, 100],
        [59.96, 31.59, 150],
      ]),
    ]);
    expect(profile).not.toBeNull();
    expect(profile!.samples).toHaveLength(2);
    expect(profile!.samples[0].km).toBe(0);
    // ~1.11 km per 0.01° of latitude.
    expect(profile!.samples[1].km).toBeCloseTo(1.11, 1);
    expect(profile!.totalKm).toBeCloseTo(1.11, 1);
  });

  it("keeps counting distance across a point that has no elevation", () => {
    // The gap is in the elevation, not in the route: the third point must not
    // slide back towards the first.
    const profile = elevationProfile([
      seg([
        [59.95, 31.59, 100],
        [59.96, 31.59, null],
        [59.97, 31.59, 200],
      ]),
    ])!;
    expect(profile.samples).toHaveLength(2);
    expect(profile.samples[1].km).toBeCloseTo(2.22, 1);
  });

  it("carries the distance over from one segment to the next", () => {
    const profile = elevationProfile([
      seg([
        [59.95, 31.59, 100],
        [59.96, 31.59, 110],
      ]),
      seg([
        [59.97, 31.59, 120],
        [59.98, 31.59, 130],
      ]),
    ])!;
    expect(profile.samples).toHaveLength(4);
    expect(profile.samples[2].km).toBeGreaterThan(profile.samples[1].km);
  });

  it("reports the range the crew reads off the axis", () => {
    const profile = elevationProfile([
      seg([
        [59.95, 31.59, 100],
        [59.96, 31.59, 250],
        [59.97, 31.59, 40],
      ]),
    ])!;
    expect(profile.minMetres).toBe(40);
    expect(profile.maxMetres).toBe(250);
  });
});

describe("profilePath", () => {
  const profile = elevationProfile([
    seg([
      [59.95, 31.59, 100],
      [59.96, 31.59, 200],
      [59.97, 31.59, 100],
    ]),
  ])!;

  it("spans the box and puts the highest point at the top", () => {
    const d = profilePath(profile, 100, 40);
    const coords = [...d.matchAll(/(-?\d+(?:\.\d+)?),(-?\d+(?:\.\d+)?)/g)].map(
      ([, x, y]) => [Number(x), Number(y)],
    );
    expect(coords).toHaveLength(3);
    expect(coords[0][0]).toBe(0);
    expect(coords[2][0]).toBeCloseTo(100, 5);
    // SVG y grows downwards: the summit is the smallest y.
    expect(coords[1][1]).toBe(0);
    expect(coords[0][1]).toBe(40);
  });

  it("draws a flat track down the middle rather than dividing by zero", () => {
    const flat = elevationProfile([
      seg([
        [59.95, 31.59, 120],
        [59.96, 31.59, 120],
      ]),
    ])!;
    const d = profilePath(flat, 100, 40);
    const ys = [...d.matchAll(/,(-?\d+(?:\.\d+)?)/g)].map(([, y]) => Number(y));
    expect(new Set(ys)).toEqual(new Set([20]));
  });

  it("starts with a move and continues with lines", () => {
    expect(profilePath(profile, 100, 40)).toMatch(
      /^M[\d.]+,[\d.]+( L[\d.]+,[\d.]+)+$/,
    );
  });
});
