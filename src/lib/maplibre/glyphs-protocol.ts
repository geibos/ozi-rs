import { addProtocol } from "maplibre-gl";

/**
 * The map's text, from glyphs shipped with the application.
 *
 * `static/glyphs/noto-sans-bold/` holds the SDF ranges of Noto Sans Bold as
 * built by `openmaptiles/fonts` v2.0, under the SIL Open Font License 1.1
 * (`static/glyphs/OFL.txt`). Only the ranges a callsign, a date and a
 * description are written in are shipped — about half a megabyte rather than
 * the fifty the full set takes with its CJK fallback.
 *
 * Every range the map asks for gets an answer, and a range that is not
 * shipped is answered with an empty set. That is the whole point of a
 * protocol rather than a plain URL: a glyph request that fails holds up the
 * tile its label belongs to, the tile carries the track lines, and in July a
 * remote glyphs URL took every line off the map whenever the laptop was
 * offline. A character outside the set is skipped; nothing else is.
 */

/** The font every map label is drawn in. */
export const LABEL_FONT = "Noto Sans Bold";

/** The style's `glyphs` URL. */
export const GLYPHS_URL = "glyphs://{fontstack}/{range}.pbf";

/** The ranges under `static/glyphs/noto-sans-bold/`. */
export const SHIPPED_GLYPH_RANGES = [
  "0-255", // Basic Latin, Latin-1: digits, `_`, «»
  "256-511", // Latin Extended-A and -B
  "768-1023", // combining marks, Greek
  "1024-1279", // Cyrillic
  "8192-8447", // general punctuation: – — “ ” …
  "8448-8703", // letterlike symbols: №
] as const;

const SHIPPED = new Set<string>(SHIPPED_GLYPH_RANGES);
const FONT_DIR = "noto-sans-bold";

/**
 * Where a glyph request's bytes live, or `null` when that range is not shipped.
 *
 * One font ships, so a layer that names another is drawn in this one rather
 * than not at all.
 */
export function glyphAssetPath(url: string): string | null {
  const range = url.match(/\/(\d+-\d+)\.pbf$/)?.[1];
  if (range === undefined || !SHIPPED.has(range)) return null;
  return `/glyphs/${FONT_DIR}/${range}.pbf`;
}

/** The bytes for one glyph request; an empty set rather than a failure. */
export async function loadGlyphRange(
  url: string,
  fetchImpl: (input: string) => Promise<Response> = fetch,
): Promise<ArrayBuffer> {
  const path = glyphAssetPath(url);
  if (path === null) return new ArrayBuffer(0);
  try {
    const response = await fetchImpl(path);
    if (!response.ok) throw new Error(`HTTP ${response.status}`);
    return await response.arrayBuffer();
  } catch (error) {
    // A shipped range that cannot be read is a packaging defect, not
    // something the operator can act on — and the names it costs are visible
    // on the map. What must not happen is the tile failing with it.
    console.warn(`glyph range ${path} unavailable:`, error);
    return new ArrayBuffer(0);
  }
}

export function registerGlyphsProtocol(): void {
  addProtocol("glyphs", async (params) => ({
    data: await loadGlyphRange(params.url),
  }));
}
