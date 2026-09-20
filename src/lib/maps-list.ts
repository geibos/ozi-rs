import type { ActiveMapDto, LizaMapPackageDto } from "./bindings";

/**
 * The maps the Library Maps tab shows.
 *
 * `currentProject.maps` covers only maps that came from a LizaAlert project.
 * A locally opened OZI map, or a map restored from the previous session after
 * the catalogue project was cleared, is active on the canvas while belonging
 * to no project — listing project maps alone left the tab showing
 * "No maps in this project" over a rendered map.
 *
 * An active map that the project does not list is prepended, so it is visible
 * without scrolling and the empty state means what it says: no maps at all.
 */
export function mapsForLibrary(
  projectMaps: LizaMapPackageDto[],
  active: ActiveMapDto | null | undefined,
): LizaMapPackageDto[] {
  if (!active) return projectMaps;
  if (projectMaps.some((m) => m.name === active.package_name)) {
    return projectMaps;
  }
  return [
    {
      name: active.package_name,
      base_zoom: active.base_zoom,
      // It is open and rendering, so its tiles are on disk by definition.
      downloaded: true,
    },
    ...projectMaps,
  ];
}
