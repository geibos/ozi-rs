<script lang="ts">
  /**
   * Waypoint Inspector — inline editor for the selected waypoint.
   *
   * No dialogs at any point (per locked design). Name / symbol / visibility
   * edits dispatch through the existing `ProjectCommand`-shaped endpoints
   * in `src/lib/api.ts`. Delete also dispatches through `ProjectCommand` —
   * there is intentionally no confirmation dialog here (Undo via Cmd+Z
   * remains available through the project command bus).
   */
  import EyeIcon from "@lucide/svelte/icons/eye";
  import { reportEditFailure } from "$lib/edit-failure";
  import EyeOffIcon from "@lucide/svelte/icons/eye-off";
  import FileOutputIcon from "@lucide/svelte/icons/file-output";
  import MapPinIcon from "@lucide/svelte/icons/map-pin";
  import MoveIcon from "@lucide/svelte/icons/move";
  import Trash2Icon from "@lucide/svelte/icons/trash-2";
  import { Button } from "$lib/components/ui/button";
  import { Input } from "$lib/components/ui/input";
  import {
    activeWaypointLayerId,
    appState,
    selectedWaypointId,
  } from "$lib/stores";
  import {
    deleteWaypoint,
    exportGpxWaypoints,
    exportWptWaypoints,
    getWaypoints,
    getWaypointsExportDefaultPath,
    renameWaypoint,
    setWaypointSymbol,
    setWaypointColor,
    toggleWaypointVisible,
  } from "$lib/api";
  import { open } from "@tauri-apps/plugin-dialog";
  import { t } from "$lib/i18n";
  import { toast } from "svelte-sonner";
  import type { WaypointData } from "$lib/types";
  import SymbolPicker from "../SymbolPicker.svelte";
  import { waypointColorHex } from "$lib/waypoint-symbols";

  // `$state<T>(...)` rather than an annotated `let`: with the annotation,
  // TypeScript's flow analysis narrows the variable to `null` at any point
  // before the first assignment, which is every inline `$derived` in this
  // file — and then the non-null branch is `never`.
  let waypoint = $state<WaypointData | null>(null);
  let nameDraft = $state("");
  let nameDirty = $state(false);

  $effect(() => {
    const id = $selectedWaypointId;
    const layerId = $activeWaypointLayerId;
    if (id === null || layerId === null || !$appState) {
      waypoint = null;
      return;
    }
    void loadWaypoint(layerId, id);
  });

  async function loadWaypoint(layerId: bigint, id: bigint) {
    try {
      const all = await getWaypoints(layerId);
      const found = all.find((w) => BigInt(w.id) === id) ?? null;
      waypoint = found;
      if (found && !nameDirty) nameDraft = found.name;
    } catch (error) {
      reportEditFailure("inspector.waypointLoadFailed", error);
      waypoint = null;
    }
  }

  async function commitName() {
    const wp = waypoint;
    const layerId = $activeWaypointLayerId;
    if (!wp || layerId === null) return;
    const trimmed = nameDraft.trim();
    if (!trimmed || trimmed === wp.name) {
      nameDraft = wp.name;
      nameDirty = false;
      return;
    }
    try {
      await renameWaypoint(layerId, BigInt(wp.id), trimmed);
      nameDirty = false;
    } catch (error) {
      toast.error("Failed to rename waypoint", { description: String(error) });
      nameDraft = wp.name;
      nameDirty = false;
    }
  }

  function handleNameInput(event: Event) {
    nameDraft = (event.currentTarget as HTMLInputElement).value;
    nameDirty = true;
  }

  async function handleSetSymbol(symbol: string | null) {
    const wp = waypoint;
    const layerId = $activeWaypointLayerId;
    if (!wp || layerId === null) return;
    try {
      await setWaypointSymbol(layerId, BigInt(wp.id), symbol);
    } catch (error) {
      reportEditFailure("inspector.waypointSymbolFailed", error);
    }
  }

  async function handleToggleVisible() {
    const wp = waypoint;
    const layerId = $activeWaypointLayerId;
    if (!wp || layerId === null) return;
    try {
      await toggleWaypointVisible(layerId, BigInt(wp.id));
    } catch (error) {
      toast.error("Failed to toggle visibility", {
        description: String(error),
      });
    }
  }

  async function handleDelete() {
    const wp = waypoint;
    const layerId = $activeWaypointLayerId;
    if (!wp || layerId === null) return;
    try {
      await deleteWaypoint(layerId, BigInt(wp.id));
      selectedWaypointId.set(null);
    } catch (error) {
      toast.error("Failed to delete waypoint", { description: String(error) });
    }
  }

  /** GPX for phones and other groups' software, WPT for OziExplorer. */
  async function handleExport(format: "gpx" | "wpt") {
    const layerId = $activeWaypointLayerId;
    if (layerId === null) return;
    try {
      const defaultPath = await getWaypointsExportDefaultPath(layerId, format);
      const path = await open({
        save: true,
        defaultPath: defaultPath ?? `waypoints.${format}`,
        filters:
          format === "gpx"
            ? [{ name: "GPX", extensions: ["gpx"] }]
            : [{ name: "OziExplorer WPT", extensions: ["wpt"] }],
      } as Parameters<typeof open>[0]);
      if (!path) return;
      if (format === "gpx") {
        await exportGpxWaypoints(layerId, path as string);
      } else {
        await exportWptWaypoints(layerId, path as string);
      }
    } catch (error) {
      toast.error($t("inspector.exportFailed"), {
        description: String(error),
      });
    }
  }

  function handleMoveOnMap() {
    toast.message("Click on the map to move this waypoint", {
      description:
        "Drag-to-move on the map is wired by the existing waypoint layer.",
    });
  }
  const swatchHex = $derived(waypointColorHex(waypoint?.color));

  async function applyColor(color: [number, number, number, number] | null) {
    const wp = waypoint;
    const layerId = $activeWaypointLayerId;
    if (!wp || layerId === null) return;
    try {
      await setWaypointColor(layerId, BigInt(wp.id), color);
    } catch (error) {
      reportEditFailure("inspector.waypointColorFailed", error);
    }
  }

  function handleColorChange(event: Event) {
    const hex = (event.currentTarget as HTMLInputElement).value;
    const [r, g, b] = [1, 3, 5].map((i) => parseInt(hex.slice(i, i + 2), 16));
    void applyColor([r, g, b, 255]);
  }
