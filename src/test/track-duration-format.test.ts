import { describe, expect, it } from "vitest";
import { formatDurationSeconds, formatTrackStats } from "../lib/track-stats";

/**
 * `duration_seconds` is the span between a track's first and last timestamp,
 * not moving time. A multi-day recording therefore produces a number like
 * 629 hours, which the owner read as a bug when the row showed "629h 22m"
 * next to "4.9 km". Spans of a day or more are formatted in days so they read
 * as a date range rather than an implausible walking time.
 */
describe("duration spans of a day or more", () => {
  it("formats a multi-day span in days and hours", () => {
    const span = 26 * 24 * 3600 + 5 * 3600 + 17 * 60;
    expect(formatDurationSeconds(span, "en")).toBe("26d 5h");
  });

  it("switches to days exactly at 24 hours", () => {
    expect(formatDurationSeconds(24 * 3600, "en")).toBe("1d 0h");
    expect(formatDurationSeconds(24 * 3600 - 60, "en")).toBe("23h 59m");
  });

  it("keeps hours and minutes below a day", () => {
    expect(formatDurationSeconds(5 * 3600 + 16 * 60, "en")).toBe("5h 16m");
    expect(formatDurationSeconds(32 * 60, "en")).toBe("32m");
    expect(formatDurationSeconds(0, "en")).toBe("0m");
  });

  it("treats a negative span as zero rather than printing a minus", () => {
    expect(formatDurationSeconds(-120, "en")).toBe("0m");
  });

  it("carries the day format into the composed row statistics", () => {
    const stats = formatTrackStats(4.9, 629 * 3600 + 22 * 60, 580, "en");
    expect(stats).toBe("4.9 km · 26d 5h · 580 pts");
    expect(formatTrackStats(4.9, 629 * 3600 + 22 * 60, 580, "ru")).toBe(
      "4.9 км · 26д 5ч · 580 тчк",
    );
  });
});

describe("a start time a person can read", () => {
  it("formats an RFC3339 timestamp for the interface language", async () => {
    const { formatTimestamp } = await import("../lib/track-stats");

    // The inspector printed "2026-07-08T09:00:00+00:00" — a machine's answer
    // to "when did this start?".
    expect(formatTimestamp("2026-07-08T09:00:00+00:00", "ru")).toMatch(
      /08\.07\.2026/,
    );
    expect(formatTimestamp("2026-07-08T09:00:00+00:00", "en")).toMatch(
      /08\/07\/2026/,
    );
  });

  it("says nothing when there is no timestamp, and passes through nonsense", async () => {
    const { formatTimestamp } = await import("../lib/track-stats");

    expect(formatTimestamp(null, "ru")).toBeNull();
    expect(formatTimestamp(undefined, "ru")).toBeNull();
    expect(formatTimestamp("not a date", "ru")).toBe("not a date");
  });
});

/**
 * A track of one point has no length and no elapsed span. It showed
 * "0.0 км · 0мин · 1 тчк" — two numbers that are not measurements of anything,
 * in front of the one that is. Such tracks became visible in the list on
 * 2026-09-21; before that they had no row at all.
 */
describe("a track with nothing to measure", () => {
  it("says only how many points it has", () => {
    expect(formatTrackStats(0, 0, 1, "ru")).toBe("1 тчк");
    expect(formatTrackStats(0, 0, 1, "en")).toBe("1 pt");
    expect(formatTrackStats(0, null, 0, "ru")).toBe("0 тчк");
  });

  it("measures a track that has two points", () => {
    expect(formatTrackStats(0.4, 120, 2, "ru")).toBe("0.4 км · 2мин · 2 тчк");
  });
});
