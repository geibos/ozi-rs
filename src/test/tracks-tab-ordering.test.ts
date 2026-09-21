// @vitest-environment jsdom
/**
 * The Tracks tab reloads on every app-state change, and during a bundle
 * download `state-changed` fires once per file, so reloads overlap. Nothing
 * ordered them: the one that started first could answer last and put its rows
 * on screen, so the list went backwards under the operator.
 */
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { render, cleanup } from "@testing-library/svelte";

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
  getSimplifiedPreview: vi.fn(async () => ({ points: [], removed: 0 })),
}));

vi.mock("@tauri-apps/plugin-dialog", () => ({ open: vi.fn(async () => null) }));

import TracksTab from "./stubs/TracksTabHarness.svelte";
import { appState } from "../lib/stores";
import type { AppStateDto, TrackSummaryDto } from "../lib/bindings";

const row = (name: string): TrackSummaryDto =>
  ({
    layer_id: 1,
    track_id: 1,
    name,
    color: "#ff0000",
    line_width: 3,
    visible: true,
    distance_km: 1,
    duration_seconds: 60,
    point_count: 2,
  }) as TrackSummaryDto;

const oneLayer = {
  tracks: [],
  waypoints: [],
  track_layers: [{ id: 1, name: "Tracks", visible: true }],
  waypoint_layers: [],
  map_layers: [],
} as unknown as AppStateDto;

async function names(container: HTMLElement): Promise<string[]> {
  for (let i = 0; i < 40; i += 1) {
    await new Promise((resolve) => setTimeout(resolve, 5));
    const found = Array.from(container.querySelectorAll(".truncate")).map(
      (el) => el.textContent?.trim() ?? "",
    );
    if (found.length > 0) return found;
  }
  return [];
}

beforeEach(async () => {
  getAppState.mockResolvedValue(oneLayer);
  listTracks.mockReset();
  await appState.refresh();
});

afterEach(cleanup);

describe("overlapping track reloads", () => {
  it("drops the rows of a reload that a newer one has overtaken", async () => {
    const held: Array<() => void> = [];
    let call = 0;
    listTracks.mockImplementation(() => {
      call += 1;
      if (call === 1) {
        return new Promise((resolve) => {
          held.push(() => resolve([row("stale")]));
        });
      }
      return Promise.resolve([row("fresh")]);
    });

    const { container } = render(TracksTab);
    await appState.refresh();

    expect(await names(container)).toContain("fresh");

    held.forEach((fire) => fire());
    await new Promise((resolve) => setTimeout(resolve, 20));

    expect(await names(container)).not.toContain("stale");
  });
});
