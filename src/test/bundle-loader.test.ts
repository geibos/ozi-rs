import { describe, expect, it } from "vitest";
import { readFileSync } from "fs";
import { join } from "path";

// Source-grep contract tests for the bundle-loader field-usability slice
// (owner feedback, 2026-07): instant catalog from cache, virtualized
// project list, preview-on-click / download-on-button split, and the
// floating per-file download-progress popup.
const loaderSource = readFileSync(
  join(__dirname, "../components/BundleLoader.svelte"),
  "utf-8",
);
const popupSource = readFileSync(
  join(__dirname, "../components/DownloadPopup.svelte"),
  "utf-8",
);
const layoutSource = readFileSync(
  join(__dirname, "../routes/+layout.svelte"),
  "utf-8",
);
const apiSource = readFileSync(join(__dirname, "../lib/api.ts"), "utf-8");
const storesSource = readFileSync(join(__dirname, "../lib/stores.ts"), "utf-8");
const i18nSource = readFileSync(join(__dirname, "../lib/i18n.ts"), "utf-8");

/** Slice `source` between two unique markers (both must exist, in order). */
function sliceBetween(source: string, from: string, to: string): string {
  const start = source.indexOf(from);
  const end = source.indexOf(to);
  expect(start, `marker not found: ${from}`).toBeGreaterThan(-1);
  expect(end, `marker not found after ${from}: ${to}`).toBeGreaterThan(start);
  return source.slice(start, end);
}

describe("row click = preview, button = download", () => {
  it("api.ts exposes previewProject delegating to commands.previewProject", () => {
    expect(apiSource).toMatch(
      /export async function previewProject\(slug: string\): Promise<void>/,
    );
    expect(apiSource).toContain('"preview_project"');
    expect(apiSource).toContain("commands.previewProject(slug)");
  });

  it("row click wires previewProject and nothing download-related", () => {
    expect(loaderSource).toContain(
      "onclick={() => handleSelectProject(row.project.slug)}",
    );
    const selectBody = sliceBetween(
      loaderSource,
      "async function handleSelectProject",
      "async function handleOpenBundle",
    );
    expect(selectBody).toContain("previewProject(slug)");
    // No download start, no cancel machinery, no progress reset on click.
    expect(selectBody).not.toContain("loadProject(");
    expect(selectBody).not.toContain("cancelDownload(");
    expect(selectBody).not.toContain("downloadProgress.set");
    expect(selectBody).not.toContain("resetBundleDownloadState");
  });

  it("shows a pending maps hint cleared by currentProject match or timeout", () => {
    expect(loaderSource).toContain('data-testid="maps-pending"');
    expect(loaderSource).toContain("PREVIEW_TIMEOUT_MS = 15_000");
    expect(loaderSource).toContain(
      "$currentProject?.name === previewPendingName",
    );
    expect(loaderSource).toContain('$t("loader.loadingMaps")');
  });

  it("open-bundle button starts the real download with cancel-and-reset semantics", () => {
    expect(loaderSource).toContain('data-testid="open-bundle"');
    expect(loaderSource).toContain('$t("loader.openBundle")');
    const openBody = sliceBetween(
      loaderSource,
      "async function handleOpenBundle",
      "async function handleOpenMap",
    );
    expect(openBody).toContain("await cancelDownload(previousId)");
    expect(openBody).toContain("resetBundleDownloadState(null)");
    expect(openBody).toContain("await loadProject(slug)");
    expect(openBody).toContain("activeDownloadId.set(id || null)");
  });
});

describe("virtualized project list", () => {
  it("windows the FILTERED array with fixed-height absolute rows over a spacer", () => {
    expect(loaderSource).toContain("const ROW_HEIGHT = 28");
    expect(loaderSource).toContain("const OVERSCAN = 15");
    expect(loaderSource).toContain("filtered.length * ROW_HEIGHT");
    expect(loaderSource).toContain("filtered.slice(startIndex, endIndex)");
    // Spacer div carrying the total height + absolutely positioned rows.
    expect(loaderSource).toContain('class="virtual-spacer"');
    expect(loaderSource).toMatch(/style=\{`height: \$\{totalHeight\}px`\}/);
    expect(loaderSource).toContain('class="list-item virtual-row"');
    expect(loaderSource).toMatch(/style=\{`top: \$\{row\.top\}px`\}/);
    expect(loaderSource).toMatch(
      /\.list-item\.virtual-row\s*\{[^}]*position: absolute/,
    );
    // 28px CSS height stays in sync with ROW_HEIGHT.
    expect(loaderSource).toMatch(
      /\.list-item\.virtual-row\s*\{[^}]*height: 28px/,
    );
  });

  it("throttles scroll recompute via requestAnimationFrame", () => {
    expect(loaderSource).toContain("requestAnimationFrame");
    expect(loaderSource).toContain("cancelAnimationFrame");
    expect(loaderSource).toContain("onscroll={handleListScroll}");
  });

  it("resets the scroll window when the filter changes", () => {
    expect(loaderSource).toContain("void debouncedProjectFilter");
    expect(loaderSource).toContain("listEl.scrollTop = 0");
  });
});

