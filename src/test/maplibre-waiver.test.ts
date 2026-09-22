import { describe, expect, it } from "vitest";
import { readdirSync, readFileSync, statSync } from "fs";
import { join } from "path";

/**
 * The premise of a security waiver, enforced.
 *
 * `scripts/npm-audit-gate.mjs` waives GHSA-jrc7-96c5-q579 — a critical XSS
 * sanitizer bypass in maplibre-gl. The advisory covers everything up to and
 * including 6.4.0, so 6.4.1 is the first release without it; `npm audit`
 * names 6.10.0 because that is the latest, not the earliest fix. Either is
 * two majors ahead of the 4.7.1 this lockfile resolves.
 *
 * The waiver rests on one fact about this code: the vulnerable sink is
 * `DOM.sanitize()`, reached through `Popup.setHTML()`, markers built from HTML
 * strings and attribution HTML, and none of those carries anything this code
 * did not write. Popups use `setText`, markers set `textContent`.
 *
 * Attribution is the case an earlier version of this test missed, and an
 * external reviewer caught on 2026-09-22: the advisory is specifically about
 * untrusted attribution strings, and the guard only looked for MapLibre's
 * `customAttribution` option, not the `attribution` a source carries. A
 * constant is fine — the OSM line in `MapView` is one — so what is forbidden
 * is an attribution assembled from anything but string literals.
 *
 * A waiver whose premise is only a promise decays the first time someone
 * writes `setHTML` without reading a script in `scripts/`. This is the promise
 * as a test: break the premise and the suite says so, naming the line, rather
 * than leaving a critical advisory waived on grounds that have quietly stopped
 * being true.
 *
 * When the upgrade past 6.4.0 lands, this test can go — or better, stay,
 * because the sink is a bad idea regardless of who has patched it.
 */
const ROOTS = ["src"];

/** The calls that reach the sanitizer the advisory is about. */
const FORBIDDEN = [
  { pattern: /\.setHTML\s*\(/, why: "Popup.setHTML reaches DOM.sanitize()" },
  {
    pattern: /\.innerHTML\s*=/,
    why: "innerHTML on an element MapLibre later sanitizes, and unsafe anyway",
  },
  {
    pattern: /customAttribution\s*:/,
    why: "custom attribution HTML reaches DOM.sanitize()",
  },
];

function sourceFiles(dir: string): string[] {
  const out: string[] = [];
  for (const entry of readdirSync(dir)) {
    if (entry === "fixtures") continue;
    const full = join(dir, entry);
    if (statSync(full).isDirectory()) out.push(...sourceFiles(full));
    else if (/\.(ts|svelte)$/.test(entry) && !full.endsWith(".test.ts")) {
      out.push(full);
    }
  }
  return out;
}

describe("the maplibre advisory waiver's premise", () => {
  it("holds: nothing reaches the sanitizer the advisory is about", () => {
    const offenders: string[] = [];

    for (const root of ROOTS) {
      for (const file of sourceFiles(root)) {
        const source = readFileSync(file, "utf-8");
        source.split("\n").forEach((line, i) => {
          // This file names the calls in order to forbid them.
          if (file.endsWith("maplibre-waiver.test.ts")) return;
          for (const { pattern, why } of FORBIDDEN) {
            if (pattern.test(line)) {
              offenders.push(`${file}:${i + 1}: ${line.trim()} — ${why}`);
            }
          }
        });
      }
    }

    expect(
      offenders,
      "these reach the sink that GHSA-jrc7-96c5-q579 is waived on the grounds " +
        "of never calling; either avoid them or take the waiver out of " +
        "scripts/npm-audit-gate.mjs and upgrade past maplibre-gl 6.4.0",
    ).toEqual([]);
  });

  it("holds: every attribution string is a literal this repo wrote", () => {
    // The advisory is about untrusted attribution. A constant cannot be
    // untrusted; anything assembled at runtime — a variable, a template
    // literal, a call — can be, and would reach `DOM.sanitize()` through a
    // path the waiver claims is never taken.
    // Written as "find the key, then look at its value" rather than one
    // negative-lookahead regex: `\s*` backtracks to zero width, so the
    // lookahead fires on the newline of a value prettier put on its own line.
    const key = /\battribution\s*:/g;
    const offenders: string[] = [];

    for (const root of ROOTS) {
      for (const file of sourceFiles(root)) {
        if (file.endsWith("maplibre-waiver.test.ts")) continue;
        const source = readFileSync(file, "utf-8");
        for (const match of source.matchAll(key)) {
          const after = source.slice(match.index + match[0].length).trimStart();
          if (after.startsWith('"') || after.startsWith("'")) continue;
          const line = source.slice(0, match.index).split("\n").length;
          offenders.push(
            `${file}:${line}: attribution is not a string literal`,
          );
        }
      }
    }

    expect(
      offenders,
      "an attribution built at runtime can carry untrusted HTML into " +
        "DOM.sanitize(), which is exactly what GHSA-jrc7-96c5-q579 is about " +
        "and what the waiver in scripts/npm-audit-gate.mjs says never happens",
    ).toEqual([]);
  });
});
