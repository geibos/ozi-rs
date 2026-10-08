// @vitest-environment jsdom
/**
 * The box tool against a stand-in map: a drag chooses the selected track's
 * points inside the box, Shift adds a second box, and the bar's buttons
 * delete the chosen points or keep only them.
 */
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { cleanup, fireEvent, render, screen } from "@testing-library/svelte";
import { get } from "svelte/store";
import { tick } from "svelte";

const { removeTrackPoints, getTrackDetail } = vi.hoisted(() => ({
  removeTrackPoints: vi.fn(async () => 2),
  getTrackDetail: vi.fn(async () => ({
    id: 7,
    name: "20261008_Lisa15",
    segments: [
      {
        id: 1,
        points: [
          // Degrees times a hundred are pixels in the stand-in projection.
          { id: 1, lat: 0.1, lon: 0.1, elevation: null, timestamp: null },
          { id: 2, lat: 0.2, lon: 0.2, elevation: null, timestamp: null },
          { id: 3, lat: 0.5, lon: 0.5, elevation: null, timestamp: null },
        ],
      },
    ],
  })),
}));

vi.mock("$lib/api", () => ({ removeTrackPoints, getTrackDetail }));

import BoxSelect from "../components/BoxSelect.svelte";
import { boxSelection, setBoxSelect } from "../lib/box-select";
import { selectedTrack, tracksGeometryVersion } from "../lib/stores";
import { setLocale } from "../lib/i18n";

type Handler = (e: unknown) => void;

/** Enough of a MapLibre map for the tool: events, a projection, a canvas. */
function standInMap() {
  const handlers = new Map<string, Set<Handler>>();
  const canvas = document.createElement("canvas");
  const map = {
    on: (type: string, fn: Handler) => {
      if (!handlers.has(type)) handlers.set(type, new Set());
      handlers.get(type)!.add(fn);
    },
    off: (type: string, fn: Handler) => handlers.get(type)?.delete(fn),
    project: ([lon, lat]: [number, number]) => ({ x: lon * 100, y: lat * 100 }),
    getCanvas: () => canvas,
    dragPan: { disable: vi.fn(), enable: vi.fn() },
    boxZoom: { disable: vi.fn(), enable: vi.fn() },
    isStyleLoaded: () => false,
    getSource: () => undefined,
  };
  const fire = (type: string, x: number, y: number, shiftKey = false) => {
    for (const fn of handlers.get(type) ?? []) {
      fn({ point: { x, y }, originalEvent: { button: 0, shiftKey } });
    }
  };
  const drag = async (
    from: [number, number],
    to: [number, number],
    shiftKey = false,
  ) => {
    fire("mousedown", ...from, shiftKey);
    fire("mousemove", ...to, shiftKey);
    fire("mouseup", ...to, shiftKey);
    await vi.waitFor(() => expect(getTrackDetail).toHaveBeenCalled());
    await tick();
  };
  return { map, drag, handlers };
}

describe("the box tool on the map", () => {
  let stand: ReturnType<typeof standInMap>;

  beforeEach(async () => {
    setLocale("ru");
    vi.clearAllMocks();
    selectedTrack.set({ layerId: 5n, trackId: 7n });
    stand = standInMap();
    render(BoxSelect, { props: { map: stand.map as never } });
    setBoxSelect(true);
    await tick();
  });

  afterEach(() => {
    setBoxSelect(false);
    selectedTrack.set(null);
    cleanup();
  });

  it("takes the drag from the map while it is on", () => {
    expect(stand.map.dragPan.disable).toHaveBeenCalled();
    expect(stand.handlers.get("mousedown")?.size).toBe(1);
  });

  it("chooses the points inside the box, and adds a second box with Shift", async () => {
    await stand.drag([0, 0], [25, 25]);
    await vi.waitFor(() => expect(get(boxSelection)?.ids.size).toBe(2));
    expect(screen.getByTestId("box-select-count").textContent).toContain("2");

    await stand.drag([40, 40], [60, 60], true);
    await vi.waitFor(() => expect(get(boxSelection)?.ids.size).toBe(3));

    await stand.drag([40, 40], [60, 60]);
    await vi.waitFor(() =>
      expect([...(get(boxSelection)?.ids ?? [])]).toEqual([3]),
    );
  });

  it("deletes the chosen points, or keeps only them", async () => {
    await stand.drag([0, 0], [25, 25]);
    await vi.waitFor(() => screen.getByTestId("box-select-delete"));
    const version = get(tracksGeometryVersion);

    await fireEvent.click(screen.getByTestId("box-select-delete"));
    await vi.waitFor(() =>
      expect(removeTrackPoints).toHaveBeenCalledWith(5n, 7n, [1n, 2n], false),
    );
    expect(get(boxSelection)).toBeNull();
    expect(get(tracksGeometryVersion)).toBe(version + 1);

    await stand.drag([0, 0], [25, 25]);
    await vi.waitFor(() => screen.getByTestId("box-select-keep"));
    await fireEvent.click(screen.getByTestId("box-select-keep"));
    await vi.waitFor(() =>
      expect(removeTrackPoints).toHaveBeenLastCalledWith(
        5n,
        7n,
        [1n, 2n],
        true,
      ),
    );
  });

  it("gives the map back when it goes off", async () => {
    setBoxSelect(false);
    await tick();
    expect(stand.map.dragPan.enable).toHaveBeenCalled();
    expect(stand.handlers.get("mousedown")?.size ?? 0).toBe(0);
  });
});
