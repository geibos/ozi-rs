/**
 * The waypoint symbols the app offers, and the glyph each one shows.
 *
 * One table, because there are two places that must agree: the picker a crew
 * chooses from, and the marker on the map. The map used to show every waypoint
 * as the same yellow dot — the symbol was stored, listed and exported, and was
 * the one place it did not appear was the place a crew looks at. A search map
 * distinguishes the task point from what was found from where the danger is;
 * ten identical dots do not.
 *
 * The values are what the backend stores and what GPX carries, so they are not
 * translated. The labels name them for a person and are translated at the call
 * site.
 */
export interface WaypointSymbol {
  value: string;
  emoji: string;
  labelKey: string;
}

export const WAYPOINT_SYMBOLS: WaypointSymbol[] = [
  { value: "flag", emoji: "🏁", labelKey: "symbol.flag" },
  { value: "camp", emoji: "🏕️", labelKey: "symbol.camp" },
  { value: "danger", emoji: "⚠️", labelKey: "symbol.danger" },
  { value: "water", emoji: "💧", labelKey: "symbol.water" },
  { value: "shelter", emoji: "🏠", labelKey: "symbol.shelter" },
  { value: "meeting-point", emoji: "👥", labelKey: "symbol.meetingPoint" },
  { value: "start", emoji: "🟢", labelKey: "symbol.start" },
  { value: "finish", emoji: "🔴", labelKey: "symbol.finish" },
  { value: "viewpoint", emoji: "👁️", labelKey: "symbol.viewpoint" },
  { value: "parking", emoji: "🅿️", labelKey: "symbol.parking" },
];

/** The default glyph: a waypoint with no symbol, and any symbol we do not know. */
export const DEFAULT_WAYPOINT_GLYPH = "📍";

/**
 * The glyph for a stored symbol value.
 *
 * A symbol this build does not know — from a GPX another tool wrote, or a
 * newer version of this one — falls back to the default pin rather than
 * disappearing: the waypoint is still there and must still be findable.
 */
export function waypointGlyph(symbol: string | null | undefined): string {
  if (!symbol) return DEFAULT_WAYPOINT_GLYPH;
  return (
    WAYPOINT_SYMBOLS.find((s) => s.value === symbol)?.emoji ??
    DEFAULT_WAYPOINT_GLYPH
  );
}

/** The default a marker takes when a waypoint carries no colour of its own. */
export const DEFAULT_WAYPOINT_HEX = "#e5c890";

/**
 * A waypoint's colour as CSS, or the default.
 *
 * Three surfaces draw a waypoint — the map marker, the row in the list and the
 * inspector — and a crew reads the list to find what they are looking at on
 * the map. They have to agree, so the conversion lives here rather than three
 * times.
 */
export function waypointColorCss(
  color: [number, number, number, number] | null | undefined,
): string {
  if (!color) return DEFAULT_WAYPOINT_HEX;
  const [r, g, b, a] = color;
  return `rgba(${r}, ${g}, ${b}, ${a / 255})`;
}

/** The same colour as a hex value, which is what an `<input type="color">` takes. */
export function waypointColorHex(
  color: [number, number, number, number] | null | undefined,
): string {
  if (!color) return DEFAULT_WAYPOINT_HEX;
  return (
    "#" +
    color
      .slice(0, 3)
      .map((c) => c.toString(16).padStart(2, "0"))
      .join("")
  );
}
