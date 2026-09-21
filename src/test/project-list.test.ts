import { describe, expect, it } from "vitest";
import { filterProjects } from "../lib/project-list";

const catalogue = [
  { slug: "2026-09-21_lavrovo", name: "2026 09 21 lavrovo", cached: true },
  { slug: "2026-09-20_kolpino", name: "2026 09 20 kolpino", cached: false },
  { slug: "2026-08-14_gatchina", name: "2026 08 14 gatchina" },
];

describe("filterProjects", () => {
  it("matches the slug as well as the name", () => {
    // The names are transliterations of the slug, so a date query has to
    // reach the slug's punctuation too.
    expect(filterProjects(catalogue, "2026-09-21", false)).toEqual([
      catalogue[0],
    ]);
    expect(filterProjects(catalogue, "kolpino", false)).toEqual([catalogue[1]]);
  });

  it("keeps only downloaded bundles when asked", () => {
    expect(filterProjects(catalogue, "", true)).toEqual([catalogue[0]]);
  });

  it("treats a missing cached flag as not downloaded", () => {
    // Entries restored from an older localStorage catalogue have no flag.
    expect(filterProjects(catalogue, "gatchina", true)).toEqual([]);
  });

  it("combines the query with the downloaded filter", () => {
    expect(filterProjects(catalogue, "2026", true)).toEqual([catalogue[0]]);
    expect(filterProjects(catalogue, "2026", false)).toHaveLength(3);
  });

  it("returns everything for a blank query", () => {
    expect(filterProjects(catalogue, "   ", false)).toHaveLength(3);
  });
});
