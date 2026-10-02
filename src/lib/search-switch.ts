/**
 * Whether opening a map moves the operator to another search.
 *
 * A project is one search (owner, 2026-10-01). The catalogue is where a crew
 * moves to the next one, so opening a map of another search over a project
 * that holds the previous search's work starts a new project first — see
 * `makeWayForSearch` in `actions/project.ts`.
 */
import type { AppStateDto } from "$lib/bindings";

/**
 * A map of `chosen` is about to open over a map of `ground`.
 *
 * No map on screen is not another search: the first map of the day belongs
 * to whatever the operator has started on.
 */
export function opensAnotherSearch(
  ground: string | null | undefined,
  chosen: string | null | undefined,
): boolean {
  return Boolean(ground) && Boolean(chosen) && ground !== chosen;
}

/**
 * The project has something that belongs to the search on screen.
 *
 * Tracks are in the state; marks are not, but adding one makes the project
 * dirty, and a project that was saved to a file is somebody's work whatever
 * it holds.
 */
export function holdsWork(
  state: Pick<AppStateDto, "tracks" | "project_dirty" | "project_path"> | null,
): boolean {
  if (state === null) return false;
  return (
    state.tracks.length > 0 ||
    state.project_dirty ||
    state.project_path !== null
  );
}
