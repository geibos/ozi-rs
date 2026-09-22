import { describe, expect, it } from "vitest";
import { focusPositions } from "../lib/map-bounds";

/**
 * "Показать всё" used to collect its waypoints by walking the MapLibre
 * markers that happened to be on the map. Those markers are render state:
 * they are put there by an asynchronous reconciler, so a click that lands
 * before the reconciler finishes frames the tracks and leaves the marks
 * outside the screen — the ШТАБ off-camera on the one action whose whole
 * job is "show me everything". External review, 2026-09-22.
 *
 * The camera now frames the data. These tests pin what "the data" means.
 */
describe("focusPositions", () => {
  const feature = (lon: number, lat: number) => ({
    geometry: { coordinates: [lon, lat] },
  });

  it("frames marks that are in the data but not yet on the map", () => {
    const points = focusPositions(
      [feature(31.0, 59.0)],
      [
        {
          layerId: "1",
          waypoints: [{ lon: 32.0, lat: 60.0, visible: true }],
        },
      ],
      () => [],
    );

    expect(points).toContainEqual({ lon: 32.0, lat: 60.0 });
    expect(points).toContainEqual({ lon: 31.0, lat: 59.0 });
  });

  it("leaves a hidden mark out of the frame", () => {
    const points = focusPositions(
      [],
      [
        {
          layerId: "1",
          waypoints: [
            { lon: 32.0, lat: 60.0, visible: false },
            { lon: 33.0, lat: 61.0, visible: true },
          ],
        },
      ],
      () => [],
    );

    expect(points).toEqual([{ lon: 33.0, lat: 61.0 }]);
  });

  it("falls back to what is drawn for a layer it could not read", () => {
    const points = focusPositions(
      [],
      [{ layerId: "7", waypoints: null }],
      (id) => (id === "7" ? [{ lon: 34.0, lat: 62.0 }] : []),
    );

    expect(points).toEqual([{ lon: 34.0, lat: 62.0 }]);
  });

  it("does not ask the map about a layer it read successfully", () => {
    const asked: string[] = [];
    focusPositions(
      [],
      [{ layerId: "1", waypoints: [{ lon: 32.0, lat: 60.0, visible: true }] }],
      (id) => {
        asked.push(id);
        return [];
      },
    );

    expect(asked).toEqual([]);
  });
});
