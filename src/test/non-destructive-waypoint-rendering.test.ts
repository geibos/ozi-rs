import { describe, expect, it } from "vitest";
import { readFileSync } from "fs";
import { join } from "path";
import { selectVisibleWaypointLayers } from "../lib/waypoint-layers";
import type { LayerSummaryDto } from "../lib/types";

const mapViewSource = readFileSync(
  join(__dirname, "../components/MapView.svelte"),
  "utf-8",
);
const storesSource = readFileSync(join(__dirname, "../lib/stores.ts"), "utf-8");
const typesSource = readFileSync(join(__dirname, "../lib/types.ts"), "utf-8");

describe("selectVisibleWaypointLayers", () => {
  it("returns all layers when no visibility flag is set (backend default)", () => {
    const layers: LayerSummaryDto[] = [
      { id: 10, name: "штаб" },
      { id: 20, name: "найдено" },
    ];
    expect(selectVisibleWaypointLayers(layers)).toEqual(layers);
  });

  it("excludes layers explicitly marked invisible", () => {
    const layers: LayerSummaryDto[] = [
      { id: 10, name: "штаб", visible: true },
      { id: 20, name: "найдено", visible: false },
      { id: 30, name: "опасности" },
    ];
    const result = selectVisibleWaypointLayers(layers);
    expect(result.map((l) => l.id)).toEqual([10, 30]);
  });

  it("treats null / undefined input as empty list", () => {
    expect(selectVisibleWaypointLayers(null)).toEqual([]);
    expect(selectVisibleWaypointLayers(undefined)).toEqual([]);
  });

  it("simulates the two-layers scenario from the layers spec", () => {
    // Layer A is active (two waypoints conceptually), B is inactive (three).
    // Both layers stay visible regardless of which is active.
    const layers: LayerSummaryDto[] = [
      { id: 1, name: "A" },
      { id: 2, name: "B" },
    ];
    expect(selectVisibleWaypointLayers(layers)).toHaveLength(2);
  });
});

describe("MapView non-destructive waypoint rendering", () => {
  it("iterates all visible waypoint layers, not only the active one", () => {
    expect(mapViewSource).toContain("visibleWaypointLayers");
    expect(mapViewSource).toContain("for (const layer of layers)");
    // The old active-only early-return must be gone.
    expect(mapViewSource).not.toMatch(
      /const layerId = \$activeWaypointLayerId;\s*if \(!\$appState[^}]*layerId === null\) return;/,
    );
  });

  it("keys waypoint markers by owning layer id and waypoint id", () => {
    expect(mapViewSource).toContain("waypointMarkerKey");
    expect(mapViewSource).toContain("`${layerId.toString()}:${waypointId}`");
    expect(mapViewSource).toContain("waypointMarkers = new Map<string, maplibregl.Marker>");
  });

  it("renders markers from inactive layers as non-draggable read-only context", () => {
    expect(mapViewSource).toContain("draggable: isActive");
    expect(mapViewSource).toContain("inactive-layer");
  });

  it("honours per-waypoint visibility flag", () => {
    expect(mapViewSource).toMatch(
      /if \(wp\.visible === false\) continue;|waypoints\.filter\(\(w\) => w\.visible !== false\)/,
    );
    expect(typesSource).toMatch(/visible[?:]?\s*:\s*boolean/);
  });

  it("routes click-to-add (placement mode) only to the active waypoint layer", () => {
    // The placement handler still resolves $activeWaypointLayerId and calls
    // addWaypoint with that id, so new waypoints land in layer A only.
    expect(mapViewSource).toContain("async function handleMapClickForWaypoint");
    expect(mapViewSource).toMatch(
      /handleMapClickForWaypoint[\s\S]+?const layerId = \$activeWaypointLayerId;[\s\S]+?await addWaypoint\(layerId,/,
    );
  });

  it("routes drag-to-move to the dragged marker's owning layer", () => {
    // The dragend handler captures `layerId` from the iteration over all
    // visible waypoint layers, not from the active-layer store.
    expect(mapViewSource).toMatch(
      /for \(const layer of layers\)[\s\S]+?const layerId = BigInt\(layer\.id\);/,
    );
    expect(mapViewSource).toMatch(
      /marker\.on\("dragend"[\s\S]+?await moveWaypoint\(layerId,/,
    );
  });

  it("exposes a derived store for visible waypoint layers", () => {
    expect(storesSource).toContain("export const visibleWaypointLayers");
    expect(storesSource).toContain("selectVisibleWaypointLayers");
  });
});
