import { describe, expect, it } from "vitest";
import { hasCyrillic, transliteratedPattern } from "../lib/translit";

/**
 * The LizaAlert catalogue names its searches in latin transliteration —
 * `2026-09-20_Schuvalovo`, `2026-07-14_Sagra` — while the crew that types
 * into the filter box thinks, and writes, in Russian. Before this, a Cyrillic
 * query matched nothing at all, in a list of about thirteen thousand rows
 * whose only way in is that box.
 *
 * There is no one transliteration: the same catalogue spells `ш` as `sch` in
 * one search and `sh` in the next. So a Cyrillic letter matches ANY of its
 * plausible latin spellings, and the query becomes a pattern rather than a
 * string.
 */
describe("hasCyrillic", () => {
  it("is true only when the text carries a Cyrillic letter", () => {
    expect(hasCyrillic("Лаврово")).toBe(true);
    expect(hasCyrillic("2026-09 Лаврово")).toBe(true);
    expect(hasCyrillic("Lavrovo")).toBe(false);
    expect(hasCyrillic("2026-09-21")).toBe(false);
    expect(hasCyrillic("")).toBe(false);
  });
});

describe("transliteratedPattern", () => {
  const matches = (query: string, name: string) =>
    transliteratedPattern(query)?.test(name) ?? false;

  it("finds the names the real catalogue uses", () => {
    // Taken from `src/test/fixtures/catalogue.json`, which the Rust core
    // writes from a real listing.
    expect(matches("Лаврово", "2026 07 08 Lavrovo")).toBe(true);
    expect(matches("Сагра", "2026 07 14 Sagra")).toBe(true);
    expect(matches("Шувалово", "2026 09 20 Schuvalovo")).toBe(true);
  });

  it("accepts the competing spellings of one letter", () => {
    // `ш` is written `sh` as often as `sch`, and `ж` turns up as `zh` and `j`.
    expect(matches("Шувалово", "2026 01 01 Shuvalovo")).toBe(true);
    expect(matches("Жуково", "2026 01 01 Zhukovo")).toBe(true);
    expect(matches("Жуково", "2026 01 01 Jukovo")).toBe(true);
    expect(matches("Химки", "2026 01 01 Khimki")).toBe(true);
    expect(matches("Химки", "2026 01 01 Himki")).toBe(true);
    expect(matches("Царское", "2026 01 01 Tsarskoe")).toBe(true);
  });

  it("matches a Cyrillic name too, when the catalogue has one", () => {
    expect(matches("Лаврово", "2026 07 08 Лаврово")).toBe(true);
  });

  it("matches inside the name, not only at its start", () => {
    expect(matches("врово", "2026 07 08 Lavrovo")).toBe(true);
  });

  it("lets a space stand for the separators a slug uses", () => {
    expect(matches("Лаврово 2026", "Lavrovo-2026")).toBe(true);
    expect(matches("2026 09 Колпино", "2026_09_Kolpino")).toBe(true);
  });

  it("still rejects a name that is not the one asked for", () => {
    expect(matches("Лаврово", "2026 09 20 Schuvalovo")).toBe(false);
    expect(matches("Колпино", "2026 07 14 Sagra")).toBe(false);
  });

  it("ignores case on both sides", () => {
    expect(matches("лАВРОВО", "2026 07 08 LAVROVO")).toBe(true);
  });

  it("is null for a query with no Cyrillic, so plain matching stays plain", () => {
    expect(transliteratedPattern("lavrovo")).toBeNull();
    expect(transliteratedPattern("2026-09-21")).toBeNull();
  });

  it("does not let a query's punctuation act as a pattern", () => {
    // `.` and `*` are ordinary characters in a search box.
    expect(matches("Лаврово.", "2026 07 08 Lavrovox")).toBe(false);
    expect(matches("Лаврово*", "2026 07 08 Lavrovooo")).toBe(false);
  });
});
