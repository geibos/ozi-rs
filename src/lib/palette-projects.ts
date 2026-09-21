import type { LizaProjectSummaryDto } from "./types";

/**
 * Which catalogue projects the command palette should render.
 *
 * The catalogue is about thirteen thousand projects since the listing started
 * paginating, and `cmdk` renders and re-filters every item it is handed on
 * each keystroke. So the filtering happens here and the palette gets a
 * screenful: the head of the list when nothing is typed, the matches by name
 * or slug when something is.
 */
export function paletteProjects<T extends { name: string; slug: string }>(
  all: T[],
  query: string,
  limit: number,
): T[] {
  const needle = query.trim().toLocaleLowerCase();
  if (needle === "") return all.slice(0, limit);
  const matches: T[] = [];
  for (const project of all) {
    if (
      project.name.toLocaleLowerCase().includes(needle) ||
      project.slug.toLocaleLowerCase().includes(needle)
    ) {
      matches.push(project);
      if (matches.length === limit) break;
    }
  }
  return matches;
}

export type PaletteProject = LizaProjectSummaryDto;
