import { invoke } from "@tauri-apps/api/core";
import type {
  AppStateDto,
  OziMetadataDto,
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
