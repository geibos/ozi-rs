import { describe, expect, it } from "vitest";
import { readFileSync, readdirSync, statSync } from "fs";
import { join } from "path";

/**
 * A toast's message must come from the dictionary.
 *
 * The third instance of the same defect in this project: the library rows'
 * tooltips were English inside a Russian window, the bundle progress arrived
 * as an English sentence from the backend, and the folder import's summary
 * did too. Each was found by looking at one screen. The rule here is that
 * when a defect turns up a third time the fix is a test over its shape.
 *
 * `no-untranslated-labels` already watches what is written into an
 * `aria-label` or a title. A toast is neither, so an entire class of sentence
 * — the ones a crew reads at the moment something failed — went unwatched.
 *
 * Only the first argument is checked. The second is the description, where
 * `String(error)` is exactly right: the backend's own words about a failure
 * are evidence, not interface text.
 */
const TOAST_CALL =
  /\btoast\.(success|error|warning|message|info|loading)\s*\(/g;

function sourceFiles(dir: string, out: string[] = []): string[] {
  for (const entry of readdirSync(dir)) {
    const path = join(dir, entry);
    if (statSync(path).isDirectory()) {
      if (entry === "test" || entry === "node_modules") continue;
      sourceFiles(path, out);
    } else if (entry.endsWith(".svelte") || entry.endsWith(".ts")) {
      out.push(path);
    }
  }
  return out;
}

/** The first argument's text, read with balanced brackets. */
function firstArgument(source: string, openParen: number): string {
  let depth = 0;
  for (let i = openParen; i < source.length; i += 1) {
    const ch = source[i];
    if (ch === "(" || ch === "[" || ch === "{") depth += 1;
    else if (ch === ")" || ch === "]" || ch === "}") {
      depth -= 1;
      if (depth === 0) return source.slice(openParen + 1, i);
    } else if (ch === "," && depth === 1) {
      return source.slice(openParen + 1, i);
    }
  }
  return "";
}

const LITERAL_START = /^\s*["'`]/;

/**
 * Whether a literal argument is one a crew would read.
 *
 * The same narrow rule `no-untranslated-labels` uses: a Latin letter typed in.
 * A template literal built entirely from translation calls — `` `${$t("a")} —
 * ${$t("b")}` `` — has none outside its interpolations and passes, so the
 * guard says nothing about how a message is assembled, only that it is not
 * typed in.
 */
function isTypedIn(argument: string): boolean {
  if (!LITERAL_START.test(argument)) return false;
  const withoutInterpolations = argument.replace(/\$\{[^}]*\}/g, "");
  return /[A-Za-z]/.test(withoutInterpolations);
}

/**
 * Comments are not code. Stripping them matters more than it looks: `api.ts`
 * explains this very rule in a comment that quotes a `toast.error("Failed to
 * …")`, and a guard that flagged its own documentation would teach people to
 * work around it.
 */
function withoutComments(source: string): string {
  return source
    .replace(/\/\*[\s\S]*?\*\//g, (block) => block.replace(/[^\n]/g, " "))
    .replace(
      /(^|[^:])\/\/[^\n]*/g,
      (line, prefix) => prefix + " ".repeat(line.length - prefix.length),
    );
}

function englishToasts(path: string): string[] {
  const source = withoutComments(readFileSync(path, "utf-8"));
  const found: string[] = [];
  for (const match of source.matchAll(TOAST_CALL)) {
    const open = match.index + match[0].length - 1;
    const argument = firstArgument(source, open);
    if (!isTypedIn(argument)) continue;
    const line = source.slice(0, match.index).split("\n").length;
    found.push(`${path}:${line} ${argument.trim().slice(0, 60)}`);
  }
  return found;
}

describe("a toast says what the dictionary says", () => {
  it("no toast is raised with a literal message", () => {
    const offenders = [
      ...sourceFiles(join(__dirname, "../components")),
      ...sourceFiles(join(__dirname, "../routes")),
      ...sourceFiles(join(__dirname, "../lib")),
    ].flatMap(englishToasts);

    expect(
      offenders,
      `A toast message must be a dictionary lookup — these are literals, and a
Russian crew reads them in English at the moment something failed:\n${offenders.join("\n")}`,
    ).toEqual([]);
  });

  it("catches a literal it is given", () => {
    // The guard is only worth having if it bites; this proves the reader
    // finds the message rather than the description beside it.
    const sample = `toast.error("Failed to do the thing", { description: String(e) });`;
    const open = sample.indexOf("(");
    expect(firstArgument(sample, open)).toBe('"Failed to do the thing"');
  });

  it("leaves a dictionary lookup and its description alone", () => {
    const sample = `toast.error($t("a.b"), { description: String(e) });`;
    expect(isTypedIn(firstArgument(sample, sample.indexOf("(")))).toBe(false);
  });

  it("allows a message assembled entirely from translations", () => {
    const sample =
      'toast.message(`${$t("a.b")} — ${$t("a.c")}`, { description: x });';
    expect(isTypedIn(firstArgument(sample, sample.indexOf("(")))).toBe(false);
  });

  it("does not read its own documentation as code", () => {
    // `api.ts` explains this rule in a comment that quotes a bad call.
    const sample = '// call sites keep toast.error("Failed to …") working\n';
    expect(withoutComments(sample)).not.toContain("Failed");
  });
});