describe("instant catalog + non-blocking refresh", () => {
  it("an empty backend snapshot never clobbers the cache-seeded catalog", () => {
    const syncBody = sliceBetween(
      storesSource,
      "export function syncProjectsFromAppState",
      "// Per-package download progress",
    );
    expect(syncBody).toContain("if (incoming.length === 0) return;");
  });

  it("startup refresh runs in background with an unobtrusive hint", () => {
    expect(layoutSource).toContain("projectsLoading.set(true)");
    expect(loaderSource).toContain('data-testid="catalog-refreshing"');
    expect(loaderSource).toContain('$t("loader.refreshing")');
    // Route-independent clear lives in stores, not in a page effect.
    expect(storesSource).toContain(
      "if (s && !s.busy) projectsLoading.set(false);",
    );
  });
});

describe("per-file download progress popup", () => {
  it("is mounted at the layout level so it survives route changes", () => {
    expect(layoutSource).toContain("<DownloadPopup />");
  });

  it("renders one bar per downloadProgress entry, filtered to the active download", () => {
    expect(popupSource).toContain('data-testid="download-popup"');
    expect(popupSource).toContain('data-testid="popup-file-row"');
    expect(popupSource).toContain("$downloadProgress.values()");
    expect(popupSource).toContain("p.download_id === $activeDownloadId");
    // Percent bar when total_bytes is known, indeterminate stripe otherwise,
    // plus a human-readable MB size.
    expect(popupSource).toContain("p.total_bytes");
    expect(popupSource).toContain("class:indeterminate={pct == null}");
    expect(popupSource).toContain("formatMb(p.downloaded_bytes)");
  });

  it("shows an x/y files header and a cancel button", () => {
    expect(popupSource).toContain('data-testid="popup-file-count"');
    expect(popupSource).toContain("file_count");
    expect(popupSource).toContain('data-testid="popup-cancel-download"');
    expect(popupSource).toContain("cancelDownload(id)");
  });

  it("downloadProgress store updates immutably so per-file bars rerender", () => {
    // A Map mutated in place would never notify subscribers — the updater
    // must allocate a fresh Map per event.
    const updaterBody = sliceBetween(
      storesSource,
      "export function updateDownloadProgress",
      "/** ID of the currently-active bundle download",
    );
    expect(updaterBody).toContain("const next = new Map(map)");
  });
});

describe("i18n coverage for the loader slice", () => {
  const NEW_KEYS = [
    "loader.projects",
    "loader.maps",
    "loader.filterPlaceholder",
    "loader.refreshing",
    "loader.noMatches",
    "loader.selectProject",
    "loader.loadingMaps",
    "loader.openBundle",
    "loader.openLocalBundle",
    "loader.setBundlesRoot",
    "loader.cachedBadge",
    "loader.openMapFailed",
    "download.title",
    "download.files",
    "download.cancel",
    "download.starting",
  ];

  it("every new key is declared in BOTH the en and ru dictionaries", () => {
    for (const key of NEW_KEYS) {
      const count = i18nSource.split(`"${key}":`).length - 1;
      expect(count, `key ${key} must appear exactly twice (en + ru)`).toBe(2);
    }
  });

  it("loader and popup render via $t, not hardcoded strings", () => {
    expect(loaderSource).toContain('$t("loader.projects")');
    expect(loaderSource).toContain('$t("loader.maps")');
    expect(loaderSource).toContain(
      'placeholder={$t("loader.filterPlaceholder")}',
    );
    expect(loaderSource).toContain('$t("loader.noMatches")');
    expect(loaderSource).toContain('$t("loader.selectProject")');
    expect(loaderSource).toContain('$t("loader.cachedBadge")');
    expect(loaderSource).toContain('$t("loader.openMapFailed")');
    expect(popupSource).toContain('$t("download.title")');
    expect(popupSource).toContain('$t("download.files")');
    expect(popupSource).toContain('$t("download.cancel")');
  });
});
