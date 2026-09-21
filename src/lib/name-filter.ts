/**
 * The one name-matching rule the Library rails share.
 *
 * Field names are typed as date plus call sign ("20260709-ЛИСА15") and
 * waypoints as free text in Russian, so the match is a case-insensitive
 * substring rather than a prefix: an operator searches by call sign as often
 * as by date, in either alphabet. Both tabs use this so a query that finds a
 * track finds a waypoint of the same name.
 */
export function filterByName<T>(
  rows: T[],
  query: string,
  name: (row: T) => string,
): T[] {
  const needle = query.trim().toLocaleLowerCase();
  if (needle === "") return rows;
  return rows.filter((row) => name(row).toLocaleLowerCase().includes(needle));
}
