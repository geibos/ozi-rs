/**
 * The fixtures are only worth having if they are the real wire shape, and if
 * they carry the states a screen has to handle. Both are asserted here so a
 * regenerated fixture that lost its hidden track, or a DTO that changed shape,
 * fails in one obvious place rather than in whichever component test happens
 * to touch it.
 */
import { describe, expect, it } from "vitest";
import {
  appStateFixture,
  tracksGeojsonFixture,
  trackDetailFixture,
  waypointsFixture,
} from "./fixtures";
import { trackFeaturesFromGeojson } from "../lib/track-features";

describe("the fixtures are the shape the backend sends", () => {
  it("carries a project with tracks, waypoints and a catalogue", () => {
    expect(appStateFixture.project_name).toBeTypeOf("string");
    expect(appStateFixture.tracks.length).toBeGreaterThan(1);
    expect(appStateFixture.track_layers.length).toBeGreaterThan(1);
    expect(appStateFixture.waypoint_layers.length).toBeGreaterThan(1);
    expect(appStateFixture.projects.length).toBeGreaterThan(1);
  });

  it("has exactly one default layer of each kind, not two sharing an id", () => {
    // A fresh project used to carry two "Tracks" layers, both id 1, and the
    // second was unreachable because everything addresses a layer by id.
    const ids = appStateFixture.track_layers.map((l) => l.id);
    expect(new Set(ids).size).toBe(ids.length);
    const waypointIds = appStateFixture.waypoint_layers.map((l) => l.id);
    expect(new Set(waypointIds).size).toBe(waypointIds.length);
  });

  it("exercises both visibility states and a Cyrillic name", () => {
    expect(appStateFixture.tracks.some((t) => t.visible)).toBe(true);
    expect(appStateFixture.tracks.some((t) => !t.visible)).toBe(true);
    expect(appStateFixture.tracks.some((t) => /[а-яё]/i.test(t.name))).toBe(
      true,
    );
  });

  it("exercises a downloaded and a not-downloaded map, both with sizes", () => {
    const maps = appStateFixture.current_project?.maps ?? [];
    expect(maps.some((m) => m.downloaded)).toBe(true);
    expect(maps.some((m) => !m.downloaded)).toBe(true);
    expect(maps.every((m) => m.size_bytes != null)).toBe(true);
  });
});

describe("the track geometry fixture", () => {
  it("is what the rail's row model can actually read", () => {
    const rows = trackFeaturesFromGeojson(tracksGeojsonFixture);

    // The regression this guards: a geometry type the row model filtered out.
    expect(rows.length).toBe(appStateFixture.tracks.length);
    expect(rows.some((r) => !r.visible)).toBe(true);
    expect(rows.every((r) => r.pointCount > 0)).toBe(true);
  });

  it("keeps a recording's segments apart", () => {
    const multi = tracksGeojsonFixture.features.find(
      (f) => f.geometry.type === "MultiLineString",
    );
    expect(multi, "a track recorded in two sittings").toBeDefined();
    const parts = (multi!.geometry as GeoJSON.MultiLineString).coordinates;
    expect(parts.length).toBeGreaterThan(1);
  });
});

describe("the detail fixtures", () => {
  it("give the inspector a two-segment track with timestamps", () => {
    expect(trackDetailFixture.segments.length).toBe(2);
    const points = trackDetailFixture.segments.flatMap((s) => s.points);
    expect(points.every((p) => p.timestamp != null)).toBe(true);
    expect(points.every((p) => p.elevation != null)).toBe(true);
  });

  it("give the waypoints tab a symbol, a plain mark and a hidden one", () => {
    expect(waypointsFixture.some((w) => w.symbol != null)).toBe(true);
    expect(waypointsFixture.some((w) => w.symbol == null)).toBe(true);
    expect(waypointsFixture.some((w) => !w.visible)).toBe(true);
  });
});
