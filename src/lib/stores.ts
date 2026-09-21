import { writable, derived } from "svelte/store";
import type {
  AppStateDto,
  BundleProgressPayload,
  CatalogCachePayload,
  DownloadProgressPayload,
  LayerSummaryDto,
  LizaProjectSummaryDto,
  SimplifiedPreview,
  TrackSummary,
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
// CJ-7: unsaved-changes signal and the current .ozp path. `projectDirty`
// drives the shell dirty indicator and the window close-guard; `projectPath`
// lets Cmd+S quick-save without a dialog once the project has a path.
export const projectDirty = derived(
  appState,
  ($s) => $s?.project_dirty ?? false,
);
export const projectPath = derived(appState, ($s) => $s?.project_path ?? null);
// Seed synchronously from the persisted catalog cache so the bundle loader
// renders the previous catalog on its first paint without waiting for any
// IPC round-trip. Falls back to an empty list on first-ever launch or any
// cache failure.
export const projectsStore = writable<LizaProjectSummaryDto[]>(
  loadCatalogCache() ?? [],
);
export const projects = derived(projectsStore, ($projects) => $projects);
export const projectsLoading = writable(false);
// Route-independent clear for the "refreshing list…" hint: the catalog
// refresh flips `lizaalert.busy` back to false on completion, which reaches
// the frontend via `state-changed` → `appState.refresh()`. Clearing here
// (instead of in a page-level effect) keeps the hint honest when the
// BundleLoader is mounted inside the workspace Sheet on `/project`.
appState.subscribe((s) => {
  if (s && !s.busy) projectsLoading.set(false);
});
export const currentProject = derived(
  appState,
  ($s) => $s?.current_project ?? null,
);
export const activeMap = derived(appState, ($s) => $s?.active_map ?? null);
export const trackLayerCount = derived(
  appState,
  ($s) => $s?.track_layer_count ?? 0,
);
/**
 * All waypoint layer summaries from the current project filtered by per-layer
 * `visible` (defaulting to true when absent). Renderers consume this to draw
 * markers from every layer simultaneously, regardless of which layer is active
 * for editing — required by the non-destructive active-layer invariant.
 */
export const visibleWaypointLayers = derived(appState, ($s) =>
  selectVisibleWaypointLayers($s?.waypoint_layers ?? null),
);
export const downloadingMaps = derived(
  appState,
  ($s) => new Set($s?.downloading_maps ?? []),
);

/**
 * Slice indicator for the active raster map. Changes only when the active
 * map's `local_path` changes — typing in the filter or download progress
 * events do NOT mutate this value.
 */
export const activeMapRef = derived(
  appState,
  ($s) => $s?.active_map?.local_path ?? null,
);

/**
 * Build a stable string fingerprint describing every field that
 * `MapView`'s tracks layer re-renders against: layer set + per-track
 * id, color, line width, visibility, and point count. Order-stable
 * regardless of layer/track insertion order in the source list.
 *
 * Exposed for unit testing in `src/test/fingerprints.test.ts` so that
 * forgetting to include a render-relevant field surfaces as a failed
 * assertion rather than a stale map.
 */
export function fingerprintTracks(
  layers: LayerSummaryDto[] | null | undefined,
  tracks: TrackSummary[] | null | undefined,
): string {
  const layerPart = (layers ?? [])
    .map((l) => `${l.id}:${l.visible === false ? 0 : 1}`)
    .sort()
    .join(",");
  const trackPart = (tracks ?? [])
    .map(
      (t) =>
        `${t.layer_id}:${t.track_id}:${t.color}:${t.line_width}:${
          t.visible ? 1 : 0
        }:${t.point_count}`,
    )
    .sort()
    .join("|");
  return `L[${layerPart}]T[${trackPart}]`;
}

/**
 * Stable string fingerprint over the waypoint-layer slice. AppStateDto
 * does NOT carry individual waypoint geometry/symbols — those are
 * fetched on demand via `getWaypoints(layerId)`. Per-waypoint diffing
 * is done by the incremental reconciler in MapView, so this fingerprint
 * only needs to invalidate when the layer set or per-layer visibility
 * changes. Download-progress events do not mutate `waypoint_layers`,
 * so this value is stable through a download burst.
 */
export function fingerprintWaypoints(
  layers: LayerSummaryDto[] | null | undefined,
): string {
  return (layers ?? [])
    .map((l) => `${l.id}:${l.visible === false ? 0 : 1}`)
    .sort()
    .join(",");
}

export const tracksFingerprint = derived(appState, ($s) =>
  fingerprintTracks($s?.track_layers, $s?.tracks),
);

export const waypointsFingerprint = derived(appState, ($s) =>
  fingerprintWaypoints($s?.waypoint_layers),
);

function syncActiveLayer(
  current: bigint | null,
  layers: Array<{ id: number }>,
): bigint | null {
  if (layers.length === 0) return null;
  if (
    current !== null &&
    layers.some((layer) => BigInt(layer.id) === current)
  ) {
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
  // Stale-while-revalidate guard: a null state (store not hydrated yet) or
  // an empty backend list (catalog refresh still in flight on cold start)
  // must NOT clobber the localStorage-seeded catalog — wiping it here made
  // the loader render an empty list until the full network refresh landed,
  // even though a perfectly clickable cached list was already available.
  const incoming = state?.projects ?? [];
  if (incoming.length === 0) return;
  projectsStore.set(incoming);
}

// Per-package download progress: package_name → { downloaded, total? }
export const downloadProgress = writable<Map<string, DownloadProgressPayload>>(
  new Map(),
);

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
 * Close out a bundle download when the backend stops being busy.
 *
 * Nothing used to clear `activeDownloadId` on success, and the progress panel
 * shows while `activeDownloadId !== null && busy`. So the panel vanished when
 * the download ended — and came back, still showing that download's finished
 * rows, the next time anything made the app busy (refreshing the project list,
 * for one). Tying the id to the busy edge makes the panel belong to the
 * operation that is actually running.
 */
let wasBusy = false;
busy.subscribe((isBusy) => {
  if (wasBusy && !isBusy) {
    activeDownloadId.set(null);
  }
  wasBusy = isBusy;
});

export function resetBundleDownloadState(downloadId: string | null) {
  activeDownloadId.set(downloadId);
  downloadProgress.set(new Map());
  currentDownload.set(null);
  bundleProgress.set(null);
}

/**
 * Aggregate bundle-level progress (file count, byte totals, phase label).
 * Written by the layout-level `bundle-progress` listener; read by the
 * bundle-loader status bar. Promoted from a page-local `$state` because
 * the listener that feeds it lives in `+layout.svelte` — see the
 * `consolidate-state-event-flow` change for the single-owner rule.
 */
export const bundleProgress = writable<BundleProgressPayload | null>(null);

/**
 * Last per-file download-progress payload, used by the bundle-loader
 * status bar to render the "N / M — package_name" current-file label
 * and the indeterminate progress bar fallback. Layout-level writer,
 * page-level reader.
 */
export const currentDownload = writable<DownloadProgressPayload | null>(null);

// UI-only state (not persisted)
export const consoleOpen = writable(false);
/**
 * Active LibraryRail tab. Session-scoped only — NOT persisted to
 * localStorage or the Rust session file. Default `'maps'`. Reopening the
 * app starts on Maps regardless of where the previous session ended.
 */
export const libraryActiveTab = writable<"maps" | "tracks" | "waypoints">(
  "maps",
);
export const addWaypointMode = writable(false);
export const activeTrackLayerId = writable<bigint | null>(null);
export const activeWaypointLayerId = writable<bigint | null>(null);
export const drawingModeActive = writable(false);
export const drawingTrackLayerId = writable<bigint | null>(null);
export const drawingTrackId = writable<bigint | null>(null);
export const drawingPointCount = writable(0);
export const drawingFinishRequested = writable(false);
export const drawingSegmentId = writable<bigint | null>(null);
export const editModeActive = writable(false);
export const selectedTrack = writable<{
  layerId: bigint;
  trackId: bigint;
} | null>(null);
export const selectedWaypointId: import("svelte/store").Writable<
  bigint | null
> = writable(null);
export const selectedPointId = writable<bigint | null>(null);
/**
 * Current map viewport bounds in lat/lon. Written by `MapView` on `moveend`
 * (plus once on map load) — deliberately NOT per-frame — and reset to `null`
 * on map teardown. Consumed by the CJ-4 "crop to map view" action in
 * `TrackInspector`, which stays disabled while the value is `null`.
 */
export const mapViewportBounds = writable<{
  minLat: number;
  minLon: number;
  maxLat: number;
  maxLon: number;
} | null>(null);
/**
 * Monotonic counter bumped after track edits that `tracksFingerprint`
 * cannot see: sorting reorders points without changing `point_count`, and
 * split/join move points between segments. Consumers treat it as a cache
 * key component — `MapView`'s tracks-slice effect re-fetches GeoJSON when
 * it changes, and `TrackInspector` / `TrackSegmentsTable` invalidate their
 * cached `TrackDetail`. Crop actions bump it too so the detail caches
 * refresh without waiting for the `state-changed` round-trip.
 */
export const tracksGeometryVersion = writable(0);
/**
 * One-shot "focus the map on this entity" request. Written by the Library
 * track rows and the Track Inspector ("Show on map"); consumed by `MapView`,
 * which fetches the track's points, calls `map.fitBounds(...)` over their
 * bbox, and resets the store to `null`. The monotonically increasing `nonce`
 * makes repeat clicks on the same track re-fire the effect.
 */
export type MapFocusRequest =
  | { kind: "track"; layerId: bigint; trackId: bigint; nonce: number }
  | { kind: "all-tracks"; nonce: number }
  | { kind: "waypoint"; lat: number; lon: number; nonce: number };

export const mapFocusRequest = writable<MapFocusRequest | null>(null);

let mapFocusNonce = 0;

/** Request that MapView pans/zooms to fit the given track. */
export function requestTrackFocus(layerId: bigint, trackId: bigint): void {
  mapFocusNonce += 1;
  mapFocusRequest.set({
    kind: "track",
    layerId,
    trackId,
    nonce: mapFocusNonce,
  });
}

/**
 * Request that MapView fits ALL track geometry into view. Fired after an
 * import so freshly-added tracks are actually shown — otherwise they render
 * off-screen (the camera stays wherever the active raster put it) and the
 * user thinks the import failed.
 */
/**
 * Request that MapView centres on a waypoint. The coordinates travel with the
 * request because the row already holds them — no round trip is needed to put
 * a mark on screen.
 */
export function requestWaypointFocus(lat: number, lon: number): void {
  mapFocusNonce += 1;
  mapFocusRequest.set({ kind: "waypoint", lat, lon, nonce: mapFocusNonce });
}

export function requestAllTracksFocus(): void {
  mapFocusNonce += 1;
  mapFocusRequest.set({ kind: "all-tracks", nonce: mapFocusNonce });
}
export const bundleLoaderOpen = writable(false);
/**
 * Slug the bundle loader should select as soon as it opens.
 *
 * The command palette's "Switch project" used to open the loader and leave the
 * operator at the top of a catalogue that is now thirteen thousand rows long,
 * with a toast asking them to find the project a second time. The loader
 * consumes this once and clears it.
 */
export const bundleLoaderPreselect = writable<string | null>(null);
/**
 * Inspector-rail visibility placeholder for the `redesign-shell-layout`
 * change. When false (default) `WorkspaceShell` omits the right rail from
 * the DOM and the canvas grows into the freed 360px. The `inspector-pane`
 * change replaces this writable with content-driven logic that flips true
 * whenever selection / detail content is present.
 */
export const inspectorOpen = writable(false);

/**
 * Cmd-K command palette open/closed flag. Mounted once at the layout level
 * (`+layout.svelte`) so the palette is reachable from any focus state,
 * including the cold-start surface. Flipped by the global `⌘K` / `Ctrl+K`
 * key handler and by the top-context-bar trigger button in
 * `WorkspaceShell`.
 */
export const commandPaletteOpen = writable(false);

/**
 * "Map info" affordance state for the Library Maps tab → Map Inspector
 * wiring. When non-null, the Inspector renders `MapInspector` for the
 * named map. The Library writes this on click; the Inspector clears it
 * when the selection moves to a track or waypoint. Independent of the
 * `activeMap` store (the user can preview info for any map without
 * opening it).
 */
export const selectedMapInfo = writable<{
  projectSlug: string | null;
  packageName: string;
  localPath: string;
} | null>(null);
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
/**
 * Active theme identifier. Default `"native-auto"` (Zinc + Teal tracking OS
 * light/dark). Persisted in `localStorage["theme"]`. Catppuccin flavour
 * values (`auto` / `latte` / `frappe` / `macchiato` / `mocha`) implicitly
 * enable the Catppuccin pack — see `selectedTheme.subscribe` below.
 */
export const selectedTheme = writable<string>(
  localStorage.getItem("theme") ?? "native-auto",
);

selectedTheme.subscribe((v) => localStorage.setItem("theme", v));

const CATPPUCCIN_THEME_VALUES = new Set([
  "auto",
  "latte",
  "frappe",
  "macchiato",
  "mocha",
]);

/**
 * Whether the user has opted into the Catppuccin theme pack. When false,
 * the theme picker only offers the native auto/light/dark entries; when
 * true the picker exposes the four Catppuccin flavours + Catppuccin Auto.
 *
 * The flag is implicitly turned ON the first time the user selects any
 * Catppuccin flavour — keeping the existing single-store API surface
 * (selecting "Mocha" still applies Mocha, no separate enable step).
 */
export const catppuccinPackEnabled = writable<boolean>(
  localStorage.getItem("catppuccinPackEnabled") === "1" ||
    CATPPUCCIN_THEME_VALUES.has(localStorage.getItem("theme") ?? ""),
);

catppuccinPackEnabled.subscribe((v) =>
  localStorage.setItem("catppuccinPackEnabled", v ? "1" : "0"),
);

selectedTheme.subscribe((v) => {
  if (CATPPUCCIN_THEME_VALUES.has(v)) catppuccinPackEnabled.set(true);
});

appState.subscribe((state) => {
  activeTrackLayerId.update((current) =>
    syncActiveLayer(current, state?.track_layers ?? []),
  );
  activeWaypointLayerId.update((current) =>
    syncActiveLayer(current, state?.waypoint_layers ?? []),
  );
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
