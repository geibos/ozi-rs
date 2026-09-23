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
import type {
  AppStateDto,
  commands,
  JsonValue,
  LayerSummaryDto,
  TrackSummaryDto,
  WaypointDto,
} from "$lib/bindings";
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
const importedTracks: TrackSummaryDto[] = [];
let importedLayerId = 10;

/**
 * The colours the backend hands to imported tracks that declare none — the
 * same list and the same order as `TRACK_PALETTE` in
 * `src-tauri/src/application/import.rs`. A day of recordings is twenty crews,
 * and a stand that drew them all one colour would hide exactly the thing the
 * palette exists to show.
 */
const STAND_TRACK_PALETTE = [
  "rgba(220,38,38,1.000)",
  "rgba(37,99,235,1.000)",
  "rgba(217,119,6,1.000)",
  "rgba(147,51,234,1.000)",
  "rgba(8,145,178,1.000)",
  "rgba(219,39,119,1.000)",
  "rgba(120,53,15,1.000)",
  "rgba(30,64,175,1.000)",
  "rgba(190,24,93,1.000)",
  "rgba(126,34,206,1.000)",
  "rgba(161,98,7,1.000)",
  "rgba(15,118,110,1.000)",
];

/**
 * A route for an imported track, so the map shows what the list says.
 *
 * Without geometry the imported rows appeared in the list and nowhere else,
 * and the map is where a day of recordings is actually read — it is the only
 * place that answers "are twelve colours actually distinguishable on a
 * topographic basemap", which is a question a palette cannot be designed
 * without.
 */
function importedGeometry(index: number): number[][] {
  const north = 59.9524 + (index % 6) * 0.004;
  const west = 31.596 + Math.floor(index / 6) * 0.03;
  return Array.from({ length: 8 }, (_, step) => [
    west + step * 0.006,
    north + Math.sin((step + index) / 2) * 0.0025,
  ]);
}

