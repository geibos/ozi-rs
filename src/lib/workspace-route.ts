/**
 * Whether the workspace is worth opening — the one rule both routes go by.
 *
 * A map with no project is a crew looking at the ground before the work
 * arrives. A project with no map is a `.ozp` from another headquarters, for
 * ground this machine may never have downloaded: tracks on the OpenStreetMap
 * backdrop are a workspace too. Neither is not.
 *
 * And the operator may ask for it with neither: a search with no map ordered
 * is processed on OpenStreetMap from the first GPX (the owner's forest search,
 * 2026-10-08) — «Работать без карты» on the launcher, `withoutMap`.
 *
 * Until 2026-10-02 only the workspace route knew the second half. The
 * launcher's start-up redirect forwarded on a map alone, so a project restored
 * without one opened in the catalogue with the work hidden behind it.
 */
import type { AppStateDto } from "$lib/bindings";

export function worthOpeningWorkspace(
  state: Pick<AppStateDto, "active_map" | "project_path"> | null,
  withoutMap = false,
): boolean {
  return (
    state !== null &&
    (withoutMap || state.active_map !== null || state.project_path !== null)
  );
}
