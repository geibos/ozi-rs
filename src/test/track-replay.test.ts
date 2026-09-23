import { describe, expect, it } from "vitest";
import {
  GAP_MS,
  formatMoment,
  positionAt,
  replayRange,
} from "../lib/track-replay";

const at = (iso: string) => Date.parse(iso);

/** A crew walking east, a point a minute. */
const walk = {
  points: [
    { lat: 59.9, lon: 30.3, timestamp: "2026-07-08T14:00:00Z" },
    { lat: 59.9, lon: 30.31, timestamp: "2026-07-08T14:01:00Z" },
    { lat: 59.9, lon: 30.32, timestamp: "2026-07-08T14:02:00Z" },
  ],
};

/**
 * A recording is a list of places with times on them, and the question a
 * headquarters asks is the other way round: where were they at half past two.
 * Answering that from the points table means scrolling a few thousand rows.
 */
describe("replayRange", () => {
  it("runs from the first moment to the last", () => {
    expect(replayRange([walk])).toEqual({
      fromMs: at("2026-07-08T14:00:00Z"),
      toMs: at("2026-07-08T14:02:00Z"),
    });
  });

  it("spans every segment of the track", () => {
    const afterABreak = {
      points: [
        { lat: 59.91, lon: 30.4, timestamp: "2026-07-08T15:00:00Z" },
        { lat: 59.91, lon: 30.41, timestamp: "2026-07-08T15:01:00Z" },
      ],
    };
    expect(replayRange([walk, afterABreak])?.toMs).toBe(
      at("2026-07-08T15:01:00Z"),
    );
  });

  it("has nothing to play back with one timed point", () => {
    expect(
      replayRange([
        {
          points: [
            { lat: 59.9, lon: 30.3, timestamp: "2026-07-08T14:00:00Z" },
            { lat: 59.9, lon: 30.31, timestamp: null },
          ],
        },
      ]),
    ).toBe(null);
  });

  it("has nothing to play back with no times at all", () => {
    expect(
      replayRange([{ points: [{ lat: 59.9, lon: 30.3, timestamp: null }] }]),
    ).toBe(null);
  });
});

describe("positionAt", () => {
  it("is exactly on a point at that point's own moment", () => {
    const where = positionAt([walk], at("2026-07-08T14:01:00Z"));
    expect(where!.lat).toBeCloseTo(59.9, 10);
    expect(where!.lon).toBeCloseTo(30.31, 10);
  });

  it("is between two points half way between their moments", () => {
    // A crew walking at four kilometres an hour is sixty metres along by the
    // half minute; the nearest point is the wrong answer by that much.
    const where = positionAt([walk], at("2026-07-08T14:00:30Z"));
    expect(where!.lon).toBeCloseTo(30.305, 10);
  });

  it("answers the start before the recording begins", () => {
    const where = positionAt([walk], at("2026-07-08T13:00:00Z"));
    expect(where!.lon).toBeCloseTo(30.3, 10);
    expect(where!.atMs).toBe(at("2026-07-08T14:00:00Z"));
  });

  it("answers the end after it finishes", () => {
    const where = positionAt([walk], at("2026-07-08T18:00:00Z"));
    expect(where!.lon).toBeCloseTo(30.32, 10);
  });

  it("says when the moment falls in a silence", () => {
    // The navigator was off for half an hour. The straight line across that is
    // a guess, and a coordinator reading a position off it should be told.
    const withGap = {
      points: [
        { lat: 59.9, lon: 30.3, timestamp: "2026-07-08T14:00:00Z" },
        { lat: 59.9, lon: 30.5, timestamp: "2026-07-08T14:30:00Z" },
      ],
    };
    expect(positionAt([withGap], at("2026-07-08T14:15:00Z"))!.inGap).toBe(true);
    // A minute between points is a recording, not a silence.
    expect(positionAt([walk], at("2026-07-08T14:00:30Z"))!.inGap).toBe(false);
  });

  it("reads a recording that arrived out of order", () => {
    // Untidy recordings are why "sort points by time" exists as an edit; the
    // replay must not need it run first.
    const shuffled = {
      points: [
        { lat: 59.9, lon: 30.32, timestamp: "2026-07-08T14:02:00Z" },
        { lat: 59.9, lon: 30.3, timestamp: "2026-07-08T14:00:00Z" },
        { lat: 59.9, lon: 30.31, timestamp: "2026-07-08T14:01:00Z" },
      ],
    };
    expect(positionAt([shuffled], at("2026-07-08T14:00:30Z"))!.lon).toBeCloseTo(
      30.305,
      10,
    );
  });

  it("ignores points with no time and points with a broken one", () => {
    const messy = {
      points: [
        { lat: 59.9, lon: 30.3, timestamp: "2026-07-08T14:00:00Z" },
        { lat: 0, lon: 0, timestamp: null },
        { lat: 0, lon: 0, timestamp: "not a date" },
        { lat: 59.9, lon: 30.32, timestamp: "2026-07-08T14:02:00Z" },
      ],
    };
    const where = positionAt([messy], at("2026-07-08T14:01:00Z"));
    expect(where!.lat).toBeCloseTo(59.9, 10);
    expect(where!.lon).toBeCloseTo(30.31, 10);
  });

  it("has no answer for a track with no times", () => {
    expect(
      positionAt([{ points: [{ lat: 59.9, lon: 30.3, timestamp: null }] }], 0),
    ).toBe(null);
  });

  it("uses a silence threshold a crew could actually produce", () => {
    // Long enough that a navigator logging every ten seconds is never in a
    // gap; short enough that a stop for a cigarette is.
    expect(GAP_MS).toBeGreaterThanOrEqual(60_000);
    expect(GAP_MS).toBeLessThanOrEqual(15 * 60_000);
  });
});

describe("formatMoment", () => {
  it("reads like a radio log", () => {
    const noon = new Date(2026, 6, 8, 14, 30, 5).getTime();
    expect(formatMoment(noon)).toBe("14:30:05");
  });

  it("pads the single digits", () => {
    const early = new Date(2026, 6, 8, 4, 5, 6).getTime();
    expect(formatMoment(early)).toBe("04:05:06");
  });
});