</script>

<div class="flex h-full flex-col gap-4 overflow-y-auto p-4">
  <header class="flex items-start gap-3">
    <SymbolPicker
      symbol={waypoint?.symbol}
      color={waypoint?.color}
      onSelect={handleSetSymbol}
    />
    <!-- The symbol says what a mark is; the colour says whose it is. Clearing
         returns it to the default rather than to a colour that looks like it. -->
    <div class="flex shrink-0 flex-col items-center gap-1">
      <input
        class="border-border h-8 w-10 rounded-sm border bg-transparent p-0"
        type="color"
        value={swatchHex}
        aria-label={$t("inspector.waypointColor")}
        data-testid="waypoint-color"
        onchange={handleColorChange}
      />
      {#if waypoint?.color}
        <button
          class="text-muted-foreground hover:text-foreground border-0 bg-transparent p-0 text-[10px]"
          onclick={() => void applyColor(null)}
          data-testid="waypoint-color-reset"
        >
          {$t("inspector.waypointColorDefault")}
        </button>
      {/if}
    </div>
    <div class="min-w-0 flex-1">
      <Input
        type="text"
        value={nameDraft}
        oninput={handleNameInput}
        onblur={commitName}
        onkeydown={(e) => {
          if (e.key === "Enter") (e.currentTarget as HTMLInputElement).blur();
          if (e.key === "Escape") {
            nameDraft = waypoint?.name ?? "";
            nameDirty = false;
            (e.currentTarget as HTMLInputElement).blur();
          }
        }}
        disabled={!waypoint}
        class="h-8 text-sm font-medium"
        aria-label={$t("inspector.waypointName")}
      />
    </div>
    <Button
      variant="ghost"
      size="icon-sm"
      onclick={handleToggleVisible}
      disabled={!waypoint}
      aria-label={waypoint?.visible
        ? $t("inspector.hideWaypoint")
        : $t("inspector.showWaypoint")}
    >
      {#if waypoint?.visible}
        <EyeIcon class="size-4" />
      {:else}
        <EyeOffIcon class="size-4" />
      {/if}
    </Button>
  </header>

  <section
    class="bg-card border-border rounded-[var(--radius-card)] border p-4"
    aria-label={$t("inspector.location")}
  >
    <h3
      class="text-muted-foreground/80 mb-3 flex items-center gap-1.5 text-[10px] font-semibold tracking-wider uppercase"
    >
      <MapPinIcon class="size-3" />
      {$t("inspector.location")}
    </h3>
    {#if waypoint}
      <dl class="grid grid-cols-[3.5rem_1fr] gap-x-3 gap-y-2 text-xs">
        <dt class="text-muted-foreground">{$t("inspector.latitude")}</dt>
        <dd class="font-mono">{waypoint.lat.toFixed(6)}</dd>
        <dt class="text-muted-foreground">{$t("inspector.longitude")}</dt>
        <dd class="font-mono">{waypoint.lon.toFixed(6)}</dd>
      </dl>
      <Button
        variant="outline"
        size="sm"
        class="mt-3 w-full justify-start gap-2"
        onclick={handleMoveOnMap}
      >
        <MoveIcon class="size-3.5" />
        {$t("inspector.moveOnMap")}
      </Button>
    {:else}
      <p class="text-muted-foreground text-xs">{$t("inspector.noWaypoint")}</p>
    {/if}
  </section>

  <section
    class="flex flex-col gap-2"
    aria-label={$t("inspector.waypointActions")}
  >
    <Button
      variant="outline"
      size="sm"
      class="justify-start gap-2"
      onclick={() => handleExport("gpx")}
      disabled={!waypoint}
    >
      <FileOutputIcon class="size-4" />
      {$t("inspector.exportGpx")}
    </Button>
    <Button
      variant="outline"
      size="sm"
      class="justify-start gap-2"
      onclick={() => handleExport("wpt")}
      disabled={!waypoint}
    >
      <FileOutputIcon class="size-4" />
      {$t("inspector.exportWpt")}
    </Button>
    <Button
      variant="outline"
      size="sm"
      class="text-destructive hover:text-destructive justify-start gap-2"
      onclick={handleDelete}
      disabled={!waypoint}
    >
      <Trash2Icon class="size-4" />
      {$t("inspector.deleteWaypoint")}
    </Button>
  </section>
</div>
