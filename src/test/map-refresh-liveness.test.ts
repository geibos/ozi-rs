import { describe, expect, it } from "vitest";
import { readFileSync } from "fs";
import { join } from "path";

const mapViewSource = readFileSync(
  join(__dirname, "../components/MapView.svelte"),
  "utf-8",
);

/**
 * Regression guard for the "tracks never render" class of bugs (owner's
 * hands-on session, 2026-07-15; also the confirmed audit finding "dead
 * map.once('load') fallback").
 *
 * MapLibre's "load" event fires ONCE per map lifetime. `isStyleLoaded()`
 * returns false at any later moment while tiles are loading, so the pattern
 *   if (!map.isStyleLoaded()) { map.once("load", apply); return; }
 * silently drops the refresh forever whenever it runs after startup. The
 * only legitimate deferral window is BEFORE the initial "load" event —
 * tracked by the explicit `mapLoaded` flag.
 */
describe("MapView refresh liveness", () => {
  it("never gates refreshes on isStyleLoaded (dead once-load fallback)", () => {
    // The comment explaining the pattern may mention the name; only actual
    // CALLS are the bug.
    expect(mapViewSource).not.toContain("map.isStyleLoaded()");
  });

  it("uses the one-shot mapLoaded flag for initial-load deferral", () => {
    expect(mapViewSource).toContain("let mapLoaded = false");
    expect(mapViewSource).toContain("mapLoaded = true");
    // The active-map and tracks effects defer via once("load") only while
    // the initial load has not happened yet.
    expect(mapViewSource).toContain("if (!mapLoaded)");
  });
});

/**
 * Applying the active map awaits the OZI metadata before it touches MapLibre.
 * Two things were wrong on that path, both found by external review on
 * 2026-09-22:
 *
 *   - `appliedMapPath` was set before the await, so a metadata read that
 *     failed still counted as applied and blocked every retry of that map for
 *     the rest of the session;
 *   - nothing checked, after the await, that the operator had not switched to
 *     a different map in the meantime, so two applies raced to remove and add
 *     the same source.
 *
 * `createLatestRun` is the rule the rest of the file already uses; its own
 * behaviour is covered in `latest-run.test.ts`.
 */
describe("applying the active map", () => {
  it("takes a run token and rechecks it after the metadata read", () => {
    expect(mapViewSource).toContain("const activeMapRuns = createLatestRun()");
    expect(mapViewSource).toMatch(
      /const run = activeMapRuns\.begin\(\)[\s\S]{0,600}await getOziMetadata\([\s\S]{0,300}activeMapRuns\.isCurrent\(run\)/,
    );
  });

  it("counts the map as applied only once the read has succeeded", () => {
    // Index comparison rather than a window: what matters is the order, and
    // the comment explaining it is long enough to slide out of any window
    // wide enough to be meaningful.
    const read = mapViewSource.indexOf("await getOziMetadata(");
    const applied = mapViewSource.indexOf("appliedMapPath = am.local_path");
    expect(read).toBeGreaterThan(-1);
    expect(applied).toBeGreaterThan(read);
  });

  it("reports a metadata read that failed instead of dropping it", () => {
    expect(mapViewSource).toContain('reportEditFailure("map.activeMapFailed"');
  });
});

/**
 * The same rule at the map's other two overlapping-reload sites.
 *
 * `one-rule-for-overlapping-reloads` extracted `createLatestRun` for all five
 * callers and then recorded, as an open item, that the map's two were not
 * covered by anything — `MapView` needs a MapLibre instance, so vitest cannot
 * mount it. What a test *can* do is pin the wiring: a token taken before the
 * first await, and checked before anything is written. The rule's own
 * behaviour is covered in `latest-run.test.ts`.
 */
describe("the map's overlapping reloads", () => {
  const sites = [
    { runs: "waypointMarkerRuns", what: "the waypoint marker refresh" },
    { runs: "trackGeometryRuns", what: "the track geometry refresh" },
  ];

  for (const { runs, what } of sites) {
    it(`takes a token and rechecks it around the await in ${what}`, () => {
      expect(mapViewSource).toContain(`const ${runs} = createLatestRun()`);
      // `begin()` first, the asynchronous boundary after it, then
      // `isCurrent` before anything is written. Order is what matters, so it
      // is read by index. One site awaits and the other chains `.then`; both
      // are the same rule, so either counts.
      const begin = mapViewSource.indexOf(`${runs}.begin()`);
      const check = mapViewSource.indexOf(`${runs}.isCurrent(`);
      expect(begin).toBeGreaterThan(-1);
      expect(check).toBeGreaterThan(begin);
      const between = mapViewSource.slice(begin, check);
      expect(
        between.includes("await ") || between.includes(".then("),
        "the token must be taken before the asynchronous boundary, not after",
      ).toBe(true);
    });
  }
});
