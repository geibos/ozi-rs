import { describe, expect, it } from "vitest";
import { readFileSync } from "fs";
import { join } from "path";

// The cold-start `/` route page now delegates the catalog + maps columns
// to `BundleLoader.svelte` (so the same component can be mounted inside
// the workspace's bundle-loader Sheet). The page itself keeps the cancel
// status bar, while project selection / refresh wiring lives in the
// component.
const loaderSource = readFileSync(
  join(__dirname, "../components/BundleLoader.svelte"),
  "utf-8"
);
const pageSource = readFileSync(
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
 */
describe("bundle loader main-thread responsiveness", () => {
  it("backend load_project returns a download_id immediately and spawns the work async", () => {
    expect(commandsSource).toMatch(
      /pub fn load_project\([^)]*\)\s*->\s*Result<String,\s*String>/m
    );
    expect(commandsSource).toContain("tauri::async_runtime::spawn");
    expect(commandsSource).toContain("uuid::Uuid::new_v4()");
    expect(commandsSource).toContain("DownloadRegistry");
    expect(commandsSource).toContain("pub fn cancel_download");
  });

  it("loadProject API wrapper returns the download_id string", () => {
    expect(apiSource).toMatch(
      /export async function loadProject\(slug: string\):\s*Promise<string>/
    );
  });

  it("handleSelectProject cancels any active download before starting a new one", () => {
    expect(loaderSource).toMatch(
      /async\s+function\s+handleSelectProject\(slug:\s*string\)\s*\{/
    );
    expect(loaderSource).toMatch(/await\s+cancelDownload\(/);
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

  it("cold-start page still mounts the cancel status bar", () => {
    // The cancel-download UI lives on the cold-start status bar; the
    // workspace `Sheet` mount uses the same component without the bar.
    expect(pageSource).toContain('data-testid="cancel-download"');
    expect(pageSource).toContain("handleCancelDownload");
  });

  it("download lifecycle state lives in stores so sibling panels stay reactive", () => {
    const storesSource = readFileSync(
      join(__dirname, "../lib/stores.ts"),
      "utf-8"
    );
    expect(storesSource).toContain("activeDownloadId");
    expect(storesSource).toContain("resetBundleDownloadState");
  });
});
