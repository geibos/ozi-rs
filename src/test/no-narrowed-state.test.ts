import { describe, expect, it } from "vitest";
import { readdirSync, readFileSync, statSync } from "fs";
import { join } from "path";

/**
 * A guard on a trap that has bitten twice.
 *
 * `let x: T | null = $state(null)` type-checks until something reads `x` in an
 * inline `$derived`. TypeScript's flow analysis narrows the variable to `null`
 * at every point before its first assignment — which is exactly where an
 * inline derived sits — so the non-null branch becomes `never` and every
 * property read on it fails with "Property … does not exist on type 'never'".
 * That message reads like a broken type and is not one, which is what makes it
 * expensive: it cost a wrong diagnosis in the waypoint-colour slice and again
 * in the point walkthrough.
 *
 * Function bodies are deferred and never see it, so a file can carry the
 * declaration for months and fail the day someone adds a derived over it.
 *
 * `$state<T>(null)` declares the type instead of narrowing to it.
 */
const ROOTS = ["src/components", "src/routes"];

function svelteFiles(dir: string): string[] {
  const out: string[] = [];
  for (const entry of readdirSync(dir)) {
    const full = join(dir, entry);
    if (statSync(full).isDirectory()) out.push(...svelteFiles(full));
    else if (entry.endsWith(".svelte")) out.push(full);
  }
  return out;
}

describe("state that a derived can read", () => {
  it("is declared with $state<T>(), not by annotating the let", () => {
    // Only `$state(null)`. An array or an object narrows too — `[]` becomes
    // `never[]` — but `never[]` is assignable to anything, so it is passed
    // along harmlessly; it is `null` that makes the non-null branch `never`
    // and breaks a property read. A guard that flagged the harmless cases
    // would be one somebody turns off.
    const annotated = /\blet\s+\w+\s*:\s*[^=;]+=\s*\$state\(null\)/g;
    const offenders: string[] = [];

    for (const root of ROOTS) {
      for (const file of svelteFiles(root)) {
        const source = readFileSync(file, "utf-8");
        for (const match of source.matchAll(annotated)) {
          const line = source.slice(0, match.index).split("\n").length;
          offenders.push(`${file}:${line}: ${match[0].trim()}…`);
        }
      }
    }

    expect(
      offenders,
      "write these as `$state<T>(…)`: an annotated let is narrowed to its " +
        "initial value in every inline $derived in the file",
    ).toEqual([]);
  });
});
