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

  it("does not match everything when a map name is empty", () => {
    expect(
      readyMapName("anything", [
        { name: "", base_zoom: 0, downloaded: false, size_bytes: null },
      ]),
    ).toBeNull();
  });
});
