/**
 * Reading a pair of coordinates the way a coordinator writes one.
 *
 * A place arrives in a message, and it arrives in whatever form the person
 * sending it had to hand: `59.9311, 30.3609` off a phone, `59°55'52"N
 * 30°21'39"E` off a screenshot, `N 59 55.87 E 30 21.65` out of OziExplorer or
 * a navigator, `59 55 52 N` off a paper sheet. They are the same place, and a
 * field that takes only one of them is a field that gets typed into wrong at
 * four in the morning.
 *
 * So this takes all of them. What it refuses is a place that is not on the
 * Earth and a string with too few or too many numbers in it — where guessing
 * would put a crew somewhere nobody meant.
 */

export interface LatLonPair {
  lat: number;
  lon: number;
}

/** Everything that can separate or decorate a coordinate. */
const HEMISPHERE = /[NSEWСЮВЗ]/i;

/** North/south in either alphabet; Russian «С» and «Ю», English `N` and `S`. */
const NORTHING = /^[NSСЮ]$/i;

function isNegativeHemisphere(letter: string): boolean {
  return /^[SWЮЗ]$/i.test(letter);
}

/**
 * Degrees, minutes and seconds collapsed into degrees.
 *
 * One number is degrees; two are degrees and decimal minutes, which is what a
 * navigator shows; three are degrees, minutes and seconds.
 */
function toDegrees(parts: number[]): number | null {
  if (parts.length === 0 || parts.length > 3) return null;
  if (parts.some((value) => !Number.isFinite(value))) return null;
  // Only the last part may carry a fraction: `59 55.87` is degrees and
  // minutes, but `59.5 55` is somebody's two separate numbers.
  if (parts.slice(0, -1).some((value) => !Number.isInteger(value))) return null;
  const [degrees, minutes = 0, seconds = 0] = parts;
  if (minutes < 0 || minutes >= 60 || seconds < 0 || seconds >= 60) return null;
  const magnitude = Math.abs(degrees) + minutes / 60 + seconds / 3600;
  return degrees < 0 ? -magnitude : magnitude;
}

interface Component {
  degrees: number;
  hemisphere: string | null;
}

/**
 * Split the text into its two components, keeping each one's letter with it.
 *
 * The comma is not a reliable separator: `59,9311 30,3609` is a decimal comma
 * in half of Europe, and `59.9311, 30.3609` is a separator. What is reliable
 * is that a coordinate pair holds two groups of numbers, so the numbers are
 * read in order and cut in half.
 */
function components(text: string): Component[] | null {
  // A decimal comma, but only between digits and only when no other comma
  // could be doing the separating.
  const normalised =
    /\d,\d/.test(text) && (text.match(/,/g) ?? []).length === 1
      ? text.replace(/(\d),(\d)/, "$1.$2")
      : text;

  const tokens = normalised
    .replace(/[°º'′"″]/g, " ")
    .replace(/[,;]/g, " ")
    .trim()
    .split(/\s+/)
    .filter((token) => token.length > 0);
  if (tokens.length === 0) return null;

  // Without a single letter in it there is no boundary between the two
  // coordinates, so the numbers are split down the middle: two numbers are a
  // degree each, four are degrees and minutes, six are degrees, minutes and
  // seconds. An odd count is a string with something else in it.
  if (!tokens.some((token) => HEMISPHERE.test(token))) {
    const numbers = tokens.map(Number);
    if (numbers.some((value) => !Number.isFinite(value))) return null;
    if (numbers.length % 2 !== 0 || numbers.length > 6) return null;
    const half = numbers.length / 2;
    const first = toDegrees(numbers.slice(0, half));
    const second = toDegrees(numbers.slice(half));
    if (first === null || second === null) return null;
    return [
      { degrees: first, hemisphere: null },
      { degrees: second, hemisphere: null },
    ];
  }

  const groups: Component[] = [];
  let numbers: number[] = [];
  let pendingHemisphere: string | null = null;

  const flush = (hemisphere: string | null) => {
    if (numbers.length === 0) return;
    const degrees = toDegrees(numbers);
    numbers = [];
    if (degrees === null) {
      groups.push({ degrees: Number.NaN, hemisphere });
      return;
    }
    groups.push({ degrees, hemisphere });
  };

  for (const token of tokens) {
    // `N59.93` and `59.93N` both happen, as do a bare `N` and a bare number.
    const leading = token.match(/^([NSEWСЮВЗ])(.*)$/i);
    const trailing = token.match(/^(.*?)([NSEWСЮВЗ])$/i);

    if (leading && leading[2] === "") {
      // A letter on its own either closes the numbers before it or opens the
      // ones after, and which it is depends on whether the group it would
      // close already has a letter:
      //
      //   `59 55 52 N 30 21 39 E` — the `N` closes the northing.
      //   `N 59 55.87 E 30 21.65` — the `E` ends the northing, which keeps
      //                             the `N` that opened it, and opens the
      //                             easting.
      if (numbers.length > 0 && pendingHemisphere === null) {
        flush(leading[1]);
      } else {
        if (numbers.length > 0) flush(pendingHemisphere);
        pendingHemisphere = leading[1];
      }
      continue;
    }
    if (leading) {
      if (numbers.length > 0) flush(pendingHemisphere);
      pendingHemisphere = leading[1];
      const value = Number(leading[2]);
      if (!Number.isFinite(value)) return null;
      numbers.push(value);
      continue;
    }
    if (trailing && trailing[1] !== "") {
      const value = Number(trailing[1]);
      if (!Number.isFinite(value)) return null;
      numbers.push(value);
      flush(pendingHemisphere ?? trailing[2]);
      pendingHemisphere = null;
      continue;
    }

    const value = Number(token);
    if (!Number.isFinite(value)) return null;
    numbers.push(value);
  }
  flush(pendingHemisphere);

  return groups;
}

/**
 * Read a pair of coordinates, or answer `null` when the text does not hold
 * exactly one place.
 *
 * Without letters the order is latitude then longitude, which is the order
 * everything from a phone to a GPX file uses. With letters the order is
 * whatever the person wrote: `30.3609E 59.9311N` is the same place.
 */
export function parseLatLon(text: string): LatLonPair | null {
  const groups = components(text);
  if (groups === null || groups.length !== 2) return null;
  if (groups.some((group) => !Number.isFinite(group.degrees))) return null;

  const signed = groups.map((group) => ({
    ...group,
    degrees:
      group.hemisphere && isNegativeHemisphere(group.hemisphere)
        ? -Math.abs(group.degrees)
        : group.degrees,
  }));

  const [first, second] = signed;
  const firstIsLat = first.hemisphere
    ? NORTHING.test(first.hemisphere)
    : !(second.hemisphere && NORTHING.test(second.hemisphere));

  const lat = firstIsLat ? first.degrees : second.degrees;
  const lon = firstIsLat ? second.degrees : first.degrees;

  // Both letters naming the same axis is a typo, not a place.
  if (first.hemisphere && second.hemisphere) {
    const bothNorthing =
      NORTHING.test(first.hemisphere) === NORTHING.test(second.hemisphere);
    if (bothNorthing) return null;
  }

  if (!Number.isFinite(lat) || !Number.isFinite(lon)) return null;
  if (Math.abs(lat) > 90 || Math.abs(lon) > 180) return null;
  return { lat, lon };
}

/** A place written back out the way this reads it best: decimal degrees. */
export function formatLatLon({ lat, lon }: LatLonPair): string {
  return `${lat.toFixed(6)}, ${lon.toFixed(6)}`;
}
