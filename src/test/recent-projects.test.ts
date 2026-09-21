// @vitest-environment jsdom
import { beforeEach, describe, expect, it } from "vitest";
import {
  MAX_RECENT_PROJECTS,
  RECENT_PROJECTS_KEY,
  forgetProject,
  getRecentProjects,
  projectDisplayName,
  rememberProject,
} from "../lib/recent-projects";

/**
 * Reopening yesterday's project meant finding it in a file dialog again. The
 * palette's existing recents are recent *maps* — a bundle and one of its
 * tiles, not a saved `.ozp`.
 */
beforeEach(() => {
  localStorage.clear();
});

const paths = () => getRecentProjects().map((p) => p.path);

describe("the projects a crew opened recently", () => {
  it("remembers one, newest first", () => {
    rememberProject("/searches/2026-09-20_Sagra.ozp", 1);
    rememberProject("/searches/2026-09-21_Veter.ozp", 2);

    expect(paths()).toEqual([
      "/searches/2026-09-21_Veter.ozp",
      "/searches/2026-09-20_Sagra.ozp",
    ]);
  });

  it("moves a project already known to the front rather than listing it twice", () => {
    rememberProject("/a.ozp", 1);
    rememberProject("/b.ozp", 2);
    rememberProject("/a.ozp", 3);

    expect(paths()).toEqual(["/a.ozp", "/b.ozp"]);
  });

  it("keeps the list bounded", () => {
    for (let i = 0; i < MAX_RECENT_PROJECTS + 4; i += 1) {
      rememberProject(`/p${i}.ozp`, i);
    }
    expect(paths()).toHaveLength(MAX_RECENT_PROJECTS);
    expect(paths()[0]).toBe(`/p${MAX_RECENT_PROJECTS + 3}.ozp`);
  });

  it("forgets one that no longer opens", () => {
    rememberProject("/gone.ozp", 1);
    rememberProject("/here.ozp", 2);
    forgetProject("/gone.ozp");
    expect(paths()).toEqual(["/here.ozp"]);
  });

  /** A broken convenience must not stop the palette opening. */
  it("returns nothing rather than throwing on rubbish in storage", () => {
    localStorage.setItem(RECENT_PROJECTS_KEY, "{not json");
    expect(getRecentProjects()).toEqual([]);

    localStorage.setItem(RECENT_PROJECTS_KEY, '{"not":"an array"}');
    expect(getRecentProjects()).toEqual([]);

    localStorage.setItem(RECENT_PROJECTS_KEY, '[{"path":42}]');
    expect(getRecentProjects()).toEqual([]);
  });

  it("shows a project by its file name, on either platform's separator", () => {
    expect(projectDisplayName("/searches/2026-09-21_Veter.ozp")).toBe(
      "2026-09-21_Veter.ozp",
    );
    expect(projectDisplayName("C:\\searches\\Veter.ozp")).toBe("Veter.ozp");
    expect(projectDisplayName("bare.ozp")).toBe("bare.ozp");
  });

  it("ignores an empty path rather than storing a nameless entry", () => {
    rememberProject("");
    expect(paths()).toEqual([]);
  });
});
