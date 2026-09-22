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

/**
 * Twelve crews' routes drawn in twelve colours still leave the question "which
 * line is ЛИСА15" — the map carries no names, because on-map labels need SDF
 * glyphs nobody has bundled yet (see `docs/backlog.md`).
 *
 * Selecting a row answers it without any assets: the chosen track gets a
 * casing under its own colour, so it stands out from the other eleven while
 * keeping the colour that identifies it. Stepping down the list lights each
 * route in turn.
 */
describe("highlighting the selected track", () => {
  function fakeMapWithFilters() {
    const layers: string[] = [];
    const filters: Record<string, unknown> = {};
    return {
      layers,
      filters,
      addSource: vi.fn(),
      getGlyphs: () => undefined,
      addLayer: (layer: { id: string }) => layers.push(layer.id),
      getLayer: (id: string) => (layers.includes(id) ? { id } : undefined),
      setFilter: (id: string, filter: unknown) => {
        filters[id] = filter;
      },
      // eslint-disable-next-line @typescript-eslint/no-explicit-any
    } as any;
  }

  it("adds the highlight beneath the coloured line, so the colour stays true", async () => {
    const { TRACKS_LAYER_SELECTED, initTracksLayer: init } = await import(
      "../lib/maplibre/tracks-layer"
    );
    const map = fakeMapWithFilters();
    init(map);
    expect(map.layers).toContain(TRACKS_LAYER_SELECTED);
    expect(map.layers.indexOf(TRACKS_LAYER_SELECTED)).toBeLessThan(
      map.layers.indexOf("tracks-lines"),
    );
  });

  it("matches exactly one track, by layer and by track", async () => {
    const { TRACKS_LAYER_SELECTED, highlightTrack, initTracksLayer: init } =
      await import("../lib/maplibre/tracks-layer");
    const map = fakeMapWithFilters();
    init(map);

    highlightTrack(map, { layerId: 2n, trackId: 7n });
    expect(map.filters[TRACKS_LAYER_SELECTED]).toEqual([
      "all",
      ["==", ["get", "layer_id"], 2],
      ["==", ["get", "track_id"], 7],
    ]);
  });

  it("matches nothing when nothing is selected", async () => {
    const { TRACKS_LAYER_SELECTED, highlightTrack, initTracksLayer: init } =
      await import("../lib/maplibre/tracks-layer");
    const map = fakeMapWithFilters();
    init(map);

    highlightTrack(map, null);
    // A filter that cannot match, rather than removing the layer: the layer's
    // position in the stack is what keeps it under the coloured line.
    expect(map.filters[TRACKS_LAYER_SELECTED]).toEqual([
      "==",
      ["get", "track_id"],
      -1,
    ]);
  });

  it("does nothing when the layer is not there", async () => {
    const { highlightTrack } = await import("../lib/maplibre/tracks-layer");
    const map = fakeMapWithFilters();
    // No init: a style reload can leave the map without the layer for a
    // moment, and a highlight arriving then must not throw at the operator.
    expect(() => highlightTrack(map, { layerId: 1n, trackId: 1n })).not.toThrow();
  });
});
