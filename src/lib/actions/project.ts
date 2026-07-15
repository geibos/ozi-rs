/**
 * Shared project actions (CJ-7 "Continuity" slice).
 *
 * Single implementation behind every save/undo/redo surface: the command
 * palette, the workspace-shell buttons and the global keyboard chords
 * (Cmd/Ctrl+S, Cmd/Ctrl+Z, Cmd/Ctrl+Shift+Z) all call these helpers so
 * dialog fallback, toasts and i18n cannot drift between entry points.
 */
import { get } from "svelte/store";
import { save as saveDialog } from "@tauri-apps/plugin-dialog";
import { toast } from "svelte-sonner";
import { redo, saveProject, undo } from "$lib/api";
import { t } from "$lib/i18n";
import { projectPath } from "$lib/stores";

/** Same filter the palette's "Open project…" dialog uses. */
const PROJECT_FILE_FILTER = { name: "OziRS project", extensions: ["json"] };

/**
 * Save without a dialog when the project already has a path on disk
 * (`projectPath` from AppStateDto); fall back to a save dialog for a
 * never-saved project.
 */
export async function quickSave(): Promise<void> {
  const path = get(projectPath);
  if (path === null) {
    await saveAs();
    return;
  }
  const translate = get(t);
  try {
    await saveProject(path);
    toast.success(translate("toast.saved"));
  } catch (error) {
    toast.error(translate("toast.saveFailed"), { description: String(error) });
  }
}

/** Always ask for a destination, then save. */
export async function saveAs(): Promise<void> {
  const translate = get(t);
  try {
    const path = await saveDialog({ filters: [PROJECT_FILE_FILTER] });
    if (!path) return; // user cancelled — not an error
    await saveProject(path);
    toast.success(translate("toast.saved"));
  } catch (error) {
    toast.error(translate("toast.saveFailed"), { description: String(error) });
  }
}

export async function doUndo(): Promise<void> {
  try {
    await undo();
  } catch (error) {
    toast.error(get(t)("toast.undoFailed"), { description: String(error) });
  }
}

export async function doRedo(): Promise<void> {
  try {
    await redo();
  } catch (error) {
    toast.error(get(t)("toast.redoFailed"), { description: String(error) });
  }
}
