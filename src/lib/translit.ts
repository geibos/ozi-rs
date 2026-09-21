/**
 * Matching a Russian query against a latin catalogue.
 *
 * The LizaAlert catalogue names its searches in transliteration —
 * `2026-09-20_Schuvalovo`, `2026-07-14_Sagra` — while the crew typing into the
 * filter box writes Russian. A plain substring match therefore found nothing
 * for the query a crew actually types, in a list of about thirteen thousand
 * rows whose only way in is that box.
 *
 * There is no single transliteration to convert to: the catalogue spells `ш`
 * as `sch` in one search and `sh` in the next, `ж` as `zh` and as `j`. So a
 * Cyrillic letter is turned into an alternation of its plausible latin
 * spellings and the query becomes a pattern. The letter itself is in the
 * alternation as well, so a catalogue entry that is written in Cyrillic still
 * matches.
 */

/**
 * Latin spellings seen for each Russian letter, longest first so the
 * alternation prefers `sch` over `s` where both could start.
 *
 * An empty string is a legitimate spelling for the signs, which most schemes
 * drop entirely.
 */
const LATIN_SPELLINGS: Record<string, string[]> = {
  а: ["a"],
  б: ["b"],
  в: ["v", "w"],
  г: ["gh", "g", "h"],
  д: ["d"],
  е: ["ye", "je", "ie", "e"],
  ё: ["yo", "jo", "io", "e"],
  ж: ["zh", "j", "g"],
  з: ["z"],
  и: ["i", "y"],
  й: ["y", "j", "i", ""],
  к: ["k", "c", "q"],
  л: ["l"],
  м: ["m"],
  н: ["n"],
  о: ["o"],
  п: ["p"],
  р: ["r"],
  с: ["ss", "s", "c"],
  т: ["t"],
  у: ["ou", "u", "oo"],
  ф: ["ph", "f"],
  х: ["kh", "ch", "h", "x"],
  ц: ["ts", "tz", "cz", "c", "z"],
  ч: ["tch", "ch", "c"],
  ш: ["sch", "sh", "ch", "s"],
  щ: ["shch", "sch", "sh"],
  ъ: ["", "'"],
  ы: ["y", "i"],
  ь: ["", "'"],
  э: ["e"],
  ю: ["yu", "ju", "iu", "u"],
  я: ["ya", "ja", "ia", "a"],
};

const CYRILLIC = /[Ѐ-ӿ]/;

/** Whether the text carries at least one Cyrillic letter. */
export function hasCyrillic(text: string): boolean {
  return CYRILLIC.test(text);
}

function escapeLiteral(char: string): string {
  return char.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
}

/**
 * A pattern matching every plausible transliteration of `query`, or `null`
 * when the query has no Cyrillic in it — then the caller's plain substring
 * match is both correct and cheaper, which matters at thirteen thousand rows
 * per keystroke.
 */
export function transliteratedPattern(query: string): RegExp | null {
  const trimmed = query.trim();
  if (trimmed === "" || !hasCyrillic(trimmed)) return null;

  let source = "";
  for (const char of trimmed.toLocaleLowerCase()) {
    const spellings = LATIN_SPELLINGS[char];
    if (spellings) {
      // The letter itself last: a Cyrillic catalogue entry matches too.
      const branches = [...spellings, char].map(escapeLiteral);
      source += `(?:${branches.join("|")})`;
    } else if (/\s/.test(char)) {
      // A space in the box stands for whatever the slug puts between words.
      source += "[\\s_-]+";
    } else {
      source += escapeLiteral(char);
    }
  }
  return new RegExp(source, "i");
}
