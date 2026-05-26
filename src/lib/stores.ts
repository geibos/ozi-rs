import { writable, derived } from "svelte/store";
import type {
  AppStateDto,
  CatalogCachePayload,
  DownloadProgressPayload,
  LizaProjectSummaryDto,
  SimplifiedPreview,
} from "./types";
import { getAppState } from "./api";
import { selectVisibleWaypointLayers } from "./waypoint-layers";

export { selectVisibleWaypointLayers };

/**
 * Versioned localStorage key for the LizaAlert project catalog cache.
 * The `v1` suffix lets a future schema change (e.g. extending
 * `LizaProjectSummaryDto`) bump to `v2` while old cache values are
 * naturally garbage-collected by the next read attempt.
 */
const CATALOG_CACHE_KEY = "liza:projects:v1";

function isValidCacheEntry(value: unknown): value is LizaProjectSummaryDto {
  if (typeof value !== "object" || value === null) return false;
  const candidate = value as Record<string, unknown>;
  return (
    typeof candidate.slug === "string" && typeof candidate.name === "string"
  );
}

/**
 * Read the LizaAlert project catalog from localStorage. Returns the cached
 * items on success or `null` on any failure (missing, corrupt, wrong shape,
 * storage unavailable). Never throws.
 */
export function loadCatalogCache(): LizaProjectSummaryDto[] | null {
  try {
    if (typeof localStorage === "undefined") return null;
    const raw = localStorage.getItem(CATALOG_CACHE_KEY);
    if (!raw) return null;
    const parsed = JSON.parse(raw) as unknown;
    if (typeof parsed !== "object" || parsed === null) return null;
    const payload = parsed as Partial<CatalogCachePayload>;
    if (!Array.isArray(payload.items)) return null;
    if (typeof payload.writtenAt !== "string") return null;
    if (!payload.items.every(isValidCacheEntry)) return null;
    return payload.items;
  } catch {
    return null;
  }
}

/**
 * Write the LizaAlert project catalog to localStorage with an ISO-8601
 * timestamp. Best-effort: storage errors (e.g. `QuotaExceededError`,
 * unavailable storage) are swallowed and logged to the dev console only.
 */
export function saveCatalogCache(items: LizaProjectSummaryDto[]): void {
  try {
    if (typeof localStorage === "undefined") return;
    const payload: CatalogCachePayload = {
      items,
      writtenAt: new Date().toISOString(),
    };
    localStorage.setItem(CATALOG_CACHE_KEY, JSON.stringify(payload));
  } catch (error) {
    console.warn("saveCatalogCache: storage write failed", error);
  }
}

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
// Seed synchronously from the persisted catalog cache so the bundle loader
// renders the previous catalog on its first paint without waiting for any
// IPC round-trip. Falls back to an empty list on first-ever launch or any
// cache failure.
export const projectsStore = writable<LizaProjectSummaryDto[]>(
  loadCatalogCache() ?? [],
);
export const projects = derived(projectsStore, ($projects) => $projects);
export const projectsLoading = writable(false);
export const currentProject = derived(appState, ($s) => $s?.current_project ?? null);
export const activeMap = derived(appState, ($s) => $s?.active_map ?? null);
export const trackLayerCount = derived(appState, ($s) => $s?.track_layer_count ?? 0);
/**
 * All waypoint layer summaries from the current project filtered by per-layer
 * `visible` (defaulting to true when absent). Renderers consume this to draw
 * markers from every layer simultaneously, regardless of which layer is active
 * for editing — required by the non-destructive active-layer invariant.
 */
export const visibleWaypointLayers = derived(appState, ($s) =>
  selectVisibleWaypointLayers($s?.waypoint_layers ?? null),
);
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

/**
 * Upsert-by-slug merge for an incoming `projects-chunk` payload.
 *
 * For each entry in `chunk`:
 *   - if a matching `slug` already exists in `projectsStore`, the existing
 *     entry's fields are replaced in place (position preserved);
 *   - otherwise the entry is appended to the end in the order it appears
 *     within the chunk.
 *
 * Entries already present in the store but absent from the chunk are
 * retained (historical missions never get pruned by a refresh). This
 * matches the `lizaalert-integration` spec for stale-while-revalidate
 * refresh semantics.
 */
export function appendProjectsChunk(chunk: LizaProjectSummaryDto[]) {
  if (chunk.length === 0) return;
  projectsStore.update((current) => {
    const indexBySlug = new Map<string, number>();
    current.forEach((project, index) => indexBySlug.set(project.slug, index));

    let next: LizaProjectSummaryDto[] | null = null;
    for (const incoming of chunk) {
      const existingIndex = indexBySlug.get(incoming.slug);
      if (existingIndex !== undefined) {
        // Replace in place — only allocate the new array on first mutation.
        if (next === null) next = current.slice();
        next[existingIndex] = incoming;
      } else {
        if (next === null) next = current.slice();
        next.push(incoming);
        indexBySlug.set(incoming.slug, next.length - 1);
      }
    }
    return next ?? current;
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

/** ID of the currently-active bundle download, if any. */
export const activeDownloadId = writable<string | null>(null);

/**
 * Files that have finished downloading inside the active bundle. Cleared at
 * the start of every new download. The UI uses this to render per-file
 * ticks and to gate the "Open bundle now" affordance.
 */
export const readyBundleFiles = writable<
  Array<{ package_name: string; local_path: string; file_index: number; file_count: number }>
>([]);

export function noteBundleFileReady(payload: {
  download_id: string;
  package_name: string;
  local_path: string;
  file_index: number;
  file_count: number;
}) {
  readyBundleFiles.update((list) => {
    if (list.some((p) => p.package_name === payload.package_name)) return list;
    return [...list, payload];
  });
}

export function resetBundleDownloadState(downloadId: string | null) {
  activeDownloadId.set(downloadId);
  readyBundleFiles.set([]);
  downloadProgress.set(new Map());
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

// Persist the catalog after the chunk stream stops growing. We can't rely
// on a `busy: true → false` transition observable from the layout: the
// backend emits `state-changed` only on refresh completion (after busy is
// already cleared), and the listener is registered *after* loadProjects()
// is invoked. Both gaps belong to the `consolidate-state-event-flow`
// change. To stay independent of that wiring rewrite, we debounce a write
// off projectsStore itself — every chunk arrival defers the write, and
// once chunks stop arriving for ~800 ms the snapshot lands.
//
// The non-empty gate still applies: a refresh that yields zero items
// (initial cold cache with the network down) must not overwrite a good
// previous cache with an empty list.
let saveCacheTimer: ReturnType<typeof setTimeout> | null = null;
projectsStore.subscribe((projects) => {
  if (projects.length === 0) return;
  if (saveCacheTimer !== null) clearTimeout(saveCacheTimer);
  saveCacheTimer = setTimeout(() => {
    saveCatalogCache(projects);
    saveCacheTimer = null;
  }, 800);
});
