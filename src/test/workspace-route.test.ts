import { describe, expect, it } from "vitest";
import { worthOpeningWorkspace } from "../lib/workspace-route";

const MAP = {
  kind: "sqlite",
  project_name: "2026 07 08 Lavrovo",
  package_name: "Topo.sqlitedb",
  local_path: "/maps/Topo.sqlitedb",
  center_lat: 59.95,
  center_lon: 31.6,
  base_zoom: 14,
};

describe("worthOpeningWorkspace", () => {
  it("opens for a map with no project — the ground before the work", () => {
    expect(worthOpeningWorkspace({ active_map: MAP, project_path: null })).toBe(
      true,
    );
  });

  it("opens for a project with no map — a colleague's .ozp", () => {
    expect(
      worthOpeningWorkspace({
        active_map: null,
        project_path: "/crew/поиск.ozp",
      }),
    ).toBe(true);
  });

  it("stays on the loader with neither", () => {
    expect(
      worthOpeningWorkspace({ active_map: null, project_path: null }),
    ).toBe(false);
  });

  it("opens with neither when the operator asked to work without a map", () => {
    expect(
      worthOpeningWorkspace({ active_map: null, project_path: null }, true),
    ).toBe(true);
  });

  it("decides nothing before the state has arrived", () => {
    expect(worthOpeningWorkspace(null)).toBe(false);
  });
});
