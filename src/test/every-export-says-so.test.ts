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
 * Found by walking CJ-6 on the stand, 2026-09-23.
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
      for (const call of EXPORT_CALLS) {
        if (!new RegExp(`await ${call}\\s*\\(`).test(source)) continue;
        // Either the shared reporter, or a success toast of its own — the day
        // export writes its own summary with counts, which says more than the
        // shared one could.
        const reports =
          source.includes("reportExported(") ||
          source.includes("toast.success");
        if (reports) continue;
        offenders.push(`${file}: calls ${call} and says nothing on success`);
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
