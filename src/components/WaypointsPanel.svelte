<script lang="ts">
  import FileOutputIcon from "@lucide/svelte/icons/file-output";
  import Trash2Icon from "@lucide/svelte/icons/trash-2";
  import XIcon from "@lucide/svelte/icons/x";
  import { Button } from "$lib/components/ui/button";
  import * as Dialog from "$lib/components/ui/dialog";
  import { Separator } from "$lib/components/ui/separator";
  import * as Tooltip from "$lib/components/ui/tooltip";
  import {
    appState,
    activeWaypointLayerId,
    waypointsPanelOpen,
    selectedWaypointId,
  } from "$lib/stores";
  import {
    getWaypoints,
    deleteWaypoint,
    renameWaypoint,
    setWaypointSymbol,
    toggleWaypointVisible,
    exportWptWaypoints,
    getWptExportDefaultPath,
  } from "$lib/api";
  import { open } from "@tauri-apps/plugin-dialog";
  import { toast } from "svelte-sonner";
  import type { WaypointData } from "$lib/types";
  import SymbolPicker from "./SymbolPicker.svelte";

  let waypoints: WaypointData[] = $state([]);
  let editingWaypoint: number | null = $state(null);
  let editName = $state("");
  let pendingDelete: WaypointData | null = $state(null);
  let deleteDialogOpen = $state(false);

  function focusOnMount(node: HTMLInputElement) {
    node.focus();
  }

  $effect(() => {
    const layerId = $activeWaypointLayerId;
    if ($appState && layerId !== null) {
      loadWaypoints();
    } else {
      waypoints = [];
    }
  });

  async function loadWaypoints() {
    try {
      const layerId = $activeWaypointLayerId;
      if ($appState && $appState.waypoint_layer_count > 0 && layerId !== null) {
        waypoints = await getWaypoints(layerId);
      } else {
        waypoints = [];
      }
    } catch (e) {
      console.error("Failed to load waypoints", e);
      toast.error("Failed to load waypoints", { description: String(e) });
      waypoints = [];
    }
  }

  function startRename(wp: WaypointData) {
    editingWaypoint = wp.id;
    editName = wp.name;
  }

  async function commitRename(wp: WaypointData) {
    const layerId = $activeWaypointLayerId;
    if (layerId === null) return;
    if (editName.trim() && editName !== wp.name) {
      await renameWaypoint(layerId, BigInt(wp.id), editName.trim());
    }
    editingWaypoint = null;
  }

  function requestDelete(wp: WaypointData) {
    pendingDelete = wp;
    deleteDialogOpen = true;
  }

  async function confirmDelete() {
    const wp = pendingDelete;
    const layerId = $activeWaypointLayerId;
    if (!wp || layerId === null) {
      deleteDialogOpen = false;
      pendingDelete = null;
      return;
    }
    await deleteWaypoint(layerId, BigInt(wp.id));
    deleteDialogOpen = false;
    pendingDelete = null;
  }

  function selectWaypoint(wp: WaypointData) {
    selectedWaypointId.set(BigInt(wp.id));
  }

  async function handleSetSymbol(wp: WaypointData, symbol: string | null) {
    const layerId = $activeWaypointLayerId;
    if (layerId === null) return;
    await setWaypointSymbol(layerId, BigInt(wp.id), symbol);
  }

  async function handleToggleVisible(wp: WaypointData) {
    const layerId = $activeWaypointLayerId;
    if (layerId === null) return;
    await toggleWaypointVisible(layerId, BigInt(wp.id));
    waypoints = waypoints.map((w) =>
      w.id === wp.id ? { ...w, visible: !w.visible } : w,
    );
  }

  async function handleExportWpt() {
    const layerId = $activeWaypointLayerId;
    if (layerId === null || waypoints.length === 0) return;
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

{#if $waypointsPanelOpen}
  <div
    class="bg-popover text-popover-foreground border-border fixed top-[60px] right-[310px] z-[200] flex max-h-[calc(100vh-80px)] w-72 resize flex-col overflow-hidden rounded-lg border shadow-lg"
  >
    <div
      class="bg-card text-muted-foreground border-border flex cursor-move items-center justify-between border-b px-2.5 py-1 text-xs select-none"
    >
      <span>Waypoints ({waypoints.length})</span>
      <div class="flex items-center gap-1">
        <Button
          variant="outline"
          size="xs"
          disabled={waypoints.length === 0}
          onclick={handleExportWpt}
          aria-label={waypoints.length === 0
            ? "No waypoints to export"
            : "Export waypoints (WPT)"}
        >
          <FileOutputIcon class="size-3" />
          WPT
        </Button>
        <Button
          variant="ghost"
          size="icon-xs"
          onclick={() => waypointsPanelOpen.set(false)}
          aria-label="Close waypoints panel"
        >
          <XIcon />
        </Button>
      </div>
    </div>
    <div class="flex-1 overflow-y-auto py-1">
      {#if waypoints.length === 0}
        <div class="text-muted-foreground p-3 text-center text-xs">
          No waypoints
        </div>
      {:else}
          {#each waypoints as wp, idx (wp.id)}
            {#if idx > 0}
              <Separator />
            {/if}
            <div
              class="hover:bg-muted flex items-center gap-1.5 px-2.5 py-1 transition-colors"
              class:opacity-50={!wp.visible}
            >
              <input
                type="checkbox"
                class="accent-primary shrink-0 cursor-pointer"
                checked={wp.visible}
                title={wp.visible ? "Hide waypoint" : "Show waypoint"}
                aria-label="Toggle waypoint visibility"
                onchange={() => handleToggleVisible(wp)}
              />
              <SymbolPicker
                symbol={wp.symbol}
                onSelect={(symbol) => handleSetSymbol(wp, symbol)}
              />

              {#if editingWaypoint === wp.id}
                <input
                  class="bg-background text-foreground border-input flex-1 rounded-sm border px-1 py-px text-xs"
                  bind:value={editName}
                  onblur={() => commitRename(wp)}
                  onkeydown={(e) => e.key === "Enter" && commitRename(wp)}
                  use:focusOnMount
                />
              {:else}
                <!-- svelte-ignore a11y_click_events_have_key_events, a11y_no_static_element_interactions -->
                <span
                  class="flex-1 cursor-text truncate text-xs"
                  class:text-muted-foreground={!wp.visible}
                  class:italic={!wp.visible}
                  onclick={() => selectWaypoint(wp)}
                  ondblclick={() => startRename(wp)}
                  title="Click to select, double-click to rename"
                >
                  {wp.name}
                </span>
              {/if}

              <div class="flex shrink-0 items-center">
                <Tooltip.Root>
                  <Tooltip.Trigger
                    class={"text-muted-foreground hover:text-destructive inline-flex size-6 items-center justify-center rounded-sm"}
                    onclick={() => requestDelete(wp)}
                    aria-label="Delete waypoint"
                  >
                    <Trash2Icon class="size-3.5" />
                  </Tooltip.Trigger>
                  <Tooltip.Content>Delete waypoint</Tooltip.Content>
                </Tooltip.Root>
              </div>
            </div>
          {/each}
      {/if}
    </div>
  </div>
{/if}

<Dialog.Root bind:open={deleteDialogOpen}>
  <Dialog.Content>
    <Dialog.Header>
      <Dialog.Title>Delete waypoint</Dialog.Title>
      <Dialog.Description>
        {#if pendingDelete}
          Are you sure you want to delete &quot;{pendingDelete.name}&quot;?
          This cannot be undone from this dialog.
        {:else}
          Are you sure you want to delete this waypoint?
        {/if}
      </Dialog.Description>
    </Dialog.Header>
    <Dialog.Footer>
      <Button
        variant="outline"
        size="sm"
        onclick={() => {
          deleteDialogOpen = false;
          pendingDelete = null;
        }}
      >
        Cancel
      </Button>
      <Button variant="destructive" size="sm" onclick={confirmDelete}>
        Delete
      </Button>
    </Dialog.Footer>
  </Dialog.Content>
</Dialog.Root>
