import { beforeEach, describe, expect, it } from "vitest";
import { get } from "svelte/store";

import {
  boxBetween,
  boxSelectActive,
  boxSelection,
  combineSelection,
  pointsInBox,
  setBoxSelect,
} from "../lib/box-select";
import { editModeActive, measuringActive } from "../lib/stores";

/** A flat projection: a degree is a hundred pixels, north is up. */
const project = (lon: number, lat: number) => ({
  x: lon * 100,
  y: -lat * 100,
});

describe("a box drawn on the map", () => {
  it("is the same box whichever way it was dragged", () => {
    expect(boxBetween({ x: 30, y: 5 }, { x: 10, y: 25 })).toEqual({
      left: 10,
      top: 5,
      right: 30,
      bottom: 25,
    });
  });

  it("chooses the points that show inside it, edges included", () => {
    const points = [
      { id: 1, lat: 0.1, lon: 0.1 },
      { id: 2, lat: 0.2, lon: 0.2 },
      { id: 3, lat: 0.3, lon: 0.3 },
      { id: 4, lat: 0.25, lon: 0.9 },
    ];
    // y grows downwards: latitude 0.3 is y = -30.
    const box = boxBetween({ x: 10, y: -30 }, { x: 25, y: -10 });
    expect(pointsInBox(points, project, box)).toEqual([1, 2]);
  });

  it("replaces the selection, or adds to it with Shift", () => {
    const previous = new Set([1, 2]);
    expect([...combineSelection(previous, [3], false)]).toEqual([3]);
    expect([...combineSelection(previous, [2, 3], true)].sort()).toEqual([
      1, 2, 3,
    ]);
  });
});

describe("the box tool", () => {
  beforeEach(() => {
    setBoxSelect(false);
    editModeActive.set(false);
    measuringActive.set(false);
  });

  it("takes the drag from the other tools", () => {
    editModeActive.set(true);
    measuringActive.set(true);
    setBoxSelect(true);
    expect(get(boxSelectActive)).toBe(true);
    expect(get(editModeActive)).toBe(false);
    expect(get(measuringActive)).toBe(false);
  });

  it("drops the selection when it goes off", () => {
    setBoxSelect(true);
    boxSelection.set({ layerId: 1n, trackId: 2n, ids: new Set([5]) });
    setBoxSelect(false);
    expect(get(boxSelection)).toBeNull();
  });
});
