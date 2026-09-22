import { describe, expect, it } from "vitest";
import { readFileSync } from "fs";
import { join } from "path";

const tracksTabSource = readFileSync(
  join(__dirname, "../components/library/TracksTab.svelte"),
  "utf-8",
);
const mapViewSource = readFileSync(
  join(__dirname, "../components/MapView.svelte"),
  "utf-8",
);
const paletteSource = readFileSync(
  join(__dirname, "../components/CommandPalette.svelte"),
  "utf-8",
);
const loaderSource = readFileSync(
  join(__dirname, "../components/BundleLoader.svelte"),
  "utf-8",
);
const projectActionsSource = readFileSync(
  join(__dirname, "../lib/actions/project.ts"),
  "utf-8",
);

/**
 * Regression guard for the owner's hands-on finding "imported tracks don't
 * show on the map": the import succeeds and the tracks render, but the camera
 * stays on the active raster, so tracks in another region are off-screen.
 *
 * Opening a saved project had the same hole and nobody had noticed, because
 * the only path that framed anything was the import. Both now ask, and the
 * request covers waypoints too — a project whose content is a headquarters
 * and a drop-off point has no track geometry to fit.
 *
 * The bbox itself is tested in `map-bounds.test.ts`; `MapView` needs a
 * MapLibre instance to mount, so what is pinned here is the wiring.
 */
describe("framing the data on the map", () => {
  it("a file import asks for the frame when at least one track arrived", () => {
    expect(tracksTabSource).toContain(
      "if (imported > 0) requestAllDataFocus()",
    );
  });

  it("a folder import asks for it once the backend summary resolves", () => {
    expect(tracksTabSource).toMatch(
      /importTracksDirectory[\s\S]{0,120}requestAllDataFocus\(\)/,
    );
  });

  it("opening a project asks for it, wherever it was asked for", () => {
    // The dialog, the remembering and the framing live in one action, which
    // the palette, its recents list and the launch screen all call. Its own
    // behaviour is covered in `open-project-action.test.ts`; what matters
    // here is that nothing loads a project around it.
    expect(projectActionsSource).toMatch(
      /await loadProjectFile\(path\);[\s\S]{0,240}requestAllDataFocus\(\)/,
    );
    expect(paletteSource).not.toContain("loadProjectFile(");
    expect(loaderSource).not.toContain("loadProjectFile(");
  });

  it("MapView consumes the request and frames tracks and waypoints together", () => {
    expect(mapViewSource).toContain('request.kind === "all-data"');
    expect(mapViewSource).toContain("focusAllData");
    expect(mapViewSource).toContain("map.fitBounds(toLngLatBounds(bounds)");
    // The waypoint positions come from the layers, not from the markers that
    // happen to be drawn: those are placed by an asynchronous reconciler, so
    // reading them framed whatever had rendered so far. What `focusPositions`
    // does with the two sources is covered in `focus-positions.test.ts`.
    expect(mapViewSource).toMatch(
      /focusPositions\([\s\S]{0,200}geojson\.features/,
    );
    expect(mapViewSource).toMatch(
      /getWaypoints\(layerId\)[\s\S]{0,400}focusPositions/,
    );
  });

  it("a single point is centred rather than fitted to its own dot", () => {
    expect(mapViewSource).toContain("isDegenerate(bounds)");
    expect(mapViewSource).toContain("centreOf(bounds)");
  });
});
