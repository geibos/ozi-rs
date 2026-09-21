import { describe, expect, it } from "vitest";
import { readFileSync } from "fs";
import { join } from "path";
import { get } from "svelte/store";
import { mapFocusRequest, requestTrackFocus } from "../lib/stores";

/**
 * Structural regression guard for the first-hands-on-session fixes
 * (onboarding + import slice):
 *
 *   A. Maps tab dead end — "No maps in this project" now carries an
 *      "Open project…" button (empty state + always-visible header
 *      button) that reuses the palette's bundle-loader mechanism.
 *   B. One unified "Import…" button (gpx/plt/zip, multi-select) instead
 *      of twin indistinguishable icon buttons.
 *   C. "Import folder…" wired to the recursive `import_tracks_directory`
 *      backend command through the `importTracksDirectory` api wrapper.
 *   D. "Show on map": row + inspector buttons write `mapFocusRequest`;
 *      MapView subscribes and fits the track bbox via `map.fitBounds`.
 *
 * (E — dash-separator name validation — is covered by
 * `track-name-validation.test.ts`.)
 *
 * Behavioural verification (dialog → IPC → map movement) is manual
 * desktop QA per `docs/agent-verification.md`; Playwright is not
 * acceptable evidence per ADR-0024.
 */

const read = (rel: string) => readFileSync(join(__dirname, rel), "utf-8");

const mapsTabSource = read("../components/library/MapsTab.svelte");
const tracksTabSource = read("../components/library/TracksTab.svelte");
const trackInspectorSource = read(
  "../components/inspector/TrackInspector.svelte",
);
const mapViewSource = read("../components/MapView.svelte");
const apiSource = read("../lib/api.ts");
const i18nSource = read("../lib/i18n.ts");

describe("Maps tab — open-project affordance (task A)", () => {
  it("renders a prominent Open project button in the empty state", () => {
    expect(mapsTabSource).toContain("maps-empty-open-project");
    expect(mapsTabSource).toContain("mapsTab.empty");
  });

  it("renders an always-visible header Open project button", () => {
    expect(mapsTabSource).toContain("maps-open-project");
  });

  it("reuses the working bundle-loader mechanism (store + cold-start route)", () => {
    expect(mapsTabSource).toContain("bundleLoaderOpen.set(true)");
    expect(mapsTabSource).toContain('goto(resolve("/"))');
  });
});

describe("Tracks tab — unified import (tasks B & C)", () => {
  it("offers a single import dialog including zip archives", () => {
    expect(tracksTabSource).toContain("library-import-tracks");
    expect(tracksTabSource).toContain('extensions: ["gpx", "plt", "zip"]');
    expect(tracksTabSource).toContain("multiple: true");
  });

  it("wires Import folder to the recursive directory import", () => {
    expect(tracksTabSource).toContain("library-import-folder");
    expect(tracksTabSource).toContain("importTracksDirectory");
  });

  it("api.ts wrapper delegates to the generated bindings command", () => {
    // The return type changed from a sentence to counts on 2026-09-22: the
    // backend had been building an English summary that went straight into a
    // toast. What this pins is the delegation, not the shape — the shape is
    // the generated binding's business.
    expect(apiSource).toContain("export async function importTracksDirectory(");
    expect(apiSource).toContain("commands.importTracksDirectory(path)");
  });

  it("the folder import is worded by the interface, not by the backend", () => {
    expect(tracksTabSource).toContain('$i18n("tracksTab.importFolderDone")');
    expect(tracksTabSource).toContain('$i18n("tracksTab.importFolderSkipped")');
    // A folder where one file was unreadable imported the rest: a caveat, not
    // a failure.
    expect(tracksTabSource).toContain("toast.warning(summary,");
  });
});

describe("Show on map — mapFocusRequest wiring (task D)", () => {
  it("stores a one-shot request with a monotonically increasing nonce", () => {
    expect(get(mapFocusRequest)).toBeNull();
    requestTrackFocus(BigInt(1), BigInt(2));
    const first = get(mapFocusRequest);
    expect(first).toEqual({
      kind: "track",
      layerId: BigInt(1),
      trackId: BigInt(2),
      nonce: first!.nonce,
    });

    // A repeat click on the SAME track must still re-fire consumers.
    requestTrackFocus(BigInt(1), BigInt(2));
    const second = get(mapFocusRequest);
    expect(second!.nonce).toBeGreaterThan(first!.nonce);

    mapFocusRequest.set(null);
  });

  it("Tracks tab rows carry the locate button", () => {
    expect(tracksTabSource).toContain("track-show-on-map");
    expect(tracksTabSource).toContain("requestTrackFocus");
  });

  it("Track Inspector carries the Show on map button", () => {
    expect(trackInspectorSource).toContain("inspector-show-on-map");
    expect(trackInspectorSource).toContain("requestTrackFocus");
  });

  it("MapView consumes the request, fits bounds, and resets the store", () => {
    expect(mapViewSource).toContain("mapFocusRequest");
    expect(mapViewSource).toContain("mapFocusRequest.set(null)");
    expect(mapViewSource).toContain("map.fitBounds(");
    expect(mapViewSource).toContain("padding: 60, maxZoom: 16");
    // Empty-track guard: toast instead of fitBounds over an empty bbox.
    expect(mapViewSource).toContain("track.noPoints");
  });
});

describe("i18n — new keys exist in BOTH dictionaries", () => {
  const requiredKeys = [
    "mapsTab.empty",
    "mapsTab.openProject",
    "tracksTab.import",
    "tracksTab.importFolder",
    "tracksTab.importFilterName",
    "tracksTab.importFailed",
    "tracksTab.importDone",
    "tracksTab.importFailedFiles",
    "tracksTab.importFolderFailed",
    "tracksTab.nameHint",
    "track.showOnMap",
    "track.noPoints",
  ];

  it.each(requiredKeys)("declares %s twice (en + ru)", (key) => {
    const occurrences = i18nSource.split(`"${key}":`).length - 1;
    expect(occurrences).toBe(2);
  });
});
