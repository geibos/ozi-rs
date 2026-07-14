import { describe, expect, it, vi, beforeEach, afterEach } from "vitest";
import { readFileSync } from "fs";
import { join } from "path";

// Waypoint visibility wiring lives in the Library Waypoints tab and its
// shared `LibraryRow.svelte` row primitive after `redesign-library-sidebar`.
const tabSource = readFileSync(
  join(__dirname, "../components/library/WaypointsTab.svelte"),
  "utf-8",
);

const rowSource = readFileSync(
  join(__dirname, "../components/library/LibraryRow.svelte"),
  "utf-8",
);

const mapViewSource = readFileSync(
  join(__dirname, "../components/MapView.svelte"),
  "utf-8",
);

const apiSource = readFileSync(join(__dirname, "../lib/api.ts"), "utf-8");
const typesSource = readFileSync(join(__dirname, "../lib/types.ts"), "utf-8");

describe("Library Waypoints tab visibility toggle", () => {
  it("imports the typed toggleWaypointVisible wrapper", () => {
    expect(tabSource).toContain("toggleWaypointVisible");
    expect(tabSource).not.toContain("invoke(");
  });

  it("renders a visibility icon button per waypoint row via LibraryRow", () => {
    // LibraryRow encapsulates the Eye / EyeOff icon-button toggle; the
    // tab passes an `onToggleVisibility` callback per row.
    expect(tabSource).toContain("onToggleVisibility");
    expect(tabSource).toContain("handleToggleVisible(r)");
    expect(rowSource).toContain("EyeIcon");
    expect(rowSource).toContain("EyeOffIcon");
  });

  it("indicates hidden state on the row without removing it from the tab", () => {
    // LibraryRow dims hidden rows via the `opacity-60` Tailwind utility;
    // the row keeps rendering in the list regardless of visibility.
    expect(rowSource).toContain("class:opacity-60={!visible}");
  });
});

describe("MapView waypoint visibility filter", () => {
  it("filters waypoints by the `visible` flag before placing markers", () => {
    expect(mapViewSource).toContain(
      "waypoints.filter((w) => w.visible !== false)",
    );
  });
});

describe("api.ts wrapper for toggle_waypoint_visible", () => {
  it("exposes toggleWaypointVisible(layerId, waypointId)", () => {
    expect(apiSource).toContain(
      "export async function toggleWaypointVisible(\n  layerId: bigint,\n  waypointId: bigint,\n)",
    );
    // `api.ts` now routes through the `invokeIpc` wrapper from
    // `src/lib/ipc.ts` so dev IPC failures surface as a structured toast.
    expect(apiSource).toContain(
      'invokeIpc("toggle_waypoint_visible", { layerId, waypointId })',
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
    // `invokeIpc` converts bigint IDs to plain numbers at the IPC boundary —
    // Tauri 2 serializes invoke args with JSON.stringify, which throws on
    // BigInt (see src/lib/ipc.ts and src/test/ipc-bigint-args.test.ts).
    expect(invokeSpy).toHaveBeenCalledWith("toggle_waypoint_visible", {
      layerId: 7,
      waypointId: 42,
    });
  });
});
