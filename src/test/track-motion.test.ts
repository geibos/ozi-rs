import { describe, expect, it } from "vitest";
import {
  ascentDescent,
  DEFAULT_MOTION_SETTINGS,
  movingSeconds,
} from "../lib/track-motion";

/**
 * Time on the move, and the climb.
 *
 * Both are questions about what GPS says when nothing is happening: a standing
 * navigator's position wanders by five to twenty metres and the elevation by
 * five to ten. The owner's thresholds (2026-10-01): a stop is less than 25 m
 * in two minutes, a climb counts from 5 m.
 */

/** Metres per degree of latitude, near enough for test geometry. */
const M_PER_DEG = 111_320;
const START = Date.parse("2026-09-28T09:00:00Z");

interface P {
  lat: number;
  lon: number;
  elevation: number | null;
  timestamp: string | null;
}

/** A deterministic wobble in [-1, 1), so the tests do not flake. */
function wobble(i: number): number {
  const x = Math.sin(i * 12.9898) * 43758.5453;
  return (x - Math.floor(x)) * 2 - 1;
}

/**
 * Points every `stepS` seconds for `seconds`, moving north at `kmh`, with the
 * position wandering by up to `jitterM` metres.
 */
function leg(
  fromS: number,
  seconds: number,
  kmh: number,
  opts: { stepS?: number; jitterM?: number; fromM?: number } = {},
): { points: P[]; endM: number } {
  const stepS = opts.stepS ?? 5;
  const jitterM = opts.jitterM ?? 0;
  const fromM = opts.fromM ?? 0;
  const points: P[] = [];
  let metres = fromM;
  for (let t = 0; t <= seconds; t += stepS) {
    metres = fromM + (kmh * 1000 * t) / 3600;
    const i = fromS + t;
    points.push({
      lat: 60 + (metres + jitterM * wobble(i)) / M_PER_DEG,
      lon: 30 + (jitterM * wobble(i + 7)) / M_PER_DEG,
      elevation: null,
      timestamp: new Date(START + (fromS + t) * 1000).toISOString(),
    });
  }
  return { points, endM: metres };
}

const minutes = (s: number | null) => (s === null ? null : s / 60);

describe("movingSeconds", () => {
  it("counts a walk with a rest stop as the walking, not the stop", () => {
    const walk1 = leg(0, 30 * 60, 3);
    // ±7 m on each axis keeps every fix within ten metres of where they stood.
    const rest = leg(30 * 60, 20 * 60, 0, { jitterM: 7, fromM: walk1.endM });
    const walk2 = leg(50 * 60, 30 * 60, 3, { fromM: rest.endM });
    const segment = {
      points: [
        ...walk1.points,
        ...rest.points.slice(1),
        ...walk2.points.slice(1),
      ],
    };
    const moving = minutes(movingSeconds([segment], DEFAULT_MOTION_SETTINGS))!;
    // The two-minute window blurs each edge of the stop by up to two minutes.
    expect(moving).toBeGreaterThan(56);
    expect(moving).toBeLessThan(64);
  });

  it("counts slow, careful combing as moving", () => {
    const comb = leg(0, 60 * 60, 1, { jitterM: 3 });
    const moving = minutes(
      movingSeconds([{ points: comb.points }], DEFAULT_MOTION_SETTINGS),
    )!;
    expect(moving).toBeGreaterThan(57);
  });

  it("says a recording that never moved was never moving", () => {
    const standing = leg(0, 30 * 60, 0, { jitterM: 7 });
    expect(
      movingSeconds([{ points: standing.points }], DEFAULT_MOTION_SETTINGS),
    ).toBe(0);
  });

  it("has nothing to say about a track without times", () => {
    const walk = leg(0, 10 * 60, 3);
    const untimed = walk.points.map((p) => ({ ...p, timestamp: null }));
    expect(
      movingSeconds([{ points: untimed }], DEFAULT_MOTION_SETTINGS),
    ).toBeNull();
  });

  it("does not count the time between two segments", () => {
    const morning = leg(0, 30 * 60, 3);
    const evening = leg(8 * 3600, 30 * 60, 3, { fromM: 5000 });
    const moving = minutes(
      movingSeconds(
        [{ points: morning.points }, { points: evening.points }],
        DEFAULT_MOTION_SETTINGS,
      ),
    )!;
    expect(moving).toBeGreaterThan(58);
    expect(moving).toBeLessThan(61);
  });

  it("follows a different stop distance", () => {
    // 1 km/h is 33 m per two minutes: moving at 25 m, stopped at 40 m.
    const comb = leg(0, 30 * 60, 1);
    const at40 = movingSeconds([{ points: comb.points }], {
      ...DEFAULT_MOTION_SETTINGS,
      stopDistanceM: 40,
    });
    expect(at40).toBe(0);
  });
});

