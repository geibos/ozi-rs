import { describe, expect, it } from "vitest";

import {
  hex,
  isReservedTrackColour,
  latinEs,
  TRACK_COLOURS,
  waypointNameProblems,
} from "../lib/standard-colours";

describe("the standard's colours", () => {
  it("offer no black for a track — it is for tasks (п. 23)", () => {
    expect(TRACK_COLOURS.some((c) => isReservedTrackColour(c.rgba))).toBe(
      false,
    );
    expect(isReservedTrackColour([0, 0, 0, 255])).toBe(true);
    expect(isReservedTrackColour([255, 0, 0, 255])).toBe(false);
  });

  it("write a swatch as #rrggbb", () => {
    expect(hex([0, 128, 128, 255])).toBe("#008080");
  });
});

describe("a waypoint name by п. 26", () => {
  it("wants the latin C for the Russian С", () => {
    expect(waypointNameProblems("Сарай").russianEs).toBe(true);
    expect(latinEs("Сарай у ССТ")).toBe("Cарай у CC\u0422");
    expect(waypointNameProblems("сарай").russianEs).toBe(false);
  });

  it("allows letters, digits, space, underscore and dash, nothing else", () => {
    expect(waypointNameProblems("Заброс 1_Б-2").badCharacters).toEqual([]);
    expect(waypointNameProblems("Камера (90°)").badCharacters).toEqual([
      "(",
      "°",
      ")",
    ]);
  });
});
