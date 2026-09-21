// @vitest-environment jsdom
/**
 * Behavioural cover for the Waypoints tab reaching parity with the Tracks
 * tab: a search that actually narrows the rendered rows, bulk visibility
 * that is one command rather than a loop, and a row that says where the
 * waypoint is and can put it on the map.
 *
 * These render the real component. The tab's earlier tests read its source
 * text, which is why nothing caught the Tracks tab going empty when the
 * geometry type changed.
 */
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { render, cleanup, fireEvent } from "@testing-library/svelte";
import { get } from "svelte/store";

const { setAllWaypointsVisible, showOnlyWaypoint, getAppState } = vi.hoisted(
  () => ({
    setAllWaypointsVisible: vi.fn(async () => {}),
    showOnlyWaypoint: vi.fn(async () => {}),
    getAppState: vi.fn(async () => null as unknown),
  }),
);

vi.mock("$lib/api", () => ({
  getAppState,
  getWaypoints: vi.fn(async (layerId: bigint) =>
    layerId === 1n
      ? [
          {
            id: 1,
            name: "ШТАБ",
            lat: 55.75123,
            lon: 37.61754,
            symbol: null,
            visible: true,
          },
          {
            id: 2,
            name: "Задача 1",
            lat: 55.8,
            lon: 37.7,
            symbol: null,
            visible: true,
          },
          {
            id: 3,
            name: "Задача 2",
            lat: 55.9,
            lon: 37.8,
            symbol: null,
            visible: false,
          },
        ]
      : [],
  ),
  setAllWaypointsVisible,
  showOnlyWaypoint,
  toggleWaypointVisible: vi.fn(async () => {}),
  renameWaypoint: vi.fn(async () => {}),
  setWaypointSymbol: vi.fn(async () => {}),
  deleteWaypoint: vi.fn(async () => {}),
  exportWptWaypoints: vi.fn(async () => {}),
  getWptExportDefaultPath: vi.fn(async () => null),
}));

vi.mock("@tauri-apps/plugin-dialog", () => ({ open: vi.fn(async () => null) }));

import WaypointsTab from "./stubs/WaypointsTabHarness.svelte";
import { appState, mapFocusRequest } from "../lib/stores";
import type { AppStateDto } from "../lib/types";

const stateWithOneLayer = {
  waypoint_layers: [{ id: 1, name: "Waypoints" }],
  track_layers: [],
  map_layers: [],
} as unknown as AppStateDto;

/** Wait for the tab's async waypoint load to land in the DOM. */
async function rows(container: HTMLElement): Promise<string[]> {
  for (let i = 0; i < 40; i += 1) {
    await new Promise((resolve) => setTimeout(resolve, 5));
    const names = Array.from(
      container.querySelectorAll(
        "[data-testid='waypoints-tab-list'] .truncate",
      ),
    ).map((el) => el.textContent?.trim() ?? "");
    if (names.length > 0) return names;
  }
  return [];
}

beforeEach(async () => {
  getAppState.mockResolvedValue(stateWithOneLayer);
  await appState.refresh();
  mapFocusRequest.set(null);
  setAllWaypointsVisible.mockClear();
  showOnlyWaypoint.mockClear();
});

afterEach(cleanup);

describe("Waypoints tab search", () => {
  it("narrows the rendered rows and reports how many are shown", async () => {
    const { container, getByTestId } = render(WaypointsTab);
    expect(await rows(container)).toEqual(["ШТАБ", "Задача 1", "Задача 2"]);

    const search = getByTestId("waypoint-search") as HTMLInputElement;
    await fireEvent.input(search, { target: { value: "задача" } });

    expect(await rows(container)).toEqual(["Задача 1", "Задача 2"]);
    expect(getByTestId("waypoint-search-count").textContent).toContain("2");
    expect(getByTestId("waypoint-search-count").textContent).toContain("3");
  });

  it("says nothing matched, distinctly from an empty project", async () => {
    const { container, getByTestId, queryByTestId } = render(WaypointsTab);
    await rows(container);

    await fireEvent.input(getByTestId("waypoint-search"), {
      target: { value: "нет такой" },
    });

    expect(queryByTestId("waypoint-search-empty")).not.toBeNull();
  });

  it("restores every row when the query is cleared", async () => {
    const { container, getByTestId } = render(WaypointsTab);
    await rows(container);

    await fireEvent.input(getByTestId("waypoint-search"), {
      target: { value: "штаб" },
    });
    expect(await rows(container)).toEqual(["ШТАБ"]);

    await fireEvent.click(getByTestId("waypoint-search-clear"));
    expect(await rows(container)).toEqual(["ШТАБ", "Задача 1", "Задача 2"]);
  });
});

describe("Waypoints tab bulk visibility", () => {
  it("hides every waypoint with one command, not one per row", async () => {
    const { container, getByTestId } = render(WaypointsTab);
    await rows(container);

    await fireEvent.click(getByTestId("waypoints-hide-all"));
    expect(setAllWaypointsVisible).toHaveBeenCalledTimes(1);
    expect(setAllWaypointsVisible).toHaveBeenCalledWith(false);

    await fireEvent.click(getByTestId("waypoints-show-all"));
    expect(setAllWaypointsVisible).toHaveBeenLastCalledWith(true);
  });
});

describe("Waypoints tab row", () => {
  it("shows the coordinates and can put the waypoint on the map", async () => {
    const { container, getAllByTestId } = render(WaypointsTab);
    await rows(container);

    expect(container.textContent).toContain("55.75123, 37.61754");

    await fireEvent.click(getAllByTestId("waypoint-show-on-map")[0]);
    expect(get(mapFocusRequest)).toMatchObject({
      kind: "waypoint",
      lat: 55.75123,
      lon: 37.61754,
    });
  });
});
