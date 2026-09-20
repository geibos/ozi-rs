import { describe, expect, it, vi } from "vitest";
import { initTracksLayer } from "../lib/maplibre/tracks-layer";

/**
 * Regression guard for "tracks never render on the map" (owner, 2026-07-16).
 * The track name labels are a symbol layer; MapLibre's addLayer throws
 * "text-field requires a style glyphs property" when the style has no glyphs
 * (none are bundled yet). That throw used to abort initTracksLayer, so the
 * track LINE layer was never usable. The line must be added unconditionally;
 * the label layer only when glyphs exist, and its failure must not propagate.
 */
function fakeMap(opts: { glyphs?: string; labelThrows?: boolean }) {
  const added: string[] = [];
  return {
    added,
    addSource: vi.fn(),
    getGlyphs: () => opts.glyphs,
    addLayer: (layer: { id: string; type: string }) => {
      if (layer.type === "symbol" && opts.labelThrows) {
        throw new Error("use of text-field requires a style glyphs property");
      }
      added.push(layer.id);
    },
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
  } as any;
}

describe("initTracksLayer", () => {
  it("adds the line layer even when the style has no glyphs", () => {
    const map = fakeMap({ glyphs: undefined });
    initTracksLayer(map);
    expect(map.added).toContain("tracks-lines");
    expect(map.added).not.toContain("tracks-labels");
  });

  it("adds the label layer when glyphs are present", () => {
    const map = fakeMap({ glyphs: "https://example/{fontstack}/{range}.pbf" });
    initTracksLayer(map);
    expect(map.added).toContain("tracks-lines");
    expect(map.added).toContain("tracks-labels");
  });

  it("never lets a label-layer failure abort line setup", () => {
    const map = fakeMap({
      glyphs: "https://example/{fontstack}/{range}.pbf",
      labelThrows: true,
    });
    expect(() => initTracksLayer(map)).not.toThrow();
    expect(map.added).toContain("tracks-lines");
  });
});
