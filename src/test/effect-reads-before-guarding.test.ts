import { describe, expect, it } from "vitest";
import { readFileSync, readdirSync, statSync } from "fs";
import { join } from "path";

/**
 * An `$effect` must read what it depends on before it gives up.
 *
 * Svelte 5 subscribes an effect to what it actually reads. An effect that
 * starts `if (!map) return;` and reads `$selectedTrack` afterwards will, on a
 * first run where the map does not exist yet, read nothing reactive at all —
 * and never run again. The selection highlight was written that way and was
 * simply dead: the row lit up, the map did not.
 *
 * Whether it bites depends on declaration order against `onMount`, which is
 * why four more like it in `MapView` were working. That is not a property to
 * rely on.
 *
 * A read counts if it is a store (`$name`) or a rune-declared `$state`
 * variable of the same file: both keep the effect subscribed.
 */
const ROOTS = ["src/components", "src/routes"];

function svelteFiles(dir: string, out: string[] = []): string[] {
  for (const entry of readdirSync(dir)) {
    const path = join(dir, entry);
    if (statSync(path).isDirectory()) svelteFiles(path, out);
    else if (entry.endsWith(".svelte")) out.push(path);
  }
  return out;
}

/** Names declared with `= $state(...)` or `= $derived(...)` in this file. */
function reactiveLocals(source: string): Set<string> {
  const names = new Set<string>();
  for (const match of source.matchAll(
    /\b(?:let|const)\s+([A-Za-z_]\w*)[^=\n]*=\s*\$(?:state|derived)\b/g,
  )) {
    names.add(match[1]);
  }
  return names;
}

/**
 * Comments blanked, line numbers kept.
 *
 * The same lesson the toast guard learned and this one had to learn again:
 * the comment explaining this very rule quotes `if (!map) return`, and a guard
 * that reads its own documentation as code flags the fix it asked for.
 */
function withoutComments(source: string): string {
  return source
    .replace(/\/\*[\s\S]*?\*\//g, (block) => block.replace(/[^\n]/g, " "))
    .replace(
      /(^|[^:])\/\/[^\n]*/g,
      (line, prefix) => prefix + " ".repeat(line.length - prefix.length),
    );
}

/** The body of each `$effect(() => { … })`, brace-balanced. */
function effectBodies(source: string): Array<{ body: string; line: number }> {
  const out: Array<{ body: string; line: number }> = [];
  for (const match of source.matchAll(/\$effect\(\(\)\s*=>\s*\{/g)) {
    const open = source.indexOf("{", match.index + match[0].length - 1);
    let depth = 0;
    for (let i = open; i < source.length; i += 1) {
      if (source[i] === "{") depth += 1;
      else if (source[i] === "}") {
        depth -= 1;
        if (depth === 0) {
          out.push({
            body: source.slice(open + 1, i),
            line: source.slice(0, match.index).split("\n").length,
          });
          break;
        }
      }
    }
  }
  return out;
}

function readsSomethingReactive(text: string, locals: Set<string>): boolean {
  if (/\$[A-Za-z_]\w*/.test(text)) return true;
  return [...locals].some((name) => new RegExp(`\\b${name}\\b`).test(text));
}

describe("an effect reads its dependencies before it gives up", () => {
  it("has no effect that returns before reading anything reactive", () => {
    const offenders: string[] = [];

    for (const root of ROOTS) {
      for (const file of svelteFiles(root)) {
        const source = withoutComments(readFileSync(file, "utf-8"));
        const locals = reactiveLocals(source);
        for (const { body, line } of effectBodies(source)) {
          const stop = body.search(/\breturn\b/);
          if (stop === -1) continue;
          const before = body.slice(0, stop);
          const after = body.slice(stop);
          if (readsSomethingReactive(before, locals)) continue;
          if (!readsSomethingReactive(after, locals)) continue;
          offenders.push(`${file}:${line} ${before.trim().slice(0, 50)}`);
        }
      }
    }

    expect(
      offenders,
      `These effects give up before reading anything reactive, so a first run
that takes the early exit leaves them subscribed to nothing:\n${offenders.join("\n")}`,
    ).toEqual([]);
  });

  it("does not read its own documentation as code", () => {
    const sample =
      "$effect(() => {\n  // if (!map) return\n  use($store);\n});";
    expect(withoutComments(sample)).not.toContain("return");
  });

  it("counts a `$state` local as a reactive read", () => {
    const source = `let filter = $state(""); $effect(() => { if (!filter) return; use($projects); });`;
    const locals = reactiveLocals(source);
    expect(locals.has("filter")).toBe(true);
    expect(readsSomethingReactive("if (!filter) ", locals)).toBe(true);
  });

  it("does not count a plain local", () => {
    const source = `let map; $effect(() => { if (!map) return; use($projects); });`;
    expect(readsSomethingReactive("if (!map) ", reactiveLocals(source))).toBe(
      false,
    );
  });
});
