import { describe, expect, it } from "vitest";
import { readFileSync } from "fs";
import { join } from "path";

/**
 * Placing marks is a mode, not a single act.
 *
 * A headquarters cutting the next outing into tasks places ten marks in a row
 * — the base camp, the finds, the hazards, the meeting point — and the mode
 * switched itself off after every one of them, including after one that
 * failed. Ten marks cost ten trips back to the toolbar. It was written down as
 * a known defect of CJ-5 in July and was still there on 2026-09-23.
 *
 * OziExplorer keeps its waypoint tool selected until another is picked, which
 * is what anyone who has used it expects. Leaving the mode here is Esc, the
 * mode chip, or starting to draw — all three clear it, and panning does not,
 * because a drag is not a click.
 *
 * This is the pattern, checked: the handler that adds a mark must not turn the
 * mode off, and the three ways out must stay. It reads the source, so it does
 * not prove that a click places a mark — delete the handler's body and it
 * still passes, as a reviewer showed on 2026-09-23. What it proves is that the
 * defect cannot come back the way it arrived, which is the job these guards
 * do; the behaviour itself is walked on the stand.
 */
const mapView = readFileSync(
  join(__dirname, "../components/MapView.svelte"),
  "utf-8",
);

function handlerBody(source: string, name: string): string {
  const start = source.indexOf(`async function ${name}(`);
  expect(start, `${name} is not in MapView`).toBeGreaterThan(-1);
  // To the next top-level `function` declaration.
  const rest = source.slice(start + 10);
  const end = rest.indexOf("\n  function ");
  return rest.slice(0, end === -1 ? undefined : end);
}

describe("the add-waypoint mode", () => {
  it("stays armed after a mark is placed", () => {
    const body = handlerBody(mapView, "handleMapClickForWaypoint");
    expect(
      body.includes("addWaypointMode.set(false)"),
      "placing a mark must not leave the mode: a headquarters places ten in a " +
        "row, and switching off after each one costs ten trips to the toolbar.",
    ).toBe(false);
  });

  it("is still left by Esc, by the mode chip and by starting to draw", () => {
    // A sticky mode with no way out is worse than the defect it replaced, so
    // each exit is checked where it lives rather than counted in one file.
    const waypointsTab = readFileSync(
      join(__dirname, "../components/library/WaypointsTab.svelte"),
      "utf-8",
    );
    const modes = readFileSync(
      join(__dirname, "../lib/actions/modes.ts"),
      "utf-8",
    );

    const missing: string[] = [];
    if (!/Escape[\s\S]{0,400}addWaypointMode\.set\(false\)/.test(mapView)) {
      missing.push("Esc no longer leaves the mode (MapView)");
    }
    if (!waypointsTab.includes("addWaypointMode.update((v) => !v)")) {
      missing.push("the mode chip no longer toggles it off (WaypointsTab)");
    }
    // Entering any other mode leaves this one, in the one place that knows
    // what the modes are.
    if (!modes.includes("addWaypointMode.set(false)")) {
      missing.push("entering another mode no longer leaves it (actions/modes)");
    }

    expect(
      missing,
      "the ways out of the mode are Esc, the mode chip and starting a draw.",
    ).toEqual([]);
  });
});
