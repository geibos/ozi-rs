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
  // Units and standard abbreviations, which are written the same in both
  // languages. "CRS" heads the projection row of a map's properties; "fps"
  // belongs to the F3 developer overlay.
  "CRS",
  "fps",
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

  /**
   * Both rules above read attributes. Nothing read the words *between* the
   * tags — which is where the actions menu on every track row lived, in
   * English, including its destructive `Delete`, along with the simplify
   * dialog and two empty states. Sixteen toasts had just been found the same
   * way; a guard on one shape says nothing about another.
   */
  it("has no English written between the tags", () => {
    const offenders: string[] = [];

    for (const root of ROOTS) {
      for (const file of svelteFiles(root)) {
        for (const node of textNodes(markupOf(readFileSync(file, "utf-8")))) {
          // Three letters in a row: enough for a word, short of `px` or `⌘K`.
          if (!/[A-Za-z]{3,}/.test(node.text)) continue;
          if (ALLOWED.has(node.text)) continue;
          offenders.push(`${file}:${node.line} ${node.text.slice(0, 60)}`);
        }
      }
    }

    expect(
      offenders,
      `These are read by a crew and are typed in:\n${offenders.join("\n")}`,
    ).toEqual([]);
  });

  /**
   * The literal rule misses a label built in an expression, and that is where
   * the last two hid: `` `Symbol: ${symbol}` `` on the waypoint rows and a
   * pinned/unpinned ternary on the inspector rail. Both were found by opening
   * the screen, which is not a method.
   *
   * So: take the attribute's expression, remove what the dictionaries supply,
   * and fail on any English word left in a string inside it.
   */
  it("has no English written into a computed aria-label, title or placeholder", () => {
    const attribute = /\b(aria-label|title|placeholder)=\{/g;
    const offenders: string[] = [];

    for (const root of ROOTS) {
      for (const file of svelteFiles(root)) {
        const source = readFileSync(file, "utf-8");
        for (const match of source.matchAll(attribute)) {
          const expression = braced(source, match.index + match[0].length - 1);
          // A dictionary lookup's key is Latin by design and is not a label,
          // and it is not always a bare literal — `$t(pinned ? "a" : "b")` is
          // still a lookup. Drop the whole call.
          const rest = withoutDictionaryCalls(expression);
          for (const literal of rest.matchAll(
            /(["'`])((?:\\.|(?!\1)[^\\])*)\1/g,
          )) {
            if (!/[A-Za-z]{2,}/.test(literal[2])) continue;
            // `{name}`-style placeholders are substituted into a translation.
            if (/^\{[a-z]+\}$/.test(literal[2])) continue;
            offenders.push(`${file}: ${match[1]}={… "${literal[2]}" …}`);
          }
        }
      }
    }

    expect(
      offenders,
      "these labels are built from English rather than from the dictionaries",
    ).toEqual([]);
  });
});

/**
 * The expression with every `$t(…)` / `$i18n(…)` call removed, arguments and
 * all, so that what remains is the text the component supplies itself.
 */
function withoutDictionaryCalls(expression: string): string {
  let out = "";
  for (let i = 0; i < expression.length; ) {
    const call = /^\$(?:t|i18n)\(/.exec(expression.slice(i));
    if (!call) {
      out += expression[i];
      i += 1;
      continue;
    }
    let depth = 0;
    let j = i + call[0].length - 1;
    for (; j < expression.length; j += 1) {
      if (expression[j] === "(") depth += 1;
      else if (expression[j] === ")") {
        depth -= 1;
        if (depth === 0) break;
      }
    }
    i = j + 1;
  }
  return out;
}

/**
 * The markup of a component: what is between `</script>` and `<style>`.
 *
 * Comments and `{…}` expressions are blanked rather than removed, so line
 * numbers still mean something, and `<style>` never enters — a CSS comment
 * full of English is not a label.
 */
function markupOf(source: string): string {
  const start = source.lastIndexOf("</script>");
  const end = source.lastIndexOf("<style>");
  const markup = source.slice(
    start >= 0 ? start + "</script>".length : 0,
    end > start ? end : undefined,
  );
  return markup
    .replace(/<!--[\s\S]*?-->/g, (c) => c.replace(/[^\n]/g, " "))
    .replace(/\{(?:[^{}]|\{[^{}]*\})*\}/g, (e) => e.replace(/[^\n]/g, " "));
}

/** The text a crew would read, node by node. */
function textNodes(markup: string): Array<{ text: string; line: number }> {
  const out: Array<{ text: string; line: number }> = [];
  for (const match of markup.matchAll(/>([^<>]+)</g)) {
    const text = match[1].replace(/\s+/g, " ").trim();
    if (text) {
      out.push({ text, line: markup.slice(0, match.index).split("\n").length });
    }
  }
  return out;
}

/** The text of a `{…}` expression starting at `open`, brace-balanced. */
function braced(source: string, open: number): string {
  let depth = 0;
  for (let i = open; i < source.length; i += 1) {
    if (source[i] === "{") depth += 1;
    else if (source[i] === "}") {
      depth -= 1;
      if (depth === 0) return source.slice(open + 1, i);
    }
  }
  return source.slice(open + 1);
}
