import { describe, expect, it } from "vitest";
import { readdirSync, readFileSync, statSync } from "fs";
import { join } from "path";

/**
 * A guard on failures that reach nobody.
 *
 * Dragging a track point, deleting one, inserting one, adding a waypoint,
 * drawing a point, cancelling a draw, fitting the tracks on screen and loading
 * either inspector all reported their failure to `console.error` and nowhere
 * else — and the error reporter is disabled outside dev builds, so in a
 * release build they were silent. The operator acted, the backend declined,
 * and nothing said so.
 *
 * The loader was given this treatment in `honest-bundle-flow`; the map and the
 * inspectors never were, and nothing was watching. This watches.
 */
const ROOTS = ["src/components", "src/routes"];

/**
 * Failures a build is right to swallow, with the reason.
 *
 * A key is `file:text-of-the-log-call`.
 */
const ALLOWED = new Map<string, string>([
  [
    "src/components/library/TracksTab.svelte:Failed to import track file",
    "Reported in aggregate after the loop: a folder import must not abort on " +
      "one bad file, and one toast per file would bury the summary. The toast " +
      "that follows names every file that failed.",
  ],
  [
    "src/components/inspector/MapInspector.svelte:MapInspector: getOziMetadata failed",
    "Not every map is OZF2: SQLite tile maps have no OZF2 metadata and this " +
      "call is expected to fail for them. Telling the operator would be noise.",
  ],
]);

function svelteFiles(dir: string): string[] {
  const out: string[] = [];
  for (const entry of readdirSync(dir)) {
    const full = join(dir, entry);
    if (statSync(full).isDirectory()) out.push(...svelteFiles(full));
    else if (entry.endsWith(".svelte")) out.push(full);
  }
  return out;
}

describe("no failure is reported only to the console", () => {
  it("has no catch block that logs and tells nobody", () => {
    // A `catch` body, allowing one nested level of braces.
    const catchBlock = /\} catch \([^)]*\) \{((?:[^{}]|\{[^{}]*\})*)\}/g;
    const logged = /console\.(?:error|warn)\(\s*["'`]([^"'`]*)/;
    const offenders: string[] = [];

    for (const root of ROOTS) {
      for (const file of svelteFiles(root)) {
        const source = readFileSync(file, "utf-8");
        for (const block of source.matchAll(catchBlock)) {
          const body = block[1];
          const log = logged.exec(body);
          if (!log) continue;
          if (body.includes("toast") || body.includes("reportEditFailure")) {
            continue;
          }
          const key = `${file}:${log[1]}`;
          if (ALLOWED.has(key)) continue;
          offenders.push(key);
        }
      }
    }

    expect(
      offenders,
      "report these to the operator, or add them to ALLOWED with a reason",
    ).toEqual([]);
  });
});
