import { describe, expect, it } from "vitest";
import { readyMapName } from "../lib/ready-maps";
import { appStateFixture } from "./fixtures";

const maps = appStateFixture.current_project?.maps ?? [];

describe("readyMapName", () => {
  it("recognises a map that landed, by its path inside the bundle", () => {
    const topo = maps[0].name;
    expect(readyMapName(`8-Android&iOS/${topo}`, maps)).toBe(topo);
  });

  it("ignores the rest of the bundle", () => {
    // Print maps, coordinates files and archives land too; none of them is
    // something the crew can open.
    expect(readyMapName("9-Map_4_print/sheet-1.pdf", maps)).toBeNull();
    expect(readyMapName("2-Coordinates.txt", maps)).toBeNull();
  });

  it("does not hand a landed file to a map whose name it merely ends with", () => {
    // The Rust side had this same defect, found by external review on
    // 2026-09-22: `bigmap.ozf2` ends with `map.ozf2`, so matching the whole
    // path with `endsWith` announced a map the crew had not downloaded. The
    // frontend copy was missed because the review read only the Rust.
    const two = [
      { name: "map.ozf2", base_zoom: 0, downloaded: false, size_bytes: null },
      {
        name: "bigmap.ozf2",
        base_zoom: 0,
        downloaded: false,
        size_bytes: null,
      },
    ];
    expect(readyMapName("8-Android&iOS/bigmap.ozf2", two)).toBe("bigmap.ozf2");
    expect(readyMapName("8-Android&iOS/map.ozf2", two)).toBe("map.ozf2");
  });

  it("recognises a map that landed in the bundle root", () => {
    const one = [
      { name: "map.ozf2", base_zoom: 0, downloaded: false, size_bytes: null },
    ];
    expect(readyMapName("map.ozf2", one)).toBe("map.ozf2");
  });

  it("does not match everything when a map name is empty", () => {
    expect(
      readyMapName("anything", [
        { name: "", base_zoom: 0, downloaded: false, size_bytes: null },
      ]),
    ).toBeNull();
  });
});
