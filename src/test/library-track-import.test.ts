import { describe, expect, it } from "vitest";
import { readFileSync } from "fs";
import { join } from "path";

/**
 * Structural regression guard for `fix-library-track-import`.
 *
 * The original `Sidebar.svelte` hosted the `Import GPX`, `Import PLT`,
 * `Create Track`, and `Add Waypoint` affordances. The
 * `redesign-library-sidebar` change deleted that component without
 * re-surfacing the entry points anywhere in `LibraryRail`, breaking the
 * core workflow. These assertions read the source of `TracksTab.svelte`
 * and `WaypointsTab.svelte` directly and verify the API + store symbols
 * still appear in the file. The same shape would have caught the
 * original regression.
 *
 * Behavioural verification (file dialog → IPC → track on the map) is
 * covered by manual desktop QA per `docs/agent-verification.md`;
 * Playwright is not acceptable evidence per ADR-0024.
 */

const tracksSource = readFileSync(
  join(__dirname, "../components/library/TracksTab.svelte"),
  "utf-8",
);

const waypointsSource = readFileSync(
  join(__dirname, "../components/library/WaypointsTab.svelte"),
  "utf-8",
);

describe("Library Tracks tab — import / create-track affordances", () => {
  it("calls importGpx via a handler", () => {
    expect(tracksSource).toContain("importGpx");
  });

  it("calls importPlt via a handler", () => {
    expect(tracksSource).toContain("importPlt");
  });

  it("calls createEmptyTrack to start drawing mode", () => {
    expect(tracksSource).toContain("createEmptyTrack");
  });

  it("references drawingModeActive for the drawing toggle", () => {
    expect(tracksSource).toContain("drawingModeActive");
  });

  it("references drawingPointCount for the Done (N points) label", () => {
    expect(tracksSource).toContain("drawingPointCount");
  });

  it("renders the GPX import button", () => {
    expect(tracksSource).toContain("library-import-gpx");
  });

  it("renders the PLT import button", () => {
    expect(tracksSource).toContain("library-import-plt");
  });

  it("renders the Create Track button", () => {
    expect(tracksSource).toContain("library-create-track");
  });
});

describe("Library Waypoints tab — add-waypoint affordance", () => {
  it("references addWaypointMode for the toggle", () => {
    expect(waypointsSource).toContain("addWaypointMode");
  });

  it("renders the Add Waypoint button", () => {
    expect(waypointsSource).toContain("library-add-waypoint");
  });
});
