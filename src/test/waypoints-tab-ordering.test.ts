// @vitest-environment jsdom
/**
 * Behavioural cover for the library tabs not letting a stale load win.
 *
 * The tab reloads every waypoint layer whenever the app state changes, and
 * during a bundle download `state-changed` fires once per file. Nothing
 * ordered those reloads: the one that started first could finish last and put
 * its rows on screen, so the list could go backwards under the operator. The
 * same shape of bug that let an abandoned bundle preview replace the map list.
 */
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { render, cleanup } from "@testing-library/svelte";

const { getWaypoints, getAppState } = vi.hoisted(() => ({
  getWaypoints: vi.fn(),
  getAppState: vi.fn(async () => null as unknown),
}));

vi.mock("$lib/api", () => ({
  getAppState,
  getWaypoints,
  getWaypointsExportDefaultPath: vi.fn(async () => null),
  deleteWaypoint: vi.fn(async () => {}),
  exportGpxWaypoints: vi.fn(async () => {}),
  exportWptWaypoints: vi.fn(async () => {}),
  renameWaypoint: vi.fn(async () => {}),
  setAllWaypointsVisible: vi.fn(async () => {}),
  setWaypointSymbol: vi.fn(async () => {}),
  showOnlyWaypoint: vi.fn(async () => {}),
  toggleWaypointVisible: vi.fn(async () => {}),
}));

vi.mock("@tauri-apps/plugin-dialog", () => ({ open: vi.fn(async () => null) }));

import WaypointsTab from "./stubs/WaypointsTabHarness.svelte";
import { appState } from "../lib/stores";
import type { AppStateDto } from "../lib/bindings";

const waypoint = (id: number, name: string) => ({
  id,
  name,
  lat: 55.75,
  lon: 37.61,
  symbol: null,
  visible: true,
});

const twoLayers = {
  tracks: [],
  waypoints: [],
  track_layers: [],
  waypoint_layers: [
    { id: 1, name: "Waypoints", visible: true },
    { id: 2, name: "Imported", visible: true },
  ],
  map_layers: [],
} as unknown as AppStateDto;

async function names(container: HTMLElement): Promise<string[]> {
  for (let i = 0; i < 40; i += 1) {
    await new Promise((resolve) => setTimeout(resolve, 5));
    const found = Array.from(
      container.querySelectorAll(
        "[data-testid='waypoints-tab-list'] .truncate",
      ),
    ).map((el) => el.textContent?.trim() ?? "");
    if (found.length > 0) return found;
  }
  return [];
}

beforeEach(async () => {
  getAppState.mockResolvedValue(twoLayers);
  getWaypoints.mockReset();
  await appState.refresh();
});

afterEach(cleanup);

describe("overlapping waypoint reloads", () => {
  it("asks every layer at once rather than one after another", async () => {
    // Each layer's answer resolves only once every layer has been asked. With
    // sequential awaits the second call never happens and this hangs out.
    let asked = 0;
    const allAsked: Array<() => void> = [];
    getWaypoints.mockImplementation(
      (layerId: bigint) =>
        new Promise((resolve) => {
          asked += 1;
          allAsked.push(() =>
            resolve([waypoint(Number(layerId), `L${layerId}`)]),
          );
          if (asked === 2) allAsked.forEach((fire) => fire());
        }),
    );

    const { container } = render(WaypointsTab);

    expect(await names(container)).toEqual(["L1", "L2"]);
  });

  it("drops the rows of a reload that a newer one has overtaken", async () => {
    // The first round is held open; the second answers immediately. When the
    // first is finally let go, its rows are stale and must not appear.
    const held: Array<() => void> = [];
    let round = 0;
    getWaypoints.mockImplementation((layerId: bigint) => {
      const mine = Math.floor(round / 2) + 1;
      round += 1;
      if (mine === 1) {
        return new Promise((resolve) => {
          held.push(() => resolve([waypoint(Number(layerId), "stale")]));
        });
      }
      return Promise.resolve([waypoint(Number(layerId), "fresh")]);
    });

    const { container } = render(WaypointsTab);
    // A second state change, as a bundle download's per-file events produce.
    await appState.refresh();

    expect(await names(container)).toEqual(["fresh", "fresh"]);

    held.forEach((fire) => fire());
    await new Promise((resolve) => setTimeout(resolve, 20));

    expect(await names(container)).toEqual(["fresh", "fresh"]);
  });
});
