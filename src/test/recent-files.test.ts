import { beforeEach, describe, expect, it, vi } from "vitest";
import {
  MAX_RECENT_FILES,
  RECENT_FILES_KEY,
  appendRecentFile,
  clearRecentFiles,
  getRecentFiles,
  type RecentFile,
} from "../lib/recentFiles";

function makeRecord(mapPath: string, openedAt = 1_700_000_000_000): RecentFile {
  return {
    projectSlug: "lizaalert",
    mapPath,
    mapName: mapPath.split("/").pop() ?? mapPath,
    openedAt,
  };
}

beforeEach(() => {
  localStorage.clear();
});

describe("recentFiles helper (palette consumer)", () => {
  it("getRecentFiles returns empty list when storage is empty", () => {
    expect(getRecentFiles()).toEqual([]);
  });

  it("appendRecentFile prepends new records (most-recent-first)", () => {
    appendRecentFile(makeRecord("/maps/a.ozi", 1));
    appendRecentFile(makeRecord("/maps/b.ozi", 2));
    const recents = getRecentFiles();
    expect(recents.map((r) => r.mapPath)).toEqual([
      "/maps/b.ozi",
      "/maps/a.ozi",
    ]);
  });

  it("dedupes by mapPath — existing entries move to the front", () => {
    appendRecentFile(makeRecord("/maps/a.ozi", 1));
    appendRecentFile(makeRecord("/maps/b.ozi", 2));
    appendRecentFile(makeRecord("/maps/a.ozi", 3));
    const recents = getRecentFiles();
    expect(recents.map((r) => r.mapPath)).toEqual([
      "/maps/a.ozi",
      "/maps/b.ozi",
    ]);
    expect(recents[0].openedAt).toBe(3);
  });

  it("caps the list at MAX_RECENT_FILES (default 8) and drops the oldest", () => {
    for (let i = 0; i < MAX_RECENT_FILES + 3; i += 1) {
      appendRecentFile(makeRecord(`/maps/m${i}.ozi`, i));
    }
    const recents = getRecentFiles();
    expect(recents).toHaveLength(MAX_RECENT_FILES);
    // The newest (index = MAX_RECENT_FILES + 2) lives at the front; the
    // oldest still present is index 3 (0..2 were dropped).
    expect(recents[0].mapPath).toBe(`/maps/m${MAX_RECENT_FILES + 2}.ozi`);
    expect(recents.at(-1)!.mapPath).toBe(`/maps/m3.ozi`);
  });

  it("treats unparseable storage as empty list and emits console.warn", () => {
    localStorage.setItem(RECENT_FILES_KEY, "not-json");
    const warn = vi.spyOn(console, "warn").mockImplementation(() => undefined);
    const recents = getRecentFiles();
    expect(recents).toEqual([]);
    expect(warn).toHaveBeenCalled();
    warn.mockRestore();
  });

  it("treats schema-invalid records as empty without throwing", () => {
    localStorage.setItem(
      RECENT_FILES_KEY,
      JSON.stringify([{ mapPath: 42 }, { unrelated: "shape" }]),
    );
    expect(getRecentFiles()).toEqual([]);
  });

  it("clearRecentFiles wipes the entry", () => {
    appendRecentFile(makeRecord("/maps/x.ozi"));
    expect(getRecentFiles()).toHaveLength(1);
    clearRecentFiles();
    expect(getRecentFiles()).toEqual([]);
  });
});
