<script lang="ts">
  /**
   * Waypoints tab inside the LibraryRail. Lists every waypoint across all
   * waypoint layers; header carries the active-waypoint-layer `Select`.
   *
   * Per-row affordances: visibility toggle, symbol button (opens
   * `SymbolPicker`), name (double-click rename), a coordinates subline, a
   * "show on map" button, and a `⋯` menu with "only this one", Export WPT
   * and Delete. Clicking a row from a non-active waypoint layer flips the
   * active waypoint layer first (`layers` capability extension).
   *
   * Visibility is per-waypoint, not per-layer: each row stores its own
   * `visible` flag, toggled through `toggle_waypoint_visible`.
   */
  import { get } from "svelte/store";
  import { locale, t } from "$lib/i18n";
  import { layerDisplayName } from "$lib/layer-names";
  import { buttonVariants } from "$lib/components/ui/button";
  import { Label } from "$lib/components/ui/label";
  import * as DropdownMenu from "$lib/components/ui/dropdown-menu";
  import * as Select from "$lib/components/ui/select";
  import * as Tooltip from "$lib/components/ui/tooltip";
  import {
    activeWaypointLayerId,
    addWaypointMode,
    appState,
    drawingModeActive,
    requestWaypointFocus,
    selectedWaypointId,
  } from "$lib/stores";
  import MapPinIcon from "@lucide/svelte/icons/map-pin";
  import SearchIcon from "@lucide/svelte/icons/search";
  import EyeIcon from "@lucide/svelte/icons/eye";
  import EyeOffIcon from "@lucide/svelte/icons/eye-off";
  import LocateIcon from "@lucide/svelte/icons/locate";
  import XIcon from "@lucide/svelte/icons/x";
  import { Input } from "$lib/components/ui/input";
  import { filterByName } from "$lib/name-filter";
  import { createLatestRun } from "$lib/latest-run";
  import { formatCoordinates as formatWaypointCoordinates } from "$lib/track-points";
  import {
    deleteWaypoint,
    exportGpxWaypoints,
    exportWptWaypoints,
    getWaypoints,
    getWaypointsExportDefaultPath,
    renameWaypoint,
    setAllWaypointsVisible,
    setWaypointSymbol,
    showOnlyWaypoint,
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
  // A search area collects the task point, the found object, dangerous spots
  // and every group's marks; past a screenful the list needs a filter for the
  // same reason the track list did.
  let waypointQuery = $state("");
  const visibleRows = $derived(
    filterByName(rows, waypointQuery, (r) => r.wp.name),
  );

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

  /** See `createLatestRun`: an overtaken reload must not put its rows back. */
  const listRuns = createLatestRun();

  async function loadAll(
    layers: { id: number; name: string }[],
  ): Promise<void> {
    const run = listRuns.begin();
    try {
      // Every layer at once. One await per layer meant a project of a dozen
      // import-created layers paid a dozen round trips in a row for a list
      // that is redrawn whole anyway.
      const perLayer = await Promise.all(
        layers.map(async (layer) => ({
          layer,
          wps: await getWaypoints(BigInt(layer.id)),
        })),
      );
      if (!listRuns.isCurrent(run)) return;

      const collected: WaypointRow[] = perLayer.flatMap(({ layer, wps }) =>
        wps.map((wp) => ({
          layerId: BigInt(layer.id),
          layerName: layer.name,
          wp,
        })),
      );
      collected.sort((a, b) => {
        if (a.layerId === b.layerId) return a.wp.id - b.wp.id;
        return a.layerId < b.layerId ? -1 : 1;
      });
      rows = collected;
    } catch (err) {
      if (!listRuns.isCurrent(run)) return;
      console.error("Failed to load waypoints", err);
      toast.error(get(t)("waypointsTab.loadFailed"), {
        description: String(err),
      });
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

  async function handleSetAllVisible(visible: boolean) {
    try {
      await setAllWaypointsVisible(visible);
    } catch (err) {
      toast.error(get(t)("waypointsTab.visibilityFailed"), {
        description: String(err),
      });
    }
  }

  async function handleShowOnly(r: WaypointRow) {
    try {
      await showOnlyWaypoint(r.layerId, BigInt(r.wp.id));
    } catch (err) {
      toast.error(get(t)("waypointsTab.visibilityFailed"), {
        description: String(err),
      });
    }
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
      toast.error(get(t)("waypointsTab.deleteFailed"), {
        description: String(err),
      });
    }
  }

  /**
   * Export one waypoint layer.
   *
   * Two formats, because the receiver decides: GPX is what a phone, a
   * navigator and the other groups' software read; WPT is OziExplorer's own.
   */
  async function handleExportLayer(layerId: bigint, format: "gpx" | "wpt") {
    const defaultPath = await getWaypointsExportDefaultPath(layerId, format);
    const filters =
      format === "gpx"
        ? [{ name: "GPX", extensions: ["gpx"] }]
        : [{ name: "OziExplorer WPT", extensions: ["wpt"] }];
    const path = await open({
      save: true,
      defaultPath: defaultPath ?? `waypoints.${format}`,
      filters,
    } as Parameters<typeof open>[0]);
    if (!path) return;
    try {
      if (format === "gpx") {
        await exportGpxWaypoints(layerId, path as string);
      } else {
        await exportWptWaypoints(layerId, path as string);
      }
    } catch (err) {
      toast.error(get(t)("waypointsTab.exportFailed"), {
        description: String(err),
      });
    }
  }
</script>

<div class="flex h-full min-h-0 flex-col">
  <header class="border-border border-b px-2 py-1.5">
    {#if waypointLayers.length > 0}
      <Label class="text-muted-foreground text-[10px]">
        {$t("waypointsTab.layer")}
      </Label>
      <div class="flex items-center gap-1">
        <Select.Root
          type="single"
          value={waypointLayerSelectValue}
          onValueChange={(v) => v && activeWaypointLayerId.set(BigInt(v))}
        >
          <Select.Trigger
            aria-label={$t("waypointsTab.layer")}
            size="sm"
            class="min-w-0 flex-1"
          >
            {(() => {
              const name = waypointLayers.find(
                (l) => String(l.id) === waypointLayerSelectValue,
              )?.name;
              return name === undefined
                ? $t("waypointsTab.pickLayer")
                : layerDisplayName(name, $locale);
            })()}
          </Select.Trigger>
          <Select.Content>
            {#each waypointLayers as layer (layer.id)}
              <Select.Item
                value={String(layer.id)}
                label={layerDisplayName(layer.name, $locale)}
              >
                {layerDisplayName(layer.name, $locale)}
              </Select.Item>
            {/each}
          </Select.Content>
        </Select.Root>

        <Tooltip.Root>
          <Tooltip.Trigger
            class={buttonVariants({
              variant: $addWaypointMode ? "default" : "ghost",
              size: $addWaypointMode ? "sm" : "icon-sm",
            })}
            aria-label={$addWaypointMode
              ? $t("waypointsTab.addCancel")
              : $t("waypointsTab.add")}
            aria-pressed={$addWaypointMode}
            disabled={$drawingModeActive || $activeWaypointLayerId === null}
            onclick={() => addWaypointMode.update((v) => !v)}
            data-testid="library-add-waypoint"
          >
            <MapPinIcon strokeWidth={1.5} />
            {#if $addWaypointMode}
              <span class="text-xs">{$t("waypointsTab.addCancel")}</span>
            {/if}
          </Tooltip.Trigger>
          <Tooltip.Content>
            {$addWaypointMode
              ? $t("waypointsTab.addCancel")
              : $t("waypointsTab.add")}
          </Tooltip.Content>
        </Tooltip.Root>
      </div>
    {/if}

    {#if rows.length > 0}
      <div class="mt-1.5 flex items-center gap-1">
        <div class="relative min-w-0 flex-1">
          <SearchIcon
            class="text-muted-foreground pointer-events-none absolute top-1/2 left-2 size-3.5 -translate-y-1/2"
            strokeWidth={1.5}
          />
          <Input
            bind:value={waypointQuery}
            class="h-7 pr-7 pl-7 text-xs"
            placeholder={$t("waypointsTab.searchPlaceholder")}
            aria-label={$t("waypointsTab.searchPlaceholder")}
            data-testid="waypoint-search"
          />
          {#if waypointQuery !== ""}
            <button
              type="button"
              class="text-muted-foreground hover:text-foreground absolute top-1/2 right-1 inline-flex size-5 -translate-y-1/2 items-center justify-center rounded-sm border-0 bg-transparent p-0"
              aria-label={$t("waypointsTab.searchClear")}
              onclick={() => (waypointQuery = "")}
              data-testid="waypoint-search-clear"
            >
              <XIcon class="size-3.5" strokeWidth={2} />
            </button>
          {/if}
        </div>
        <Tooltip.Root>
          <Tooltip.Trigger
            class="text-muted-foreground hover:text-foreground inline-flex size-6 shrink-0 items-center justify-center rounded-sm border-0 bg-transparent p-0"
            aria-label={$t("waypointsTab.showAll")}
            onclick={() => handleSetAllVisible(true)}
            data-testid="waypoints-show-all"
          >
            <EyeIcon class="size-3.5" strokeWidth={1.5} />
          </Tooltip.Trigger>
          <Tooltip.Content>{$t("waypointsTab.showAll")}</Tooltip.Content>
        </Tooltip.Root>
        <Tooltip.Root>
          <Tooltip.Trigger
            class="text-muted-foreground hover:text-foreground inline-flex size-6 shrink-0 items-center justify-center rounded-sm border-0 bg-transparent p-0"
            aria-label={$t("waypointsTab.hideAll")}
            onclick={() => handleSetAllVisible(false)}
            data-testid="waypoints-hide-all"
          >
            <EyeOffIcon class="size-3.5" strokeWidth={1.5} />
          </Tooltip.Trigger>
          <Tooltip.Content>{$t("waypointsTab.hideAll")}</Tooltip.Content>
        </Tooltip.Root>
        {#if waypointQuery !== ""}
          <span
            class="text-muted-foreground shrink-0 font-mono text-[10px] tabular-nums"
            data-testid="waypoint-search-count"
          >
            {$t("waypointsTab.searchCount")
              .replace("{shown}", String(visibleRows.length))
              .replace("{total}", String(rows.length))}
          </span>
        {/if}
      </div>
    {/if}
  </header>

  <div class="flex-1 overflow-y-auto py-1" data-testid="waypoints-tab-list">
    {#if rows.length === 0}
      <div class="text-muted-foreground p-3 text-center text-xs">
        {$t("waypointsTab.empty")}
      </div>
    {:else if visibleRows.length === 0}
      <div
        class="text-muted-foreground p-3 text-center text-xs"
        data-testid="waypoint-search-empty"
      >
        {$t("waypointsTab.searchEmpty")}
      </div>
    {:else}
      {#each visibleRows as r (rowKey(r))}
        <LibraryRow
          name={r.wp.name}
          visible={r.wp.visible}
          visibilityLabel={(r.wp.visible
            ? $t("waypointsTab.hide")
            : $t("waypointsTab.show")
          ).replace("{name}", r.wp.name)}
          selected={isSelected(r)}
          onToggleVisibility={() => handleToggleVisible(r)}
          onSelect={() => handleSelectRow(r)}
          onRename={(name) => handleRename(r, name)}
        >
          {#snippet leadingControl()}
            <SymbolPicker
              symbol={r.wp.symbol}
              color={r.wp.color}
              onSelect={(sym) => handleSetSymbol(r, sym)}
            />
          {/snippet}
          {#snippet subline()}
            <span class="text-muted-foreground font-mono text-[10px]">
              {formatWaypointCoordinates(r.wp)}
            </span>
          {/snippet}
          {#snippet trailingControl()}
            <Tooltip.Root>
              <Tooltip.Trigger
                class="text-muted-foreground hover:text-foreground inline-flex size-6 items-center justify-center rounded-sm border-0 bg-transparent p-0"
                aria-label={$t("track.showOnMap")}
                onclick={() => requestWaypointFocus(r.wp.lat, r.wp.lon)}
                data-testid="waypoint-show-on-map"
              >
                <LocateIcon class="size-3.5" strokeWidth={1.5} />
              </Tooltip.Trigger>
              <Tooltip.Content>{$t("track.showOnMap")}</Tooltip.Content>
            </Tooltip.Root>
          {/snippet}
          {#snippet actions()}
            <DropdownMenu.Item onSelect={() => handleShowOnly(r)}>
              {$t("waypointsTab.onlyThis")}
            </DropdownMenu.Item>
            <DropdownMenu.Separator />
            <DropdownMenu.Item
              onSelect={() => handleExportLayer(r.layerId, "gpx")}
            >
              {$t("waypointsTab.exportGpx")}
            </DropdownMenu.Item>
            <DropdownMenu.Item
              onSelect={() => handleExportLayer(r.layerId, "wpt")}
            >
              {$t("waypointsTab.exportWpt")}
            </DropdownMenu.Item>
            <DropdownMenu.Separator />
            <DropdownMenu.Item
              variant="destructive"
              onSelect={() => handleDelete(r)}
            >
              {$t("waypointsTab.delete")}
            </DropdownMenu.Item>
          {/snippet}
        </LibraryRow>
      {/each}
    {/if}
  </div>
</div>
