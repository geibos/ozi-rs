import { describe, expect, it } from "vitest";
import { readFileSync } from "fs";
import { join } from "path";

/**
 * The stand fires `state-changed` after exactly the commands the application
 * fires it after.
 *
 * Nothing in the interface knows a command landed except this event. A screen
 * that reloads its own data hides that; a screen that waits — the unsaved
 * indicator, the layer rail, the map's fingerprint — does not. The stand
 * listed two commands while `src-tauri/src/commands/mod.rs` emitted after
 * fifty-four, so those screens sat still here and moved in the packaged
 * application. That is the inversion the stand exists to prevent: a defect
 * found here that is not real costs an afternoon, and worse, teaches whoever
 * walked it to distrust the stand.
 *
 * Read out of the Rust rather than restated, so adding an emit on that side
 * fails here instead of quietly widening the gap again.
 */
function commandsThatEmitStateChanged(rust: string): string[] {
  const functions: Array<{ at: number; name: string }> = [];
  for (const match of rust.matchAll(/\npub fn ([a-z0-9_]+)/g)) {
    functions.push({ at: match.index ?? 0, name: match[1] });
  }

  const names = new Set<string>();
  for (const match of rust.matchAll(/emit\("state-changed"/g)) {
    const at = match.index ?? 0;
    // The emit belongs to the last function that opened before it, including
    // emits inside a closure the function spawned.
    let owner: string | null = null;
    for (const fn of functions) {
      if (fn.at >= at) break;
      owner = fn.name;
    }
    if (owner) names.add(owner);
  }
  return [...names].sort();
}

describe("the stand's state-changed events", () => {
  it("fire after the same commands the application fires them after", () => {
    const rust = readFileSync(
      join(__dirname, "../../src-tauri/src/commands/mod.rs"),
      "utf-8",
    );
    const expected = commandsThatEmitStateChanged(rust);
    expect(expected.length).toBeGreaterThan(40);

    const stand = readFileSync(join(__dirname, "stand/tauri-core.ts"), "utf-8");
    const declared = stand.match(
      /const EMITS_STATE_CHANGED = new Set\(\[([\s\S]*?)\]\);/,
    );
    expect(declared, "EMITS_STATE_CHANGED is not where this test looks").not.toBe(
      null,
    );
    const listed = [
      ...(declared as RegExpMatchArray)[1].matchAll(/"([a-z0-9_]+)"/g),
    ]
      .map((m) => m[1])
      .sort();

    const missing = expected.filter((name) => !listed.includes(name));
    const extra = listed.filter((name) => !expected.includes(name));

    expect(
      { missing, extra },
      "the application tells the interface a command landed by emitting " +
        "`state-changed`. A command missing here leaves the stand's screens " +
        "showing the state from before the click; an extra one refreshes " +
        "where the real application does not.",
    ).toEqual({ missing: [], extra: [] });
  });
});
