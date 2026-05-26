<script lang="ts">
  /**
   * Waypoints tab inside the LibraryRail. Lists every waypoint across all
   * waypoint layers; header carries the active-waypoint-layer `Select`.
   *
   * Per-row affordances: visibility toggle, symbol button (opens
   * `SymbolPicker`), name (double-click rename), `⋯` menu with Export WPT
   * and Delete. Clicking a row from a non-active waypoint layer flips the
   * active waypoint layer first (`layers` capability extension).
   *
   * Visibility is per-waypoint, not per-layer: each row stores its own
   * `visible` flag, toggled through `toggle_waypoint_visible`.
   */
  import { Label } from "$lib/components/ui/label";
  import * as DropdownMenu from "$lib/components/ui/dropdown-menu";
  import * as Select from "$lib/components/ui/select";
  import {
    activeWaypointLayerId,
    appState,
    selectedWaypointId,
  } from "$lib/stores";
  import {
    deleteWaypoint,
    exportWptWaypoints,
    getWaypoints,
    getWptExportDefaultPath,
    renameWaypoint,
    setWaypointSymbol,
    toggleWaypointVisible,
  } from "$lib/api";
  import { open } from "@tauri-apps/plugin-dialog";
  import { toast } from "svelte-sonner";
  import type { WaypointData } from "$lib/types";
  import LibraryRow from "./LibraryRow.svelte";
  import SymbolPicker from "../SymbolPicker.svelte";

  interface WaypointRow {
    layerId: bigint;
    layerName: string;
    wp: WaypointData;
  }

  let rows: WaypointRow[] = $state([]);

  const waypointLayers = $derived($appState?.waypoint_layers ?? []);
  const waypointLayerSelectValue = $derived(
    $activeWaypointLayerId !== null ? $activeWaypointLayerId.toString() : "",
  );

  $effect(() => {
    // Re-load when the app state changes (layer count, or refresh signal)
    // and when the active layer changes. We collect waypoints from every
    // layer so the tab shows the full project, sorted by (layer, id).
    const layers = $appState?.waypoint_layers ?? [];
    if (!$appState || layers.length === 0) {
      rows = [];
      return;
    }
    void loadAll(layers);
  });

  async function loadAll(
    layers: { id: number; name: string }[],
  ): Promise<void> {
    try {
      const collected: WaypointRow[] = [];
      for (const layer of layers) {
        const wps = await getWaypoints(BigInt(layer.id));
        for (const wp of wps) {
          collected.push({
            layerId: BigInt(layer.id),
            layerName: layer.name,
            wp,
          });
        }
      }
      collected.sort((a, b) => {
        if (a.layerId === b.layerId) return a.wp.id - b.wp.id;
        return a.layerId < b.layerId ? -1 : 1;
      });
      rows = collected;
    } catch (err) {
      console.error("Failed to load waypoints", err);
      toast.error("Failed to load waypoints", { description: String(err) });
      rows = [];
    }
  }

  function rowKey(r: WaypointRow): string {
    return `${r.layerId}:${r.wp.id}`;
  }

  function isSelected(r: WaypointRow): boolean {
    return $selectedWaypointId === BigInt(r.wp.id);
  }

  async function handleToggleVisible(r: WaypointRow) {
    await toggleWaypointVisible(r.layerId, BigInt(r.wp.id));
    rows = rows.map((row) =>
      rowKey(row) === rowKey(r)
        ? { ...row, wp: { ...row.wp, visible: !row.wp.visible } }
        : row,
    );
  }

  function handleSelectRow(r: WaypointRow) {
    // layers spec extension: switch the active layer if needed.
    if ($activeWaypointLayerId !== r.layerId) {
      activeWaypointLayerId.set(r.layerId);
    }
    selectedWaypointId.set(BigInt(r.wp.id));
  }

  async function handleRename(r: WaypointRow, newName: string) {
    await renameWaypoint(r.layerId, BigInt(r.wp.id), newName);
  }

  async function handleSetSymbol(r: WaypointRow, symbol: string | null) {
    await setWaypointSymbol(r.layerId, BigInt(r.wp.id), symbol);
  }

  async function handleDelete(r: WaypointRow) {
    try {
      await deleteWaypoint(r.layerId, BigInt(r.wp.id));
    } catch (err) {
      toast.error("Failed to delete waypoint", { description: String(err) });
    }
  }

  async function handleExportWptForLayer(layerId: bigint) {
    const defaultPath = await getWptExportDefaultPath(layerId);
    const path = await open({
      save: true,
      defaultPath: defaultPath ?? "waypoints.wpt",
      filters: [{ name: "OziExplorer WPT", extensions: ["wpt"] }],
    } as Parameters<typeof open>[0]);
    if (path) {
      await exportWptWaypoints(layerId, path as string);
    }
  }
</script>

<div class="flex h-full min-h-0 flex-col">
  <header class="border-border border-b px-2 py-1.5">
    {#if waypointLayers.length > 0}
      <Label class="text-muted-foreground text-[10px]">Waypoint layer</Label>
      <Select.Root
        type="single"
        value={waypointLayerSelectValue}
        onValueChange={(v) => v && activeWaypointLayerId.set(BigInt(v))}
      >
        <Select.Trigger aria-label="Waypoint layer" size="sm" class="w-full">
          {waypointLayers.find(
            (l) => String(l.id) === waypointLayerSelectValue,
          )?.name ?? "Pick layer"}
        </Select.Trigger>
        <Select.Content>
          {#each waypointLayers as layer (layer.id)}
            <Select.Item value={String(layer.id)} label={layer.name}>
              {layer.name}
            </Select.Item>
          {/each}
        </Select.Content>
      </Select.Root>
    {/if}
  </header>

  <div class="flex-1 overflow-y-auto py-1" data-testid="waypoints-tab-list">
    {#if rows.length === 0}
      <div class="text-muted-foreground p-3 text-center text-xs">
        No waypoints
      </div>
    {:else}
      {#each rows as r (rowKey(r))}
        <LibraryRow
          name={r.wp.name}
          visible={r.wp.visible}
          visibilityLabel={r.wp.visible
            ? `Hide waypoint ${r.wp.name}`
            : `Show waypoint ${r.wp.name}`}
          selected={isSelected(r)}
          onToggleVisibility={() => handleToggleVisible(r)}
          onSelect={() => handleSelectRow(r)}
          onRename={(name) => handleRename(r, name)}
        >
          {#snippet leadingControl()}
            <SymbolPicker
              symbol={r.wp.symbol}
              onSelect={(sym) => handleSetSymbol(r, sym)}
            />
          {/snippet}
          {#snippet actions()}
            <DropdownMenu.Item
              onSelect={() => handleExportWptForLayer(r.layerId)}
            >
              Export WPT
            </DropdownMenu.Item>
            <DropdownMenu.Separator />
            <DropdownMenu.Item
              variant="destructive"
              onSelect={() => handleDelete(r)}
            >
              Delete
            </DropdownMenu.Item>
          {/snippet}
        </LibraryRow>
      {/each}
    {/if}
  </div>
</div>
