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
export const CATALOG_CACHE_KEY = "liza:projects:v1";

function isValidCacheEntry(
  value: unknown,
): value is Partial<LizaProjectSummaryDto> {
  if (typeof value !== "object" || value === null) return false;
  const candidate = value as Record<string, unknown>;
  return (
    typeof candidate.slug === "string" && typeof candidate.name === "string"
  );
}

/**
 * A cache entry written before the `cached` flag existed has none, and a flag
 * written yesterday may be stale anyway. Default it to false: the backend
 * re-sends every row with a fresh flag as soon as the catalogue is read.
 */
function normalizeCacheEntry(
  entry: Partial<LizaProjectSummaryDto>,
): LizaProjectSummaryDto {
  return {
    slug: entry.slug as string,
    name: entry.name as string,
    cached: entry.cached === true,
  };
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
    return payload.items.map(normalizeCacheEntry);
  } catch {
    return null;
  }
}

/**
 * Write the LizaAlert project catalog to localStorage with an ISO-8601
 * timestamp. Best-effort: storage errors (e.g. `QuotaExceededError`,
 * unavailable storage) are swallowed and logged to the dev console only.
 */
/**
 * When the list on screen last came from the server.
 *
 * Not "when the cache was last written": the cache is rewritten 800 ms after
 * any non-empty change to the list, and that includes the very first one,
 * where the list was restored from the cache itself. Stamping the write time
 * meant a launch with no network re-dated yesterday's list as today's — and
 * the date exists precisely so a crew can tell those apart. External review,
 * 2026-09-22.
 */
let lastRefreshedAt: string | null = null;

/** A completed catalogue walk landed; the list on screen is from the server. */
export function markCatalogueRefreshed(at: string = new Date().toISOString()) {
  lastRefreshedAt = at;
}

export function saveCatalogCache(
  items: LizaProjectSummaryDto[],
  refreshedAt?: string,
): void {
  try {
    if (typeof localStorage === "undefined") return;
    if (refreshedAt !== undefined) lastRefreshedAt = refreshedAt;
    // A write with no refresh behind it keeps whatever date the cache had:
    // the contents may have been merged, but nothing newer was fetched. Only
    // a first-ever write, which by definition follows a walk, takes now.
    const writtenAt =
      lastRefreshedAt ?? loadCatalogWrittenAt() ?? new Date().toISOString();
    lastRefreshedAt = writtenAt;
    const payload: CatalogCachePayload = { items, writtenAt };
    localStorage.setItem(CATALOG_CACHE_KEY, JSON.stringify(payload));
    catalogueWrittenAt.set(writtenAt);
  } catch (error) {
    console.warn("saveCatalogCache: storage write failed", error);
  }
}

/**
 * When the cached list was last written, as the ISO string the cache holds.
 *
 * The timestamp has been in the payload since the cache existed and was
 * thrown away on read. Offline it is the difference between a list a crew can
 * act on and a list they cannot: a search published yesterday is missing from
 * both a stale list and a wrong one.
 */
export function loadCatalogWrittenAt(): string | null {
  try {
    if (typeof localStorage === "undefined") return null;
    const raw = localStorage.getItem(CATALOG_CACHE_KEY);
    if (!raw) return null;
    const parsed = JSON.parse(raw) as Partial<CatalogCachePayload>;
    return typeof parsed?.writtenAt === "string" ? parsed.writtenAt : null;
  } catch {
    return null;
  }
}

function createAppStore() {
  const { subscribe, set } = writable<AppStateDto | null>(null);
  // The same generation stamp `latest-run.ts` holds for component reloads,
  // inlined here because the store is the shared state. Two refreshes
  // overlapped freely and the slower one's answer replaced the newer state —
  // and overlapping is the normal case, not a rare one: a bundle download
  // emits `state-changed` once per file. External review, 2026-09-22.
  let latest = 0;

  return {
    subscribe,
    async refresh() {
      latest += 1;
      const mine = latest;
      const state = await getAppState();
      if (mine !== latest) return;
      set(state);
    },
  };
}

