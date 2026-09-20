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

/**
 * Regression guard for the owner's hands-on finding "imported tracks don't
 * show on the map". Import succeeds and the tracks render — but the camera
 * stays on the active raster, so tracks at a different region are off-screen.
 * Both import paths must fit the camera to all tracks, and MapView must
 * consume the "all-tracks" focus request.
 */
describe("auto-fit camera after import", () => {
  it("file import fits all tracks when at least one imported", () => {
    expect(tracksTabSource).toContain("requestAllTracksFocus");
    expect(tracksTabSource).toContain(
      "if (imported > 0) requestAllTracksFocus()",
    );
  });

  it("folder import fits all tracks on success", () => {
    // The folder handler calls it right after the backend summary resolves.
    expect(tracksTabSource).toMatch(
      /importTracksDirectory[\s\S]{0,120}requestAllTracksFocus\(\)/,
    );
  });

  it("MapView consumes the all-tracks focus request via fitBounds", () => {
    expect(mapViewSource).toContain('request.kind === "all-tracks"');
    expect(mapViewSource).toContain("focusAllTracks");
    expect(mapViewSource).toContain("map.fitBounds");
  });
});
