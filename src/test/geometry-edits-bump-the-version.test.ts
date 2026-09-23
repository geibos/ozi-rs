import { describe, expect, it } from "vitest";
import { readdirSync, readFileSync, statSync } from "fs";
import { join } from "path";

/**
 * Every edit that changes a track's shape invalidates what is drawn from it.
 *
 * The inspector caches a track's detail and the map caches its geometry, both
 * keyed on `tracksGeometryVersion`. An operation that changes the shape and
 * forgets to bump it leaves the statistics saying one thing — "three points"
 * — while the segment table beside them still lists five, and the line on the
 * map still has the points the operator just removed. It reads as "nothing
 * happened", which is worse than an error.
 *
 * Five of the six geometry operations bumped it. Simplify did not, and it was
 * found by walking CJ-4 on the stand on 2026-09-23 rather than by any test:
 * the operation lives in a different component from the other five, so the
 * pattern "call the command, then remember to bump" had a place to fail.
 *
 * This is that pattern, checked. It does not prove the version is bumped at
 * the right moment; it proves nobody forgot.
 */
const GEOMETRY_CHANGING = [
  "simplifyTrack",
  "sortTrackPoints",
  "cropTrackToExtent",
  "cropTrackToTime",
  "trimTrackAtPoint",
  "splitSegment",
  "joinSegments",
] as const;

/**
 * The map's own editing handlers are exempt: a drag, an insert or a delete on
 * the canvas reloads the points it is drawing directly
 * (`reloadEditableTrackPoints`), because the operator is looking at that
 * track and a version bump would rebuild the whole layer under their hand.
 */
const EXEMPT_FILES = ["MapView.svelte"];

function componentFiles(dir: string): string[] {
  const out: string[] = [];
  for (const entry of readdirSync(dir)) {
    const full = join(dir, entry);
    if (statSync(full).isDirectory()) out.push(...componentFiles(full));
    else if (entry.endsWith(".svelte")) out.push(full);
  }
  return out;
}

describe("an edit that changes a track's shape", () => {
  it("bumps the geometry version everywhere it is called", () => {
    const offenders: string[] = [];

    for (const file of componentFiles("src/components")) {
      if (EXEMPT_FILES.some((name) => file.endsWith(name))) continue;
      const source = readFileSync(file, "utf-8");

      for (const command of GEOMETRY_CHANGING) {
        // A call, not the import line and not a mention in a comment.
        const called = new RegExp(`await ${command}\\s*\\(`).test(source);
        if (!called) continue;
        if (source.includes("tracksGeometryVersion.update")) continue;
        offenders.push(`${file}: calls ${command} and never bumps the version`);
      }
    }

    expect(
      offenders,
      "these change a track's shape without invalidating what is drawn from " +
        "it, so the segment table and the map line go on showing points that " +
        "are gone. Bump `tracksGeometryVersion` after the call.",
    ).toEqual([]);
  });
});
