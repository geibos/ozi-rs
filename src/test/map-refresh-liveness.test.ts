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