export const appState = createAppStore();

/**
 * The catalogue walk is running.
 *
 * Split from `bundleBusy` on 2026-09-22: one flag for both meant the
 * launch-time walk disabled the only download button for minutes, although
 * the two do unrelated work.
 */
export const listingBusy = derived(appState, ($s) => $s?.listing_busy ?? false);
/** A bundle is being downloaded or opened from disk. */
export const bundleBusy = derived(appState, ($s) => $s?.bundle_busy ?? false);
/** Either of the two — for anything that just wants "something is happening". */
export const busy = derived(
  appState,
  ($s) => ($s?.listing_busy ?? false) || ($s?.bundle_busy ?? false),
);
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

/**
 * The close guard's question, and the operator's answer.
 *
 * The guard runs inside the window's close-request handler, which must call
 * `preventDefault()` before its first `await`; the question it then asks is a
 * dialog of our own, because the operating system's confirm box has two
 * buttons and the answer the operator almost always wants — save, then quit —
 * is the third. `askBeforeClosing()` opens it and resolves with the choice.
 */
export const closeGuardOpen = writable(false);

let closeGuardResolve: ((choice: CloseGuardChoice) => void) | null = null;

export type CloseGuardChoice = "save" | "discard" | "stay";

export function askBeforeClosing(): Promise<CloseGuardChoice> {
  // A second request while one is open answers the first with "stay": two
  // dialogs over one window is worse than one question asked again.
  if (closeGuardResolve) closeGuardResolve("stay");
  closeGuardOpen.set(true);
  return new Promise<CloseGuardChoice>((resolve) => {
    closeGuardResolve = resolve;
  });
}

export function resolveCloseGuard(choice: CloseGuardChoice): void {
  closeGuardOpen.set(false);
  const resolve = closeGuardResolve;
  closeGuardResolve = null;
  resolve?.(choice);
}
// Seed synchronously from the persisted catalog cache so the bundle loader
// renders the previous catalog on its first paint without waiting for any
// IPC round-trip. Falls back to an empty list on first-ever launch or any
// cache failure.
export const projectsStore = writable<LizaProjectSummaryDto[]>(
  loadCatalogCache() ?? [],
);
export const projects = derived(projectsStore, ($projects) => $projects);
/** ISO timestamp of the cached list on screen, seeded from the cache. */
export const catalogueWrittenAt = writable<string | null>(
  loadCatalogWrittenAt(),
);
export const projectsLoading = writable(false);

/**
 * Why the catalogue could not be refreshed, when it could not.
 *
 * Offline the list still shows — it comes from the cache, which is the right
 * answer — but nothing said the refresh had failed, so a crew could not tell
 * today's list from one saved days ago. That is the difference between
 * knowing a bundle exists and assuming it does not.
 */
