// @vitest-environment jsdom
/**
 * The bundle progress panel is gated on `activeDownloadId !== null && busy`.
 * Nothing cleared the id when a download finished, so the panel came back —
 * showing the finished download's rows — the next time anything made the app
 * busy, refreshing the project list for one.
 */
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { get } from "svelte/store";

const { getAppState } = vi.hoisted(() => ({
  getAppState: vi.fn(async () => null as unknown),
}));

vi.mock("$lib/api", () => ({ getAppState }));

import { appState, activeDownloadId, busy } from "../lib/stores";
import type { AppStateDto } from "../lib/types";

const state = (over: Record<string, unknown> = {}) =>
  ({
    busy: false,
    status: "",
    diagnostics: [],
    track_layers: [],
    waypoint_layers: [],
    map_layers: [],
    ...over,
  }) as unknown as AppStateDto;

beforeEach(async () => {
  getAppState.mockResolvedValue(state());
  await appState.refresh();
  activeDownloadId.set(null);
});

afterEach(() => {
  activeDownloadId.set(null);
});

describe("the progress panel belongs to the running download", () => {
  it("drops the download id when the backend stops being busy", async () => {
    activeDownloadId.set("download-1");

    getAppState.mockResolvedValue(state({ busy: true }));
    await appState.refresh();
    expect(get(busy)).toBe(true);
    expect(get(activeDownloadId)).toBe("download-1");

    getAppState.mockResolvedValue(state({ busy: false }));
    await appState.refresh();

    expect(get(busy)).toBe(false);
    expect(get(activeDownloadId)).toBeNull();
  });

  it("does not resurrect a finished download when something else makes the app busy", async () => {
    activeDownloadId.set("download-1");
    getAppState.mockResolvedValue(state({ busy: true }));
    await appState.refresh();
    getAppState.mockResolvedValue(state({ busy: false }));
    await appState.refresh();

    // A later refresh of the project list makes the app busy again.
    getAppState.mockResolvedValue(state({ busy: true }));
    await appState.refresh();

    expect(get(activeDownloadId)).toBeNull();
  });
});
