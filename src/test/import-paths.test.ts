import { describe, expect, it, vi, beforeEach } from "vitest";

/**
 * Two surfaces take a day's files into the project — the Import… picker and,
 * since 2026-09-23, dropping them on the window — and CJ-3 is the same journey
 * either way. The dispatch lives in one module so the two cannot drift, which
 * is exactly how the picker came to accept `.wpt` while nothing else did.
 *
 * What used to stand in for this was a pair of assertions that the component's
 * source text contained `endsWith(".plt")`. That checks the spelling of a
 * branch, not that a `.plt` reaches the PLT importer, and it went on passing
 * when the branch moved out of the component.
 */
const calls: string[] = [];

vi.mock("$lib/api", () => ({
  importGpx: vi.fn(async (p: string) => {
    calls.push(`gpx:${p}`);
    return "";
  }),
  importPlt: vi.fn(async (p: string) => {
    calls.push(`plt:${p}`);
    return "";
  }),
  importWpt: vi.fn(async (p: string) => {
    calls.push(`wpt:${p}`);
    return "";
  }),
  importTracksDirectory: vi.fn(async (p: string) => {
    calls.push(`dir:${p}`);
    return { tracks: 0, waypoints: 0, files: 0, skipped: [] };
  }),
}));

const { importPaths, isImportablePath, IMPORTABLE_EXTENSIONS } =
  await import("../lib/actions/import-paths");

describe("importPaths", () => {
  beforeEach(() => {
    calls.length = 0;
  });

  it("sends each file to the reader that understands it", async () => {
    const outcome = await importPaths([
      "/searches/day3.gpx",
      "/searches/старый.PLT",
      "/searches/ШТАБ.wpt",
      "/searches/all.zip",
    ]);

    expect(calls).toEqual([
      "gpx:/searches/day3.gpx",
      // Case comes from whatever wrote the file; a navigator from 2009 shouts.
      "plt:/searches/старый.PLT",
      "wpt:/searches/ШТАБ.wpt",
      // ZIP goes through the GPX command, which unpacks it.
      "gpx:/searches/all.zip",
    ]);
    expect(outcome.imported).toBe(4);
    expect(outcome.failed).toEqual([]);
  });

  it("treats anything without a known extension as a folder", async () => {
    // A memory card handed over is a folder, and the recursive import walks
    // per-date subfolders.
    const outcome = await importPaths(["/Volumes/GARMIN/2026-07-08"]);

    expect(calls).toEqual(["dir:/Volumes/GARMIN/2026-07-08"]);
    expect(outcome.usedFolderImport).toBe(true);
  });

  it("does not stop at the first file that fails", async () => {
    const api = await import("$lib/api");
    vi.mocked(api.importPlt).mockRejectedValueOnce(new Error("unreadable"));

    const outcome = await importPaths([
      "/searches/broken.plt",
      "/searches/day4.gpx",
    ]);

    // Nine files of a day that worked matter more than the one that did not.
    expect(outcome.imported).toBe(1);
    expect(outcome.failed).toEqual(["broken.plt"]);
    expect(calls).toContain("gpx:/searches/day4.gpx");
  });

  it("reports a failure by the file's name, not its whole path", async () => {
    const api = await import("$lib/api");
    vi.mocked(api.importGpx).mockRejectedValueOnce(new Error("bad xml"));

    const outcome = await importPaths([
      "/Volumes/Внешний диск/поиски/2026-07-08/ЛИСА15.gpx",
    ]);

    expect(outcome.failed).toEqual(["ЛИСА15.gpx"]);
  });
});

describe("isImportablePath", () => {
  it("knows the four formats the surfaces accept", () => {
    expect(IMPORTABLE_EXTENSIONS).toEqual(["gpx", "plt", "wpt", "zip"]);
    for (const ext of IMPORTABLE_EXTENSIONS) {
      expect(isImportablePath(`/searches/day.${ext}`)).toBe(true);
      expect(isImportablePath(`/searches/day.${ext.toUpperCase()}`)).toBe(true);
    }
  });

  it("does not claim a map or a folder", () => {
    expect(isImportablePath("/searches/topo.sqlitedb")).toBe(false);
    expect(isImportablePath("/searches/2026-07-08")).toBe(false);
  });
});
