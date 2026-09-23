import { beforeEach, describe, expect, it } from "vitest";
import { get } from "svelte/store";
import {
  RECENT_PROJECTS_KEY,
  forgetProject,
  recentProjects,
  rememberProject,
} from "../lib/recent-projects";

/**
 * The recent list is a store, not a snapshot.
 *
 * `BundleLoader` read `getRecentProjects()` into a plain `const` at component
 * init, which in Svelte 5 is read once and never again. So opening a
 * colleague's `.ozp` from that very screen left the list without it — and the
 * loader is also the Sheet that opens over the workspace, where the component
 * can stay mounted for a whole session. Found walking CJ-8 on 2026-09-23.
 *
 * A list of recent things that is a snapshot of one moment is the same lie as
 * a copy of a list that changes.
 */
describe("the recent projects store", () => {
  beforeEach(() => {
    localStorage.removeItem(RECENT_PROJECTS_KEY);
    // Re-publish an empty list: the store was seeded at import.
    rememberProject("/seed.ozp");
    forgetProject("/seed.ozp");
  });

  it("gains a project the moment it is remembered", () => {
    expect(get(recentProjects)).toEqual([]);

    rememberProject("/inbox/2026-09-23_Сагра_второй_штаб.ozp");

    expect(get(recentProjects).map((p) => p.path)).toEqual([
      "/inbox/2026-09-23_Сагра_второй_штаб.ozp",
    ]);
    expect(get(recentProjects)[0].name).toBe(
      "2026-09-23_Сагра_второй_штаб.ozp",
    );
  });

  it("puts the one just opened first", () => {
    rememberProject("/a.ozp", 1);
    rememberProject("/b.ozp", 2);
    rememberProject("/a.ozp", 3);

    expect(get(recentProjects).map((p) => p.path)).toEqual([
      "/a.ozp",
      "/b.ozp",
    ]);
  });

  it("loses a project that no longer opens", () => {
    rememberProject("/gone.ozp");
    rememberProject("/here.ozp");

    forgetProject("/gone.ozp");

    expect(get(recentProjects).map((p) => p.path)).toEqual(["/here.ozp"]);
  });

  it("notifies a subscriber rather than only changing on the next read", () => {
    const seen: number[] = [];
    const stop = recentProjects.subscribe((list) => seen.push(list.length));

    rememberProject("/one.ozp");
    rememberProject("/two.ozp");
    stop();

    // The initial value plus one per change — a screen showing the list is
    // told, rather than finding out when it happens to be rebuilt.
    expect(seen).toEqual([0, 1, 2]);
  });
});
