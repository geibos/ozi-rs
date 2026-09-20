import { describe, expect, it } from "vitest";
import { readFileSync } from "fs";
import { join } from "path";

const mapViewSource = readFileSync(
  join(__dirname, "../components/MapView.svelte"),
  "utf-8",
);

/**
 * Regression guard for the owner's finding "the map JPG overlays the tracks".
 * A raster `map-tiles` layer must never cover the track lines/labels. The
 * beforeId insertion alone proved unreliable across style changes, so the
 * component re-asserts the invariant by raising the track layers to the top
 * after adding the raster and after every track refresh.
 */
describe("track layers stay above the raster", () => {
  it("defines a raiseTrackLayers helper that moves both track layers to top", () => {
    expect(mapViewSource).toContain("function raiseTrackLayers()");
    expect(mapViewSource).toContain('"tracks-lines", "tracks-labels"');
    expect(mapViewSource).toContain("map.moveLayer(id)");
  });

  it("raises track layers right after adding the active raster", () => {
    // raiseTrackLayers() must appear after the map-tiles addLayer, before
    // the subsequent fitBounds/flyTo camera move.
    const addLayerIdx = mapViewSource.indexOf('id: "map-tiles"');
    const raiseIdx = mapViewSource.indexOf("raiseTrackLayers()", addLayerIdx);
    const fitIdx = mapViewSource.indexOf("if (fitBoundsTarget)", addLayerIdx);
    expect(addLayerIdx).toBeGreaterThan(-1);
    expect(raiseIdx).toBeGreaterThan(addLayerIdx);
    expect(raiseIdx).toBeLessThan(fitIdx);
  });

  it("raises track layers after refreshing track geometry", () => {
    // Both the on-load refresh and the slice effect re-assert z-order.
    const count = (mapViewSource.match(/raiseTrackLayers\(\)/g) || []).length;
    expect(count).toBeGreaterThanOrEqual(3);
  });
});
