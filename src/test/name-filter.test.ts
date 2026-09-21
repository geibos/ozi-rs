import { describe, expect, it } from "vitest";
import { filterByName } from "../lib/name-filter";
import { filterTrackFeatures } from "../lib/track-features";

const tracks = [
  { name: "20260709-ЛИСА15" },
  { name: "20260709-lisa2" },
  { name: "20260710-ШТАБ" },
];

describe("filterByName", () => {
  it("matches a substring anywhere in the name, ignoring case", () => {
    expect(filterByName(tracks, "ЛИСА", (t) => t.name)).toEqual([
      { name: "20260709-ЛИСА15" },
    ]);
    expect(filterByName(tracks, "lisa", (t) => t.name)).toEqual([
      { name: "20260709-lisa2" },
    ]);
    expect(filterByName(tracks, "0709", (t) => t.name)).toHaveLength(2);
  });

  it("returns every row for an empty or blank query", () => {
    expect(filterByName(tracks, "", (t) => t.name)).toHaveLength(3);
    expect(filterByName(tracks, "   ", (t) => t.name)).toHaveLength(3);
  });

  it("reads the name through the accessor, not a fixed field", () => {
    const rows = [{ wp: { name: "Задача 1" } }, { wp: { name: "Штаб" } }];
    expect(filterByName(rows, "задача", (r) => r.wp.name)).toEqual([
      { wp: { name: "Задача 1" } },
    ]);
  });
});

describe("both Library tabs share one matching rule", () => {
  it("filterTrackFeatures agrees with filterByName", () => {
    for (const query of ["лиса", "ШТАБ", "0709", "", "нет"]) {
      expect(filterTrackFeatures(tracks, query)).toEqual(
        filterByName(tracks, query, (t) => t.name),
      );
    }
  });
});
