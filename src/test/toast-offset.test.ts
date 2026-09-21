import { describe, expect, it } from "vitest";
import { readFileSync } from "fs";
import { join } from "path";
import {
  DOWNLOAD_POPUP_MARGIN_PX,
  TOAST_EDGE_OFFSET_PX,
  TOAST_POPUP_GAP_PX,
  toastOffset,
} from "../lib/toast-offset";

/**
 * Both the toaster and the download progress panel are pinned to the
 * bottom-right corner, and the toaster's stacking context (z-index
 * 999999999 from svelte-sonner) always wins over the panel's z-index 60.
 * Measured on the stand at 1024x640: the panel occupied 692..1012 x
 * 550..628 and an error toast covered 644..1000 x 542..616 — the toast hid
 * the panel's title and the "files / bytes" row while the download was
 * still running.
 *
 * The panel is the one that cannot move: it must clear the map controls
 * (top-left, bottom-left) and stay above the bundle-loader sheet. So the
 * toaster steps aside for exactly as long as a download is on screen.
 */
describe("toast offset", () => {
  it("uses the library's own edge offset when nothing is in the corner", () => {
    expect(toastOffset(0)).toEqual({
      top: "32px",
      right: "32px",
      bottom: "32px",
      left: "32px",
    });
  });

  it("lifts toasts clear of the download panel, with a gap", () => {
    // 78px panel + 12px panel margin + 12px gap.
    expect(toastOffset(78).bottom).toBe("102px");
  });

  it("only moves the bottom edge", () => {
    const lifted = toastOffset(200);
    expect(lifted.top).toBe("32px");
    expect(lifted.right).toBe("32px");
    expect(lifted.left).toBe("32px");
  });

  it("keeps the offset growing with the panel", () => {
    const short = Number.parseInt(toastOffset(78).bottom, 10);
    const tall = Number.parseInt(toastOffset(240).bottom, 10);
    expect(tall - short).toBe(240 - 78);
  });

  it("never drops below the edge offset for a panel shorter than it", () => {
    // A one-row panel is ~64px; lifted it still clears the edge offset.
    expect(Number.parseInt(toastOffset(1).bottom, 10)).toBeGreaterThanOrEqual(
      TOAST_EDGE_OFFSET_PX,
    );
  });

  it("derives the lift from the panel's own margin and the gap", () => {
    expect(toastOffset(100).bottom).toBe(
      `${100 + DOWNLOAD_POPUP_MARGIN_PX + TOAST_POPUP_GAP_PX}px`,
    );
  });
});

/**
 * The measurement above only matters if the three pieces stay connected: the
 * panel has to publish its height, and the layout has to hand the offset to
 * the toaster. Either wire cut, and the toast goes back to covering the panel
 * with nothing failing.
 */
describe("toast offset wiring", () => {
  const read = (rel: string) =>
    readFileSync(join(__dirname, "..", rel), "utf-8");

  it("DownloadPopup measures itself into the store", () => {
    const source = read("components/DownloadPopup.svelte");
    expect(source).toContain("bind:clientHeight={panelHeight}");
    expect(source).toContain(
      "downloadPopupHeight.set($activeDownloadId === null ? 0 : panelHeight)",
    );
  });

  it("the layout offsets the toaster by the measured height", () => {
    const source = read("routes/+layout.svelte");
    expect(source).toContain(
      'import { toastOffset } from "$lib/toast-offset";',
    );
    expect(source).toContain("toastOffset($downloadPopupHeight)");
    expect(source).toContain("offset={toastEdge}");
  });
});
