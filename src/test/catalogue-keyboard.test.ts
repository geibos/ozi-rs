// @vitest-environment jsdom
/**
 * The catalogue is thirteen thousand rows and the list is virtualized, so only
 * the rows on screen exist in the DOM. There is no tab order over it and never
 * can be: a crew on a laptop in a field HQ had to find their search with the
 * trackpad. These drive it from the keyboard instead — type, arrow, Enter.
 */
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { render, cleanup, fireEvent } from "@testing-library/svelte";

const { previewProject } = vi.hoisted(() => ({
  previewProject: vi.fn(async () => {}),
}));

vi.mock("$lib/api", () => ({
  previewProject,
  cancelDownload: vi.fn(async () => {}),
  cancelProjectListing: vi.fn(async () => {}),
  loadProjects: vi.fn(async () => {}),
  loadProject: vi.fn(async () => ""),
  openLocalBundle: vi.fn(async () => ""),
  openSelectedMap: vi.fn(async () => ""),
  setBundlesRoot: vi.fn(async () => {}),
}));

vi.mock("@tauri-apps/plugin-dialog", () => ({ open: vi.fn(async () => null) }));

import BundleLoader from "../components/BundleLoader.svelte";
import { bundleLoaderView, projectsStore } from "../lib/stores";

const CATALOGUE = Array.from({ length: 40 }, (_, i) => ({
  slug: `2026-09-${String((i % 28) + 1).padStart(2, "0")}_search-${i}`,
  name: `Search ${i}`,
  cached: false,
}));

function list(container: HTMLElement): HTMLElement {
  const el = container.querySelector("[data-testid='project-virtual-list']");
  if (!el) throw new Error("the loader rendered without its project list");
  return el as HTMLElement;
}

/**
 * The row the list is pointing at, by slug.
 *
 * The id rather than the rendered row: the point of the windowing is that a
 * row far down the list is not in the DOM, and the position is kept against
 * the filtered array rather than against what happens to be rendered.
 */
function activeSlug(container: HTMLElement): string | null {
  const id = list(container).getAttribute("aria-activedescendant");
  return id ? id.replace(/^project-row-/, "") : null;
}

/** Whether the pointed-at row is one of the rows currently rendered. */
function activeRowIsRendered(container: HTMLElement): boolean {
  const id = list(container).getAttribute("aria-activedescendant");
  // Not `querySelector`: jsdom has no `CSS.escape`, and the ids carry dates.
  return Array.from(container.querySelectorAll("[role='option']")).some(
    (el) => el.id === id,
  );
}

beforeEach(() => {
  vi.stubGlobal(
    "ResizeObserver",
    class {
      observe() {}
      unobserve() {}
      disconnect() {}
    },
  );
  previewProject.mockClear();
  projectsStore.set(CATALOGUE);
  bundleLoaderView.set({
    filter: "",
    onlyCached: false,
    selectedSlug: "",
    skipped: {},
    scrollTop: 0,
  });
});

afterEach(cleanup);

describe("walking the catalogue from the keyboard", () => {
  it("is a listbox that points at the row it is on", async () => {
    const { container } = render(BundleLoader);
    const el = list(container);

    expect(el.getAttribute("role")).toBe("listbox");
    expect(el.getAttribute("tabindex")).toBe("0");
    expect(activeSlug(container)).toBeNull();

    await fireEvent.keyDown(el, { key: "ArrowDown" });
    expect(activeSlug(container)).toBe(CATALOGUE[0].slug);
    expect(
      activeRowIsRendered(container),
      "a row at the top of the list is on screen, so it must exist to point at",
    ).toBe(true);

    await fireEvent.keyDown(el, { key: "ArrowDown" });
    expect(activeSlug(container)).toBe(CATALOGUE[1].slug);

    await fireEvent.keyDown(el, { key: "ArrowUp" });
    expect(activeSlug(container)).toBe(CATALOGUE[0].slug);
  });

  it("does not walk off either end", async () => {
    const { container } = render(BundleLoader);
    const el = list(container);

    await fireEvent.keyDown(el, { key: "ArrowUp" });
    expect(activeSlug(container)).toBe(CATALOGUE[0].slug);

    await fireEvent.keyDown(el, { key: "End" });
    expect(activeSlug(container)).toBe(CATALOGUE[CATALOGUE.length - 1].slug);

    await fireEvent.keyDown(el, { key: "ArrowDown" });
    expect(activeSlug(container)).toBe(CATALOGUE[CATALOGUE.length - 1].slug);

    await fireEvent.keyDown(el, { key: "Home" });
    expect(activeSlug(container)).toBe(CATALOGUE[0].slug);
  });

  it("previews the row it is on when Enter is pressed", async () => {
    const { container } = render(BundleLoader);
    const el = list(container);

    await fireEvent.keyDown(el, { key: "ArrowDown" });
    await fireEvent.keyDown(el, { key: "ArrowDown" });
    await fireEvent.keyDown(el, { key: "Enter" });

    expect(previewProject).toHaveBeenCalledWith(CATALOGUE[1].slug);
  });

  it("goes nowhere on Enter before a row has been reached", async () => {
    const { container } = render(BundleLoader);
    await fireEvent.keyDown(list(container), { key: "Enter" });
    expect(previewProject).not.toHaveBeenCalled();
  });

  it("walks into the list from the search box", async () => {
    const { container } = render(BundleLoader);
    const search = container.querySelector("input") as HTMLInputElement;

    await fireEvent.keyDown(search, { key: "ArrowDown" });
    expect(activeSlug(container)).toBe(CATALOGUE[0].slug);
  });
});
