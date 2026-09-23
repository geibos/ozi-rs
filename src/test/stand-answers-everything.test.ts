import { describe, expect, it } from "vitest";
import { readFileSync } from "fs";
import { join } from "path";

const bindings = readFileSync(join(__dirname, "../lib/bindings.ts"), "utf-8");
const standCore = readFileSync(join(__dirname, "stand/tauri-core.ts"), "utf-8");

/**
 * Every command the frontend can send has an answer on the stand.
 *
 * The stand refuses a command it has no answer for, loudly, which is the right
 * behaviour and was never the problem. The problem is that nobody finds out
 * until they walk the screen that sends it — and four such gaps turned up in
 * two days, each while walking a feature that had just been built:
 * `add_waypoint`, `import_wpt`, `new_project` and `simplify_track`.
 *
 * The last was the worst. Twelve commands had no answer, and they were the
 * whole of CJ-4's editing spine — drawing, moving a point, deleting one,
 * splitting a segment, joining, sorting by time, both crops and simplifying.
 * The journey a coordinator spends most of their time in could not be walked
 * here at all, and nothing said so.
 *
 * A command with no answer is a screen nobody can look at. This fails in a
 * second instead.
 */
/**
 * Commands that are answered somewhere other than the answer table, with the
 * reason. Empty today: the tile commands, which are the obvious candidates,
 * are invoked by the MapLibre protocol handlers rather than through the
 * generated bindings, so they never reach this list at all.
 */
const ANSWERED_ELSEWHERE = new Map<string, string>();

function standCommandNames(source: string): Set<string> {
  const names = new Set<string>();
  // `command_name: (args) => …` in the answer table.
  for (const match of source.matchAll(/^ {2}([a-z0-9_]+):/gm)) {
    names.add(match[1]);
  }
  // `"command_name",` in the accepted-without-data list.
  for (const match of source.matchAll(/^\s*"([a-z0-9_]+)",$/gm)) {
    names.add(match[1]);
  }
  return names;
}

describe("the stand answers every command", () => {
  it("has an answer, or a recorded reason, for each generated command", () => {
    const commands = [
      ...new Set(
        [...bindings.matchAll(/TAURI_INVOKE\("([a-z0-9_]+)"/g)].map(
          (m) => m[1],
        ),
      ),
    ].sort();
    expect(commands.length).toBeGreaterThan(50);

    const answered = standCommandNames(standCore);
    const missing = commands.filter(
      (name) => !answered.has(name) && !ANSWERED_ELSEWHERE.has(name),
    );

    expect(
      missing,
      "these commands have no answer in src/test/stand/tauri-core.ts, so the " +
        "screens that send them cannot be walked on the stand. Add an answer " +
        "that changes what is on screen — a stand that answers 'accepted' " +
        "tells the same lie as one that answers nothing — or add the command " +
        "to ANSWERED_ELSEWHERE with the reason.",
    ).toEqual([]);
  });

  /**
   * Tauri rejects an `invoke` with the `Err(String)` the command returned, and
   * the interface shows what it is handed. The stand threw `Error` objects, so
   * every failure screen here read "{}" where the packaged application shows
   * the reason — which makes an error screen, the thing least often looked at,
   * the thing least worth looking at. The one exception is a command with no
   * fixture: that is a defect in the stand, not in the application, and it
   * should arrive loudly and with a stack.
   */
  it("rejects a command the way Tauri rejects one", () => {
    const thrown = [...standCore.matchAll(/throw new Error\(([\s\S]{0,80})/g)].map(
      (m) => m[1].replace(/\s+/g, " ").trim(),
    );
    const unexpected = thrown.filter(
      (text) => !text.includes("stand: no fixture answers"),
    );
    expect(
      unexpected,
      "throw the string the Rust command would have returned, not an Error: " +
        "an interface that renders the rejection shows `{}` for an Error.",
    ).toEqual([]);
  });

  it("does not carry reasons for commands that no longer exist", () => {
    const commands = new Set(
      [...bindings.matchAll(/TAURI_INVOKE\("([a-z0-9_]+)"/g)].map((m) => m[1]),
    );
    const stale = [...ANSWERED_ELSEWHERE.keys()].filter(
      (name) => !commands.has(name),
    );
    expect(stale, "an exemption that outlives its command is noise").toEqual(
      [],
    );
  });
});
