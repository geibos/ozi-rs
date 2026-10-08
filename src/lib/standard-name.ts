/**
 * Track names by the detachment's cartographic standard.
 *
 * п. 14: latin letters, digits, `_` and `-` only. п. 15: `ГГГГММДД_Позывной`,
 * the date without separators. п. 17: the group's callsign in transliteration
 * — `Lisa`, `Veter`, `Kinolog`, `Bort`, `copter`/`kopter`/`bpla`, `Pegas`.
 * п. 18: a second navigator of the same group gets a number or a personal
 * callsign after `_` (`20200421_Lisa1_2`, `20200421_Lisa2_Mohnatiy`).
 *
 * Tracks arrive named however the phone or the crew named them — `Лиса 19
 * Мина`, `20261006-Veter1-Maura`, `File` — and the operator renames every one
 * by hand (owner's recordings, 2026-10-06 and 2026-10-08). The date is the
 * day the group went out: their first point, in the local day, not the day
 * the file was sent.
 */

const STANDARD_TRACK_NAME = /^\d{8}_[A-Za-z0-9][A-Za-z0-9_-]*$/;

/** Whether a track name follows п. 14–15. */
export function isStandardTrackName(name: string): boolean {
  return STANDARD_TRACK_NAME.test(name);
}

/**
 * Russian to latin the way the standard's own example spells it —
 * `Мохнатый` → `Mohnatiy`: `х` is `h`, `ы` is `i`, `й` is `y`.
 */
const LATIN: Record<string, string> = {
  а: "a",
  б: "b",
  в: "v",
  г: "g",
  д: "d",
  е: "e",
  ё: "yo",
  ж: "zh",
  з: "z",
  и: "i",
  й: "y",
  к: "k",
  л: "l",
  м: "m",
  н: "n",
  о: "o",
  п: "p",
  р: "r",
  с: "s",
  т: "t",
  у: "u",
  ф: "f",
  х: "h",
  ц: "ts",
  ч: "ch",
  ш: "sh",
  щ: "sch",
  ъ: "",
  ы: "i",
  ь: "",
  э: "e",
  ю: "yu",
  я: "ya",
};

export function transliterate(text: string): string {
  let out = "";
  for (const ch of text) {
    const lower = ch.toLowerCase();
    const latin = LATIN[lower];
    if (latin === undefined) {
      out += ch;
    } else if (ch !== lower && latin.length > 0) {
      out += latin[0].toUpperCase() + latin.slice(1);
    } else {
      out += latin;
    }
  }
  return out;
}

/** Group types and how the standard spells them (п. 17, 19). */
const GROUPS: Record<string, string> = {
  лиса: "Lisa",
  lisa: "Lisa",
  ветер: "Veter",
  veter: "Veter",
  кинолог: "Kinolog",
  kinolog: "Kinolog",
  борт: "Bort",
  bort: "Bort",
  бпла: "Bpla",
  bpla: "Bpla",
  коптер: "Kopter",
  kopter: "Kopter",
  copter: "Copter",
  пегас: "Pegas",
  pegas: "Pegas",
  автоном: "Avtonom",
  avtonom: "Avtonom",
};

/** Car models a crew writes after a Ветер's number; not callsigns. */
const CARS = new Set([
  "гранта",
  "granta",
  "нива",
  "niva",
  "веста",
  "vesta",
  "лада",
  "lada",
  "уаз",
  "uaz",
  "патриот",
  "patriot",
  "дастер",
  "duster",
  "логан",
  "logan",
  "солярис",
  "solaris",
  "рио",
  "rio",
  "поло",
  "polo",
  "приора",
  "priora",
  "калина",
  "kalina",
  "ларгус",
  "largus",
  "шнива",
  "буханка",
  "газель",
  "gazel",
  "тойота",
  "toyota",
  "хендай",
  "hyundai",
  "киа",
  "kia",
]);

/** Words a file name carries that are not part of anybody's callsign. */
const NOISE = new Set(["track", "трек", "gpx", "plt", "file", "файл"]);

const NAME_DATE = /^(\d{4})[-_.]?(\d{2})[-_.]?(\d{2})(?=$|[^0-9])/;

/** `ГГГГММДД` of an instant, in the given time zone (the machine's if none). */
function localDay(instant: string, timeZone?: string): string | null {
  const date = new Date(instant);
  if (Number.isNaN(date.getTime())) return null;
  const parts = new Intl.DateTimeFormat("en-CA", {
    timeZone,
    year: "numeric",
    month: "2-digit",
    day: "2-digit",
  }).formatToParts(date);
  const get = (type: string) => parts.find((p) => p.type === type)?.value;
  return `${get("year")}${get("month")}${get("day")}`;
}

