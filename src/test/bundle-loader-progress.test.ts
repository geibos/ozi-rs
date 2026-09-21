import { describe, expect, it } from "vitest";
import { readFileSync } from "fs";
import { join } from "path";

// Bundle-loader UI lives in two files after `redesign-library-sidebar`:
//   - `BundleLoader.svelte` — columns (catalog + maps), reused by the
//     cold-start `/` route and the workspace's bundle-loader Sheet.
//   - `routes/+page.svelte` — cold-start wrapper that adds the status
//     bar with the cancel button and current-file label.
const loaderSource = readFileSync(
  join(__dirname, "../components/BundleLoader.svelte"),
  "utf-8",
);
const pageSource = readFileSync(
  join(__dirname, "../routes/+page.svelte"),
  "utf-8",
);
const layoutSource = readFileSync(
  join(__dirname, "../routes/+layout.svelte"),
  "utf-8",
);
const storesSource = readFileSync(join(__dirname, "../lib/stores.ts"), "utf-8");
const apiSource = readFileSync(join(__dirname, "../lib/api.ts"), "utf-8");
const typesSource = readFileSync(join(__dirname, "../lib/types.ts"), "utf-8");
const commandsSource = readFileSync(
  join(__dirname, "../../src-tauri/src/commands/mod.rs"),
  "utf-8",
);

describe("bundle loader per-file progress UI", () => {
  it("renders a current-file label sourced from download-progress events", () => {
    expect(pageSource).toContain("currentFileLabel");
    expect(pageSource).toContain("file_index");
    expect(pageSource).toContain("file_count");
    expect(pageSource).toContain('data-testid="current-file-label"');
    // Source of truth: the `download-progress` listener lives in
    // `+layout.svelte` (single-owner rule, `consolidate-state-event-flow`)
    // and writes the payload into the `currentDownload` store; the page
    // reads `$currentDownload`.
    expect(layoutSource).toMatch(/currentDownload\.set\(event\.payload\)/);
    expect(pageSource).toContain("$currentDownload");
  });

  it("per-file readiness streams into AppStateDto live (stream-bundle-file-availability)", () => {
    // The FileReady branch must emit both bundle-file-ready (still kept for
    // analytics / future consumers) AND state-changed so the Maps column can
    // flip the row's badge from blue % to green cached the moment the file
    // is on disk — not at end-of-bundle.
    expect(commandsSource).toMatch(
      /DownloadNotification::FileReady[\s\S]+?app\.emit\(\s*"bundle-file-ready"[\s\S]+?app\.emit\(\s*"state-changed"/,
    );
    // The frontend no longer subscribes to bundle-file-ready at the page
    // level — that side channel is gone in favour of the AppStateDto
    // refresh that state-changed triggers.
    expect(pageSource).not.toContain("noteBundleFileReady");
    expect(pageSource).not.toMatch(/listen<[^>]*>\(\s*"bundle-file-ready"/);
    expect(loaderSource).not.toContain("noteBundleFileReady");
    expect(loaderSource).not.toMatch(/listen<[^>]*>\(\s*"bundle-file-ready"/);
    // The ready-files affordance and the Open bundle now button are gone;
    // the Maps column's per-row badge replaces them.
    expect(loaderSource).not.toContain('data-testid="ready-files"');
    expect(loaderSource).not.toContain('data-testid="open-bundle-now"');
    expect(storesSource).not.toContain("readyBundleFiles");
    expect(storesSource).not.toContain("noteBundleFileReady");
  });

  it("renders an indeterminate animation when total_bytes is missing", () => {
    expect(pageSource).toContain('data-testid="indeterminate-bar"');
    expect(pageSource).toContain("indeterminate-bar");
    expect(pageSource).toContain("$currentDownload.total_bytes == null");
  });

  it("exposes a cancel button wired to cancelDownload(activeDownloadId)", () => {
    expect(pageSource).toContain('data-testid="cancel-download"');
    expect(pageSource).toContain("cancelDownload");
    expect(pageSource).toContain("handleCancelDownload");
    expect(apiSource).toContain("export async function cancelDownload");
    expect(apiSource).toContain('"cancel_download"');
  });

  it("carries download_id in all event payloads", () => {
    expect(typesSource).toContain("download_id: string");
    expect(typesSource).toContain("BundleFileReadyPayload");
    expect(commandsSource).toContain("download_id: String");
    expect(commandsSource).toContain("struct BundleFileReadyPayload");
  });

  it("declares Rust download-progress fields including file_index/file_count", () => {
    expect(commandsSource).toContain("file_index: Option<usize>");
    expect(commandsSource).toContain("file_count: Option<usize>");
  });
});
