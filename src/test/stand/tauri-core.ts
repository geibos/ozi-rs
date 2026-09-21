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
import { standEmit } from "./tauri-event";
import { playBundleDownload } from "./download-script";
import {
  appStateFixture,
  coldStartFixture,
  trackDetailFixture,
  tracksGeojsonFixture,
  waypointsFixture,
} from "../fixtures";

type Args = Record<string, unknown> | undefined;

/** A 1×1 transparent PNG, for tiles the stand cannot serve. */
const TRANSPARENT_PNG = Uint8Array.from([
  0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a, 0x00, 0x00, 0x00, 0x0d, 0x49,
  0x48, 0x44, 0x52, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x08, 0x06,
  0x00, 0x00, 0x00, 0x1f, 0x15, 0xc4, 0x89, 0x00, 0x00, 0x00, 0x0a, 0x49, 0x44,
  0x41, 0x54, 0x78, 0x9c, 0x63, 0x00, 0x01, 0x00, 0x00, 0x05, 0x00, 0x01, 0x0d,
  0x0a, 0x2d, 0xb4, 0x00, 0x00, 0x00, 0x00, 0x49, 0x45, 0x4e, 0x44, 0xae, 0x42,
  0x60, 0x82,
]).buffer;

/** Commands that change state: the stand accepts them and reports no data. */
/**
 * Commands whose real implementation finishes by emitting `state-changed`.
 *
 * Without it the stand leaves the app mid-flight: `load_projects` sets the
 * "refreshing…" hint, and only a state update clears it, so the hint sat there
 * forever and the first screen always looked like it was still loading.
 */
const EMITS_STATE_CHANGED = new Set([
  "load_projects",
  "preview_project",
  "set_bundles_root",
]);

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
  "export_all_tracks_gpx",
  "export_track_plt",
  "export_gpx_waypoints",
  "export_wpt_waypoints",
]);

/**
 * The layers the fixture actually populated. A handler that ignores its
 * arguments is a mock that lies: answering every layer with the same
 * waypoints listed each of them twice in the rail, which looked exactly like
 * an app defect until the transcript showed two calls with different ids.
 */
const FIXTURE_TRACK_LAYER = 1;
const FIXTURE_WAYPOINT_LAYER = 1;
const FIXTURE_TRACK = 1;

/**
 * Which state the stand serves, from the URL: `?state=cold` opens the app as a
 * crew first sees it, with the catalogue loaded and nothing open. Without it
 * the workspace always wins, because the default fixture has an active map and
 * the cold-start route redirects.
 */
function requestedState(): "cold" | "workspace" {
  if (typeof location === "undefined") return "workspace";
  return new URLSearchParams(location.search).get("state") === "cold"
    ? "cold"
    : "workspace";
}

const HANDLERS: Record<string, (args: Args) => unknown> = {
  get_app_state: () =>
    requestedState() === "cold" ? coldStartFixture : appStateFixture,
  get_tracks_geojson: () => tracksGeojsonFixture,
  get_track_detail: (args) =>
    args?.layerId === FIXTURE_TRACK_LAYER && args?.trackId === FIXTURE_TRACK
      ? trackDetailFixture
      : { id: args?.trackId ?? 0, name: "", segments: [] },
  get_waypoints: (args) =>
    args?.layerId === FIXTURE_WAYPOINT_LAYER ? waypointsFixture : [],
  get_track_export_default_path: () => null,
  get_wpt_export_default_path: () => null,
  get_waypoints_export_default_path: () => null,
  get_simplified_preview: () => ({ points: [], removed: 0 }),
  // The stand has no tile store. A transparent 1×1 PNG stands in, so the map
  // shows the basemap instead of a wall of error toasts — cartographic
  // fidelity is out of scope here and says so in the README.
  get_sqlite_tile: () => TRANSPARENT_PNG,
  get_ozi_tile: () => TRANSPARENT_PNG,
  open_selected_map: () => "",
  // Pressing the download button plays the event sequence a real bundle
  // download emits, so the panel, the "map is ready" announcement and the
  // panel closing can be looked at without a network.
  load_project: () => {
    const id = `stand-${Date.now()}`;
    playBundleDownload(id);
    return id;
  },
  open_local_bundle: () => "",
  cancel_download: () => true,
};

/** Commands the stand answered, in order — a screen's IPC transcript. */
export const standCalls: Array<{ command: string; args: Args }> = [];

// The transcript on `window.__stand`, so a stand session can see what a screen
// asked for from the console — which is how "the button does nothing" turns
// into "the command never fired" without guessing.
if (typeof window !== "undefined") {
  (window as unknown as Record<string, unknown>).__stand = {
    calls: standCalls,
  };
}

export async function invoke<T>(command: string, args?: Args): Promise<T> {
  standCalls.push({ command, args });

  if (EMITS_STATE_CHANGED.has(command)) {
    queueMicrotask(() => standEmit("state-changed", undefined));
  }

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
