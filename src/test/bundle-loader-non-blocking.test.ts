import { describe, expect, it } from "vitest";
import { readFileSync } from "fs";
import { join } from "path";

const loaderSource = readFileSync(
  join(__dirname, "../routes/+page.svelte"),
  "utf-8"
);
const apiSource = readFileSync(join(__dirname, "../lib/api.ts"), "utf-8");
const commandsSource = readFileSync(
  join(__dirname, "../../src-tauri/src/commands/mod.rs"),
  "utf-8"
);

/**
 * The non-blocking contract is twofold:
 *   1. The Tauri command for load_project must return a download_id, not
 *      block until the bundle is done.
 *   2. The Svelte loader must support cancel-and-restart when the user
 *      switches projects mid-download — `cancelDownload(previousId)` is
 *      awaited BEFORE the new `loadProject(slug)` is started, so the
 *      backend never sees overlapping downloads.
 *
 * Both halves are checked structurally so the test catches regressions
 * to either the old "fire-and-forget loadProject" loader or to a future
 * loader that forgets to cancel the previous download.
 */
describe("bundle loader main-thread responsiveness", () => {
  it("backend load_project returns a download_id immediately and spawns the work async", () => {
    // Handler signature: Result<String, String>
    expect(commandsSource).toMatch(
      /pub fn load_project\([^)]*\)\s*->\s*Result<String,\s*String>/m
    );
    // Body must spawn on the async runtime, not block.
    expect(commandsSource).toContain("tauri::async_runtime::spawn");
    // Wire format: download_id is a v4 uuid.
    expect(commandsSource).toContain("uuid::Uuid::new_v4()");
    // Cancellation surface is registered.
    expect(commandsSource).toContain("DownloadRegistry");
    expect(commandsSource).toContain("pub fn cancel_download");
  });

  it("loadProject API wrapper returns the download_id string", () => {
    expect(apiSource).toMatch(
      /export async function loadProject\(slug: string\):\s*Promise<string>/
    );
  });

  it("handleSelectProject cancels any active download before starting a new one", () => {
    // Async signature is required to support `await cancelDownload(...)`
    // before the new `loadProject(...)` call.
    expect(loaderSource).toMatch(
      /async\s+function\s+handleSelectProject\(slug:\s*string\)\s*\{/
    );
    // cancelDownload must be awaited (sequenced) so the backend never sees
    // overlapping downloads.
    expect(loaderSource).toMatch(/await\s+cancelDownload\(/);
    // loadProject must be called for the newly-selected slug.
    expect(loaderSource).toMatch(/loadProject\(slug\)/);
  });

  it("project list is not blanket-disabled while a download is in flight", () => {
    // The blanket `disabled={$busy}` on project list-item buttons was
    // removed so the user can switch projects mid-download. The refresh
    // button keeps its `disabled={$busy}` because that path duplicates an
    // in-flight loadProjects call.
    expect(loaderSource).not.toMatch(
      /class="list-item"[^>]*disabled=\{\$busy\}/
    );
    expect(loaderSource).toMatch(
      /onclick=\{handleRefresh\}[^>]*disabled=\{\$busy\}/
    );
  });

  it("download lifecycle state lives in stores so sibling panels stay reactive", () => {
    const storesSource = readFileSync(
      join(__dirname, "../lib/stores.ts"),
      "utf-8"
    );
    expect(storesSource).toContain("activeDownloadId");
    expect(storesSource).toContain("readyBundleFiles");
    expect(storesSource).toContain("resetBundleDownloadState");
  });
});
