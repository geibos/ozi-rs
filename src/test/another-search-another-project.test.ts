import { beforeEach, describe, expect, it, vi } from "vitest";
import { get } from "svelte/store";
import type { AppStateDto } from "../lib/bindings";

/**
 * Opening the next search's map from the catalogue starts its own project.
 *
 * A project is one search (owner, 2026-10-01). Without this, choosing the
 * next search and opening its map drew the previous search's routes over a
 * district they have nothing to do with, and the next save wrote both
 * operations into one file.
 */
const backend = vi.hoisted(() => ({
  state: null as AppStateDto | null,
  newProject: vi.fn(async () => {}),
  saveProject: vi.fn(async (_path: string) => {}),
  openSelectedMap: vi.fn(async (_map: string) => ""),
  saveDialog: vi.fn(async (): Promise<string | null> => null),
  toastSuccess: vi.fn(),
}));

vi.mock("$lib/api", () => ({
  getAppState: async () => backend.state,
  newProject: () => backend.newProject(),
  saveProject: (path: string) => backend.saveProject(path),
  openSelectedMap: (map: string) => backend.openSelectedMap(map),
  loadProjectFile: vi.fn(),
  undo: vi.fn(),
  redo: vi.fn(),
}));

vi.mock("@tauri-apps/plugin-dialog", () => ({
  save: () => backend.saveDialog(),
  open: vi.fn(async () => null),
  confirm: vi.fn(async () => true),
}));

vi.mock("svelte-sonner", () => ({
  toast: Object.assign(vi.fn(), {
    success: (...args: unknown[]) => backend.toastSuccess(...args),
    error: vi.fn(),
    message: vi.fn(),
  }),
}));

import { openMapShowingDownload } from "../lib/actions/open-map";
import {
  appState,
  closeGuardOpen,
  closeGuardPurpose,
  resolveCloseGuard,
  type CloseGuardChoice,
} from "../lib/stores";

const LAVROVO = "2026-09-28 Лаврово";
const KIRISHI = "2026-10-01 Кириши";

function state(overrides: Partial<AppStateDto> = {}): AppStateDto {
  return {
    project_name: "Untitled Project",
    project_saved: true,
    project_dirty: false,
    project_path: null,
    status: "",
    listing_busy: false,
    bundle_busy: false,
    downloading_maps: [],
    current_project: {
      slug: "kirishi",
      name: KIRISHI,
      center_lat: 59.4,
      center_lon: 32.0,
      maps: [],
      contents: [],
    },
    active_map: {
      kind: "sqlite",
      project_name: LAVROVO,
      package_name: "Lavrovo_Topo_z16.sqlitedb",
      local_path: "/bundles/lavrovo/Lavrovo_Topo_z16.sqlitedb",
      center_lat: 59.95,
      center_lon: 31.6,
      base_zoom: 14,
    },
    diagnostics: [],
    track_layers: [{ id: 1, name: "Tracks" }],
    waypoint_layers: [{ id: 1, name: "Waypoints" }],
    track_layer_count: 1,
    waypoint_layer_count: 1,
    tracks: [],
    ...overrides,
  } as AppStateDto;
}

const A_TRACK = [
  { layer_id: 1, track_id: 1 },
] as unknown as AppStateDto["tracks"];

async function given(s: AppStateDto) {
  backend.state = s;
  await appState.refresh();
}

/** Answer the close-guard question the moment it is asked. */
function answerWith(choice: CloseGuardChoice) {
  const unsubscribe = closeGuardOpen.subscribe((open) => {
    if (open) queueMicrotask(() => resolveCloseGuard(choice));
  });
  return unsubscribe;
}

