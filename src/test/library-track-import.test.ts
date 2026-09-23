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
  it("calls importGpx via the unified import handler", () => {
    expect(tracksSource).toContain("importGpx");
  });

  it("calls importPlt via the unified import handler", () => {
    expect(tracksSource).toContain("importPlt");
  });

  it("starts drawing mode through the shared action", () => {
    // The routine moved to `$lib/actions/modes` on 2026-09-23 so the Tracks
    // tab, the mode chips above the map and the command palette all enter
    // drawing the same way. The tab asks for the mode; the action creates the
    // track.
    expect(tracksSource).toContain('setInteractionMode("draw")');
    const modes = readFileSync(
      join(__dirname, "../lib/actions/modes.ts"),
      "utf-8",
    );
    expect(modes).toContain("createEmptyTrack");
  });

  it("references drawingModeActive for the drawing toggle", () => {
    expect(tracksSource).toContain("drawingModeActive");
  });

  it("references drawingPointCount for the Done (N points) label", () => {
    expect(tracksSource).toContain("drawingPointCount");
  });

  it("renders ONE unified Import button instead of the twin GPX/PLT buttons", () => {
    expect(tracksSource).toContain("library-import-tracks");
    expect(tracksSource).not.toContain("library-import-gpx");
    expect(tracksSource).not.toContain("library-import-plt");
  });

  it("offers every importable format in the unified file filter", () => {
    // The list itself lives in `$lib/actions/import-paths`, so the picker and
    // the window's drop target cannot offer different things. What the list
    // contains, and where each extension goes, is covered by behaviour in
    // `import-paths.test.ts`.
    expect(tracksSource).toContain("IMPORTABLE_EXTENSIONS");
  });

  it("allows multi-select in the unified import dialog", () => {
    expect(tracksSource).toContain("multiple: true");
  });

  it("routes the picked files through the shared import dispatch", () => {
    // Which reader each extension reaches is a behaviour, and it is tested as
    // one in `import-paths.test.ts`. Asserting that this component's source
    // spells `endsWith(".plt")` checked the spelling of a branch, and went on
    // passing after the branch moved out of the component.
    expect(tracksSource).toContain("importPaths(paths)");
  });

  it("renders the Import folder button wired to the recursive backend import", () => {
    expect(tracksSource).toContain("library-import-folder");
    expect(tracksSource).toContain("importTracksDirectory");
    expect(tracksSource).toContain("directory: true");
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
