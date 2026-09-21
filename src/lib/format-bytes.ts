import type { Locale } from "./i18n";

/**
 * Human byte sizes, in the language the app is running in.
 *
 * Three copies of this lived in the loader, the download popup and the
 * cold-start status bar, all printing English units into a Russian window and
 * disagreeing about MB vs MiB. The site states binary units, so these do too.
 */
const UNITS: Record<Locale, [string, string, string, string]> = {
  ru: ["Б", "КиБ", "МиБ", "ГиБ"],
  en: ["B", "KiB", "MiB", "GiB"],
};

export function formatBytes(bytes: number, locale: Locale): string {
  const units = UNITS[locale] ?? UNITS.en;
  if (!Number.isFinite(bytes) || bytes < 0) return "—";
  if (bytes >= 1024 ** 3)
    return `${(bytes / 1024 ** 3).toFixed(1)} ${units[3]}`;
  if (bytes >= 1024 ** 2)
    return `${(bytes / 1024 ** 2).toFixed(1)} ${units[2]}`;
  if (bytes >= 1024) return `${Math.round(bytes / 1024)} ${units[1]}`;
  return `${Math.round(bytes)} ${units[0]}`;
}

/** The same, for a size that may be unknown. */
export function formatOptionalBytes(
  bytes: number | null | undefined,
  locale: Locale,
): string | null {
  if (bytes === null || bytes === undefined) return null;
  return formatBytes(bytes, locale);
}
