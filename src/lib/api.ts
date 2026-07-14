import { invokeIpc } from "./ipc";
import type {
  AppStateDto,
  OziMetadataDto,
  TrackDetail,
  WaypointData,
  SimplifiedPreview,
} from "./types";

// Every IPC call goes through `invokeIpc` (see `src/lib/ipc.ts`) so that
// rejections surface as a sticky `data-testid="ipc-error"` toast in dev
// builds. In production the wrapper is transparent. User-facing
// `toast.error("Failed to …")` calls at call sites continue to render.

export async function getAppState(): Promise<AppStateDto> {
  return invokeIpc("get_app_state");
}

export async function getTracksGeojson(): Promise<GeoJSON.FeatureCollection> {
  return invokeIpc("get_tracks_geojson");
}

export async function loadProjects(): Promise<void> {
  return invokeIpc("load_projects");
}

/**
 * Begin downloading a LizaAlert project bundle.
 *
 * Returns immediately with a `download_id` that callers use to correlate
 * `download-progress` / `bundle-file-ready` / `bundle-progress` events and
 * to cancel via {@link cancelDownload}. The actual download proceeds on a
 * background task — the returned promise resolves once the Tauri command
 * handler returns, NOT once the bundle has finished downloading. Callers
 * MUST NOT `await` the download via this promise; subscribe to events
 * instead so the main thread stays responsive.
 */
export async function loadProject(slug: string): Promise<string> {
  return invokeIpc("load_project", { slug });
}

/**
 * Abort an in-flight bundle download. Returns `true` if the download was
 * known and a cancel signal was delivered; files already on disk remain
 * untouched and a subsequent {@link loadProject} for the same bundle will
 * resume by fetching only the missing files.
 */
export async function cancelDownload(downloadId: string): Promise<boolean> {
  return invokeIpc("cancel_download", { downloadId });
}

export async function openSelectedMap(mapName: string): Promise<void> {
  return invokeIpc("open_selected_map", { mapName });
}

export async function openLocalBundle(dir: string): Promise<string> {
  return invokeIpc("open_local_bundle", { dir });
}

export async function setBundlesRoot(path: string): Promise<void> {
  return invokeIpc("set_bundles_root", { path });
}

export async function saveProject(path: string): Promise<void> {
  return invokeIpc("save_project", { path });
}

export async function loadProjectFile(path: string): Promise<void> {
  return invokeIpc("load_project_file", { path });
}

export async function importGpx(path: string): Promise<string> {
  return invokeIpc("import_gpx", { path });
}

export async function importPlt(path: string): Promise<string> {
  return invokeIpc("import_plt", { path });
}

export async function exportGpx(layerId: bigint, path: string): Promise<void> {
  return invokeIpc("export_gpx", { layerId, path });
}

export async function getTrackExportDefaultPath(
  trackName: string,
  extension: "gpx" | "plt",
): Promise<string | null> {
  return invokeIpc("get_track_export_default_path", { trackName, extension });
}

export async function exportTrackPlt(
  layerId: bigint,
  trackId: bigint,
  path: string,
): Promise<void> {
  return invokeIpc("export_track_plt", { layerId, trackId, path });
}

export async function exportWptWaypoints(
  layerId: bigint,
  path: string,
): Promise<void> {
  return invokeIpc("export_wpt_waypoints", { layerId, path });
}

export async function getWptExportDefaultPath(
  layerId: bigint,
): Promise<string | null> {
  return invokeIpc("get_wpt_export_default_path", { layerId });
}

export async function undo(): Promise<void> {
  return invokeIpc("undo");
}

export async function redo(): Promise<void> {
  return invokeIpc("redo");
}

export async function renameTrack(
  layerId: bigint,
  trackId: bigint,
  newName: string,
): Promise<void> {
  return invokeIpc("rename_track", { layerId, trackId, newName });
}

export async function setTrackColor(
  layerId: bigint,
  trackId: bigint,
  color: [number, number, number, number],
): Promise<void> {
  return invokeIpc("set_track_color", { layerId, trackId, color });
}

export async function toggleTrackVisible(
  layerId: bigint,
  trackId: bigint,
): Promise<void> {
  return invokeIpc("toggle_track_visible", { layerId, trackId });
}

export async function toggleWaypointVisible(
  layerId: bigint,
  waypointId: bigint,
): Promise<void> {
  return invokeIpc("toggle_waypoint_visible", { layerId, waypointId });
}

export async function revealBundle(): Promise<void> {
  return invokeIpc("reveal_bundle");
}

export async function getOziMetadata(mapPath: string): Promise<OziMetadataDto> {
  return invokeIpc("get_ozi_metadata", { mapPath });
}

/** Return raw tile bytes from SQLite bundle. Used by sqlite-protocol.ts. */
export async function getSqliteTile(
  path: string,
  baseZoom: number,
  z: number,
  x: number,
  y: number,
): Promise<ArrayBuffer> {
  return invokeIpc("get_sqlite_tile", { path, baseZoom, z, x, y });
}

