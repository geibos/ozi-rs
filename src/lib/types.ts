// DTOs mirroring Rust structs in src-tauri/src/commands/mod.rs

export interface DiagnosticDto {
  level: "info" | "error";
  message: string;
}

export interface LizaProjectSummaryDto {
  slug: string;
  name: string;
}

export interface LizaMapPackageDto {
  name: string;
  base_zoom: number;
  downloaded: boolean;
}

export interface LizaProjectDto {
  name: string;
  center_lat: number;
  center_lon: number;
  maps: LizaMapPackageDto[];
}

export interface ActiveMapDto {
  kind: "sqlite" | "ozi";
  project_name: string;
  package_name: string;
  local_path: string;
  center_lat: number;
  center_lon: number;
  base_zoom: number;
}

export interface AppStateDto {
  project_name: string;
  project_saved: boolean;
  status: string;
  busy: boolean;
  downloading_maps: string[];
  projects: LizaProjectSummaryDto[];
  current_project: LizaProjectDto | null;
  active_map: ActiveMapDto | null;
  diagnostics: DiagnosticDto[];
  track_layer_count: number;
  waypoint_layer_count: number;
}

export interface OziLevelDto {
  level_index: number;
  width: number;
  height: number;
  tile_width: number;
  tile_height: number;
  tile_columns: number;
  tile_rows: number;
}

export interface OziMetadataDto {
  map_path: string;
  title: string;
  projection: string;
  datum: string;
  calibration_points: string[];
  levels: OziLevelDto[];
}

export interface DownloadProgressPayload {
  package_name: string;
  downloaded_bytes: number;
  total_bytes?: number;
}
