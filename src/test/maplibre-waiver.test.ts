import { describe, expect, it } from "vitest";
import { readdirSync, readFileSync, statSync } from "fs";
import { join } from "path";

/**
 * The premise of a security waiver, enforced.
 *
 * `scripts/npm-audit-gate.mjs` waives GHSA-jrc7-96c5-q579 — a critical XSS
 * sanitizer bypass in maplibre-gl, vulnerable in everything up to and
 * including 6.4.0 and fixed only in 6.10.0, two majors ahead of the pinned 4.x.
 * The waiver rests on one fact about this code: the vulnerable sink is
 * `DOM.sanitize()`, reached through `Popup.setHTML()`, markers built from HTML
 * strings and custom attribution, and none of those is called here. Popups use
 * `setText`, markers set `textContent`.
 *
 * A waiver whose premise is only a promise decays the first time someone
 * writes `setHTML` without reading a script in `scripts/`. This is the promise
 * as a test: break the premise and the suite says so, naming the line, rather
 * than leaving a critical advisory waived on grounds that have quietly stopped
 * being true.
 *
 * When the upgrade to 6.10 lands, this test can go — or better, stay, because
 * the sink is a bad idea regardless of who has patched it.
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
        "scripts/npm-audit-gate.mjs and upgrade to maplibre-gl 6.10",
    ).toEqual([]);
  });
});
