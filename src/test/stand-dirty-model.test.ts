import { describe, expect, it } from "vitest";
import { readFileSync } from "fs";
import { join } from "path";
import { standCommandChangesTheProject } from "./stand/tauri-core";

/**
 * Which commands leave a coordinator's work unsaved.
 *
 * The indicator in the title bar is the only thing between a night's editing
 * and a closed laptop, and the stand served a fixed `project_dirty: false` —
 * so CJ-7, the journey that is entirely about whether work survives, could
 * not be walked here at all. The model added on 2026-09-23 got it wrong on the
 * first try in the other direction: `load_projects` runs at every start and
 * was classified as an edit, so the stand opened with the dot already on. An
 * indicator that is always on says as little as one that is always off.
 *
 * Hence the partition below, written out in full. The code defaults an
 * unknown command to "changes the project", which is the safe way to be wrong
 * at runtime; this test refuses to let a command stay unclassified, which is
 * the safe way to be wrong at review.
 */
const LEAVES_THE_PROJECT_ALONE = [
  // Reads.
  "get_app_state",
  "get_ozi_metadata",
  "get_simplified_preview",
  "get_track_detail",
  "get_track_export_default_path",
  "get_tracks_geojson",
  "get_waypoints",
  "get_waypoints_export_default_path",
  "get_wpt_export_default_path",
  "list_tracks",
  // Writing a file out does not change the project it was written from.
  "export_all_tracks_gpx",
  "export_gpx",
  "export_gpx_waypoints",
  "export_track_plt",
  "export_wpt_waypoints",
  // Taking something back, including a drawing abandoned half way: the
  // application discards those points without leaving them in the history.
  "cancel_download",
  "cancel_drawing",
  "cancel_project_listing",
  // Showing a file to the operator.
  "reveal_bundle",
  "reveal_path",
  // The catalogue and the raster under the work, which are the session, not
  // the project: nothing here is work anybody would lose.
  "preview_project",
  "load_projects",
  "load_project",
  "open_local_bundle",
  "open_selected_map",
  "set_bundles_root",
  // After these the project in hand is the project on disk.
  "save_project",
  "load_project_file",
  "new_project",
];

describe("the stand's unsaved-changes model", () => {
  const bindings = readFileSync(join(__dirname, "../lib/bindings.ts"), "utf-8");
  const commands = [
    ...new Set(
      [...bindings.matchAll(/TAURI_INVOKE\("([a-z0-9_]+)"/g)].map((m) => m[1]),
    ),
  ].sort();

  it("classifies every command the frontend can send", () => {
    const expected = new Set(LEAVES_THE_PROJECT_ALONE);
    const surprises = commands
      .filter((name) => standCommandChangesTheProject(name) === expected.has(name))
      .map(
        (name) =>
          `${name}: the stand says it ${
            expected.has(name) ? "changes" : "leaves alone"
          } the project, the list beside it says the opposite`,
      );

    expect(
      surprises,
      "a new command defaults to 'changes the project', which turns the " +
        "unsaved dot on. If it does not — a read, an export, a catalogue " +
        "call — add it to LEAVES_THE_PROJECT_ALONE in both this test and " +
        "src/test/stand/tauri-core.ts.",
    ).toEqual([]);
  });

  it("does not carry names for commands that no longer exist", () => {
    const stale = LEAVES_THE_PROJECT_ALONE.filter(
      (name) => !commands.includes(name),
    );
    expect(stale, "a classification that outlives its command is noise").toEqual(
      [],
    );
  });

  it("leaves the project clean through a start that only reads", () => {
    // The commands a cold start actually sends, in order. None of them is an
    // edit, so the dot must still be off when the first screen appears.
    const startup = [
      "get_app_state",
      "list_tracks",
      "get_waypoints",
      "load_projects",
      "get_tracks_geojson",
    ];
    expect(startup.filter(standCommandChangesTheProject)).toEqual([]);
  });
});
