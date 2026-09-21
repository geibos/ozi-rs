/**
 * When the cached catalogue was written.
 *
 * The cache has carried an ISO timestamp since it was introduced and nothing
 * ever showed it. Offline — the state the cache exists for — a crew reads a
 * list of searches without knowing whether it is from this morning or from
 * three weeks ago, and a search created yesterday reads as a search that does
 * not exist.
 *
 * Absolute rather than relative: "3 часа назад" needs Russian plural rules to
 * be written correctly, and a clock time is what a crew compares against
 * "the coordinator said it went up at two".
 */
export function formatCatalogueAge(
  writtenAt: string | null,
  locale: string,
): string | null {
  if (!writtenAt) return null;
  const when = new Date(writtenAt);
  if (Number.isNaN(when.getTime())) return null;
  return new Intl.DateTimeFormat(locale, {
    day: "2-digit",
    month: "2-digit",
    hour: "2-digit",
    minute: "2-digit",
  }).format(when);
}
