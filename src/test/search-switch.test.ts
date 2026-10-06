import { describe, expect, it } from "vitest";
import { holdsWork, opensAnotherSearch } from "../lib/search-switch";

/**
 * A project is one search (owner, 2026-10-01). Opening a map of another search
 * over a project that holds the previous one's work starts a new project; the
 * two questions are whether the map is another search's, and whether there is
 * work to make way.
 */
describe("opensAnotherSearch", () => {
  it("says yes for a map of a search other than the one on screen", () => {
    expect(opensAnotherSearch("2026-09-28 Лаврово", "2026-10-01 Кириши")).toBe(
      true,
    );
  });

  it("says no for another sheet of the same search", () => {
    expect(opensAnotherSearch("2026-09-28 Лаврово", "2026-09-28 Лаврово")).toBe(
      false,
    );
  });

  it("says no when no map is open yet — the first map of the day", () => {
    expect(opensAnotherSearch(null, "2026-10-01 Кириши")).toBe(false);
    expect(opensAnotherSearch(undefined, "2026-10-01 Кириши")).toBe(false);
  });

  it("says no when no search is chosen", () => {
    expect(opensAnotherSearch("2026-09-28 Лаврово", null)).toBe(false);
  });
});

describe("holdsWork", () => {
  const empty = { tracks: [], project_dirty: false, project_path: null };

  it("finds nothing in an empty, never-saved project", () => {
    expect(holdsWork(empty)).toBe(false);
    expect(holdsWork(null)).toBe(false);
  });

  it("counts a track", () => {
    expect(holdsWork({ ...empty, tracks: [{} as never] as never[] })).toBe(
      true,
    );
  });

  it("counts unsaved changes, which is where a mark-only project shows", () => {
    expect(holdsWork({ ...empty, project_dirty: true })).toBe(true);
  });

  it("counts a project with a file on disk, whatever is in it", () => {
    expect(
      holdsWork({ ...empty, project_path: "/Users/crew/lavrovo.ozp" }),
    ).toBe(true);
  });
});
