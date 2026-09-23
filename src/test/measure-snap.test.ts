import { describe, expect, it } from "vitest";
import {
  SNAP_RADIUS_PX,
  measuredBetween,
  snapToWaypoint,
  type SnapCandidate,
} from "../lib/measure-snap";
import type { LatLon } from "../lib/geo";

/** A projection where one degree is a thousand pixels. */
const project = (at: LatLon) => ({ x: at.lon * 1000, y: -at.lat * 1000 });

const marks: SnapCandidate[] = [
  { name: "ШТАБ", lat: 59.9, lon: 30.3 },
  { name: "ЗАБРОС", lat: 59.92, lon: 30.34 },
];

/**
 * "How far from the headquarters to the drop-off" is the commonest measurement
 * a coordinator makes, and clicking as near each mark as the hand manages is
 * close enough for a sketch and not for a task: fifteen pixels at a
 * two-kilometre view is eighty metres, and the number goes out over the radio.
 */
describe("snapToWaypoint", () => {
  it("takes the mark's own position when the click lands on it", () => {
    // Ten pixels off, inside the radius.
    const result = snapToWaypoint({ lat: 59.9, lon: 30.31 }, marks, project);
    expect(result).toEqual({ name: "ШТАБ", lat: 59.9, lon: 30.3 });
  });

  it("leaves a click that landed nowhere alone", () => {
    const click = { lat: 59.8, lon: 30.1 };
    expect(snapToWaypoint(click, marks, project)).toEqual({
      ...click,
      name: null,
    });
  });

  it("takes the nearer of two marks", () => {
    const crowd: SnapCandidate[] = [
      { name: "дальняя", lat: 59.9, lon: 30.3 },
      { name: "ближняя", lat: 59.9, lon: 30.3055 },
    ];
    const result = snapToWaypoint({ lat: 59.9, lon: 30.306 }, crowd, project);
    expect(result.name).toBe("ближняя");
  });

  it("does not care which order the marks were drawn in", () => {
    const together: SnapCandidate[] = [
      { name: "первая", lat: 59.9, lon: 30.3 },
      { name: "вторая", lat: 59.9, lon: 30.3 },
    ];
    const click = { lat: 59.9, lon: 30.3002 };
    expect(snapToWaypoint(click, together, project).name).toBe("первая");
    expect(
      snapToWaypoint(click, [...together].reverse(), project).name,
    ).toBe("вторая");
  });

  it("measures the reach on screen, not on the ground", () => {
    // The same click, at two zooms. Zoomed out, the mark is far in pixels and
    // the tape does not catch; zoomed in it does — which is the aiming the
    // operator is actually doing.
    const click = { lat: 59.9, lon: 30.32 };
    const zoomedOut = (at: LatLon) => ({ x: at.lon * 100, y: -at.lat * 100 });
    const zoomedIn = (at: LatLon) => ({ x: at.lon * 10000, y: -at.lat * 10000 });
    expect(snapToWaypoint(click, marks, zoomedOut).name).toBe("ШТАБ");
    expect(snapToWaypoint(click, marks, zoomedIn).name).toBe(null);
  });

  it("catches nothing when there is nothing to catch", () => {
    const click = { lat: 59.9, lon: 30.3 };
    expect(snapToWaypoint(click, [], project).name).toBe(null);
  });

  it("uses a radius somebody can actually hit", () => {
    // Fourteen pixels is about a fingertip on a trackpad and smaller than the
    // marker itself; a wider one catches marks the operator was aiming past.
    expect(SNAP_RADIUS_PX).toBeGreaterThanOrEqual(10);
    expect(SNAP_RADIUS_PX).toBeLessThanOrEqual(20);
  });
});

describe("measuredBetween", () => {
  it("names both marks when both ends caught one", () => {
    expect(
      measuredBetween([
        { lat: 59.9, lon: 30.3, name: "ШТАБ" },
        { lat: 59.92, lon: 30.34, name: "ЗАБРОС" },
      ]),
    ).toBe("ШТАБ → ЗАБРОС");
  });

  it("says nothing when one end is a bare click", () => {
    expect(
      measuredBetween([
        { lat: 59.9, lon: 30.3, name: "ШТАБ" },
        { lat: 59.92, lon: 30.34, name: null },
      ]),
    ).toBe(null);
  });

  it("says nothing about a measurement along a route", () => {
    expect(
      measuredBetween([
        { lat: 59.9, lon: 30.3, name: "ШТАБ" },
        { lat: 59.91, lon: 30.32, name: "поворот" },
        { lat: 59.92, lon: 30.34, name: "ЗАБРОС" },
      ]),
    ).toBe(null);
  });
});
