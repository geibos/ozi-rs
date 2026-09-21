/**
 * Filtering for the bundle loader's project list.
 *
 * Extracted from the component so the rules are testable: the list is the
 * only way into a bundle and it carries about thirteen thousand rows.
 */
export interface ProjectRow {
  slug: string;
  name: string;
  cached?: boolean;
}

/**
 * Narrow the catalogue to what the operator asked for.
 *
 * The query matches the name or the slug — names are latin transliterations
 * of the slug, so a query that looks like a date has to reach both — and
 * `onlyCached` keeps the bundles that are already on disk, which is the only
 * list that means anything without a signal.
 */
export function filterProjects<T extends ProjectRow>(
  projects: T[],
  query: string,
  onlyCached: boolean,
): T[] {
  const needle = query.trim().toLocaleLowerCase();
  return projects.filter((project) => {
    if (onlyCached && project.cached !== true) return false;
    if (needle === "") return true;
    return (
      project.name.toLocaleLowerCase().includes(needle) ||
      project.slug.toLocaleLowerCase().includes(needle)
    );
  });
}
