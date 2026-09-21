import { beforeEach, describe, expect, it, vi } from "vitest";
import { get } from "svelte/store";

// `vi.mock` is hoisted above every `const`, so the spies live on a namespace
// the factories reach lazily.
const spies = vi.hoisted(() => ({
  loadProjectFile: vi.fn(async (_path: string) => {}),
  openDialog: vi.fn(async () => null as string | null),
  errorToast: vi.fn(),
}));

vi.mock("$lib/api", () => ({
  loadProjectFile: spies.loadProjectFile,
  saveProject: vi.fn(async () => {}),
  undo: vi.fn(async () => {}),
  redo: vi.fn(async () => {}),
}));
vi.mock("@tauri-apps/plugin-dialog", () => ({
  open: spies.openDialog,
  save: vi.fn(async () => null),
}));
vi.mock("svelte-sonner", () => ({
  toast: { error: spies.errorToast, success: vi.fn(), message: vi.fn() },
}));

const { loadProjectFile, openDialog, errorToast } = spies;

import { openProjectFile } from "../lib/actions/project";
import { mapFocusRequest } from "../lib/stores";
import {
  RECENT_PROJECTS_KEY,
  getRecentProjects,
} from "../lib/recent-projects";

/**
 * A saved `.ozp` could be opened from exactly one place: the command palette.
 * Nothing on the launch screen offered it, so a crew arriving in the morning
 * with yesterday's work had to know ⌘K existed. The same reasoning put the
 * language switch in the status bar.
 *
 * Three surfaces now ask for it, so the dialog, the remembering and the
 * framing live in one function rather than three.
 */
describe("openProjectFile", () => {
  beforeEach(() => {
    loadProjectFile.mockReset();
    openDialog.mockReset();
    errorToast.mockReset();
    localStorage.removeItem(RECENT_PROJECTS_KEY);
    mapFocusRequest.set(null);
  });

  it("does nothing at all when the dialog is cancelled", async () => {
    openDialog.mockResolvedValue(null);
    await openProjectFile();
    expect(loadProjectFile).not.toHaveBeenCalled();
    expect(get(mapFocusRequest)).toBeNull();
    expect(errorToast).not.toHaveBeenCalled();
  });

  it("loads the file, remembers it and frames the map on it", async () => {
    openDialog.mockResolvedValue("/searches/2026-09-20.ozp");
    await openProjectFile();
    expect(loadProjectFile).toHaveBeenCalledWith("/searches/2026-09-20.ozp");
    expect(getRecentProjects().map((p) => p.path)).toEqual([
      "/searches/2026-09-20.ozp",
    ]);
    expect(get(mapFocusRequest)?.kind).toBe("all-data");
  });

  it("reports a failure and remembers nothing", async () => {
    openDialog.mockResolvedValue("/searches/gone.ozp");
    loadProjectFile.mockRejectedValue(new Error("no such file"));
    await openProjectFile();
    expect(errorToast).toHaveBeenCalled();
    expect(getRecentProjects()).toEqual([]);
    expect(get(mapFocusRequest)).toBeNull();
  });

  it("opens a known path without a dialog", async () => {
    // The recents list hands it the path; asking again would be absurd.
    await openProjectFile("/searches/yesterday.ozp");
    expect(openDialog).not.toHaveBeenCalled();
    expect(loadProjectFile).toHaveBeenCalledWith("/searches/yesterday.ozp");
    expect(get(mapFocusRequest)?.kind).toBe("all-data");
  });

  it("drops a recent path that no longer opens", async () => {
    localStorage.setItem(
      RECENT_PROJECTS_KEY,
      JSON.stringify([
        { path: "/searches/gone.ozp", name: "gone.ozp", openedAt: 1 },
      ]),
    );
    loadProjectFile.mockRejectedValue(new Error("no such file"));
    await openProjectFile("/searches/gone.ozp");
    expect(getRecentProjects()).toEqual([]);
    expect(errorToast).toHaveBeenCalled();
  });
});
