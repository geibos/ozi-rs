import { describe, expect, it } from "vitest";
import {
  filterTrackFeatures,
  trackFeaturesFromSummaries,
} from "../lib/track-features";
import type { TrackSummaryDto } from "../lib/bindings";

/**
 * The Tracks tab used to build its rows from the map's GeoJSON. That cost
 * every coordinate of every track over IPC to draw a list of names, and it
 * inherited the map's omission of a track with nothing drawable — so a
 * one-point track had no row, and could not be renamed or deleted.
 *
 * The rows come from `list_tracks` now: one per track, no geometry.
 */
describe("track rows built from the listing", () => {
  const summary = (over: Partial<TrackSummaryDto> = {}): TrackSummaryDto =>
    ({
      layer_id: 2,
      track_id: 7,
      name: "20260709-ЛИСА10",
      color: "#ff0000",
      line_width: 3,
      visible: true,
      distance_km: 11.5,
      duration_seconds: 24000,
      point_count: 1193,
      ...over,
    }) as TrackSummaryDto;

  it("carries what a row shows", () => {
    const rows = trackFeaturesFromSummaries([summary()]);
    expect(rows).toHaveLength(1);
    expect(rows[0].name).toBe("20260709-ЛИСА10");
    expect(rows[0].layerId).toBe(2n);
    expect(rows[0].trackId).toBe(7n);
    expect(rows[0].distanceKm).toBeCloseTo(11.5);
    expect(rows[0].pointCount).toBe(1193);
  });

  it("gives a track the map cannot draw a row like any other", () => {
    const rows = trackFeaturesFromSummaries([
      summary({ track_id: 8, name: "точка отсечки", point_count: 1 }),
    ]);
    expect(rows).toHaveLength(1);
    expect(rows[0].pointCount).toBe(1);
  });

  it("treats a missing duration as absent rather than zero", () => {
    const rows = trackFeaturesFromSummaries([
      summary({ duration_seconds: null }),
    ]);
    expect(rows[0].durationSeconds).toBeNull();
  });

  it("orders rows by layer then track so one layer's rows cluster", () => {
    const rows = trackFeaturesFromSummaries([
      summary({ layer_id: 10, track_id: 1, name: "b" }),
      summary({ layer_id: 2, track_id: 9, name: "a2" }),
      summary({ layer_id: 2, track_id: 1, name: "a1" }),
    ]);
    expect(rows.map((r) => r.name)).toEqual(["a1", "a2", "b"]);
  });
});

/**
 * A field project carries dozens of tracks named by date and call sign
 * ("20260709-ЛИСА15"), spread over one layer per imported file. Scanning that
 * list by eye is the slowest step in cleaning up a search, so the rail needs a
 * filter that matches the way those names are typed: partial, case-insensitive
 * and Cyrillic-aware.
 */
describe("filtering track rows", () => {
  const rows = [
    { name: "20260709-ЛИСА15" },
    { name: "20260709_Veter2" },
    { name: "20260710 лиса19" },
    { name: "Походы на открытом воздухе" },
  ] as Parameters<typeof filterTrackFeatures>[0];

  it("returns everything for an empty or blank query", () => {
    expect(filterTrackFeatures(rows, "")).toHaveLength(4);
    expect(filterTrackFeatures(rows, "   ")).toHaveLength(4);
  });

  it("matches part of a name anywhere in it", () => {
    expect(filterTrackFeatures(rows, "Veter").map((r) => r.name)).toEqual([
      "20260709_Veter2",
    ]);
    expect(filterTrackFeatures(rows, "0709").map((r) => r.name)).toEqual([
      "20260709-ЛИСА15",
      "20260709_Veter2",
    ]);
  });

  it("ignores case in both alphabets", () => {
    expect(filterTrackFeatures(rows, "лиса").map((r) => r.name)).toEqual([
      "20260709-ЛИСА15",
      "20260710 лиса19",
    ]);
    expect(filterTrackFeatures(rows, "VETER")).toHaveLength(1);
  });

  it("ignores surrounding whitespace in the query", () => {
    expect(filterTrackFeatures(rows, "  лиса15 ")).toHaveLength(1);
  });

  it("returns nothing when no name matches", () => {
    expect(filterTrackFeatures(rows, "zzz")).toHaveLength(0);
  });
});
