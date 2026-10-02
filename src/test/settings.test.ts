// @vitest-environment jsdom
import { beforeEach, describe, expect, it, vi } from "vitest";
import { get } from "svelte/store";

/**
 * The thresholds the track statistics use, as the operator sets them.
 *
 * Kept on the machine like the theme and the language: they describe how this
 * crew's GPS behaves, not the project, and a project handed to another HQ
 * should not change their statistics.
 */
async function freshModule() {
  vi.resetModules();
  return import("../lib/settings");
}

describe("motion settings", () => {
  beforeEach(() => {
    localStorage.clear();
  });

  it("starts from the owner's defaults: 25 m in two minutes, 5 m of climb", async () => {
    const { motionSettings } = await freshModule();
    expect(get(motionSettings)).toEqual({
      stopDistanceM: 25,
      stopWindowS: 120,
      climbThresholdM: 5,
    });
  });

  it("keeps a changed value across a restart", async () => {
    const first = await freshModule();
    first.setMotionSetting("stopDistanceM", 40);
    const second = await freshModule();
    expect(get(second.motionSettings).stopDistanceM).toBe(40);
  });

  it("ignores a value that is not a positive number", async () => {
    const { motionSettings, setMotionSetting } = await freshModule();
    for (const bad of [0, -5, Number.NaN, Number.POSITIVE_INFINITY, ""]) {
      setMotionSetting("climbThresholdM", bad as number);
    }
    expect(get(motionSettings).climbThresholdM).toBe(5);
  });

  it("falls back to the defaults when what is stored cannot be read", async () => {
    localStorage.setItem("ozi:motion-settings", "{not json");
    const { motionSettings } = await freshModule();
    expect(get(motionSettings).stopDistanceM).toBe(25);
  });

  it("keeps the good values from a store that has some bad ones", async () => {
    localStorage.setItem(
      "ozi:motion-settings",
      JSON.stringify({ stopDistanceM: 30, stopWindowS: -1, climbThresholdM: "x" }),
    );
    const { motionSettings } = await freshModule();
    expect(get(motionSettings)).toEqual({
      stopDistanceM: 30,
      stopWindowS: 120,
      climbThresholdM: 5,
    });
  });
});
