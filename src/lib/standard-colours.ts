/**
 * The cartographic standard's colours and the waypoint name rule.
 *
 * Colours are OziExplorer's named colours — the ones its picker offers and an
 * operator says aloud ("передний чёрный, задний лаймовый", owner, 2026-10-08):
 * green is Ozi's `Green` (0,128,0), distinct from `Lime` (0,255,0); "цвет
 * морской волны" is taken as `Teal` (0,128,128), which nobody has confirmed
 * against the original's Russian picker.
 */

export type Rgba = [number, number, number, number];

export interface StandardColour {
  key: string;
  rgba: Rgba;
}

/**
 * Tracks (п. 20–23): foot groups red, blue, sometimes green; Ветра pink;
 * Борты and БПЛА yellow, light blue or white at width 1. Black is for tasks
 * and is not offered.
 */
export const TRACK_COLOURS: StandardColour[] = [
  { key: "red", rgba: [255, 0, 0, 255] },
  { key: "blue", rgba: [0, 0, 255, 255] },
  { key: "green", rgba: [0, 128, 0, 255] },
  { key: "pink", rgba: [255, 0, 255, 255] },
  { key: "yellow", rgba: [255, 255, 0, 255] },
  { key: "aqua", rgba: [0, 255, 255, 255] },
  { key: "white", rgba: [255, 255, 255, 255] },
];

/**
 * Marks (п. 28): what matters to the search, confirmed finds among it, red;
 * unconfirmed finds sea-green; groups' positions and features of the ground
 * green. Lime is the owner's drop-off point (заброс), not the standard's.
 */
export const WAYPOINT_COLOURS: StandardColour[] = [
  { key: "important", rgba: [255, 0, 0, 255] },
  { key: "unconfirmed", rgba: [0, 128, 128, 255] },
  { key: "ground", rgba: [0, 128, 0, 255] },
  { key: "dropOff", rgba: [0, 255, 0, 255] },
];

/** `#rrggbb` for a swatch. */
export function hex([r, g, b]: Rgba | number[]): string {
  return `#${[r, g, b].map((v) => v.toString(16).padStart(2, "0")).join("")}`;
}

/** Black is reserved for tasks (п. 23); a search track must not use it. */
export function isReservedTrackColour([r, g, b]: Rgba | number[]): boolean {
  return r < 40 && g < 40 && b < 40;
}

/**
 * What is wrong with a waypoint name by п. 26, if anything: the Russian
 * capital С, which must be the latin C, and characters outside letters,
 * digits, space, `_` and `-`.
 */
export function waypointNameProblems(name: string): {
  russianEs: boolean;
  badCharacters: string[];
} {
  const bad = new Set<string>();
  for (const ch of name) {
    if (!/[\p{L}\p{N} _-]/u.test(ch)) bad.add(ch);
  }
  return { russianEs: name.includes("С"), badCharacters: [...bad] };
}

/** The name with every Russian capital С made latin (п. 26). */
export function latinEs(name: string): string {
  return name.replaceAll("С", "C");
}
