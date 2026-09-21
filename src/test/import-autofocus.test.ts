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
    expect(tracksTabSource).toContain("if (imported > 0) requestAllDataFocus()");
  });

  it("a folder import asks for it once the backend summary resolves", () => {
    expect(tracksTabSource).toMatch(
      /importTracksDirectory[\s\S]{0,120}requestAllDataFocus\(\)/,
    );
  });

  it("opening a project asks for it, from the dialog and from the recents", () => {
    // Two call sites, because the recents path drops a stale entry and the
    // dialog path does not.
    expect(
      paletteSource.match(/requestAllDataFocus\(\);/g) ?? [],
    ).toHaveLength(2);
    expect(paletteSource).toMatch(
      /await loadProjectFile\(path as string\);[\s\S]{0,300}requestAllDataFocus\(\)/,
    );
    expect(paletteSource).toMatch(
      /await loadProjectFile\(path\);[\s\S]{0,120}requestAllDataFocus\(\)/,
    );
  });

  it("MapView consumes the request and frames tracks and waypoints together", () => {
    expect(mapViewSource).toContain('request.kind === "all-data"');
    expect(mapViewSource).toContain("focusAllData");
    expect(mapViewSource).toContain("map.fitBounds(toLngLatBounds(bounds)");
    // The waypoint positions come from the markers already on the map.
    expect(mapViewSource).toMatch(
      /for \(const marker of waypointMarkers\.values\(\)\)[\s\S]{0,160}points\.push/,
    );
  });

  it("a single point is centred rather than fitted to its own dot", () => {
    expect(mapViewSource).toContain("isDegenerate(bounds)");
    expect(mapViewSource).toContain("centreOf(bounds)");
  });
});
