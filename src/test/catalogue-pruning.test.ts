import { beforeEach, describe, expect, it } from "vitest";
import { get } from "svelte/store";
import {
  appendProjectsChunk,
  beginCatalogueRefresh,
  finishCatalogueRefresh,
  projectsStore,
} from "../lib/stores";
import type { LizaProjectSummaryDto } from "../lib/types";

/**
 * A search taken down upstream used to stay in the list for good: chunks are
 * merged and nothing ever removed anything, so it sat in the list, then in the
 * cache, and clicking it failed.
 *
 * Only a walk that ran to the end knows what no longer exists. A stopped one
 * read a prefix of the catalogue and must remove nothing — otherwise pressing
 * "stop" would delete most of the list.
 */
const row = (slug: string): LizaProjectSummaryDto => ({
  slug,
  name: slug.replace(/_/g, " "),
  cached: false,
});

beforeEach(() => {
  projectsStore.set([row("2026-09-01_old"), row("2026-09-20_current")]);
});

const slugs = () => get(projectsStore).map((p) => p.slug);

describe("pruning the catalogue after a refresh", () => {
  it("drops what a completed walk did not send", () => {
    beginCatalogueRefresh();
    appendProjectsChunk([row("2026-09-20_current")]);
    finishCatalogueRefresh(true);

    expect(slugs()).toEqual(["2026-09-20_current"]);
  });

  it("keeps everything when the walk was stopped", () => {
    beginCatalogueRefresh();
    appendProjectsChunk([row("2026-09-20_current")]);
    finishCatalogueRefresh(false);

    expect(slugs()).toEqual(["2026-09-01_old", "2026-09-20_current"]);
  });

  it("adds what the walk found that was not there before", () => {
    beginCatalogueRefresh();
    appendProjectsChunk([row("2026-09-20_current"), row("2026-09-21_new")]);
    finishCatalogueRefresh(true);

    expect(slugs()).toEqual(["2026-09-20_current", "2026-09-21_new"]);
  });

  /**
   * The cached chunk is emitted before the walk starts. Counting it as proof
   * that a search still exists would defeat the whole thing.
   */
  it("does not count a chunk that arrived before the refresh began", () => {
    appendProjectsChunk([row("2026-09-01_old")]);
    beginCatalogueRefresh();
    appendProjectsChunk([row("2026-09-20_current")]);
    finishCatalogueRefresh(true);

    expect(slugs()).toEqual(["2026-09-20_current"]);
  });

  it("prunes nothing when no refresh was in progress", () => {
    finishCatalogueRefresh(true);
    expect(slugs()).toEqual(["2026-09-01_old", "2026-09-20_current"]);
  });
});
