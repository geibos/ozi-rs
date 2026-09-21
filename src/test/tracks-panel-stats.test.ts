import { describe, expect, it } from "vitest";
import { readFileSync } from "fs";
import { join } from "path";
import { formatTrackStats } from "../lib/track-stats";

// Track-stats rendering moved from the floating `TracksPanel.svelte` to
// the `LibraryRail` Tracks tab (`redesign-library-sidebar`).
const source = readFileSync(
  join(__dirname, "../components/library/TracksTab.svelte"),
  "utf-8",
);

describe("Library Tracks tab statistics formatter", () => {
  it("renders distance, duration, and point count joined by · when timestamps are present", () => {
    expect(formatTrackStats(12.3, 5040, 156, "en")).toBe(
      "12.3 km · 1h 24m · 156 pts",
    );
    // The crews read Russian; the units follow the interface.
    expect(formatTrackStats(12.3, 5040, 156, "ru")).toBe(
      "12.3 км · 1ч 24мин · 156 тчк",
    );
  });

  it("omits the duration segment (and its separator) when duration is null", () => {
    const formatted = formatTrackStats(12.3, null, 156, "en");
    expect(formatted).toBe("12.3 km · 156 pts");
    expect(formatted).not.toMatch(/\d+h\s\d+m/);
    expect(formatted).not.toMatch(/·\s\d+m\s·/);
  });

  it("omits the duration segment when duration is undefined", () => {
    expect(formatTrackStats(0.5, undefined, 4, "en")).toBe("0.5 km · 4 pts");
  });

  it("shows minutes-only when the duration is under one hour", () => {
    expect(formatTrackStats(3.2, 2700, 50, "en")).toBe("3.2 km · 45m · 50 pts");
  });

  it("rounds distance to one decimal place", () => {
    expect(formatTrackStats(12.345, 60, 10, "en")).toBe(
      "12.3 km · 1m · 10 pts",
    );
    expect(formatTrackStats(0, 0, 0, "en")).toBe("0.0 km · 0m · 0 pts");
  });
});

// Reading GeoJSON properties into row fields, and treating a missing
// duration as absent, moved to `src/lib/track-features.ts` and are covered
// behaviourally by `track-features.test.ts`.
describe("Library Tracks tab statistics rendering", () => {
  it("imports the formatter and exposes a track-stats element", () => {
    expect(source).toContain("import { formatTrackStats }");
    expect(source).toContain('data-testid="track-stats"');
  });

  it("passes the loaded statistics through formatTrackStats", () => {
    expect(source).toContain("formatTrackStats(");
    expect(source).toContain("t.distanceKm");
    expect(source).toContain("t.durationSeconds");
    expect(source).toContain("t.pointCount");
  });
});
