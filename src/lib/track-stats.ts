// Track statistics formatting for the Tracks panel and the Inspector.
//
// Distance is one decimal place in kilometres, duration is the span between
// the first and last point timestamp — `<d>d <h>h` from a day upwards,
// `<h>h <m>m` below that, `<m>m` under an hour — and the point count carries a
// unit. The bullet separator joins only the parts that are present, so the
// duration is dropped when there are no timestamps.
//
// Units are localized: a Russian crew reading "12.3 km · 1h 24m · 156 pts" in
// an otherwise Russian window is reading someone else's app.
import type { Locale } from "./i18n";

interface Units {
  km: string;
  d: string;
  h: string;
  m: string;
  pts: string;
}

const UNITS: Record<Locale, Units> = {
  ru: { km: "км", d: "д", h: "ч", m: "мин", pts: "тчк" },
  en: { km: "km", d: "d", h: "h", m: "m", pts: "pts" },
};

function units(locale: Locale): Units {
  return UNITS[locale] ?? UNITS.en;
}

export function formatDistanceKm(distanceKm: number, locale: Locale): string {
  return `${distanceKm.toFixed(1)} ${units(locale).km}`;
}

export function formatDurationSeconds(
  durationSeconds: number,
  locale: Locale,
): string {
  const u = units(locale);
  const totalSeconds = Math.max(0, Math.trunc(durationSeconds));
  const hours = Math.floor(totalSeconds / 3600);
  const minutes = Math.floor((totalSeconds % 3600) / 60);
  // A span of a day or more comes from a multi-day recording, not from a
  // multi-day walk. Printed as hours it reads as an error ("629h 22m" beside
  // "4.9 km"); printed as days it reads as the date range it is.
  if (hours >= 24) {
    return `${Math.floor(hours / 24)}${u.d} ${hours % 24}${u.h}`;
  }
  if (hours > 0) {
    return `${hours}${u.h} ${minutes}${u.m}`;
  }
  return `${minutes}${u.m}`;
}

export function formatPointCount(pointCount: number, locale: Locale): string {
  return `${pointCount} ${units(locale).pts}`;
}

/**
 * The compact track-stats label rendered next to the track name.
 *
 * Examples:
 *   `12.3 км · 1ч 24мин · 156 тчк` when timestamps are present
 *   `12.3 км · 156 тчк`            when `durationSeconds` is null/undefined
 */
export function formatTrackStats(
  distanceKm: number,
  durationSeconds: number | null | undefined,
  pointCount: number,
  locale: Locale,
): string {
  const segments = [formatDistanceKm(distanceKm, locale)];
  if (durationSeconds !== null && durationSeconds !== undefined) {
    segments.push(formatDurationSeconds(durationSeconds, locale));
  }
  segments.push(formatPointCount(pointCount, locale));
  return segments.join(" · ");
}

/**
 * A point's timestamp, as a person reads it.
 *
 * The inspector printed the raw RFC3339 string ("2026-07-08T09:00:00+00:00"),
 * which is a machine's answer to "when did this start?".
 */
export function formatTimestamp(
  timestamp: string | null | undefined,
  locale: Locale,
): string | null {
  if (!timestamp) return null;
  const date = new Date(timestamp);
  if (Number.isNaN(date.getTime())) return timestamp;
  return new Intl.DateTimeFormat(locale === "ru" ? "ru-RU" : "en-GB", {
    day: "2-digit",
    month: "2-digit",
    year: "numeric",
    hour: "2-digit",
    minute: "2-digit",
  }).format(date);
}