function importOneLayer(label: string, trackCount: number): string {
  importedLayerId += 1;
  for (let i = 0; i < trackCount; i += 1) {
    importedTracks.push({
      layer_id: importedLayerId,
      track_id: importedTracks.length + 100,
      name: `${label} ${i + 1}`,
      color:
        STAND_TRACK_PALETTE[
          (tracksListFixture.length + importedTracks.length) %
            STAND_TRACK_PALETTE.length
        ],
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

/** Waypoints this stand session has placed, appended to the fixture's. */
const placedWaypoints: WaypointDto[] = [];

/**
 * The layer lists as this stand session has edited them.
 *
 * Layer management is the one thing an operator does to what the import made,
 * and until 2026-09-23 no command existed for it at all. `null` means the
 * session has not touched them and the fixture's lists stand.
 */
let standTrackLayers: LayerSummaryDto[] | null = null;
let standWaypointLayers: LayerSummaryDto[] | null = null;
let nextStandLayerId = 900;

function previewedAppState(): AppStateDto {
  const fixture =
    requestedState() === "cold" ? coldStartFixture : appStateFixture;
  // The imported rows belong in the state too, not only in `list_tracks`:
  // MapView redraws off a fingerprint taken from `AppStateDto.tracks`, so an
  // import that left this alone appeared in the list and never on the map.
  let base: AppStateDto = importedTracks.length
    ? { ...fixture, tracks: [...fixture.tracks, ...importedTracks] }
    : fixture;
  if (standTrackLayers || standWaypointLayers) {
    const trackLayers = standTrackLayers ?? base.track_layers;
    const waypointLayers = standWaypointLayers ?? base.waypoint_layers;
    base = {
      ...base,
      track_layers: trackLayers,
      waypoint_layers: waypointLayers,
      track_layer_count: trackLayers.length,
      waypoint_layer_count: waypointLayers.length,
      // A removed layer takes its rows with it, or the list would go on
      // showing tracks whose layer is gone — which is exactly the lie the
      // stand exists to catch.
      tracks: base.tracks.filter((row) =>
        trackLayers.some((layer) => layer.id === row.layer_id),
      ),
    };
  }
  const project = base.current_project;
  if (previewedSlug === null || !project) return base;
  // The whole project, with its slug moved: the loader matches on the slug, so
  // a partial object here would have been a project with nothing in it.
  return {
    ...base,
    current_project: { ...project, slug: previewedSlug },
  };
}

/**
 * A command name as the wire uses it, from the camelCase the bindings use.
 *
 * `getTracksGeojson` → `get_tracks_geojson`.
 */
type SnakeCase<S extends string> = S extends `${infer Head}${infer Tail}`
  ? Head extends Uppercase<Head>
    ? Head extends Lowercase<Head>
      ? `${Head}${SnakeCase<Tail>}`
      : `_${Lowercase<Head>}${SnakeCase<Tail>}`
    : `${Head}${SnakeCase<Tail>}`
  : S;

/** What a command answers with, unwrapped from the generated `Result`. */
type Answer<K extends keyof typeof commands> = Extract<
  Awaited<ReturnType<(typeof commands)[K]>>,
  { status: "ok" }
>["data"];

/**
 * The stand's answers, typed against the generated bindings.
 *
 * Two stubs have had the wrong shape: `export_all_tracks_gpx` answered nothing
 * where the caller reads two counts, and `get_simplified_preview` answered
 * `{points, removed}` where the DTO is `{original_count, simplified_count,
 * segments}` — which threw inside `MapView`. The README says a command with no
 * answer throws loudly because a mock returning `undefined` is the failure
 * this exercise exists to stop; a mock returning the wrong shape is that same
 * failure, and only the compiler catches it every time.
 *
 * The raw-byte tile commands are the exception: their binding says `number[]`
 * and the transport hands the app an `ArrayBuffer`, which is what the real
 * IPC does.
 */
type StandAnswers = {
  [K in keyof typeof commands as SnakeCase<K & string>]?: (
    args: Args,
  ) => Answer<K>;
} & {
  get_sqlite_tile: (args: Args) => ArrayBuffer;
  get_ozi_tile: (args: Args) => ArrayBuffer;
};

const HANDLERS: StandAnswers = {
  get_app_state: () => previewedAppState(),
  // The binding says `JsonValue` because the Rust side answers with dynamic
  // JSON; the fixture is a typed FeatureCollection, which is the stricter of
  // the two and what every reader here wants.
  get_tracks_geojson: () =>
    ({
      ...tracksGeojsonFixture,
      features: [
        ...tracksGeojsonFixture.features,
        ...importedTracks.map((track, index) => ({
          type: "Feature",
          geometry: {
            type: "MultiLineString",
            coordinates: [importedGeometry(index)],
          },
          properties: { ...track },
        })),
      ],
    }) as unknown as JsonValue,
  // The rows the Tracks tab reads — its own fixture, not the map's features.
  // Deriving them from the geometry would have made the stand inherit the very
  // omission this listing exists to undo.
  list_tracks: () => [...tracksListFixture, ...importedTracks],
  get_track_detail: (args) =>
    args?.layerId === FIXTURE_TRACK_LAYER && args?.trackId === FIXTURE_TRACK
      ? trackDetailFixture
      : { id: Number(args?.trackId ?? 0), name: "", segments: [] },
  get_waypoints: (args) =>
    args?.layerId === FIXTURE_WAYPOINT_LAYER
      ? [...waypointsFixture, ...placedWaypoints]
      : [],
  // Placing a waypoint by bearing and distance was the one on-map tool that
  // could not be walked here: the stand had no answer for `add_waypoint`, so
  // the tool ended in the failure toast every time. It failed loudly, which is
  // the stand working as designed, but it left the feature unlooked-at.
  // Verification session, 2026-09-22.
  add_waypoint: (args) => {
    placedWaypoints.push({
      id: 900 + placedWaypoints.length,
      name: typeof args?.name === "string" ? args.name : "Точка",
      lat: Number(args?.lat ?? 0),
      lon: Number(args?.lon ?? 0),
      symbol: null,
      visible: true,
      color: null,
    });
    standEmit("state-changed", undefined);
    return null;
  },
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

  // ── Layer management ───────────────────────────────────────────────────
  create_track_layer: (args) => {
    nextStandLayerId += 1;
    standTrackLayers = [
      ...(standTrackLayers ?? appStateFixture.track_layers),
      { id: nextStandLayerId, name: String(args?.name ?? "Layer") },
    ];
    standEmit("state-changed", undefined);
    return nextStandLayerId;
  },
  create_waypoint_layer: (args) => {
    nextStandLayerId += 1;
    standWaypointLayers = [
      ...(standWaypointLayers ?? appStateFixture.waypoint_layers),
      { id: nextStandLayerId, name: String(args?.name ?? "Layer") },
    ];
    standEmit("state-changed", undefined);
    return nextStandLayerId;
  },
  rename_track_layer: (args) => {
    standTrackLayers = (standTrackLayers ?? appStateFixture.track_layers).map(
      (layer) =>
        layer.id === Number(args?.layerId)
          ? { ...layer, name: String(args?.newName ?? layer.name) }
          : layer,
    );
    standEmit("state-changed", undefined);
    return null;
  },
  rename_waypoint_layer: (args) => {
    standWaypointLayers = (
      standWaypointLayers ?? appStateFixture.waypoint_layers
    ).map((layer) =>
      layer.id === Number(args?.layerId)
        ? { ...layer, name: String(args?.newName ?? layer.name) }
        : layer,
    );
    standEmit("state-changed", undefined);
    return null;
  },
  delete_track_layer: (args) => {
    standTrackLayers = (
      standTrackLayers ?? appStateFixture.track_layers
    ).filter((layer) => layer.id !== Number(args?.layerId));
    standEmit("state-changed", undefined);
    return null;
  },
  delete_waypoint_layer: (args) => {
    standWaypointLayers = (
      standWaypointLayers ?? appStateFixture.waypoint_layers
    ).filter((layer) => layer.id !== Number(args?.layerId));
    standEmit("state-changed", undefined);
    return null;
  },
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

  const handler = (
    HANDLERS as Record<string, ((args: Args) => unknown) | undefined>
  )[command];
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
