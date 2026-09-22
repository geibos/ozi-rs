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
import { playBundleDownload, playMapDownload } from "./download-script";
import {
  appStateFixture,
  coldStartFixture,
  trackDetailFixture,
  tracksGeojsonFixture,
  tracksListFixture,
  catalogueFixture,
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
const EMITS_STATE_CHANGED = new Set(["preview_project", "set_bundles_root"]);

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
  "set_waypoint_color",
  "delete_track",
  "delete_waypoint",
  "cancel_drawing",
  "undo",
  "redo",
  "save_project",
  "set_bundles_root",
  "reveal_bundle",
  "cancel_project_listing",
  "trim_track_at_point",
  "export_gpx",
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
/**
 * Which failure the stand should play, from the URL: `?fail=download` ends the
 * download in an error, `?fail=catalogue` makes the listing unreachable. An
 * error state is a screen too, and it had never been looked at.
 */
const FAILURE: string | null =
  typeof location === "undefined"
    ? null
    : new URLSearchParams(location.search).get("fail");

// Read once, at load: the app navigates between its own routes and would
// otherwise lose the flag the moment it did.
function requestedFailure(): string | null {
  return FAILURE;
}

/** The last path component, which is what an import summary names. */
function standFileLabel(path: unknown, fallback: string): string {
  if (typeof path !== "string" || path === "") return fallback;
  return path.split("/").filter(Boolean).pop() ?? fallback;
}

function requestedState(): "cold" | "workspace" {
  if (typeof location === "undefined") return "workspace";
  return new URLSearchParams(location.search).get("state") === "cold"
    ? "cold"
    : "workspace";
}

/**
 * The project the stand is currently previewing.
 *
 * `preview_project` used to be answered with a bare "accepted", so the loader
 * asked for a project, got a state-changed event, and found the same project
 * still there — the pending hint could never clear and the screen sat
 * spinning. The slug is what the loader matches on, so the stand has to move
 * it.
 */
/**
 * What an import has added during this stand session.
 *
 * The stand serves fixtures, so an import used to be unanswerable and threw.
 * Now that a file dialog can answer with a path, the import flow is walkable —
 * but only if importing visibly changes the screen, which is the whole thing
 * an operator is checking. These rows are appended to the fixture's, and a
 * reload clears them, as a fresh launch would.
 */
const importedTracks: Array<Record<string, unknown>> = [];
let importedLayerId = 10;

function importOneLayer(label: string, trackCount: number): string {
  importedLayerId += 1;
  for (let i = 0; i < trackCount; i += 1) {
    importedTracks.push({
      layer_id: importedLayerId,
      track_id: importedTracks.length + 100,
      name: `${label} ${i + 1}`,
      color: "rgba(37,99,235,1.000)",
      line_width: 3.0,
      visible: true,
      point_count: 42,
      distance_km: 3.7,
      duration_seconds: 5400,
    });
  }
  standEmit("state-changed", undefined);
  return `Imported ${trackCount} tracks from ${label}`;
}

let previewedSlug: string | null = null;

function previewedAppState(): unknown {
  const base = requestedState() === "cold" ? coldStartFixture : appStateFixture;
  if (previewedSlug === null) return base;
  const state = base as { current_project?: { slug: string } | null };
  if (!state.current_project) return base;
  return {
    ...base,
    current_project: { ...state.current_project, slug: previewedSlug },
  };
}

const HANDLERS: Record<string, (args: Args) => unknown> = {
  get_app_state: () => previewedAppState(),
  get_tracks_geojson: () => tracksGeojsonFixture,
  // The rows the Tracks tab reads — its own fixture, not the map's features.
  // Deriving them from the geometry would have made the stand inherit the very
  // omission this listing exists to undo.
  list_tracks: () => [...tracksListFixture, ...importedTracks],
  get_track_detail: (args) =>
    args?.layerId === FIXTURE_TRACK_LAYER && args?.trackId === FIXTURE_TRACK
      ? trackDetailFixture
      : { id: args?.trackId ?? 0, name: "", segments: [] },
  get_waypoints: (args) =>
    args?.layerId === FIXTURE_WAYPOINT_LAYER ? waypointsFixture : [],
  get_track_export_default_path: () => null,
  get_wpt_export_default_path: () => null,
  get_waypoints_export_default_path: () => null,
  // The real shape, not an invented one. This answered `{points, removed}`
  // while `SimplifiedPreviewDto` is `{original_count, simplified_count,
  // segments}`, so opening the simplify dialog threw
  // `Cannot read properties of undefined (reading 'map')` inside MapView and
  // the dialog showed two empty numbers. A stub with the wrong shape is the
  // same failure as a stub that returns `undefined`, wearing a hat.
  get_simplified_preview: () => {
    const segments = trackDetailFixture.segments.map((segment) => {
      const kept = segment.points.filter((_, index) => index % 2 === 0);
      return {
        id: segment.id,
        original_count: segment.points.length,
        simplified_count: kept.length,
        kept_points: kept,
      };
    });
    return {
      original_count: segments.reduce((n, s) => n + s.original_count, 0),
      simplified_count: segments.reduce((n, s) => n + s.simplified_count, 0),
      segments,
    };
  },
  // The stand has no tile store. A transparent 1×1 PNG stands in, so the map
  // shows the basemap instead of a wall of error toasts — cartographic
  // fidelity is out of scope here and says so in the README.
  get_sqlite_tile: () => TRANSPARENT_PNG,
  get_ozi_tile: () => TRANSPARENT_PNG,
  // A map already on disk opens with no download id; one that is not starts a
  // download and returns its id, which is what the caller needs in order to
  // show the panel and offer a cancel.
  open_selected_map: (args) => {
    const mapName = String(args?.mapName ?? "");
    const map = (appStateFixture.current_project?.maps ?? []).find(
      (m) => m.name === mapName,
    );
    if (!map || map.downloaded) return "";
    const id = `stand-map-${Date.now()}`;
    playMapDownload(id, mapName);
    return id;
  },
  // Pressing the download button plays the event sequence a real bundle
  // download emits, so the panel, the "map is ready" announcement and the
  // panel closing can be looked at without a network.
  load_project: () => {
    const id = `stand-${Date.now()}`;
    playBundleDownload(id, requestedFailure() === "download");
    return id;
  },
  // Answers with counts, not "accepted": the Tracks tab reads them into the
  // toast, and a stub that returned nothing crashed on `.tracks`.
  export_all_tracks_gpx: () => ({
    tracks: tracksListFixture.length,
    waypoints: waypointsFixture.length,
  }),
  // Imports: the command answers with the summary the Tracks tab shows, and
  // the rows appear. Without an answer here the stand threw, which is correct
  // for a command nobody has thought about and wrong for the first thing a
  // crew does with a day's recordings.
  import_gpx: (args) =>
    importOneLayer(standFileLabel(args?.path, "20260708_Veter2.gpx"), 1),
  import_plt: (args) =>
    importOneLayer(standFileLabel(args?.path, "20260708_Veter2.plt"), 1),
  import_tracks_directory: (args) => {
    const label = standFileLabel(args?.path, "20260921");
    importOneLayer(label, 3);
    // Counts, not a sentence: the interface does the wording now. One file is
    // reported unreadable, because the caveat branch is a screen too.
    return { files: 4, tracks: 3, waypoints: 2, skipped: ["ЛИСА17.plt"] };
  },
  // Opening a saved project: the stand has one project, so this reports
  // success and leaves the fixture in place. What the flow is checked for is
  // the framing, the recents and the toast, all of which are frontend.
  load_project_file: () => {
    standEmit("state-changed", undefined);
    return null;
  },
  open_local_bundle: () => "",
  // The catalogue arrives as a stream, so the stand sends one: the cached
  // chunk first, then the walk's boundaries around the walk's own chunk. The
  // interface prunes on a complete walk, and a stand that skipped the
  // boundaries would never exercise that.
  load_projects: () => {
    standEmit("projects-chunk", catalogueFixture);
    queueMicrotask(() => {
      standEmit("catalogue-refresh-started", undefined);
      standEmit("projects-chunk", catalogueFixture);
      standEmit("catalogue-refresh-finished", { complete: true });
      standEmit("state-changed", undefined);
    });
    return null;
  },
  preview_project: (args) => {
    previewedSlug = typeof args?.slug === "string" ? args.slug : null;
    return null;
  },
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

  if (
    requestedFailure() === "catalogue" &&
    (command === "load_projects" || command === "preview_project")
  ) {
    throw new Error(
      "bundle listing unreachable and not cached: error sending request for url (https://maps.lizaalert.ru/maps/)",
    );
  }

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
