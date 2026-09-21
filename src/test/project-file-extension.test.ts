// @vitest-environment jsdom
import { beforeEach, describe, expect, it, vi } from "vitest";

interface SaveOptions {
  filters: { name: string; extensions: string[] }[];
}

const { saveDialog, saveProject } = vi.hoisted(() => ({
  saveDialog: vi.fn(async (_options?: unknown): Promise<string | null> => null),
  saveProject: vi.fn(async (_path: string): Promise<void> => {}),
}));

vi.mock("@tauri-apps/plugin-dialog", () => ({ save: saveDialog }));
vi.mock("$lib/api", () => ({
  saveProject,
  undo: vi.fn(async () => {}),
  redo: vi.fn(async () => {}),
}));
vi.mock("svelte-sonner", () => ({
  toast: { success: vi.fn(), error: vi.fn() },
}));
// `projectPath` is derived from the app state in the real store; the actions
// only read it, so a writable stands in.
vi.mock("$lib/stores", async () => {
  const { writable } = await import("svelte/store");
  return { projectPath: writable<string | null>(null) };
});

import { saveAs, quickSave } from "../lib/actions/project";
import { getRecentProjects } from "../lib/recent-projects";
import { projectPath } from "$lib/stores";
import {
  PROJECT_OPEN_EXTENSIONS,
  PROJECT_SAVE_EXTENSION,
} from "../lib/project-file";

/**
 * The project format is `.ozp` — AGENTS.md, `docs/project-map.md` and
 * `project-persistence` all say so, and the spec requires a save "to a
 * user-chosen `.ozp` file".
 *
 * Both dialogs filtered on `json`. A crew who had an `.ozp` — from an older
 * build, from a colleague, from anywhere — could not see it in the open
 * dialog, because a filter hides what it does not match. And what they saved
 * was not the format the documentation names.
 */
describe("the project file's extension", () => {
  beforeEach(() => {
    saveDialog.mockClear();
    saveProject.mockClear();
  });

  it("is what Save offers", async () => {
    saveDialog.mockResolvedValueOnce("/tmp/search.ozp");
    await saveAs();

    const options = saveDialog.mock.calls[0][0] as SaveOptions;
    expect(options.filters[0].extensions).toEqual([PROJECT_SAVE_EXTENSION]);
    expect(PROJECT_SAVE_EXTENSION).toBe("ozp");
    expect(saveProject).toHaveBeenCalledWith("/tmp/search.ozp");
  });

  /**
   * Open accepts the older extension too. Everything this app has saved until
   * now is a `.json`, and a filter that hid those would lose a crew their
   * work far more surely than the wrong extension ever did.
   */
  it("is what Open accepts, alongside what earlier builds wrote", () => {
    expect(PROJECT_OPEN_EXTENSIONS).toContain("ozp");
    expect(PROJECT_OPEN_EXTENSIONS).toContain("json");
    expect(PROJECT_OPEN_EXTENSIONS[0]).toBe("ozp");
  });

  it("does not save when the dialog is cancelled", async () => {
    saveDialog.mockResolvedValueOnce(null);
    await saveAs();
    expect(saveProject).not.toHaveBeenCalled();
  });
});

/**
 * A project a crew has just written is the one they are most likely to want
 * next, and a save is where a never-saved project first gets a path at all —
 * so both saves record it, not just the open dialog.
 */
describe("saving records the project among the recents", () => {
  beforeEach(() => {
    localStorage.clear();
    (projectPath as unknown as { set(v: string | null): void }).set(null);
  });

  it("records the destination a Save As chose", async () => {
    saveDialog.mockResolvedValueOnce("/searches/2026-09-21_Veter.ozp");
    await saveAs();

    expect(getRecentProjects().map((p) => p.path)).toEqual([
      "/searches/2026-09-21_Veter.ozp",
    ]);
  });

  it("moves a project saved again to the front of the list", async () => {
    saveDialog.mockResolvedValueOnce("/a.ozp");
    await saveAs();
    saveDialog.mockResolvedValueOnce("/b.ozp");
    await saveAs();

    (projectPath as unknown as { set(v: string | null): void }).set("/a.ozp");
    await quickSave();

    expect(getRecentProjects().map((p) => p.path)).toEqual([
      "/a.ozp",
      "/b.ozp",
    ]);
  });

  it("records nothing when the save fails", async () => {
    saveDialog.mockResolvedValueOnce("/unwritable.ozp");
    saveProject.mockRejectedValueOnce(new Error("read-only volume"));
    await saveAs();

    expect(getRecentProjects()).toEqual([]);
  });
});
