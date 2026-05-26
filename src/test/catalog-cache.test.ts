import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { get } from "svelte/store";
import {
  appendProjectsChunk,
  loadCatalogCache,
  projectsStore,
  saveCatalogCache,
} from "../lib/stores";
import type { LizaProjectSummaryDto } from "../lib/types";

const CACHE_KEY = "liza:projects:v1";

const ALPHA: LizaProjectSummaryDto = { slug: "alpha", name: "Alpha" };
const BRAVO: LizaProjectSummaryDto = { slug: "bravo", name: "Bravo" };
const CHARLIE: LizaProjectSummaryDto = { slug: "charlie", name: "Charlie" };

beforeEach(() => {
  localStorage.clear();
  projectsStore.set([]);
});

afterEach(() => {
  vi.restoreAllMocks();
});

describe("catalog cache primitives", () => {
  it("round-trips items via save → load", () => {
    saveCatalogCache([ALPHA, BRAVO]);
    const loaded = loadCatalogCache();
    expect(loaded).toEqual([ALPHA, BRAVO]);
  });

  it("returns null for a missing cache entry", () => {
    expect(loadCatalogCache()).toBeNull();
  });

  it("returns null for malformed JSON", () => {
    localStorage.setItem(CACHE_KEY, "{not valid json");
    expect(loadCatalogCache()).toBeNull();
  });

  it("returns null when the payload has the wrong shape", () => {
    localStorage.setItem(CACHE_KEY, JSON.stringify({ items: [{ slug: 1 }] }));
    expect(loadCatalogCache()).toBeNull();
  });

  it("returns null when items is missing", () => {
    localStorage.setItem(CACHE_KEY, JSON.stringify({ writtenAt: "x" }));
    expect(loadCatalogCache()).toBeNull();
  });

  it("returns null when writtenAt is missing", () => {
    localStorage.setItem(CACHE_KEY, JSON.stringify({ items: [ALPHA] }));
    expect(loadCatalogCache()).toBeNull();
  });

  it("does not throw when localStorage.setItem rejects with a quota error", () => {
    vi.spyOn(localStorage, "setItem").mockImplementation(() => {
      throw new DOMException("Quota exceeded", "QuotaExceededError");
    });
    expect(() => saveCatalogCache([ALPHA])).not.toThrow();
  });
});

describe("projectsStore hydration before first paint (task 2.3)", () => {
  it("seeds the projects store from the cache at module import time", async () => {
    // Populate the cache BEFORE the dynamic import so the module's
    // top-level initializer reads it during evaluation.
    localStorage.setItem(
      CACHE_KEY,
      JSON.stringify({
        items: [ALPHA, BRAVO],
        writtenAt: "2026-05-20T00:00:00.000Z",
      }),
    );

    vi.resetModules();
    const fresh = await import("../lib/stores");

    // First read of projectsStore — no IPC has run yet because the test
    // never imported `api`'s mock and never called `appState.refresh()`.
    expect(get(fresh.projectsStore)).toEqual([ALPHA, BRAVO]);
  });
});

describe("appendProjectsChunk upsert-by-slug (task 3.3)", () => {
  it("appends all-new slugs to the end of the list", () => {
    projectsStore.set([ALPHA]);
    appendProjectsChunk([BRAVO, CHARLIE]);
    expect(get(projectsStore)).toEqual([ALPHA, BRAVO, CHARLIE]);
  });

  it("updates known slugs in place and appends only the new ones", () => {
    projectsStore.set([ALPHA, BRAVO]);
    const updatedAlpha = { slug: "alpha", name: "Alpha (renamed)" };
    appendProjectsChunk([updatedAlpha, CHARLIE]);
    expect(get(projectsStore)).toEqual([updatedAlpha, BRAVO, CHARLIE]);
  });

  it("retains existing entries that the chunk does not reference", () => {
    projectsStore.set([ALPHA, BRAVO, CHARLIE]);
    // A "refresh" that returned only BRAVO must not drop ALPHA or CHARLIE.
    appendProjectsChunk([BRAVO]);
    expect(get(projectsStore)).toEqual([ALPHA, BRAVO, CHARLIE]);
  });

  it("ignores an empty chunk without mutating the store", () => {
    projectsStore.set([ALPHA]);
    appendProjectsChunk([]);
    expect(get(projectsStore)).toEqual([ALPHA]);
  });
});

describe("cache write on busy: true → false (task 4.3)", () => {
  it("writes once with the current catalog on the falling edge of busy", async () => {
    // Fresh module so the previousBusy tracker starts at false.
    vi.resetModules();

    // Mock the api module so appState.refresh() can return our shaped DTO
    // synchronously without an IPC call.
    let nextBusy = false;
    const baseState = {
      project_name: "",
      project_saved: true,
      status: "",
      busy: false,
      downloading_maps: [],
      projects: [],
      current_project: null,
      active_map: null,
      diagnostics: [],
      track_layers: [],
      waypoint_layers: [],
      track_layer_count: 0,
      waypoint_layer_count: 0,
      tracks: [],
    };
    vi.doMock("../lib/api", () => ({
      getAppState: vi.fn(async () => ({ ...baseState, busy: nextBusy })),
    }));

    const fresh = await import("../lib/stores");
    const setItemSpy = vi.spyOn(localStorage, "setItem");

    // Seed projects so the gate "non-empty list" is satisfied.
    fresh.projectsStore.set([ALPHA, BRAVO]);

    // busy: false → true (no cache write — only the falling edge writes)
    nextBusy = true;
    await fresh.appState.refresh();

    // busy: true → false (this transition should trigger a single write)
    nextBusy = false;
    await fresh.appState.refresh();

    const cacheWrites = setItemSpy.mock.calls.filter(
      ([key]) => key === CACHE_KEY,
    );
    expect(cacheWrites).toHaveLength(1);

    const payload = JSON.parse(cacheWrites[0][1] as string);
    expect(payload.items).toEqual([ALPHA, BRAVO]);
    expect(typeof payload.writtenAt).toBe("string");
    expect(new Date(payload.writtenAt).toISOString()).toBe(payload.writtenAt);

    vi.doUnmock("../lib/api");
  });

  it("does not write when projects list is empty on the falling edge", async () => {
    vi.resetModules();

    let nextBusy = false;
    const baseState = {
      project_name: "",
      project_saved: true,
      status: "",
      busy: false,
      downloading_maps: [],
      projects: [],
      current_project: null,
      active_map: null,
      diagnostics: [],
      track_layers: [],
      waypoint_layers: [],
      track_layer_count: 0,
      waypoint_layer_count: 0,
      tracks: [],
    };
    vi.doMock("../lib/api", () => ({
      getAppState: vi.fn(async () => ({ ...baseState, busy: nextBusy })),
    }));

    const fresh = await import("../lib/stores");
    const setItemSpy = vi.spyOn(localStorage, "setItem");

    // Leave projectsStore empty (the gate must prevent the write).
    fresh.projectsStore.set([]);

    nextBusy = true;
    await fresh.appState.refresh();
    nextBusy = false;
    await fresh.appState.refresh();

    const cacheWrites = setItemSpy.mock.calls.filter(
      ([key]) => key === CACHE_KEY,
    );
    expect(cacheWrites).toHaveLength(0);

    vi.doUnmock("../lib/api");
  });
});
