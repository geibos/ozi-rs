import { describe, expect, it } from "vitest";
import { createLatestRun } from "../lib/latest-run";

describe("only the newest run may write", () => {
  it("keeps the newest token current and retires the ones before it", () => {
    const runs = createLatestRun();

    const first = runs.begin();
    expect(runs.isCurrent(first)).toBe(true);

    const second = runs.begin();
    expect(runs.isCurrent(second)).toBe(true);
    expect(runs.isCurrent(first)).toBe(false);
  });

  it("does not make a finished run current again", () => {
    const runs = createLatestRun();
    const first = runs.begin();
    runs.begin();

    // The overtaken run answering later must still find itself stale.
    expect(runs.isCurrent(first)).toBe(false);
  });

  it("gives each caller its own counter", () => {
    const tracks = createLatestRun();
    const waypoints = createLatestRun();

    const trackRun = tracks.begin();
    waypoints.begin();
    waypoints.begin();

    expect(tracks.isCurrent(trackRun)).toBe(true);
  });

  it("treats a token nobody issued as stale", () => {
    const runs = createLatestRun();
    runs.begin();
    expect(runs.isCurrent(0)).toBe(false);
    expect(runs.isCurrent(99)).toBe(false);
  });
});
