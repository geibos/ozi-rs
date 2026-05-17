import { describe, expect, it, vi, beforeEach, afterEach } from "vitest";
import { readFileSync } from "fs";
import { join } from "path";

const panelSource = readFileSync(
  join(__dirname, "../components/WaypointsPanel.svelte"),
  "utf-8"
);

const mapViewSource = readFileSync(
  join(__dirname, "../components/MapView.svelte"),
  "utf-8"
);

const apiSource = readFileSync(join(__dirname, "../lib/api.ts"), "utf-8");
const typesSource = readFileSync(join(__dirname, "../lib/types.ts"), "utf-8");

describe("WaypointsPanel visibility toggle", () => {
  it("imports the typed toggleWaypointVisible wrapper", () => {
    expect(panelSource).toContain("toggleWaypointVisible");
    expect(panelSource).not.toContain("invoke(");
  });

  it("renders a visibility checkbox per waypoint row bound to wp.visible", () => {
    expect(panelSource).toContain('type="checkbox"');
    // Chrome moved to Tailwind utilities (accent-primary / cursor-pointer /
    // shrink-0) — the legacy `class="visibility-toggle"` selector is gone.
    expect(panelSource).toContain("accent-primary");
    expect(panelSource).toContain("checked={wp.visible}");
    expect(panelSource).toContain("handleToggleVisible(wp)");
  });

  it("indicates hidden state on the row without removing it from the panel", () => {
    // Legacy `.hidden-waypoint` CSS class is gone; the hidden state now
    // dims the row with the Tailwind `opacity-50` utility (which is theme-
    // and flavour-independent, and does not unmount the row).
    expect(panelSource).toContain("class:opacity-50={!wp.visible}");
  });
});

describe("MapView waypoint visibility filter", () => {
  it("filters waypoints by the `visible` flag before placing markers", () => {
    expect(mapViewSource).toContain("waypoints.filter((w) => w.visible !== false)");
  });
});

describe("api.ts wrapper for toggle_waypoint_visible", () => {
  it("exposes toggleWaypointVisible(layerId, waypointId)", () => {
    expect(apiSource).toContain(
      "export async function toggleWaypointVisible(\n  layerId: bigint,\n  waypointId: bigint\n)"
    );
    expect(apiSource).toContain(
      'invoke("toggle_waypoint_visible", { layerId, waypointId })'
    );
  });
});

describe("WaypointData type", () => {
  it("includes a required visible boolean field", () => {
    expect(typesSource).toMatch(/visible:\s*boolean/);
  });
});

describe("toggleWaypointVisible IPC contract", () => {
  beforeEach(() => {
    vi.resetModules();
  });

  afterEach(() => {
    vi.clearAllMocks();
  });

  it("invokes toggle_waypoint_visible with the correct argument shape", async () => {
    const invokeSpy = vi.fn().mockResolvedValue(undefined);
    vi.doMock("@tauri-apps/api/core", () => ({ invoke: invokeSpy }));

    const api = await import("../lib/api");
    await api.toggleWaypointVisible(7n, 42n);

    expect(invokeSpy).toHaveBeenCalledOnce();
    expect(invokeSpy).toHaveBeenCalledWith("toggle_waypoint_visible", {
      layerId: 7n,
      waypointId: 42n,
    });
  });
});