function capitalise(word: string): string {
  return word.charAt(0).toUpperCase() + word.slice(1).toLowerCase();
}

/**
 * The callsign part of a name: the group word with its number, then the rest
 * transliterated, each word capitalised, joined by `_`.
 */
function callsign(rest: string): string | null {
  const tokens = rest
    .split(/[\s_\-.,]+/u)
    .filter((t) => t.length > 0 && !NOISE.has(t.toLowerCase()));
  const parts: string[] = [];
  for (let i = 0; i < tokens.length; i += 1) {
    const token = tokens[i];
    // `лиса15`, `Lisa15`, `ЛИСА15`: the group and its number written together.
    const joined = /^([\p{L}]+)(\d+)$/u.exec(token);
    const word = (joined ? joined[1] : token).toLowerCase();
    const group = GROUPS[word];
    if (group) {
      let number = joined ? joined[2] : "";
      if (!number && /^\d+$/.test(tokens[i + 1] ?? "")) {
        number = tokens[i + 1];
        i += 1;
      }
      parts.push(group + number);
    } else if (/^\d+$/.test(token)) {
      parts.push(token);
    } else if (CARS.has(token.toLowerCase())) {
      // A Ветер's car is not a callsign: the owner renamed `Ветер 2 Гранта`
      // to `Veter2` (2026-10-06), and kept `Veter1_Maura`, a driver's.
      continue;
    } else {
      const latin = transliterate(token).replace(/[^A-Za-z0-9]/g, "");
      if (latin) parts.push(capitalise(latin));
    }
  }
  return parts.length > 0 ? parts.join("_") : null;
}

/**
 * A name by the standard for a track called `name` whose first point was
 * recorded at `startTime`, or `null` when there is no date or no callsign to
 * build one from — the operator then names it, as before.
 *
 * The first point's day wins over a date written into the name: a file named
 * the day it was sent is still the track of the day the group went out.
 */
export function suggestTrackName(
  name: string,
  startTime: string | null,
  timeZone?: string,
  fallbackDate?: string,
): string | null {
  const trimmed = name.trim().replace(/\.(gpx|plt)$/i, "");
  const inName = NAME_DATE.exec(trimmed);
  const rest = inName ? trimmed.slice(inName[0].length) : trimmed;
  const date =
    (startTime ? localDay(startTime, timeZone) : null) ??
    (inName ? `${inName[1]}${inName[2]}${inName[3]}` : null) ??
    fallbackDate ??
    null;
  const sign = callsign(rest);
  if (!date || !sign) return null;
  return `${date}_${sign}`;
}

/**
 * Names made distinct the way п. 18 has it: a group with several navigators
 * numbers its tracks `_1`, `_2`… Two files of one group — the navigator's and
 * the phone's — suggest the same name, and «Сохранить в 10-Tracks» refuses
 * two tracks with one file. Every name of a clashing group is numbered, in
 * the order given; a name already taken by a track not being renamed is
 * numbered past it.
 */
export function distinctNames(
  names: readonly string[],
  taken: ReadonlySet<string> = new Set(),
): string[] {
  const counts = new Map<string, number>();
  for (const name of names) counts.set(name, (counts.get(name) ?? 0) + 1);
  const used = new Set(taken);
  const next = new Map<string, number>();
  return names.map((name) => {
    if ((counts.get(name) ?? 0) < 2 && !used.has(name)) {
      used.add(name);
      return name;
    }
    let n = next.get(name) ?? 1;
    while (used.has(`${name}_${n}`)) n += 1;
    next.set(name, n + 1);
    const numbered = `${name}_${n}`;
    used.add(numbered);
    return numbered;
  });
}

/** Whether a name holds a group's word — `Лиса`, `ветер5`, `Bort`… */
export function namesAGroup(name: string): boolean {
  return name
    .split(/[\s_\-.,\d]+/u)
    .some((word) => GROUPS[word.toLowerCase()] !== undefined);
}

/**
 * The standard's name from the first of `candidates` that names a group —
 * the track's own name, then its layer's, which is the file it came from — or
 * failing that from the first that gives anything. A phone calls its track
 * `заброс` and the crew calls the file `Лиса4` (owner's tracks, 2026-10-06).
 */
export function suggestFromNames(
  candidates: readonly string[],
  startTime: string | null,
  timeZone?: string,
): string | null {
  const grouped = candidates.find(namesAGroup);
  if (grouped !== undefined) {
    const named = suggestTrackName(grouped, startTime, timeZone);
    if (named) return named;
  }
  for (const candidate of candidates) {
    const named = suggestTrackName(candidate, startTime, timeZone);
    if (named) return named;
  }
  return null;
}
