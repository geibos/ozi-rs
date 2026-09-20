import { describe, expect, it } from "vitest";
import { mapsForLibrary } from "../lib/maps-list";
import type { ActiveMapDto, LizaMapPackageDto } from "../lib/bindings";

const pkg = (name: string, downloaded = true): LizaMapPackageDto => ({
  name,
  base_zoom: 17,
  downloaded,
});

const activeMap = (packageName: string): ActiveMapDto => ({
  kind: "ozi",
  project_name: "Local OZI",
  package_name: packageName,
  local_path: `/tmp/${packageName}`,
  center_lat: 55.0,
  center_lon: 37.0,
  base_zoom: 17,
});

/**
 * The Maps tab used to list `currentProject.maps` alone, so a locally opened
 * OZI map — which belongs to no LizaAlert project — left the tab claiming
 * "No maps in this project" while that very map was rendered on the canvas.
 */
describe("maps listed in the Library Maps tab", () => {
  it("lists the active map when no catalogue project is loaded", () => {
    const list = mapsForLibrary([], activeMap("Lavrovo_Satell_z17"));
    expect(list.map((m) => m.name)).toEqual(["Lavrovo_Satell_z17"]);
    expect(list[0].downloaded).toBe(true);
  });

  it("does not duplicate the active map when the project already lists it", () => {
    const list = mapsForLibrary(
      [pkg("Topo_z16"), pkg("Lavrovo_Satell_z17")],
      activeMap("Lavrovo_Satell_z17"),
    );
    expect(list.map((m) => m.name)).toEqual(["Topo_z16", "Lavrovo_Satell_z17"]);
  });

  it("puts an off-catalogue active map first so it is visible without scrolling", () => {
    const list = mapsForLibrary([pkg("Topo_z16")], activeMap("Local_OZF"));
    expect(list.map((m) => m.name)).toEqual(["Local_OZF", "Topo_z16"]);
  });

  it("returns the project maps unchanged when nothing is active", () => {
    const list = mapsForLibrary([pkg("Topo_z16"), pkg("Sat_z17", false)], null);
    expect(list.map((m) => m.name)).toEqual(["Topo_z16", "Sat_z17"]);
    expect(list[1].downloaded).toBe(false);
  });

  it("is empty only when there is neither a project map nor an active map", () => {
    expect(mapsForLibrary([], null)).toEqual([]);
  });
});
