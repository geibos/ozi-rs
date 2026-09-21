import { beforeEach, describe, expect, it, vi } from "vitest";
import { get } from "svelte/store";

const openSelectedMap = vi.fn(async (_mapName: string) => "");
vi.mock("$lib/api", () => ({
  openSelectedMap: (mapName: string) => openSelectedMap(mapName),
}));

import { openMapShowingDownload } from "../lib/actions/open-map";
import {
  activeDownloadId,
  downloadProgress,
  resetBundleDownloadState,
} from "../lib/stores";

/**
 * `open_selected_map` returns a download id when the map was not on disk, so
 * the caller can show progress and offer a cancel. Two of the three call sites
 * used it; the command palette threw it away, which left a download running
 * with no panel, no cancel and nothing on screen at all — the palette closes,
 * and the map cannot open until the bytes arrive.
 *
 * One helper, so the fourth call site cannot get it wrong either.
 */
describe("openMapShowingDownload", () => {
  beforeEach(() => {
    openSelectedMap.mockReset();
    resetBundleDownloadState(null);
  });

  it("puts a started download on screen and reports that the map is not open", async () => {
    openSelectedMap.mockResolvedValue("download-7");
    await expect(openMapShowingDownload("Satell_z17.sqlitedb")).resolves.toBe(
      false,
    );
    expect(get(activeDownloadId)).toBe("download-7");
  });

  it("clears a previous download's rows so the panel shows this one", async () => {
    resetBundleDownloadState("older");
    downloadProgress.set(
      new Map([
        [
          "stale.sqlitedb",
          {
            download_id: "older",
            package_name: "stale.sqlitedb",
            downloaded_bytes: 5,
            total_bytes: 10,
          },
        ],
      ]),
    );
    openSelectedMap.mockResolvedValue("download-8");
    await openMapShowingDownload("Satell_z17.sqlitedb");
    expect(get(activeDownloadId)).toBe("download-8");
    expect(get(downloadProgress).size).toBe(0);
  });

  it("touches nothing when the map opened from disk", async () => {
    openSelectedMap.mockResolvedValue("");
    await expect(openMapShowingDownload("Topo_z16.sqlitedb")).resolves.toBe(
      true,
    );
    expect(get(activeDownloadId)).toBeNull();
  });

  it("lets the failure reach the caller, which is what toasts it", async () => {
    openSelectedMap.mockRejectedValue(new Error("no route to host"));
    await expect(openMapShowingDownload("Topo_z16.sqlitedb")).rejects.toThrow(
      "no route to host",
    );
    expect(get(activeDownloadId)).toBeNull();
  });
});
