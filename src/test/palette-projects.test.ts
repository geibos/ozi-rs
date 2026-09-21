import { describe, expect, it } from "vitest";
import { paletteProjects } from "../lib/palette-projects";

const catalogue = Array.from({ length: 13_000 }, (_, i) => ({
  slug: `2026-09-${String((i % 28) + 1).padStart(2, "0")}_place${i}`,
  name: `2026-09-${String((i % 28) + 1).padStart(2, "0")} place${i}`,
}));

describe("paletteProjects", () => {
  it("hands the palette a screenful, never the whole catalogue", () => {
    expect(paletteProjects(catalogue, "", 40)).toHaveLength(40);
    expect(paletteProjects(catalogue, "place", 40)).toHaveLength(40);
  });

  it("matches on the name or the slug, ignoring case", () => {
    const rows = [
      { slug: "2026-09-21_lavrovo", name: "2026-09-21 lavrovo" },
      { slug: "2026-09-20_kolpino", name: "2026-09-20 kolpino" },
    ];
    expect(paletteProjects(rows, "LAVROVO", 40)).toEqual([rows[0]]);
    expect(paletteProjects(rows, "09-20", 40)).toEqual([rows[1]]);
  });

  it("stops scanning once the limit is reached", () => {
    // 13k rows with a query that matches everything: a full filter would walk
    // the whole array on every keystroke.
    const result = paletteProjects(catalogue, "2026", 5);
    expect(result).toHaveLength(5);
    expect(result[0]).toBe(catalogue[0]);
  });

  it("returns nothing when nothing matches", () => {
    expect(paletteProjects(catalogue, "нет такого", 40)).toEqual([]);
  });
});