export const catalogueError = writable<string | null>(null);
// Route-independent clear for the "refreshing list…" hint: the catalog
// refresh flips `lizaalert.busy` back to false on completion, which reaches
// the frontend via `state-changed` → `appState.refresh()`. Clearing here
// (instead of in a page-level effect) keeps the hint honest when the
// BundleLoader is mounted inside the workspace Sheet on `/project`.
appState.subscribe((s) => {
  if (s && !s.listing_busy) projectsLoading.set(false);
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
/**
 * The slugs this catalogue walk has sent, while one is running.
 *
 * `null` means no walk is in progress, and then nothing is collected and
 * nothing can be pruned. The cached chunk that `load_projects` emits from disk
 * arrives before the walk starts, deliberately outside this window: counting
 * yesterday's cache as proof that a search still exists would defeat the
 * point.
 */
let walkedSlugs: Set<string> | null = null;

/** A catalogue walk has started: begin collecting what it sends. */
export function beginCatalogueRefresh(): void {
  walkedSlugs = new Set();
}

/**
 * A catalogue walk has ended.
 *
 * Only a walk that ran to the end knows what no longer exists, so only a
 * complete one prunes. A stopped walk read a prefix of the catalogue; pruning
 * on that would make "stop" mean "delete most of the list".
 */
export function finishCatalogueRefresh(complete: boolean): void {
  const seen = walkedSlugs;
  walkedSlugs = null;
  if (!complete || seen === null) return;
  // The list on screen is now the server's answer, so the date it carries may
  // move. A stopped walk read only a prefix and does not earn a new date.
  markCatalogueRefreshed();
  projectsStore.update((current) =>
    current.filter((project) => seen.has(project.slug)),
  );
}

export function appendProjectsChunk(chunk: LizaProjectSummaryDto[]) {
  if (chunk.length === 0) return;
  if (walkedSlugs !== null) {
    for (const project of chunk) walkedSlugs.add(project.slug);
  }
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

/**
 * Measured height of the download progress panel, or 0 when it is not on
 * screen. Written by `DownloadPopup`, read by the layout so the toaster can
 * step above it — see `$lib/toast-offset` for why the panel is the one that
 * stays put.
 */
export const downloadPopupHeight = writable(0);

/** ID of the currently-active bundle download, if any. */
export const activeDownloadId = writable<string | null>(null);

/**
 * Close out the download the panel is showing.
 *
 * Both download paths emit `download-finished` with their id; the layout
 * routes it here. Before that, nothing cleared `activeDownloadId` at all, so
 * the panel came back with a finished download's rows the next time anything
 * made the app busy. Ignoring a stale id matters: a bundle download that ends
 * while a map download is running must not close the map's panel.
 */
export function finishDownload(downloadId: string): boolean {
  let matched = false;
  activeDownloadId.update((current) => {
    if (current !== downloadId) return current;
    matched = true;
    return null;
  });
  if (matched) {
    downloadProgress.set(new Map());
    currentDownload.set(null);
    bundleProgress.set(null);
  }
  return matched;
}

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
 * Apply a `bundle-progress` event, keeping what the new phase does not restate.
 *
 * Only the downloading phase reports byte totals; extracting and indexing
 * report a message and nothing else. Overwriting wholesale therefore blanked
 * "43 MiB of 120 MiB" the moment the last file landed, which is exactly when
 * the operator looks at it. A total that has not changed is still true.
 */
export function applyBundleProgress(payload: BundleProgressPayload): void {
  bundleProgress.update((previous) => {
    if (previous === null || previous.download_id !== payload.download_id) {
      return payload;
    }
    return {
      ...payload,
      completed: payload.completed ?? previous.completed,
      total: payload.total ?? previous.total,
      downloaded_bytes: payload.downloaded_bytes ?? previous.downloaded_bytes,
      total_bytes: payload.total_bytes ?? previous.total_bytes,
    };
  });
}

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

/**
 * The on-map measuring tool: click points, read the running distance.
 *
 * ADR-0020 puts distance measurement in the MVP. Its entry point is the
 * command palette — `ui-shell` requires the mode chips above the canvas to
 * stay inert scaffolding, and `product-scope` names the palette as a place a
 * workspace action may live.
 *
 * The points are scratch: they are not a track, they are never saved, and they
 * go when the tool is switched off. A measurement a crew wants to keep is a
 * track, which they can already draw.
 */
export const measuringActive = writable(false);
export const measuredPoints = writable<{ lat: number; lon: number }[]>([]);

/**
 * What the tape is measuring: a length, or an area.
 *
 * Two tools, not one readout carrying both. Owner, 2026-09-23: "часто надо
 * померить длину" — how long is this ride, how far from the road to the
 * stream — and for an open path the enclosed area is not a small number, it
 * is a meaningless one. Showing it beside the length invites reading it.
 *
 * The area mode closes the shape and reports what it encloses, with the
 * perimeter beside it, because a sector is described by both: "прочесать 2.4
 * км², обойти по кромке 6 км".
 */
export type MeasureMode = "distance" | "area";
export const measureMode = writable<MeasureMode>("distance");

/**
 * The radius ring: click a centre, click again to set the radius.
 *
 * A search draws these constantly — everything within five hundred metres of
 * the last known position — and it is the other on-map tool ADR-0020 declares.
 * Scratch like the tape: not a project object, gone when the tool goes off.
 */
export const ringActive = writable(false);
export const ringCentre = writable<{ lat: number; lon: number } | null>(null);
export const ringRadiusKm = writable(0);

/**
 * Turn one tool on and the other off.
 *
 * The two take the same click, so they cannot both be listening: a click meant
 * for the ring's centre must not also extend the tape. One function so that
 * invariant lives in one place rather than in each caller.
 */
export function setMeasuring(
  active: boolean,
  mode: MeasureMode = "distance",
): void {
  measuredPoints.set([]);
  measureMode.set(mode);
  measuringActive.set(active);
  if (active) {
    setRingOff();
    setProjectionOff();
  }
}

export function setRing(active: boolean): void {
  ringCentre.set(null);
  ringRadiusKm.set(0);
  ringActive.set(active);
  if (active) {
    measuredPoints.set([]);
    measuringActive.set(false);
    setProjectionOff();
  }
}

/**
 * Placing a waypoint by projection: a point, an azimuth and a distance.
 *
 * The third on-map tool `product-scope` declares, and the one that is not
 * click-driven — "from the task point, 240° and 1.2 km" is dictated over a
 * radio, not pointed at. So it takes an origin by click and the rest by
 * typing, and shows where the result will land before it is committed.
 */
export const projectionActive = writable(false);
export const projectionOrigin = writable<{ lat: number; lon: number } | null>(
  null,
);
export const projectionBearing = writable(0);
export const projectionDistanceM = writable(0);

export function setProjection(active: boolean): void {
  projectionOrigin.set(null);
  projectionActive.set(active);
  if (active) {
    measuredPoints.set([]);
    measuringActive.set(false);
    setRingOff();
  }
}

function setProjectionOff(): void {
  projectionOrigin.set(null);
  projectionActive.set(false);
}

function setRingOff(): void {
  ringCentre.set(null);
  ringRadiusKm.set(0);
  ringActive.set(false);
}
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
  | { kind: "all-data"; nonce: number }
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

/**
 * Centre the map on a track point — the same request, since what the map needs
 * is a pair of coordinates and it does not care what kind of thing they name.
 *
 * Used when stepping through a recording point by point: a walkthrough that
 * left the map where it was would be a list, not a walkthrough.
 */
export const focusTrackPoint = requestWaypointFocus;

/**
 * Frame everything the project holds — tracks and waypoints alike.
 *
 * Asked for after an import and after a project is opened. A project whose
 * only content is waypoints (a headquarters and a drop-off point) has to be
 * framed too, which is why this is not "all tracks" any more.
 */
export function requestAllDataFocus(): void {
  mapFocusNonce += 1;
  mapFocusRequest.set({ kind: "all-data", nonce: mapFocusNonce });
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
 * Where the operator was in the bundle loader when it last closed.
 *
 * On the workspace route the loader lives inside a Sheet, and a Sheet unmounts
 * its content. Everything the component held went with it: the search text,
 * the "only downloaded" toggle, the selected project, the contents they had
 * unchecked, and the place in a thirteen-thousand-row list. Closing the loader
 * to look at the map — the ordinary thing to do — meant finding the search
 * again by name afterwards.
 *
 * It is deliberately session state, not persisted: a crew opening the app
 * tomorrow starts on today's search, not on yesterday's.
 */
export interface BundleLoaderView {
  filter: string;
  onlyCached: boolean;
  selectedSlug: string;
  skipped: Record<string, true>;
  scrollTop: number;
}

export const bundleLoaderView = writable<BundleLoaderView>({
  filter: "",
  onlyCached: false,
  selectedSlug: "",
  skipped: {},
  scrollTop: 0,
});
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
  /** Metres — what the slider shows. The algorithm works in kilometres and
   * the conversion belongs on the Rust side of the boundary, where the
   * parameter is named for its unit. */
  toleranceM: number;
  preview: SimplifiedPreview | null;
}>({
  active: false,
  layerId: BigInt(0),
  trackId: BigInt(0),
  toleranceM: 10,
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