describe("ascentDescent", () => {
  function profile(elevations: number[]): Array<{ points: P[] }> {
    return [
      {
        points: elevations.map((elevation, i) => ({
          lat: 60 + i / M_PER_DEG,
          lon: 30,
          elevation,
          timestamp: null,
        })),
      },
    ];
  }

  it("keeps noise on flat ground out of the climb", () => {
    // Two metres either side: a spread of four, inside the 5 m threshold.
    // Noise wider than the threshold is not noise to any threshold, which is
    // why the threshold is a setting.
    const flat = Array.from({ length: 500 }, (_, i) => 100 + 2 * wobble(i));
    expect(ascentDescent(profile(flat), DEFAULT_MOTION_SETTINGS)).toEqual({
      ascentM: 0,
      descentM: 0,
    });
  });

  it("counts a hill up and down", () => {
    const up = Array.from({ length: 101 }, (_, i) => 100 + i);
    const down = Array.from({ length: 101 }, (_, i) => 200 - i);
    const result = ascentDescent(
      profile([...up, ...down.slice(1)]),
      DEFAULT_MOTION_SETTINGS,
    )!;
    expect(result.ascentM).toBeGreaterThanOrEqual(95);
    expect(result.ascentM).toBeLessThanOrEqual(100);
    expect(result.descentM).toBeGreaterThanOrEqual(95);
    expect(result.descentM).toBeLessThanOrEqual(100);
  });

  it("counts a noisy hill as the hill, not the noise", () => {
    const hill = Array.from(
      { length: 400 },
      (_, i) => 100 + i / 4 + 2 * wobble(i),
    );
    const { ascentM, descentM } = ascentDescent(
      profile(hill),
      DEFAULT_MOTION_SETTINGS,
    )!;
    // A 100 m climb. Summed point by point the noise alone would add hundreds.
    expect(ascentM).toBeGreaterThan(95);
    expect(ascentM).toBeLessThan(105);
    expect(descentM).toBe(0);
  });

  it("skips points without a height and has nothing to say without any", () => {
    expect(
      ascentDescent(
        [{ points: [{ lat: 60, lon: 30, elevation: null, timestamp: null }] }],
        DEFAULT_MOTION_SETTINGS,
      ),
    ).toBeNull();
    const gappy = profile([100, 110, 120]);
    gappy[0].points.splice(1, 0, {
      lat: 60,
      lon: 30,
      elevation: null,
      timestamp: null,
    });
    expect(ascentDescent(gappy, DEFAULT_MOTION_SETTINGS)!.ascentM).toBe(20);
  });

  it("follows a different climb threshold", () => {
    const steps = [100, 104, 108, 112, 100];
    expect(
      ascentDescent(profile(steps), DEFAULT_MOTION_SETTINGS)!.ascentM,
    ).toBe(12);
    expect(
      ascentDescent(profile(steps), {
        ...DEFAULT_MOTION_SETTINGS,
        climbThresholdM: 20,
      }),
    ).toEqual({ ascentM: 0, descentM: 0 });
  });
});
