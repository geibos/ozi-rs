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

import { saveAs } from "../lib/actions/project";
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
