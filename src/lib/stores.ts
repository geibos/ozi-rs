import { writable, derived } from "svelte/store";
import type {
  AppStateDto,
  DownloadProgressPayload,
  LizaProjectSummaryDto,
  SimplifiedPreview,
} from "./types";
import { getAppState } from "./api";

function createAppStore() {
  const { subscribe, set } = writable<AppStateDto | null>(null);

  return {
    subscribe,
    async refresh() {
      const state = await getAppState();
      set(state);
    },
  };
}

export const appState = createAppStore();

export const busy = derived(appState, ($s) => $s?.busy ?? false);
export const status = derived(appState, ($s) => $s?.status ?? "");
export const diagnostics = derived(appState, ($s) => $s?.diagnostics ?? []);
export const projectsStore = writable<LizaProjectSummaryDto[]>([]);
export const projects = derived(projectsStore, ($projects) => $projects);
export const projectsLoading = writable(false);
export const currentProject = derived(appState, ($s) => $s?.current_project ?? null);
export const activeMap = derived(appState, ($s) => $s?.active_map ?? null);
export const trackLayerCount = derived(appState, ($s) => $s?.track_layer_count ?? 0);
export const downloadingMaps = derived(appState, ($s) => new Set($s?.downloading_maps ?? []));

function syncActiveLayer(
  current: bigint | null,
  layers: Array<{ id: number }>
): bigint | null {
  if (layers.length === 0) return null;
  if (current !== null && layers.some((layer) => BigInt(layer.id) === current)) {
    return current;
  }
  return BigInt(layers[0].id);
}

export function appendProjectsChunk(chunk: LizaProjectSummaryDto[]) {
  projectsStore.update((current) => {
    const known = new Set(current.map((project) => project.slug));
    const additions = chunk.filter((project) => !known.has(project.slug));
    return additions.length > 0 ? [...current, ...additions] : current;
  });
}

export function syncProjectsFromAppState(state: AppStateDto | null) {
  projectsStore.set(state?.projects ?? []);
}

// Per-package download progress: package_name → { downloaded, total? }
export const downloadProgress = writable<Map<string, DownloadProgressPayload>>(new Map());

export function updateDownloadProgress(payload: DownloadProgressPayload) {
  downloadProgress.update((map) => {
    const next = new Map(map);
    if (payload.downloaded_bytes === 0 && !payload.total_bytes) {
      next.delete(payload.package_name);
    } else {
      next.set(payload.package_name, payload);
    }
    return next;
  });
}

// UI-only state (not persisted)
export const consoleOpen = writable(false);
export const tracksPanelOpen = writable(true);
export const waypointsPanelOpen = writable(false);
export const addWaypointMode = writable(false);
export const activeTrackLayerId = writable<bigint | null>(null);
export const activeWaypointLayerId = writable<bigint | null>(null);
export const drawingModeActive = writable(false);
export const drawingTrackLayerId = writable<bigint | null>(null);
export const drawingTrackId = writable<bigint | null>(null);
export const drawingPointCount = writable(0);
export const drawingFinishRequested = writable(false);
export const drawingSegmentId = writable<bigint | null>(null);
export const trackPointsPanelOpen = writable(false);
export const editModeActive = writable(false);
export const selectedTrack = writable<{ layerId: bigint; trackId: bigint } | null>(null);
export const selectedWaypointId: import("svelte/store").Writable<bigint | null> = writable(null);
export const selectedPointId = writable<bigint | null>(null);
export const bundleLoaderOpen = writable(false);
export const simplifyState = writable<{
  active: boolean;
  layerId: bigint;
  trackId: bigint;
  tolerance: number;
  preview: SimplifiedPreview | null;
}>({
  active: false,
  layerId: BigInt(0),
  trackId: BigInt(0),
  tolerance: 10,
  preview: null,
});
export const selectedTheme = writable<string>(
  localStorage.getItem("theme") ?? "auto"
);

selectedTheme.subscribe((v) => localStorage.setItem("theme", v));

appState.subscribe((state) => {
  activeTrackLayerId.update((current) => syncActiveLayer(current, state?.track_layers ?? []));
  activeWaypointLayerId.update((current) => syncActiveLayer(current, state?.waypoint_layers ?? []));
});
