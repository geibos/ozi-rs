/**
 * The stand's replacement for `@tauri-apps/api/core`.
 *
 * `just stand` serves the real frontend in a browser with this module aliased
 * over the Tauri transport, so a screen can be opened, looked at and
 * photographed without building the app, without a backend and without
 * Appium — which is what every visual check has cost so far.
 *
 * The answers come from `src/test/fixtures`, which the Rust core writes
 * (`just fixtures`), so what the stand shows is what the app would receive.
 * A command with no answer here throws loudly rather than returning
 * `undefined`: a screen that renders because a mock quietly answered nothing
 * is exactly the kind of evidence this whole exercise exists to stop
 * producing.
 */
import {
  appStateFixture,
  trackDetailFixture,
  tracksGeojsonFixture,
  waypointsFixture,
} from "../fixtures";

type Args = Record<string, unknown> | undefined;

/** Commands that change state: the stand accepts them and reports no data. */
const ACCEPTED_WITHOUT_DATA = new Set([
  "set_all_tracks_visible",
  "show_only_track",
  "set_all_waypoints_visible",
  "show_only_waypoint",
  "toggle_track_visible",
  "toggle_waypoint_visible",
  "set_track_color",
  "set_track_line_width",
  "rename_track",
  "rename_waypoint",
  "set_waypoint_symbol",
  "delete_track",
  "delete_waypoint",
  "cancel_drawing",
  "undo",
  "redo",
  "save_project",
  "set_bundles_root",
  "reveal_bundle",
  "load_projects",
  "preview_project",
  "export_gpx",
  "export_track_plt",
  "export_gpx_waypoints",
  "export_wpt_waypoints",
]);

const HANDLERS: Record<string, (args: Args) => unknown> = {
  get_app_state: () => appStateFixture,
  get_tracks_geojson: () => tracksGeojsonFixture,
  get_track_detail: () => trackDetailFixture,
  get_waypoints: () => waypointsFixture,
  get_track_export_default_path: () => null,
  get_wpt_export_default_path: () => null,
  get_waypoints_export_default_path: () => null,
  get_simplified_preview: () => ({ points: [], removed: 0 }),
  // No OZF2 file exists on the stand, and the inspector already has a branch
  // for that — the honest answer is "no calibration metadata", not a
  // fabricated one.
  get_ozi_metadata: () => null,
  open_selected_map: () => "",
  load_project: () => "",
  open_local_bundle: () => "",
  cancel_download: () => true,
};

/** Commands the stand answered, in order — a screen's IPC transcript. */
export const standCalls: Array<{ command: string; args: Args }> = [];

export async function invoke<T>(command: string, args?: Args): Promise<T> {
  standCalls.push({ command, args });

  const handler = HANDLERS[command];
  if (handler) return handler(args) as T;
  if (ACCEPTED_WITHOUT_DATA.has(command)) return null as T;

  throw new Error(
    `stand: no fixture answers "${command}". Add it to src/test/stand/tauri-core.ts ` +
      `— an unanswered command must fail loudly, not return undefined.`,
  );
}

/** `Channel` exists in the real module; nothing in the app constructs one yet. */
export class Channel<T = unknown> {
  onmessage: ((message: T) => void) | null = null;
}
