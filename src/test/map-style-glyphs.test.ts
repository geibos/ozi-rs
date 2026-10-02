import { describe, expect, it } from "vitest";
import { readFileSync } from "fs";
import { join } from "path";

const mapViewSource = readFileSync(
  join(__dirname, "../components/MapView.svelte"),
  "utf-8",
);

/**
 * Regression guard for the owner's "треки не отображаются на карте" finding.
 *
 * Root cause: the map style declared a REMOTE glyphs URL
 * (`https://demotiles.maplibre.org/font/...`). The `tracks` GeoJSON source
 * feeds both the `tracks-lines` (line) layer and the `tracks-labels` (symbol)
 * layer. A MapLibre source tile only finishes parsing once EVERY layer's
 * dependencies resolve — and the symbol layer's glyph fetch hangs in the
 * offline desktop webview. So the shared tile never completed and the LINE
 * never rendered either (`querySourceFeatures("tracks") === 0`), even though
 * the source held valid geometry. Removing the remote glyphs URL makes
 * `map.getGlyphs()` falsy, so `initTracksLayer` skips the symbol layer and the
 * line tiles cleanly.
 *
 * The app is offline-first. Since 2026-10-01 it ships its own SDF glyphs and
 * the style points at them through the `glyphs://` protocol, which answers
 * every range — an unshipped one with an empty set — so a glyph request can
 * no longer hold a tile up. A remote URL would bring the July failure back.
 */
describe("map style glyphs", () => {
  it("does not declare a remote glyphs URL that would poison track tiling", () => {
    // No remote font endpoint of any kind in the map style.
    expect(mapViewSource).not.toMatch(/glyphs:\s*["']https?:\/\//);
    expect(mapViewSource).not.toContain("demotiles.maplibre.org");
  });

  it("points glyphs at the application's own protocol", () => {
    expect(mapViewSource).toMatch(/glyphs:\s*GLYPHS_URL/);
  });
});
