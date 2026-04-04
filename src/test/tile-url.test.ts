import { describe, it, expect } from "vitest";
import { parseSqliteTileUrl } from "../lib/maplibre/tile-url";

describe("parseSqliteTileUrl", () => {
  it("parses a normal absolute path", () => {
    const result = parseSqliteTileUrl(
      "sqlite:///Users/user/Documents/LizaAlert Maps/project/map.sqlitedb/15/10/20/30"
    );
    expect(result).toEqual({
      filePath: "/Users/user/Documents/LizaAlert Maps/project/map.sqlitedb",
      baseZoom: 15,
      z: 10,
      x: 20,
      y: 30,
    });
  });

  it("parses path with spaces", () => {
    const result = parseSqliteTileUrl(
      "sqlite:///path/to/My Maps/bundle.sqlitedb/14/5/6/7"
    );
    expect(result?.filePath).toBe("/path/to/My Maps/bundle.sqlitedb");
    expect(result?.baseZoom).toBe(14);
  });

  it("returns null for a missing base_zoom component (old 4-part format)", () => {
    // Old URL without base_zoom — must not silently misparse
    const result = parseSqliteTileUrl(
      "sqlite:///path/to/map.sqlitedb/10/20/30"
    );
    // Regex requires 5 numeric segments — this has only 3 → null
    expect(result).toBeNull();
  });

  it("returns null for a completely malformed URL", () => {
    expect(parseSqliteTileUrl("sqlite://")).toBeNull();
    expect(parseSqliteTileUrl("not-a-url")).toBeNull();
  });

  it("base_zoom, z, x, y are integers not strings", () => {
    const result = parseSqliteTileUrl("sqlite:///map.sqlitedb/15/12/345/678");
    expect(typeof result?.baseZoom).toBe("number");
    expect(typeof result?.z).toBe("number");
  });
});
