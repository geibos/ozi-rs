import { describe, expect, it } from "vitest";
import { readFileSync } from "fs";
import { join } from "path";

const loaderSource = readFileSync(
  join(__dirname, "../views/BundleLoaderView.svelte"),
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
 *   2. The Svelte component must NOT `await loadProject(...)` — instead it
 *      kicks off the request and rerenders purely from events.
 *
 * Both halves are checked structurally so the test catches a regression
 * to the previous "await loadProject(slug)" loader.
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

  it("BundleLoaderView does NOT await the loadProject promise inline", () => {
    // The previous loader had `await loadProject(slug)` directly inside an
    // event handler. The new contract forbids that — the handler must be
    // synchronous (modulo a fire-and-forget then-chain) so the Svelte main
    // thread does not stall.
    expect(loaderSource).not.toMatch(/await\s+loadProject\(/);
    expect(loaderSource).toMatch(/loadProject\(slug\)\s*\.then/);
    // handleSelectProject is intentionally a sync function (no async kw).
    expect(loaderSource).toMatch(/function\s+handleSelectProject\(slug:\s*string\)\s*\{/);
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
