import { describe, expect, it } from "vitest";
import { readdirSync, readFileSync, statSync } from "fs";
import { join } from "path";

/**
 * A guard on the defect that reached the screen twice in one day: an English
 * literal written straight into a component.
 *
 * `aria-label`, `title` and `placeholder` are the ones that slip through,
 * because none of them is visible while writing the markup — the first two are
 * what a screen reader speaks and what this project's customer-journey smoke
 * matches on, since WKWebView publishes a control's `aria-label` rather than
 * its inner text. They sat in English inside a Russian window on the rows a
 * crew touches most.
 *
 * The rule is narrow on purpose: a literal string value containing a Latin
 * letter. Anything computed, translated or interpolated passes, so the test
 * says nothing about how a label is built — only that it is not typed in.
 */
const ROOTS = ["src/components", "src/routes"];

/** Attribute values that are not shown to anyone and never translated. */
const ALLOWED = new Set([
  // A mode group that is inert by construction; the label says so to a
  // reviewer, and the group is disabled and read by nobody.
  "Mode (inert placeholder)",
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

describe("no user-facing label is typed in", () => {
  it("has no literal aria-label, title or placeholder in any component", () => {
    const pattern =
      /\b(aria-label|title|placeholder)="([^"{}]*[A-Za-z][^"{}]*)"/g;
    const offenders: string[] = [];

    for (const root of ROOTS) {
      for (const file of svelteFiles(root)) {
        const source = readFileSync(file, "utf-8");
        for (const match of source.matchAll(pattern)) {
          const value = match[2];
          if (ALLOWED.has(value)) continue;
          offenders.push(`${file}: ${match[1]}="${value}"`);
        }
      }
    }

    expect(
      offenders,
      "translate these, or add them to ALLOWED with a reason",
    ).toEqual([]);
  });
});
