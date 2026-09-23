import { describe, expect, it } from "vitest";
import { readdirSync, readFileSync, statSync } from "fs";
import { join } from "path";

/**
 * An export that says nothing is indistinguishable from one that did not run.
 *
 * CJ-6 ends with a file handed to a group. Four of the export paths — the
 * single-file ones, which are exactly the ones used to hand one crew's track
 * over — wrote the file and told the operator nothing at all. The day export
 * had a success toast from the first day; the others never did, and nobody
 * noticed because the failure looks like success until somebody goes to check
 * whether the file exists.
 *
 * Found by walking CJ-6 on the stand, 2026-09-23. The day export was found
 * later the same day, walking a whole session end to end: it said "three
 * tracks and three marks" and never said where they went, which is the half a
 * coordinator handing the file over actually needs. It passed this guard,
 * because the guard read the file rather than the call — see below.
 */
const EXPORT_CALLS = [
  "exportGpx",
  "exportTrackPlt",
  "exportWptWaypoints",
  "exportGpxWaypoints",
  "exportAllTracksGpx",
] as const;

function componentFiles(dir: string): string[] {
  const out: string[] = [];
  for (const entry of readdirSync(dir)) {
    const full = join(dir, entry);
    if (statSync(full).isDirectory()) out.push(...componentFiles(full));
    else if (entry.endsWith(".svelte")) out.push(full);
  }
  return out;
}

describe("every export tells the operator it happened", () => {
  it("reports success wherever a file is written", () => {
    const offenders: string[] = [];

    for (const file of componentFiles("src/components")) {
      const source = readFileSync(file, "utf-8");
      const lines = source.split("\n");

      for (const call of EXPORT_CALLS) {
        const pattern = new RegExp(`await ${call}\\s*\\(`);
        lines.forEach((line, index) => {
          if (!pattern.test(line)) return;
          // The twenty lines after the call, not the whole file. The first
          // version of this checked the file and let the day export through
          // for a year's worth of a day: `TracksTab.svelte` contains both
          // `reportExported(` and a `toast.success` belonging to other
          // things, so every export in it passed whatever it actually did.
          // A guard that checks the neighbourhood instead of the file is the
          // difference between catching that and not.
          const after = lines.slice(index, index + 20).join("\n");
          if (after.includes("reportExported(")) return;
          offenders.push(
            `${file}:${index + 1}: calls ${call} and does not report it`,
          );
        });
      }
    }

    expect(
      offenders,
      "an export that says nothing cannot be told from one that did not run. " +
        "Use `reportExported` from $lib/actions/export-result, which also " +
        "offers to show the file.",
    ).toEqual([]);
  });
});
