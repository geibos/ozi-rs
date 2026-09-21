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
  tracksListFixture,
  trackDetailFixture,
  waypointsFixture,
} from "./fixtures";
import { trackFeaturesFromSummaries } from "../lib/track-features";

describe("the fixtures are the shape the backend sends", () => {
  it("carries a project with tracks and waypoints, and no catalogue", () => {
    expect(appStateFixture.project_name).toBeTypeOf("string");
    expect(appStateFixture.tracks.length).toBeGreaterThan(1);
    expect(appStateFixture.track_layers.length).toBeGreaterThan(1);
    expect(appStateFixture.waypoint_layers.length).toBeGreaterThan(1);
    // The catalogue left the state snapshot: it is thirteen thousand rows and
    // this is fetched on every `state-changed`. It arrives as its own stream.
    expect(appStateFixture).not.toHaveProperty("projects");
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

  it("carries the open project's slug, not only its name", () => {
    // Without it the loader had a project and no way to ask for that
    // project: opened from the workspace, where no row has been clicked,
    // the only download button in the app returned before calling anything.
    expect(appStateFixture.current_project?.slug).toBeTruthy();
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
    const rows = trackFeaturesFromSummaries(tracksListFixture);

    expect(rows.length).toBe(appStateFixture.tracks.length);
    expect(rows.some((r) => !r.visible)).toBe(true);
  });

  /**
   * The map drops a track it cannot draw; the list must not. Holding the two
   * fixtures against each other is what keeps the stand from quietly showing
   * the same rows the old, geometry-derived list did.
   */
  it("lists a track the map has no feature for", () => {
    const listed = tracksListFixture.map((r) => r.name);
    const drawn = tracksGeojsonFixture.features.map(
      (f) => (f.properties ?? {}).name as string,
    );

    expect(listed.length).toBeGreaterThan(drawn.length);
    const undrawable = listed.filter((name) => !drawn.includes(name));
    expect(undrawable).toHaveLength(1);
    expect(
      tracksListFixture.find((r) => r.name === undrawable[0])?.point_count,
    ).toBe(1);
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
