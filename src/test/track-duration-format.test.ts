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
    expect(formatDurationSeconds(span)).toBe("26d 5h");
  });

  it("switches to days exactly at 24 hours", () => {
    expect(formatDurationSeconds(24 * 3600)).toBe("1d 0h");
    expect(formatDurationSeconds(24 * 3600 - 60)).toBe("23h 59m");
  });

  it("keeps hours and minutes below a day", () => {
    expect(formatDurationSeconds(5 * 3600 + 16 * 60)).toBe("5h 16m");
    expect(formatDurationSeconds(32 * 60)).toBe("32m");
    expect(formatDurationSeconds(0)).toBe("0m");
  });

  it("treats a negative span as zero rather than printing a minus", () => {
    expect(formatDurationSeconds(-120)).toBe("0m");
  });

  it("carries the day format into the composed row statistics", () => {
    const stats = formatTrackStats(4.9, 629 * 3600 + 22 * 60, 580);
    expect(stats).toBe("4.9 km · 26d 5h · 580 pts");
  });
});
