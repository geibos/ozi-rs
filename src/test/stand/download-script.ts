import { standEmit } from "./tauri-event";
import { appStateFixture } from "../fixtures";

/**
 * A bundle download, played out on the stand.
 *
 * The download panel, the "this map is ready" announcement and the panel
 * closing at the end had never been seen: catching them needs a real bundle
 * in flight, and on the owner's link a bundle lands in seconds. This replays
 * the event sequence the backend emits — the same names, the same payload
 * shapes — slowly enough to look at.
 *
 * It is a script, not a simulation: nothing here downloads anything, and it
 * proves only how the screens behave given those events.
 */
const STEP_MS = 700;

export function playBundleDownload(downloadId: string): void {
  const maps = appStateFixture.current_project?.maps ?? [];
  const files = [
    { name: "2-Coordinates.txt", bytes: 165 },
    {
      name: "8-Android&iOS/" + (maps[0]?.name ?? "topo.sqlitedb"),
      bytes: 16_672_522,
    },
    {
      name: "8-Android&iOS/" + (maps[1]?.name ?? "satell.sqlitedb"),
      bytes: 194_093_875,
    },
  ];
  const totalBytes = files.reduce((sum, f) => sum + f.bytes, 0);

  let step = 0;
  const at = (fn: () => void) => {
    step += 1;
    setTimeout(fn, step * STEP_MS);
  };

  at(() =>
    standEmit("bundle-progress", {
      download_id: downloadId,
      message: "Scanning",
      phase: "scanning",
    }),
  );

  at(() =>
    standEmit("bundle-progress", {
      download_id: downloadId,
      message: `Downloading ${files.length} files in parallel`,
      phase: "downloading",
      completed: 0,
      total: files.length,
      downloaded_bytes: 0,
      total_bytes: totalBytes,
    }),
  );

  let done = 0;
  let downloaded = 0;
  files.forEach((file, index) => {
    // Half, then whole: enough to see a bar move rather than jump.
    at(() =>
      standEmit("download-progress", {
        download_id: downloadId,
        package_name: file.name,
        downloaded_bytes: Math.round(file.bytes / 2),
        total_bytes: file.bytes,
        file_index: index,
        file_count: files.length,
      }),
    );
    at(() => {
      standEmit("download-progress", {
        download_id: downloadId,
        package_name: file.name,
        downloaded_bytes: file.bytes,
        total_bytes: file.bytes,
        file_index: index,
        file_count: files.length,
      });
      standEmit("bundle-file-ready", {
        download_id: downloadId,
        package_name: file.name,
        local_path: `/tmp/${file.name}`,
        file_index: index,
        file_count: files.length,
      });
      done += 1;
      downloaded += file.bytes;
      standEmit("bundle-progress", {
        download_id: downloadId,
        message: `Downloaded ${done} of ${files.length} files`,
        phase: "downloading",
        completed: done,
        total: files.length,
        downloaded_bytes: downloaded,
        total_bytes: totalBytes,
      });
    });
  });

  at(() =>
    standEmit("bundle-progress", {
      download_id: downloadId,
      message: "Extracting",
      phase: "extracting",
    }),
  );
  at(() =>
    standEmit("download-finished", {
      download_id: downloadId,
      ok: true,
      message: null,
    }),
  );
}
