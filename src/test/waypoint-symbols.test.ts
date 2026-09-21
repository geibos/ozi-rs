import { describe, expect, it } from "vitest";
import {
  DEFAULT_WAYPOINT_GLYPH,
  WAYPOINT_SYMBOLS,
  waypointGlyph,
} from "../lib/waypoint-symbols";

/**
 * The map used to draw every waypoint as the same yellow dot: the symbol was
 * stored, listed and exported, and the one place it did not appear was the
 * place a crew looks at.
 */
describe("the glyph a waypoint shows", () => {
  it("is the one its symbol names", () => {
    expect(waypointGlyph("danger")).toBe("⚠️");
    expect(waypointGlyph("flag")).toBe("🏁");
  });

  it("is the default pin when there is no symbol", () => {
    expect(waypointGlyph(null)).toBe(DEFAULT_WAYPOINT_GLYPH);
    expect(waypointGlyph(undefined)).toBe(DEFAULT_WAYPOINT_GLYPH);
    expect(waypointGlyph("")).toBe(DEFAULT_WAYPOINT_GLYPH);
  });

  /**
   * A GPX from another tool carries whatever symbol that tool uses. The
   * waypoint is still on the map and must still be findable, so an unknown
   * symbol falls back rather than vanishing.
   */
  it("falls back for a symbol this build does not know", () => {
    expect(waypointGlyph("geocache-traditional")).toBe(DEFAULT_WAYPOINT_GLYPH);
  });

  it("offers each symbol once, with a glyph and a label key", () => {
    const values = WAYPOINT_SYMBOLS.map((s) => s.value);
    expect(new Set(values).size).toBe(values.length);
    for (const symbol of WAYPOINT_SYMBOLS) {
      expect(symbol.emoji, symbol.value).not.toBe("");
      expect(symbol.labelKey, symbol.value).toMatch(/^symbol\./);
    }
  });
});
