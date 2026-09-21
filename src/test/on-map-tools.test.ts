// @vitest-environment jsdom
import { beforeEach, describe, expect, it } from "vitest";
import { get } from "svelte/store";
import {
  measuredPoints,
  measuringActive,
  ringActive,
  ringCentre,
  ringRadiusKm,
  setMeasuring,
  setRing,
} from "../lib/stores";

/**
 * The tape and the ring take the same click, so they cannot both be listening:
 * a click meant for the ring's centre must not also extend the tape. Switching
 * one on switches the other off, and both discard what they were holding —
 * these are scratch tools, and a stale point left behind would be measured
 * into the next measurement.
 */
beforeEach(() => {
  setMeasuring(false);
  setRing(false);
});

describe("the two on-map tools", () => {
  it("does not leave the other one listening", () => {
    setMeasuring(true);
    expect(get(measuringActive)).toBe(true);
    expect(get(ringActive)).toBe(false);

    setRing(true);
    expect(get(ringActive)).toBe(true);
    expect(get(measuringActive)).toBe(false);
  });

  it("discards the tape's points when the ring takes over", () => {
    setMeasuring(true);
    measuredPoints.set([
      { lat: 1, lon: 1 },
      { lat: 2, lon: 2 },
    ]);

    setRing(true);
    expect(get(measuredPoints)).toEqual([]);
  });

  it("discards the ring when the tape takes over", () => {
    setRing(true);
    ringCentre.set({ lat: 1, lon: 1 });
    ringRadiusKm.set(5);

    setMeasuring(true);
    expect(get(ringCentre)).toBeNull();
    expect(get(ringRadiusKm)).toBe(0);
  });

  it("clears what it was holding when switched off", () => {
    setMeasuring(true);
    measuredPoints.set([{ lat: 1, lon: 1 }]);
    setMeasuring(false);
    expect(get(measuredPoints)).toEqual([]);

    setRing(true);
    ringCentre.set({ lat: 1, lon: 1 });
    setRing(false);
    expect(get(ringCentre)).toBeNull();
  });
});
