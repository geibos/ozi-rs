// @vitest-environment jsdom
/**
 * Behavioural cover for the loader keeping the operator's place.
 *
 * On the workspace route the loader lives inside a Sheet, and a Sheet unmounts
 * its content. Everything the component held went with it, so closing the
 * loader to look at the map — the ordinary thing to do — sent the operator
 * back to the top of a thirteen-thousand-row catalogue with an empty search
 * box. This mounts the real component, types a search, throws the component
 * away exactly as the Sheet does, and mounts it again.
 */
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { render, cleanup, fireEvent } from "@testing-library/svelte";
import { get } from "svelte/store";

vi.mock("$lib/api", () => ({
  cancelDownload: vi.fn(async () => {}),
  loadProjects: vi.fn(async () => {}),
  loadProject: vi.fn(async () => ""),
  openLocalBundle: vi.fn(async () => ""),
  openSelectedMap: vi.fn(async () => ""),
  previewProject: vi.fn(async () => {}),
  setBundlesRoot: vi.fn(async () => {}),
}));

vi.mock("@tauri-apps/plugin-dialog", () => ({ open: vi.fn(async () => null) }));

import BundleLoader from "../components/BundleLoader.svelte";
import { bundleLoaderView, projectsStore } from "../lib/stores";

const CATALOGUE = [
  { slug: "2026-09-20_lisiy-nos", name: "Lisiy Nos", cached: false },
  { slug: "2026-09-21_veter", name: "Veter", cached: true },
  { slug: "2026-09-21_zarya", name: "Zarya", cached: false },
];

function searchBox(container: HTMLElement): HTMLInputElement {
  const input = container.querySelector("input");
  if (!input) throw new Error("the loader rendered without a search box");
  return input as HTMLInputElement;
}

beforeEach(() => {
  // jsdom has no ResizeObserver, and the loader measures its list viewport
  // with one. Nothing here depends on the measurement.
  vi.stubGlobal(
    "ResizeObserver",
    class {
      observe() {}
      unobserve() {}
      disconnect() {}
    },
  );
  projectsStore.set(CATALOGUE);
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
});

describe("the loader keeps the operator's place", () => {
  it("brings the search back after the Sheet has unmounted it", async () => {
    const first = render(BundleLoader);
    await fireEvent.input(searchBox(first.container), {
      target: { value: "Veter" },
    });

    expect(get(bundleLoaderView).filter).toBe("Veter");

    // What closing the Sheet does.
    cleanup();

    const second = render(BundleLoader);
    expect(searchBox(second.container).value).toBe("Veter");
  });

  it("does not carry the search into a fresh session", () => {
    const { container } = render(BundleLoader);
    expect(searchBox(container).value).toBe("");
  });
});
