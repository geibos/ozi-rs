import { describe, expect, it } from "vitest";
import { readFileSync } from "fs";
import { join } from "path";

/**
 * Every registered command has a line in the commands reference.
 *
 * `CLAUDE.md` sends every contributor to `docs/commands-reference.md` for how
 * an edit reaches the project, and twenty of seventy-one commands were not on
 * it — layer management, `.wpt` import, starting a new search, the note beside
 * a mark, trimming a track. Somebody looking for one of those found nothing,
 * and "not in the reference" reads as "not there".
 *
 * The page had said, honestly, that it named only the commands a reader was
 * likely to look for, because a hand-kept copy of a list that changes goes
 * stale — which is exactly what happened to the feature-status preamble and is
 * why that one now points at the registry instead. A reference cannot point at
 * the registry, though: a reader who wants to know what `trim_track_at_point`
 * does is not helped by being told where the list lives. So the page is
 * complete, and this keeps it complete.
 */
describe("the commands reference", () => {
  const lib = readFileSync(
    join(__dirname, "../../src-tauri/src/lib.rs"),
    "utf-8",
  );
  const reference = readFileSync(
    join(__dirname, "../../docs/commands-reference.md"),
    "utf-8",
  );

  /**
   * The registry names commands as `commands::<name>`, and a command living in
   * a submodule as `commands::<module>::<name>` — the tiles block, and since
   * 2026-09-23 the report one. Only the last segment is a command name.
   *
   * The first version of this named `tiles` explicitly, so adding a second
   * submodule made it report `report` as a missing command. Taking the last
   * segment works for any number of them.
   */
  function registeredCommands(source: string): string[] {
    const names = new Set<string>();
    for (const match of source.matchAll(
      /commands::((?:[a-z0-9_]+::)*[a-z0-9_]+)/g,
    )) {
      const segments = match[1].split("::");
      names.add(segments[segments.length - 1]);
    }
    return [...names].sort();
  }

  it("has a line for every command the backend registers", () => {
    const registered = registeredCommands(lib);
    expect(registered.length).toBeGreaterThan(60);

    const missing = registered.filter(
      (name) => !reference.includes(`\`${name}\``),
    );
    expect(
      missing,
      "these commands are registered in src-tauri/src/lib.rs and have no line " +
        "in docs/commands-reference.md, which is the page every contributor " +
        "is sent to. A command missing from it reads as a command that does " +
        "not exist.",
    ).toEqual([]);
  });

  it("does not describe commands that were removed", () => {
    const registered = new Set(registeredCommands(lib));
    // Command-shaped names in the tables: a row's first cell, in backticks.
    const described = [...reference.matchAll(/^\| `([a-z0-9_]+)` \|/gm)].map(
      (m) => m[1],
    );
    const stale = described.filter((name) => !registered.has(name));
    expect(
      stale,
      "a row for a command that no longer exists sends a reader looking for " +
        "something that is not there",
    ).toEqual([]);
  });
});
