import { invoke } from "@tauri-apps/api/core";
import type {
  AppStateDto,
  OziMetadataDto,
  TrackDetail,
  WaypointData,
  SimplifiedPreview,
} from "./types";

export async function getAppState(): Promise<AppStateDto> {
  return invoke("get_app_state");
}

export async function getTracksGeojson(): Promise<GeoJSON.FeatureCollection> {
  return invoke("get_tracks_geojson");
}

export async function loadProjects(): Promise<void> {
  return invoke("load_projects");
}

export async function loadProject(slug: string): Promise<void> {
  return invoke("load_project", { slug });
}

export async function openSelectedMap(mapName: string): Promise<void> {
  return invoke("open_selected_map", { mapName });
}

export async function openLocalBundle(dir: string): Promise<void> {
  return invoke("open_local_bundle", { dir });
}

export async function setBundlesRoot(path: string): Promise<void> {
  return invoke("set_bundles_root", { path });
}

export async function saveProject(path: string): Promise<void> {
  return invoke("save_project", { path });
}

export async function loadProjectFile(path: string): Promise<void> {
  return invoke("load_project_file", { path });
}

export async function importGpx(path: string): Promise<string> {
  return invoke("import_gpx", { path });
}

export async function importPlt(path: string): Promise<string> {
  return invoke("import_plt", { path });
}

export async function exportGpx(layerId: bigint, path: string): Promise<void> {
  return invoke("export_gpx", { layerId, path });
}

export async function getTrackExportDefaultPath(
  trackName: string,
  extension: "gpx" | "plt"
): Promise<string | null> {
  return invoke("get_track_export_default_path", { trackName, extension });
}

export async function exportTrackPlt(
  layerId: bigint,
  trackId: bigint,
  path: string
): Promise<void> {
  return invoke("export_track_plt", { layerId, trackId, path });
}

export async function exportWptWaypoints(
  layerId: bigint,
  path: string
): Promise<void> {
  return invoke("export_wpt_waypoints", { layerId, path });
}

export async function getWptExportDefaultPath(
  layerId: bigint
): Promise<string | null> {
  return invoke("get_wpt_export_default_path", { layerId });
}

export async function undo(): Promise<void> {
  return invoke("undo");
}

export async function redo(): Promise<void> {
  return invoke("redo");
}

export async function renameTrack(
  layerId: bigint,
  trackId: bigint,
  newName: string
): Promise<void> {
  return invoke("rename_track", { layerId, trackId, newName });
}

export async function setTrackColor(
  layerId: bigint,
  trackId: bigint,
  color: [number, number, number, number]
): Promise<void> {
  return invoke("set_track_color", { layerId, trackId, color });
}

export async function toggleTrackVisible(
  layerId: bigint,
  trackId: bigint
): Promise<void> {
  return invoke("toggle_track_visible", { layerId, trackId });
}

export async function revealBundle(): Promise<void> {
  return invoke("reveal_bundle");
}

export async function getOziMetadata(mapPath: string): Promise<OziMetadataDto> {
  return invoke("get_ozi_metadata", { mapPath });
}

/** Return raw tile bytes from SQLite bundle. Used by sqlite-protocol.ts. */
export async function getSqliteTile(
  path: string,
  baseZoom: number,
  z: number,
  x: number,
  y: number
): Promise<ArrayBuffer> {
  return invoke("get_sqlite_tile", { path, baseZoom, z, x, y });
}

/** Return PNG-encoded tile bytes from OZF2 file. Used by ozi-protocol.ts. */
export async function getOziTile(
  mapPath: string,
  level: number,
  tileX: number,
  tileY: number
): Promise<ArrayBuffer> {
  return invoke("get_ozi_tile", { mapPath, level, tileX, tileY });
}

/** Return a 256×256 PNG for Web Mercator tile (tx, ty, tz) reprojected from OZF2. */
export async function getOziTileProjected(
  mapPath: string,
  tx: number,
  ty: number,
  tz: number,
): Promise<ArrayBuffer> {
  return invoke("get_ozi_tile_projected", { mapPath, tx, ty, tz });
}

export async function moveTrackPoint(
  layerId: bigint,
  trackId: bigint,
  segmentId: bigint,
  pointId: bigint,
  position: [number, number]
): Promise<void> {
  return invoke("move_track_point", { layerId, trackId, segmentId, pointId, position });
}

export async function deleteTrackPoint(
  layerId: bigint,
  trackId: bigint,
  segmentId: bigint,
  pointId: bigint
): Promise<void> {
  return invoke("delete_track_point", { layerId, trackId, segmentId, pointId });
}

export async function insertTrackPoint(
  layerId: bigint,
  trackId: bigint,
  segmentId: bigint,
  index: number,
  position: [number, number]
): Promise<void> {
  return invoke("insert_track_point", { layerId, trackId, segmentId, index, position });
}

export async function splitSegment(
  layerId: bigint,
  trackId: bigint,
  segmentId: bigint,
  pointId: bigint
): Promise<void> {
  return invoke("split_segment", { layerId, trackId, segmentId, pointId });
}

export async function joinSegments(
  layerId: bigint,
  trackId: bigint,
  segIdA: bigint,
  segIdB: bigint
): Promise<void> {
  return invoke("join_segments", { layerId, trackId, segIdA, segIdB });
}

export async function deleteTrack(layerId: bigint, trackId: bigint): Promise<void> {
  return invoke("delete_track", { layerId, trackId });
}

export async function deleteWaypoint(layerId: bigint, waypointId: bigint): Promise<void> {
  return invoke("delete_waypoint", { layerId, waypointId });
}

export async function renameWaypoint(
  layerId: bigint,
  waypointId: bigint,
  newName: string
): Promise<void> {
  return invoke("rename_waypoint", { layerId, waypointId, newName });
}

export async function setWaypointSymbol(
  layerId: bigint,
  waypointId: bigint,
  symbol: string | null
): Promise<void> {
  return invoke("set_waypoint_symbol", { layerId, waypointId, symbol });
}

export async function simplifyTrack(
  layerId: bigint,
  trackId: bigint,
  tolerance: number
): Promise<void> {
  return invoke("simplify_track", { layerId, trackId, tolerance });
}

export async function setTrackLineWidth(
  layerId: bigint,
  trackId: bigint,
  width: number
): Promise<void> {
  return invoke("set_track_line_width", { layerId, trackId, width });
}

export async function getTrackDetail(
  layerId: bigint,
  trackId: bigint
): Promise<TrackDetail> {
  return invoke("get_track_detail", { layerId, trackId });
}

export async function addWaypoint(
  layerId: bigint,
  lat: number,
  lon: number,
  name: string,
): Promise<void> {
  return invoke("add_waypoint", { layerId, lat, lon, name });
}

export async function moveWaypoint(
  layerId: bigint,
  waypointId: bigint,
  position: [number, number]
): Promise<void> {
  return invoke("move_waypoint", { layerId, waypointId, position });
}

export async function getWaypoints(layerId: bigint): Promise<WaypointData[]> {
  return invoke("get_waypoints", { layerId });
}

export async function getSimplifiedPreview(
  layerId: bigint,
  trackId: bigint,
  tolerance: number
): Promise<SimplifiedPreview> {
  return invoke("get_simplified_preview", { layerId, trackId, tolerance });
}

export async function createEmptyTrack(
  layerId: bigint,
  name: string
): Promise<bigint> {
  return invoke("create_empty_track", { layerId, name });
}
