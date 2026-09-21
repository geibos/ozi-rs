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

import {
  appState,
  activeDownloadId,
  busy,
  downloadProgress,
  finishDownload,
} from "../lib/stores";
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
  it("clears the panel for the download that finished", () => {
    activeDownloadId.set("download-1");
    downloadProgress.set(
      new Map([
        [
          "map.sqlitedb",
          {
            download_id: "download-1",
            package_name: "map.sqlitedb",
            downloaded_bytes: 1,
            total_bytes: 2,
          },
        ],
      ]),
    );

    expect(finishDownload("download-1")).toBe(true);

    expect(get(activeDownloadId)).toBeNull();
    expect(get(downloadProgress).size).toBe(0);
  });

  it("ignores a finish for a download the panel is not showing", () => {
    // A bundle download ending must not close the panel of a map download
    // that is still running.
    activeDownloadId.set("map-download");

    expect(finishDownload("some-older-bundle")).toBe(false);
    expect(get(activeDownloadId)).toBe("map-download");
  });

  it("does not resurrect a finished download when something else makes the app busy", async () => {
    activeDownloadId.set("download-1");
    finishDownload("download-1");

    // A later refresh of the project list makes the app busy again.
    getAppState.mockResolvedValue(state({ busy: true }));
    await appState.refresh();

    expect(get(busy)).toBe(true);
    expect(get(activeDownloadId)).toBeNull();
  });
});
