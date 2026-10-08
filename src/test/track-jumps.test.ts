import { describe, expect, it } from "vitest";

import { findSuspects, formatLeg, legsOf } from "../lib/track-jumps";

/** Metres of latitude per degree, near enough for building fixtures. */
const M_PER_DEG_LAT = 111_195;

interface P {
  id: number;
  lat: number;
  lon: number;
  elevation: number | null;
  timestamp: string | null;
}

/**
 * A straight walk north from 59.9 N, one point every `stepM` metres and
 * `stepS` seconds, with `edit` to bend chosen points out of line.
 */
function walk(
  count: number,
  {
    stepM = 10,
    stepS = 5,
    firstId = 1,
    timed = true,
  }: { stepM?: number; stepS?: number; firstId?: number; timed?: boolean } = {},
): P[] {
  const start = Date.parse("2026-10-06T09:00:00Z");
  return Array.from({ length: count }, (_, i) => ({
    id: firstId + i,
    lat: 59.9 + (i * stepM) / M_PER_DEG_LAT,
    lon: 30.3,
    elevation: null,
    timestamp: timed ? new Date(start + i * stepS * 1000).toISOString() : null,
  }));
}

/** Move a point `metres` east of where it is. */
function pushEast(point: P, metres: number): void {
  const mPerDegLon = M_PER_DEG_LAT * Math.cos((point.lat * Math.PI) / 180);
  point.lon += metres / mPerDegLon;
}

/** Move every point from `from` on `metres` further north. */
function shiftNorth(points: P[], from: number, metres: number): void {
  for (const p of points.slice(from)) p.lat += metres / M_PER_DEG_LAT;
}

describe("legs between points", () => {
  it("gives the distance and the speed from the previous point", () => {
    const points = walk(2, { stepM: 351, stepS: 20 });
    const legs = legsOf(points);
    expect(legs[0]).toBeNull();
    expect(legs[1]!.distanceM).toBeCloseTo(351, 0);
    expect(legs[1]!.seconds).toBe(20);
    expect(legs[1]!.speedKmh).toBeCloseTo(63.2, 1);
  });

  it("has no speed without times, or when the clock did not move", () => {
    expect(legsOf(walk(2, { timed: false }))[1]!.speedKmh).toBeNull();
    expect(legsOf(walk(2, { stepS: 0 }))[1]!.speedKmh).toBeNull();
  });
});

describe("jumps and outliers", () => {
  it("finds a spike by its apex, and not at its neighbours", () => {
    const points = walk(40);
    pushEast(points[20], 400);
    const suspects = findSuspects([{ id: 1, points }]);
    expect(suspects).toHaveLength(1);
    expect(suspects[0].kind).toBe("outlier");
    expect(suspects[0].pointId).toBe(21);
    expect(suspects[0].before.distanceM).toBeGreaterThan(390);
    expect(suspects[0].after!.distanceM).toBeGreaterThan(390);
  });

  it("finds a gap as a jump at the point after it", () => {
    const points = walk(40);
    shiftNorth(points, 25, 900);
    const suspects = findSuspects([{ id: 1, points }]);
    expect(suspects).toHaveLength(1);
    expect(suspects[0]).toMatchObject({
      kind: "jump",
      pointId: 26,
      previousPointId: 25,
      segmentId: 1,
    });
    expect(suspects[0].before.distanceM).toBeCloseTo(910, -1);
  });

  it("leaves a clean track alone", () => {
    expect(findSuspects([{ id: 1, points: walk(100) }])).toEqual([]);
  });

  it("does not call a short leg a jump on a dense track", () => {
    // A navigator logging every second while standing: 1 m steps. A 60 m leg
    // is sixty steps but not a jump — a jump is at least 100 m.
    const points = walk(60, { stepM: 1, stepS: 1 });
    shiftNorth(points, 30, 60);
    expect(findSuspects([{ id: 1, points }])).toEqual([]);
  });

  it("does not treat the line between two segments as a leg", () => {
    const first = walk(20);
    const second = walk(20, { firstId: 100 });
    shiftNorth(second, 0, 5_000);
    expect(
      findSuspects([
        { id: 1, points: first },
        { id: 2, points: second },
      ]),
    ).toEqual([]);
  });

  it("lists suspects in track order across segments", () => {
    const first = walk(30);
    pushEast(first[10], 300);
    const second = walk(30, { firstId: 100 });
    shiftNorth(second, 15, 2_000);
    const kinds = findSuspects([
      { id: 1, points: first },
      { id: 2, points: second },
    ]).map((s) => [s.kind, s.segmentId, s.pointId]);
    expect(kinds).toEqual([
      ["outlier", 1, 11],
      ["jump", 2, 115],
    ]);
  });

  it("finds nothing in a track too short to have a median", () => {
    expect(findSuspects([{ id: 1, points: walk(1) }])).toEqual([]);
    expect(findSuspects([])).toEqual([]);
  });
});

describe("a break between days", () => {
  it("is found where the clock jumps by hours, whatever the distance", () => {
    // A navigator that came from the last search: its old points first, then
    // five days later this search's, a few metres away.
    const points = walk(30);
    const later = Date.parse(points[10].timestamp!) + 5 * 86_400_000;
    points.slice(10).forEach((p, i) => {
      p.timestamp = new Date(later + i * 5000).toISOString();
    });
    const suspects = findSuspects([{ id: 1, points }]);
    expect(suspects).toHaveLength(1);
    expect(suspects[0]).toMatchObject({
      kind: "break",
      pointId: 11,
      previousPointId: 10,
    });
    expect(suspects[0].before.seconds).toBeCloseTo(5 * 86_400 + 5, -1);
  });

  it("is a break, not a jump, when the gap is far as well", () => {
    const points = walk(30);
    shiftNorth(points, 20, 5_000);
    const later = Date.parse(points[20].timestamp!) + 86_400_000;
    points.slice(20).forEach((p, i) => {
      p.timestamp = new Date(later + i * 5000).toISOString();
    });
    expect(findSuspects([{ id: 1, points }]).map((s) => s.kind)).toEqual([
      "break",
    ]);
  });

  it("leaves a pause of an hour alone", () => {
    const points = walk(30);
    const later = Date.parse(points[15].timestamp!) + 3_600_000;
    points.slice(15).forEach((p, i) => {
      p.timestamp = new Date(later + i * 5000).toISOString();
    });
    expect(findSuspects([{ id: 1, points }])).toEqual([]);
  });
});

describe("a leg, as the points table writes it", () => {
  const leg = (distanceM: number, speedKmh: number | null) => ({
    distanceM,
    seconds: null,
    speedKmh,
  });

  it("gives metres under a kilometre, and the speed", () => {
    expect(formatLeg(leg(351.4, 63.2), "ru")).toBe("351\u00a0м · 63\u00a0км/ч");
    expect(formatLeg(leg(351.4, 63.2), "en")).toBe("351\u00a0m · 63\u00a0km/h");
  });

  it("gives kilometres from a kilometre, and no speed when there is none", () => {
    expect(formatLeg(leg(1_634, null), "ru")).toBe("1.6\u00a0км");
  });
});
