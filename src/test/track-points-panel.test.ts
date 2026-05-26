// The legacy `TrackPointsPanel.svelte` floating card was retired by the
// `redesign-library-sidebar` change. Its data-loading and formatting
// helpers were extracted into the pure-TypeScript module
// `src/lib/track-points.ts` so the future Inspector pane (delivered by
// `redesign-inspector-pane`) can import them without a Svelte coupling.
import { describe, expect, it } from "vitest";
import { readFileSync, existsSync } from "fs";
import { join } from "path";
import {
  formatCoordinates,
  formatElevation,
  formatTimestamp,
  paginateSegment,
  paginateTrackDetail,
  TRACK_POINTS_INITIAL_LIMIT,
} from "../lib/track-points";
import type { PointDetail, SegmentDetail, TrackDetail } from "../lib/types";

const moduleSource = readFileSync(
  join(__dirname, "../lib/track-points.ts"),
  "utf-8"
);

describe("track-points module", () => {
  it("is a plain TypeScript module with no Svelte- or DOM-specific code", () => {
    // The Inspector pane will import this module from a Svelte component;
    // the module itself must remain pure so it can be unit-tested without
    // a DOM and reused by any future surface.
    expect(moduleSource).not.toMatch(/from\s+["']\$lib\/stores["']/);
    expect(moduleSource).not.toMatch(/svelte\/store/);
    expect(moduleSource).not.toMatch(/document\./);
    expect(moduleSource).not.toMatch(/window\./);
    expect(moduleSource).not.toMatch(/\$state\(|\$derived\(|\$effect\(/);
  });

  it("retires the floating TrackPointsPanel.svelte source file", () => {
    const legacyPath = join(
      __dirname,
      "../components/TrackPointsPanel.svelte"
    );
    expect(existsSync(legacyPath)).toBe(false);
  });

  it("formats coordinates to five decimal places", () => {
    const point: PointDetail = { id: 1, lat: 55.7558123, lon: 37.6173456 };
    expect(formatCoordinates(point)).toBe("55.75581, 37.61735");
  });

  it("formats elevation when present and returns null when missing", () => {
    expect(formatElevation({ id: 1, lat: 0, lon: 0, elevation: 123.456 })).toBe(
      "123.5m"
    );
    expect(formatElevation({ id: 1, lat: 0, lon: 0 })).toBeNull();
  });

  it("renders timestamps verbatim, never inventing placeholder text", () => {
    expect(
      formatTimestamp({
        id: 1,
        lat: 0,
        lon: 0,
        timestamp: "2024-06-01T10:00:00Z",
      })
    ).toBe("2024-06-01T10:00:00Z");
    expect(formatTimestamp({ id: 1, lat: 0, lon: 0 })).toBeNull();
  });

  it("paginates a single segment to the initial limit until expanded", () => {
    const points: PointDetail[] = Array.from({ length: 1500 }, (_, i) => ({
      id: i,
      lat: 0,
      lon: 0,
    }));
    const segment: SegmentDetail = { id: 7, points };
    const collapsed = paginateSegment(segment, false);
    expect(collapsed.displayed.length).toBe(TRACK_POINTS_INITIAL_LIMIT);
    expect(collapsed.remaining).toBe(500);
    const expanded = paginateSegment(segment, true);
    expect(expanded.displayed.length).toBe(1500);
    expect(expanded.remaining).toBe(0);
  });

  it("paginates an entire track detail per-segment", () => {
    const detail: TrackDetail = {
      id: 1,
      name: "T",
      segments: [
        {
          id: 1,
          points: Array.from({ length: 1500 }, (_, i) => ({
            id: i,
            lat: 0,
            lon: 0,
          })),
        },
        {
          id: 2,
          points: Array.from({ length: 100 }, (_, i) => ({
            id: i,
            lat: 0,
            lon: 0,
          })),
        },
      ],
    };
    const paged = paginateTrackDetail(detail, new Set([2]));
    expect(paged[0].remaining).toBe(500);
    expect(paged[1].remaining).toBe(0);
  });
});
