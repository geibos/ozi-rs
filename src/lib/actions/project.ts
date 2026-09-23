/**
 * Shared project actions (CJ-7 "Continuity" slice).
 *
 * Single implementation behind every save/undo/redo surface: the command
 * palette, the workspace-shell buttons and the global keyboard chords
 * (Cmd/Ctrl+S, Cmd/Ctrl+Z, Cmd/Ctrl+Shift+Z) all call these helpers so
 * dialog fallback, toasts and i18n cannot drift between entry points.
 */
import { get } from "svelte/store";
import {
  confirm as confirmDialog,
  open as openFileDialog,
  save as saveDialog,
} from "@tauri-apps/plugin-dialog";
import { toast } from "svelte-sonner";
import { loadProjectFile, newProject, redo, saveProject, undo } from "$lib/api";
import { t } from "$lib/i18n";
import {
  activeTrackLayerId,
  activeWaypointLayerId,
  appState,
  projectDirty,
  projectPath,
  requestAllDataFocus,
} from "$lib/stores";
import {
  PROJECT_OPEN_EXTENSIONS,
  PROJECT_SAVE_EXTENSION,
} from "$lib/project-file";
import { forgetProject, rememberProject } from "$lib/recent-projects";

/** See `project-file.ts`: the format is `.ozp`, and both dialogs say so. */
const PROJECT_FILE_FILTER = {
  name: "OziRS project",
  extensions: [PROJECT_SAVE_EXTENSION],
};

/** Opening also accepts the legacy `.json` — see `project-file.ts`. */
const PROJECT_OPEN_FILTER = {
  name: "OziRS project",
  extensions: PROJECT_OPEN_EXTENSIONS,
};

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
    rememberProject(path);
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
    rememberProject(path);
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

/**
 * Open a saved project, from a dialog or from a path the recents already know.
 *
 * Until 2026-09-22 this lived only in the command palette, so a crew arriving
 * with yesterday's work had to know ⌘K existed to get back to it. Three
 * surfaces ask for it now — the palette, its recents list and the launch
 * screen — so the dialog, the remembering and the framing are here rather
 * than copied.
 *
 * A path that fails is dropped from the recents: an entry that errors every
 * time it is chosen is worse than no entry.
 *
 * Answers whether a project was opened, so the caller can act on it. The
 * launcher has to leave for the workspace when one lands, and it cannot tell
 * from the outside: a cancelled dialog and a file that would not read both
 * leave the stores as they were, and so does re-opening the project that is
 * already open — which is exactly what the recent list is for.
 */
export async function openProjectFile(known?: string): Promise<boolean> {
  const translate = get(t);
  let path = known ?? null;
  try {
    if (path === null) {
      const chosen = await openFileDialog({
        filters: [PROJECT_OPEN_FILTER],
      } as Parameters<typeof openFileDialog>[0]);
      if (!chosen) return false; // cancelled — not an error
      path = chosen as string;
    }
    await loadProjectFile(path);
    rememberProject(path);
    // Without this the project's tracks land wherever the camera happens to
    // point, which looks exactly like a project that did not load.
    requestAllDataFocus();
    return true;
  } catch (error) {
    if (known !== undefined) forgetProject(known);
    toast.error(translate("toast.openFailed"), { description: String(error) });
    return false;
  }
}

/**
 * Start the next search.
 *
 * A project is one search. Without this a crew that finished an operation and
 * began the next kept adding to the same document, and yesterday's routes
 * stayed on the map under today's.
 *
 * The question before discarding is the one the close guard asks, for the same
 * reason: an hour of marking is an hour of marking whether the window is
 * closing or the operator is moving on.
 */
export async function startNewProject(): Promise<void> {
  const translate = get(t);
  try {
    if (get(projectDirty)) {
      const go = await confirmDialog(translate("newProject.message"), {
        title: translate("newProject.title"),
        kind: "warning",
        okLabel: translate("newProject.discard"),
        cancelLabel: translate("newProject.cancel"),
      });
      if (!go) return;
    }
    await newProject();
    // The old ids point at layers that no longer exist. Reading the new
    // project's defaults back is what keeps drawing and waypoint placement
    // pointed at something.
    await appState.refresh();
    const state = get(appState);
    activeTrackLayerId.set(
      state?.track_layers?.[0] ? BigInt(state.track_layers[0].id) : null,
    );
    activeWaypointLayerId.set(
      state?.waypoint_layers?.[0] ? BigInt(state.waypoint_layers[0].id) : null,
    );
    toast.success(translate("newProject.done"));
  } catch (error) {
    toast.error(translate("newProject.failed"), { description: String(error) });
  }
}