describe("opening another search's map", () => {
  beforeEach(() => {
    backend.newProject.mockReset();
    backend.saveProject.mockReset();
    backend.openSelectedMap.mockReset().mockResolvedValue("");
    backend.saveDialog.mockReset().mockResolvedValue(null);
    backend.toastSuccess.mockReset();
    // The new project's state, as the backend answers after `new_project`.
    backend.newProject.mockImplementation(async () => {
      backend.state = state({ project_path: null, tracks: [] });
    });
  });

  it("starts a new project over a saved one without asking, and says for which search", async () => {
    await given(state({ project_path: "/crew/lavrovo.ozp", tracks: A_TRACK }));
    const ask = vi.fn();
    const stop = closeGuardOpen.subscribe((open) => open && ask());

    await expect(openMapShowingDownload("Kirishi_Topo.sqlitedb")).resolves.toBe(
      true,
    );

    expect(ask).not.toHaveBeenCalled();
    expect(backend.newProject).toHaveBeenCalledOnce();
    expect(backend.openSelectedMap).toHaveBeenCalledWith(
      "Kirishi_Topo.sqlitedb",
    );
    expect(backend.newProject.mock.invocationCallOrder[0]).toBeLessThan(
      backend.openSelectedMap.mock.invocationCallOrder[0],
    );
    expect(String(backend.toastSuccess.mock.calls[0]?.[0])).toContain(KIRISHI);
    stop();
  });

  it("leaves the project alone for another sheet of the same search", async () => {
    await given(
      state({
        tracks: A_TRACK,
        project_dirty: true,
        current_project: { ...state().current_project!, name: LAVROVO },
      }),
    );
    await openMapShowingDownload("Lavrovo_Satell_z17.sqlitedb");
    expect(backend.newProject).not.toHaveBeenCalled();
    expect(backend.openSelectedMap).toHaveBeenCalledOnce();
  });

  it("leaves an empty, never-saved project alone", async () => {
    await given(state());
    await openMapShowingDownload("Kirishi_Topo.sqlitedb");
    expect(backend.newProject).not.toHaveBeenCalled();
    expect(backend.openSelectedMap).toHaveBeenCalledOnce();
  });

  it("asks about unsaved work, in the words of this question rather than quitting", async () => {
    await given(state({ project_dirty: true, tracks: A_TRACK }));
    let purposeWhenAsked: string | null = null;
    const stop = closeGuardOpen.subscribe((open) => {
      if (!open) return;
      purposeWhenAsked = get(closeGuardPurpose);
      queueMicrotask(() => resolveCloseGuard("discard"));
    });

    await openMapShowingDownload("Kirishi_Topo.sqlitedb");

    expect(purposeWhenAsked).toBe("newSearch");
    expect(backend.newProject).toHaveBeenCalledOnce();
    expect(backend.openSelectedMap).toHaveBeenCalledOnce();
    stop();
  });

  it("opens nothing and resets nothing when the operator stays", async () => {
    await given(state({ project_dirty: true, tracks: A_TRACK }));
    const stop = answerWith("stay");

    await expect(openMapShowingDownload("Kirishi_Topo.sqlitedb")).resolves.toBe(
      false,
    );

    expect(backend.newProject).not.toHaveBeenCalled();
    expect(backend.openSelectedMap).not.toHaveBeenCalled();
    stop();
  });

  it("saves, then starts the new project and opens the map", async () => {
    await given(
      state({
        project_dirty: true,
        project_path: "/crew/lavrovo.ozp",
        tracks: A_TRACK,
      }),
    );
    const stop = answerWith("save");

    await openMapShowingDownload("Kirishi_Topo.sqlitedb");

    expect(backend.saveProject).toHaveBeenCalledWith("/crew/lavrovo.ozp");
    expect(backend.saveProject.mock.invocationCallOrder[0]).toBeLessThan(
      backend.newProject.mock.invocationCallOrder[0],
    );
    expect(backend.openSelectedMap).toHaveBeenCalledOnce();
    stop();
  });

  it("treats a cancelled save dialog as staying", async () => {
    await given(state({ project_dirty: true, tracks: A_TRACK }));
    backend.saveDialog.mockResolvedValue(null);
    const stop = answerWith("save");

    await expect(openMapShowingDownload("Kirishi_Topo.sqlitedb")).resolves.toBe(
      false,
    );

    expect(backend.saveProject).not.toHaveBeenCalled();
    expect(backend.newProject).not.toHaveBeenCalled();
    expect(backend.openSelectedMap).not.toHaveBeenCalled();
    stop();
  });

  it("treats a failed save as staying", async () => {
    await given(
      state({
        project_dirty: true,
        project_path: "/crew/lavrovo.ozp",
        tracks: A_TRACK,
      }),
    );
    backend.saveProject.mockRejectedValue(new Error("disk full"));
    const stop = answerWith("save");

    await openMapShowingDownload("Kirishi_Topo.sqlitedb");

    expect(backend.newProject).not.toHaveBeenCalled();
    expect(backend.openSelectedMap).not.toHaveBeenCalled();
    stop();
  });
});
