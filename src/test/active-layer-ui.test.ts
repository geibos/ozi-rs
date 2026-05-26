import { describe, expect, it } from "vitest";
import { readFileSync } from "fs";
import { join } from "path";

const commandsSource = readFileSync(
  join(__dirname, "../../src-tauri/src/commands/mod.rs"),
  "utf-8"
);
const typesSource = readFileSync(join(__dirname, "../lib/types.ts"), "utf-8");
const storesSource = readFileSync(join(__dirname, "../lib/stores.ts"), "utf-8");
// The legacy `Sidebar.svelte` / `WaypointsPanel.svelte` floating-panel
// surfaces were removed by `redesign-library-sidebar`. Their active-layer
// wiring now lives inside the three Library tabs.
const tracksTabSource = readFileSync(
  join(__dirname, "../components/library/TracksTab.svelte"),
  "utf-8"
);
const waypointsTabSource = readFileSync(
  join(__dirname, "../components/library/WaypointsTab.svelte"),
  "utf-8"
);
const mapViewSource = readFileSync(join(__dirname, "../components/MapView.svelte"), "utf-8");

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

  it("renders minimal active track and waypoint layer selectors in the Library tabs", () => {
    expect(tracksTabSource).toContain("activeTrackLayerId");
    expect(tracksTabSource).toContain("$appState?.track_layers");
    expect(tracksTabSource).toContain("Track layer");
    expect(waypointsTabSource).toContain("activeWaypointLayerId");
    expect(waypointsTabSource).toContain("$appState?.waypoint_layers");
    expect(waypointsTabSource).toContain("Waypoint layer");
  });

  it("routes drawing workflows through the selected track layer", () => {
    expect(mapViewSource).not.toContain("insertTrackPoint(1n");
    expect(storesSource).toContain("drawingTrackLayerId");
  });

  it("routes waypoint workflows through the selected waypoint layer", () => {
    expect(mapViewSource).not.toContain("getWaypoints(1n");
    expect(mapViewSource).not.toContain("addWaypoint(1n");
    expect(mapViewSource).not.toContain("moveWaypoint(1n");
    expect(waypointsTabSource).not.toContain("currentLayerId = 1n");
    expect(waypointsTabSource).toContain("activeWaypointLayerId");
  });

  it("keeps Svelte components behind typed API wrappers", () => {
    expect(tracksTabSource).not.toContain("invoke(");
    expect(waypointsTabSource).not.toContain("invoke(");
    expect(mapViewSource).not.toContain("invoke(");
  });
});
