import { describe, expect, it } from "vitest";
import { readFileSync } from "fs";
import { join } from "path";

const commandsSource = readFileSync(
  join(__dirname, "../../src-tauri/src/commands/mod.rs"),
  "utf-8"
);
const typesSource = readFileSync(join(__dirname, "../lib/types.ts"), "utf-8");
const storesSource = readFileSync(join(__dirname, "../lib/stores.ts"), "utf-8");
const sidebarSource = readFileSync(join(__dirname, "../components/Sidebar.svelte"), "utf-8");
const mapViewSource = readFileSync(join(__dirname, "../components/MapView.svelte"), "utf-8");
const waypointsPanelSource = readFileSync(
  join(__dirname, "../components/WaypointsPanel.svelte"),
  "utf-8"
);

describe("active layer UI wiring", () => {
  it("exposes minimal track and waypoint layer lists in app state DTOs", () => {
    expect(commandsSource).toContain("pub struct LayerSummaryDto");
    expect(commandsSource).toContain("pub track_layers: Vec<LayerSummaryDto>");
    expect(commandsSource).toContain("pub waypoint_layers: Vec<LayerSummaryDto>");
    expect(typesSource).toContain("export interface LayerSummaryDto");
    expect(typesSource).toContain("track_layers: LayerSummaryDto[]");
    expect(typesSource).toContain("waypoint_layers: LayerSummaryDto[]");
  });

  it("keeps active layer selection in UI-only stores with app-state fallback", () => {
    expect(storesSource).toContain("activeTrackLayerId");
    expect(storesSource).toContain("activeWaypointLayerId");
    expect(storesSource).toContain("track_layers");
    expect(storesSource).toContain("waypoint_layers");
  });

  it("renders minimal active track and waypoint layer selectors in the sidebar", () => {
    expect(sidebarSource).toContain("activeTrackLayerId");
    expect(sidebarSource).toContain("activeWaypointLayerId");
    expect(sidebarSource).toContain("$appState?.track_layers");
    expect(sidebarSource).toContain("$appState?.waypoint_layers");
    expect(sidebarSource).toContain("Track layer");
    expect(sidebarSource).toContain("Waypoint layer");
  });

  it("routes drawing workflows through the selected track layer", () => {
    expect(sidebarSource).not.toContain("createEmptyTrack(1n");
    expect(sidebarSource).not.toContain("getTrackDetail(1n");
    expect(mapViewSource).not.toContain("insertTrackPoint(1n");
    expect(storesSource).toContain("drawingTrackLayerId");
  });

  it("routes waypoint workflows through the selected waypoint layer", () => {
    expect(mapViewSource).not.toContain("getWaypoints(1n");
    expect(mapViewSource).not.toContain("addWaypoint(1n");
    expect(mapViewSource).not.toContain("moveWaypoint(1n");
    expect(waypointsPanelSource).not.toContain("currentLayerId = 1n");
    expect(waypointsPanelSource).toContain("activeWaypointLayerId");
  });

  it("keeps Svelte components behind typed API wrappers", () => {
    expect(sidebarSource).not.toContain("invoke(");
    expect(mapViewSource).not.toContain("invoke(");
    expect(waypointsPanelSource).not.toContain("invoke(");
  });
});
