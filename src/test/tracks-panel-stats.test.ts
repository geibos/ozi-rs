import { describe, expect, it } from "vitest";
import { readFileSync } from "fs";
import { join } from "path";
import { formatTrackStats } from "../lib/track-stats";

const source = readFileSync(
  join(__dirname, "../components/TracksPanel.svelte"),
  "utf-8"
);

describe("TracksPanel statistics formatter", () => {
  it("renders distance, duration, and point count joined by · when timestamps are present", () => {
    // 12.3 km, 1h 24m = 5040s, 156 pts → matches the spec scenario verbatim.
    expect(formatTrackStats(12.3, 5040, 156)).toBe("12.3 km · 1h 24m · 156 pts");
  });

  it("omits the duration segment (and its separator) when duration is null", () => {
    const formatted = formatTrackStats(12.3, null, 156);
    expect(formatted).toBe("12.3 km · 156 pts");
    // No `<m>m` or `<h>h <m>m` token sneaks in when duration is missing.
    expect(formatted).not.toMatch(/\d+h\s\d+m/);
    expect(formatted).not.toMatch(/·\s\d+m\s·/);
  });

  it("omits the duration segment when duration is undefined", () => {
    expect(formatTrackStats(0.5, undefined, 4)).toBe("0.5 km · 4 pts");
  });

  it("shows minutes-only when the duration is under one hour", () => {
    // 45 * 60 = 2700s
    expect(formatTrackStats(3.2, 2700, 50)).toBe("3.2 km · 45m · 50 pts");
  });

  it("rounds distance to one decimal place", () => {
    expect(formatTrackStats(12.345, 60, 10)).toBe("12.3 km · 1m · 10 pts");
    expect(formatTrackStats(0, 0, 0)).toBe("0.0 km · 0m · 0 pts");
  });
});

describe("TracksPanel statistics rendering", () => {
  it("imports the formatter and exposes a track-stats element", () => {
    expect(source).toContain('import { formatTrackStats }');
    // The `class="track-stats"` selector is gone (chrome moved to Tailwind
    // utilities reading semantic tokens); the test-only data-testid hook is
    // still the contract for downstream UI/E2E assertions.
    expect(source).toContain('data-testid="track-stats"');
  });

  it("loads the new GeoJSON properties for distance, duration, and point count", () => {
    expect(source).toContain("distance_km");
    expect(source).toContain("duration_seconds");
    expect(source).toContain("point_count");
    expect(source).toContain("distanceKm:");
    expect(source).toContain("durationSeconds:");
    expect(source).toContain("pointCount:");
  });

  it("passes the loaded statistics through formatTrackStats", () => {
    expect(source).toContain("formatTrackStats(");
    expect(source).toContain("track.distanceKm");
    expect(source).toContain("track.durationSeconds");
    expect(source).toContain("track.pointCount");
  });

  it("treats null/undefined duration as missing so the segment is hidden", () => {
    // The mapping converts null/undefined raw duration into null on the model,
    // which formatTrackStats then drops from the rendered output.
    expect(source).toContain("rawDuration === null || rawDuration === undefined");
  });
});
