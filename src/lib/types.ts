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

export interface LayerSummaryDto {
  id: number;
  name: string;
  // Optional per-layer visibility flag. When absent (current backend),
  // the layer is treated as visible. Reserved for future per-layer toggling.
  visible?: boolean;
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
  track_layers: LayerSummaryDto[];
  waypoint_layers: LayerSummaryDto[];
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
  bounds: [number, number, number, number] | null;
  native_zoom: number;
  min_zoom: number;
}

export interface DownloadProgressPayload {
  package_name: string;
  downloaded_bytes: number;
  total_bytes?: number;
}

export interface BundleProgressPayload {
  message: string;
  phase: "scanning" | "downloading" | "extracting" | "indexing";
  completed?: number;
  total?: number;
  downloaded_bytes?: number;
  total_bytes?: number;
}

export interface PointDetail {
  id: number;
  lat: number;
  lon: number;
  elevation?: number;
  timestamp?: string;
}

export interface SegmentDetail {
  id: number;
  points: PointDetail[];
}

export interface TrackDetail {
  id: number;
  name: string;
  segments: SegmentDetail[];
}

export interface WaypointData {
  id: number;
  name: string;
  lat: number;
  lon: number;
  symbol?: string;
  // Optional per-waypoint visibility flag. When absent (current backend),
  // the waypoint is treated as visible. Reserved for future per-waypoint toggling.
  visible?: boolean;
}

export interface SimplifiedSegmentPreview {
  id: number;
  original_count: number;
  simplified_count: number;
  kept_points: PointDetail[];
}

export interface SimplifiedPreview {
  original_count: number;
  simplified_count: number;
  segments: SimplifiedSegmentPreview[];
}