/** Return PNG-encoded tile bytes from OZF2 file. Used by ozi-protocol.ts. */
export async function getOziTile(
  mapPath: string,
  level: number,
  tileX: number,
  tileY: number,
): Promise<ArrayBuffer> {
  return invokeIpc("get_ozi_tile", { mapPath, level, tileX, tileY });
}

/** Return a 256×256 PNG for Web Mercator tile (tx, ty, tz) reprojected from OZF2. */
export async function getOziTileProjected(
  mapPath: string,
  tx: number,
  ty: number,
  tz: number,
): Promise<ArrayBuffer> {
  return invokeIpc("get_ozi_tile_projected", { mapPath, tx, ty, tz });
}

export async function moveTrackPoint(
  layerId: bigint,
  trackId: bigint,
  segmentId: bigint,
  pointId: bigint,
  position: [number, number],
): Promise<void> {
  return invokeIpc("move_track_point", {
    layerId,
    trackId,
    segmentId,
    pointId,
    position,
  });
}

export async function deleteTrackPoint(
  layerId: bigint,
  trackId: bigint,
  segmentId: bigint,
  pointId: bigint,
): Promise<void> {
  return invokeIpc("delete_track_point", {
    layerId,
    trackId,
    segmentId,
    pointId,
  });
}

export async function insertTrackPoint(
  layerId: bigint,
  trackId: bigint,
  segmentId: bigint,
  index: number,
  position: [number, number],
): Promise<void> {
  return invokeIpc("insert_track_point", {
    layerId,
    trackId,
    segmentId,
    index,
    position,
  });
}

export async function splitSegment(
  layerId: bigint,
  trackId: bigint,
  segmentId: bigint,
  pointId: bigint,
): Promise<void> {
  return invokeIpc("split_segment", { layerId, trackId, segmentId, pointId });
}

export async function joinSegments(
  layerId: bigint,
  trackId: bigint,
  segIdA: bigint,
  segIdB: bigint,
): Promise<void> {
  return invokeIpc("join_segments", { layerId, trackId, segIdA, segIdB });
}

export async function deleteTrack(
  layerId: bigint,
  trackId: bigint,
): Promise<void> {
  return invokeIpc("delete_track", { layerId, trackId });
}

export async function deleteWaypoint(
  layerId: bigint,
  waypointId: bigint,
): Promise<void> {
  return invokeIpc("delete_waypoint", { layerId, waypointId });
}

export async function renameWaypoint(
  layerId: bigint,
  waypointId: bigint,
  newName: string,
): Promise<void> {
  return invokeIpc("rename_waypoint", { layerId, waypointId, newName });
}

export async function setWaypointSymbol(
  layerId: bigint,
  waypointId: bigint,
  symbol: string | null,
): Promise<void> {
  return invokeIpc("set_waypoint_symbol", { layerId, waypointId, symbol });
}

export async function simplifyTrack(
  layerId: bigint,
  trackId: bigint,
  tolerance: number,
): Promise<void> {
  return invokeIpc("simplify_track", { layerId, trackId, tolerance });
}

export async function setTrackLineWidth(
  layerId: bigint,
  trackId: bigint,
  width: number,
): Promise<void> {
  return invokeIpc("set_track_line_width", { layerId, trackId, width });
}

export async function getTrackDetail(
  layerId: bigint,
  trackId: bigint,
): Promise<TrackDetail> {
  return invokeIpc("get_track_detail", { layerId, trackId });
}

export async function addWaypoint(
  layerId: bigint,
  lat: number,
  lon: number,
  name: string,
): Promise<void> {
  return invokeIpc("add_waypoint", { layerId, lat, lon, name });
}

export async function moveWaypoint(
  layerId: bigint,
  waypointId: bigint,
  position: [number, number],
): Promise<void> {
  return invokeIpc("move_waypoint", { layerId, waypointId, position });
}

export async function getWaypoints(layerId: bigint): Promise<WaypointData[]> {
  return invokeIpc("get_waypoints", { layerId });
}

export async function getSimplifiedPreview(
  layerId: bigint,
  trackId: bigint,
  tolerance: number,
): Promise<SimplifiedPreview> {
  return invokeIpc("get_simplified_preview", { layerId, trackId, tolerance });
}

export async function createEmptyTrack(
  layerId: bigint,
  name: string,
): Promise<bigint> {
  // The wire value is a JSON number (serde-serialized u64), not a bigint —
  // convert at the boundary so the declared return type holds and strict
  // comparisons against bigint stores (e.g. `$selectedTrackId === trackId`)
  // behave correctly.
  const trackId = await invokeIpc<number>("create_empty_track", {
    layerId,
    name,
  });
  return BigInt(trackId);
}
