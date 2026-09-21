import type { Locale } from "./i18n";

/**
 * How a layer's name is shown.
 *
 * A project always carries one track layer and one waypoint layer, created by
 * the core and named "Tracks" and "Waypoints" — English, in the selector of a
 * Russian window. The names are part of the saved file and of the `layers`
 * invariant, so they are translated for display rather than renamed: a project
 * written today still opens in an older build, and a build in English still
 * shows the name it stored.
 *
 * A layer the operator named "Tracks" themselves is translated too. That is
 * the price of not touching stored data, and it costs nothing: the name means
 * the same thing.
 */
const DEFAULT_NAMES: Record<string, Record<Locale, string>> = {
  Tracks: { ru: "Треки", en: "Tracks" },
  Waypoints: { ru: "Точки", en: "Waypoints" },
};

export function layerDisplayName(name: string, locale: Locale): string {
  return DEFAULT_NAMES[name]?.[locale] ?? name;
}
