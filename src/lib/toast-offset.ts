/**
 * Where the toaster sits relative to the download progress panel.
 *
 * Both live in the bottom-right corner and svelte-sonner's viewport carries
 * `z-index: 999999999`, so a toast always draws over `DownloadPopup`'s
 * `z-index: 60`. The panel is the one that cannot move: the map keeps its
 * controls in the left corners, and the panel has to stay above the
 * bundle-loader sheet. So the toaster steps aside instead — but only while a
 * download is on screen, so a toast keeps its usual place the rest of the
 * time.
 */

/** svelte-sonner's own default edge offset (`VIEWPORT_OFFSET`). */
export const TOAST_EDGE_OFFSET_PX = 32;

/** `.download-popup`'s `bottom` in `DownloadPopup.svelte`. */
export const DOWNLOAD_POPUP_MARGIN_PX = 12;

/** Breathing room between the panel's top edge and the lowest toast. */
export const TOAST_POPUP_GAP_PX = 12;

export type ToastOffset = {
  top: string;
  right: string;
  bottom: string;
  left: string;
};

/**
 * @param downloadPopupHeight Measured height of the download panel in px,
 *   or 0 when no download is on screen.
 */
export function toastOffset(downloadPopupHeight: number): ToastOffset {
  // `Math.max` rather than a bare sum: a panel measured mid-mount can report
  // a height of a few px, and a toast must never end up closer to the edge
  // than it sits with no panel at all.
  const bottom = Math.max(
    downloadPopupHeight > 0
      ? downloadPopupHeight + DOWNLOAD_POPUP_MARGIN_PX + TOAST_POPUP_GAP_PX
      : 0,
    TOAST_EDGE_OFFSET_PX,
  );
  return {
    top: `${TOAST_EDGE_OFFSET_PX}px`,
    right: `${TOAST_EDGE_OFFSET_PX}px`,
    bottom: `${bottom}px`,
    left: `${TOAST_EDGE_OFFSET_PX}px`,
  };
}
