// @vitest-environment jsdom
/**
 * Behavioural cover for stopping the catalogue refresh.
 *
 * The walk is up to a thousand pages and holds the application busy for all of
 * it, so on a field link the only download button in the app is disabled for
 * minutes after launch. A crew that needs a bundle now has to be able to say
 * they have waited long enough.
 */
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { render, cleanup, fireEvent } from "@testing-library/svelte";

const { cancelProjectListing } = vi.hoisted(() => ({
  cancelProjectListing: vi.fn(async () => true),
}));

vi.mock("$lib/api", () => ({
  cancelDownload: vi.fn(async () => {}),
  cancelProjectListing,
  loadProjects: vi.fn(async () => {}),
  loadProject: vi.fn(async () => ""),
  openLocalBundle: vi.fn(async () => ""),
  openSelectedMap: vi.fn(async () => ""),
  previewProject: vi.fn(async () => {}),
  setBundlesRoot: vi.fn(async () => {}),
}));

vi.mock("@tauri-apps/plugin-dialog", () => ({ open: vi.fn(async () => null) }));

import BundleLoader from "../components/BundleLoader.svelte";
import { bundleLoaderView, projectsLoading, projectsStore } from "../lib/stores";

beforeEach(() => {
  vi.stubGlobal(
    "ResizeObserver",
    class {
      observe() {}
      unobserve() {}
      disconnect() {}
    },
  );
  cancelProjectListing.mockClear();
  projectsStore.set([]);
  bundleLoaderView.set({
    filter: "",
    onlyCached: false,
    selectedSlug: "",
    skipped: {},
    scrollTop: 0,
  });
});

afterEach(() => {
  cleanup();
  projectsLoading.set(false);
});

describe("stopping the catalogue refresh", () => {
  it("offers a stop while the refresh is running, and asks the backend for it", async () => {
    projectsLoading.set(true);
    const { getByTestId } = render(BundleLoader);

    await fireEvent.click(getByTestId("stop-refresh"));

    expect(cancelProjectListing).toHaveBeenCalledTimes(1);
  });

  it("offers nothing to stop when no refresh is running", () => {
    projectsLoading.set(false);
    const { queryByTestId } = render(BundleLoader);

    expect(queryByTestId("stop-refresh")).toBeNull();
  });
});
