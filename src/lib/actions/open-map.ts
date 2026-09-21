/**
 * Opening a map, from wherever the operator asked for it.
 *
 * `open_selected_map` opens the map from disk when it is there and starts a
 * download when it is not, returning the download id in the second case so the
 * caller can put the progress panel on screen with its cancel. Two of the three
 * call sites did that; the command palette dropped the id, which left a
 * download running with nothing on screen — the palette closes, the map cannot
 * open until the bytes arrive, and there is no way to stop it.
 *
 * One helper rather than three copies, for the same reason `latest-run.ts`
 * exists: the rule is easy to get right once and easy to forget at the next
 * call site.
 */
import { openSelectedMap } from "$lib/api";
import { resetBundleDownloadState } from "$lib/stores";

/**
 * @returns `true` when the map is open now, `false` when a download started
 *   and the map will open once it lands. Failures are thrown, because what to
 *   say about them differs per surface.
 */
export async function openMapShowingDownload(
  mapName: string,
): Promise<boolean> {
  const downloadId = await openSelectedMap(mapName);
  if (downloadId) {
    // Also clears the previous download's rows, so the panel shows this one.
    resetBundleDownloadState(downloadId);
    return false;
  }
  return true;
}
