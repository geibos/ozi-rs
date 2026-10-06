// @vitest-environment jsdom
/**
 * The simplify panel speaks Ozi's language.
 *
 * The detachment's standard has every foot patrol's track filtered in
 * OziExplorer "by index 4". Measured on eleven tracks from a real search
 * (2026-10-06), that is Douglas–Peucker at about 2 m; the panel opens there,
 * says so, and offers to go back after the slider has been moved.
 */
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { cleanup, fireEvent, render, screen } from "@testing-library/svelte";
import { get } from "svelte/store";
import { tick } from "svelte";

// jsdom has no ResizeObserver, and the panel's slider measures itself.
globalThis.ResizeObserver ??= class {
  observe() {}
  unobserve() {}
  disconnect() {}
} as unknown as typeof ResizeObserver;

const { listTracks, getAppState } = vi.hoisted(() => ({
  listTracks: vi.fn(),
  getAppState: vi.fn(async () => null as unknown),
}));

vi.mock("$lib/api", () => ({
  getAppState,
  listTracks,
  getTrackExportDefaultPath: vi.fn(async () => null),
  deleteTrack: vi.fn(async () => {}),
  exportGpx: vi.fn(async () => {}),
  exportAllTracksGpx: vi.fn(async () => 0),
  exportTrackPlt: vi.fn(async () => {}),
  renameTrack: vi.fn(async () => {}),
  setAllTracksVisible: vi.fn(async () => {}),
  setTrackColor: vi.fn(async () => {}),
  setTrackLineWidth: vi.fn(async () => {}),
  showOnlyTrack: vi.fn(async () => {}),
  toggleTrackVisible: vi.fn(async () => {}),
  simplifyTrack: vi.fn(async () => {}),
  sortTrackPoints: vi.fn(async () => {}),
  cropTrackToExtent: vi.fn(async () => {}),
  cropTrackToTime: vi.fn(async () => {}),
  getSimplifiedPreview: vi.fn(async () => ({
    original_count: 100,
    simplified_count: 40,
    geojson: { type: "FeatureCollection", features: [] },
  })),
}));

vi.mock("@tauri-apps/plugin-dialog", () => ({ open: vi.fn(async () => null) }));

import TracksTab from "./stubs/TracksTabHarness.svelte";
import { appState, simplifyState } from "../lib/stores";
import { setLocale } from "../lib/i18n";
import { OZI_INDEX_4_TOLERANCE_M, openSimplify } from "../lib/simplify";
import type { AppStateDto, TrackSummaryDto } from "../lib/bindings";

const row = {
  layer_id: 1,
  track_id: 1,
  name: "20261006_Lisa2",
  color: "#ff0000",
  line_width: 3,
  visible: true,
  distance_km: 1,
  duration_seconds: 60,
  point_count: 1995,
} as TrackSummaryDto;

const oneLayer = {
  tracks: [],
  waypoints: [],
  track_layers: [{ id: 1, name: "Tracks", visible: true }],
  waypoint_layers: [],
  map_layers: [],
} as unknown as AppStateDto;

async function settle() {
  for (let i = 0; i < 20; i += 1) {
    await new Promise((r) => setTimeout(r, 5));
    await tick();
  }
}

describe("the simplify panel and Ozi's index 4", () => {
  beforeEach(async () => {
    setLocale("ru");
    getAppState.mockResolvedValue(oneLayer);
    listTracks.mockResolvedValue([row]);
    await appState.refresh();
  });

  afterEach(() => {
    simplifyState.update((s) => ({ ...s, active: false }));
    cleanup();
  });

  it("is 2 m", () => {
    expect(OZI_INDEX_4_TOLERANCE_M).toBe(2);
  });

  it("opens at index 4 whichever way it is opened", () => {
    simplifyState.update((s) => ({ ...s, toleranceM: 250 }));
    openSimplify(1n, 1n);
    expect(get(simplifyState)).toMatchObject({
      active: true,
      layerId: 1n,
      trackId: 1n,
      toleranceM: 2,
      preview: null,
    });
  });

  it("says what 2 m means and offers no button while it is there", async () => {
    render(TracksTab);
    await settle();
    openSimplify(1n, 1n);
    await settle();
    expect(screen.getByTestId("simplify-ozi-hint").textContent).toContain(
      "2 м — как фильтр трека OziExplorer с индексом 4",
    );
    expect(screen.queryByTestId("simplify-ozi-reset")).toBeNull();
  });

  it("goes back to index 4 after the slider has moved", async () => {
    render(TracksTab);
    await settle();
    openSimplify(1n, 1n);
    simplifyState.update((s) => ({ ...s, toleranceM: 40 }));
    await settle();
    await fireEvent.click(screen.getByTestId("simplify-ozi-reset"));
    expect(get(simplifyState).toleranceM).toBe(2);
  });
});
