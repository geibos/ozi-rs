import { transliteratedPattern } from "./translit";
import type { LizaProjectSummaryDto } from "./types";

/**
 * Which catalogue projects the command palette should render.
 *
 * The catalogue is about thirteen thousand projects since the listing started
 * paginating, and `cmdk` renders and re-filters every item it is handed on
 * each keystroke. So the filtering happens here and the palette gets a
 * screenful: the head of the list when nothing is typed, the matches by name
 * or slug when something is.
 *
 * A Russian query goes through `transliteratedPattern`, for the same reason
 * the loader's filter does: the catalogue is spelled in latin and the crew is
 * not. The palette is the second way into the catalogue, so it cannot answer
 * differently from the first.
 */
export function paletteProjects<T extends { name: string; slug: string }>(
  all: T[],
  query: string,
  limit: number,
): T[] {
  const needle = query.trim().toLocaleLowerCase();
  if (needle === "") return all.slice(0, limit);
  const pattern = transliteratedPattern(query);
  const matches: T[] = [];
  for (const project of all) {
    const hit = pattern
      ? pattern.test(project.name) || pattern.test(project.slug)
      : project.name.toLocaleLowerCase().includes(needle) ||
        project.slug.toLocaleLowerCase().includes(needle);
    if (hit) {
      matches.push(project);
      if (matches.length === limit) break;
    }
  }
  return matches;
}

export type PaletteProject = LizaProjectSummaryDto;
