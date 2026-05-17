<script lang="ts">
  import ExternalLinkIcon from "@lucide/svelte/icons/external-link";
  import FolderOpenIcon from "@lucide/svelte/icons/folder-open";
  import ListIcon from "@lucide/svelte/icons/list";
  import ListOrderedIcon from "@lucide/svelte/icons/list-ordered";
  import MapIcon from "@lucide/svelte/icons/map";
  import MapPinIcon from "@lucide/svelte/icons/map-pin";
  import MapPinPlusIcon from "@lucide/svelte/icons/map-pin-plus";
  import PencilLineIcon from "@lucide/svelte/icons/pencil-line";
  import RedoIcon from "@lucide/svelte/icons/redo-2";
  import SaveIcon from "@lucide/svelte/icons/save";
  import TerminalIcon from "@lucide/svelte/icons/terminal";
  import UndoIcon from "@lucide/svelte/icons/undo-2";
  import UploadIcon from "@lucide/svelte/icons/upload";
  import { Button } from "$lib/components/ui/button";
  import { ScrollArea } from "$lib/components/ui/scroll-area";
  import * as Select from "$lib/components/ui/select";
  import { Separator } from "$lib/components/ui/separator";
  import * as Tooltip from "$lib/components/ui/tooltip";
  import {
    appState,
    status,
    activeMap,
    tracksPanelOpen,
    trackPointsPanelOpen,
    waypointsPanelOpen,
    addWaypointMode,
    drawingModeActive,
    activeTrackLayerId,
    activeWaypointLayerId,
    drawingTrackLayerId,
    drawingTrackId,
    drawingPointCount,
    drawingFinishRequested,
    drawingSegmentId,
    editModeActive,
    consoleOpen,
  } from "$lib/stores";
  import {
    importGpx,
    importPlt,
    saveProject,
    loadProjectFile,
    undo,
    redo,
    revealBundle,
    createEmptyTrack,
    getTrackDetail,
  } from "$lib/api";
  import ThemePicker from "./ThemePicker.svelte";
  import { open, save } from "@tauri-apps/plugin-dialog";
  import { goto } from "$app/navigation";
  import { resolve } from "$app/paths";

  const trackLayers = $derived($appState?.track_layers ?? []);
  const waypointLayers = $derived($appState?.waypoint_layers ?? []);

  const trackLayerSelectValue = $derived(
    $activeTrackLayerId !== null ? $activeTrackLayerId.toString() : "",
  );
  const waypointLayerSelectValue = $derived(
    $activeWaypointLayerId !== null ? $activeWaypointLayerId.toString() : "",
  );

  const statusIsError = $derived(
    $status.toLowerCase().includes("error") ||
      $status.toLowerCase().includes("failed"),
  );

  async function handleImportGpx() {
    const path = await open({
      multiple: false,
      filters: [{ name: "GPX", extensions: ["gpx", "zip"] }],
    });
    if (path) await importGpx(path as string);
  }

  async function handleImportPlt() {
    const path = await open({
      multiple: false,
      filters: [{ name: "PLT track", extensions: ["plt"] }],
    });
    if (path) await importPlt(path as string);
  }

  async function handleSave() {
    const path = await save({
      defaultPath: `${$appState?.project_name ?? "project"}.ozp`,
      filters: [{ name: "OZI Project", extensions: ["ozp"] }],
    });
    if (path) await saveProject(path);
  }

  async function handleOpen() {
    const path = await open({
      multiple: false,
      filters: [{ name: "OZI Project", extensions: ["ozp"] }],
    });
    if (path) await loadProjectFile(path as string);
  }

  async function toggleTrackDrawingMode() {
    if ($drawingModeActive) {
      drawingFinishRequested.set(true);
      return;
    }

    const layerId = $activeTrackLayerId;
    if (layerId === null) return;

    try {
      editModeActive.set(false);
      addWaypointMode.set(false);
      const trackId = await createEmptyTrack(layerId, "New Track");
      const detail = await getTrackDetail(layerId, trackId);
      drawingTrackLayerId.set(layerId);
      drawingTrackId.set(trackId);
      drawingSegmentId.set(BigInt(detail.segments[0].id));
      drawingPointCount.set(0);
      drawingModeActive.set(true);
    } catch (error) {
      console.error("Failed to start track drawing mode", error);
    }
  }
</script>

<aside
  class="bg-card text-card-foreground border-border flex h-full w-56 shrink-0 flex-col overflow-hidden border-r"
>
  <header
    class="bg-popover text-popover-foreground border-border flex items-center justify-between border-b px-2.5 py-2"
  >
    <span class="text-primary text-sm font-semibold">ozi-rs</span>
    <ThemePicker />
  </header>

  <ScrollArea class="flex-1">
    <Tooltip.Provider delayDuration={300}>
      <section class="flex flex-col gap-1.5 px-2 py-2">
        <h2
          class="text-muted-foreground mb-0.5 text-[10px] font-semibold tracking-wider uppercase"
        >
          Project
        </h2>
        <div class="flex flex-wrap gap-1">
          <Button variant="outline" size="xs" onclick={handleOpen}>
            <FolderOpenIcon />
            Open
          </Button>
          <Button variant="outline" size="xs" onclick={handleSave}>
            <SaveIcon />
            Save
          </Button>
          <Tooltip.Root>
            <Tooltip.Trigger>
              {#snippet child({ props })}
                <Button {...props} variant="outline" size="icon-xs" onclick={undo}>
                  <UndoIcon />
                </Button>
              {/snippet}
            </Tooltip.Trigger>
            <Tooltip.Content>Undo</Tooltip.Content>
          </Tooltip.Root>
          <Tooltip.Root>
            <Tooltip.Trigger>
              {#snippet child({ props })}
                <Button {...props} variant="outline" size="icon-xs" onclick={redo}>
                  <RedoIcon />
                </Button>
              {/snippet}
            </Tooltip.Trigger>
            <Tooltip.Content>Redo</Tooltip.Content>
          </Tooltip.Root>
        </div>
      </section>

      <Separator />

      <section class="flex flex-col gap-1.5 px-2 py-2">
        <h2
          class="text-muted-foreground mb-0.5 text-[10px] font-semibold tracking-wider uppercase"
        >
          Map
        </h2>
        <Button
          variant="default"
          size="sm"
          class="w-full justify-start"
          onclick={() => goto(resolve("/"))}
        >
          <MapIcon />
          Maps…
        </Button>

        {#if $activeMap}
          <div class="bg-muted flex flex-col gap-0.5 rounded-md p-1.5">
            <div class="text-muted-foreground text-[10px]">Active</div>
            <div class="text-foreground truncate text-xs font-medium">
              {$activeMap.project_name}
            </div>
            <div class="text-muted-foreground truncate text-[11px]">
              {$activeMap.package_name}
            </div>
            <Button
              variant="outline"
              size="xs"
              class="w-full justify-start"
              onclick={revealBundle}
            >
              <ExternalLinkIcon />
              Reveal in Finder
            </Button>
          </div>
        {/if}
      </section>

      <Separator />

      <section class="flex flex-col gap-1.5 px-2 py-2">
        <h2
          class="text-muted-foreground mb-0.5 text-[10px] font-semibold tracking-wider uppercase"
        >
          Tracks
        </h2>
        {#if trackLayers.length > 0}
          <label class="flex flex-col gap-1">
            <span class="text-muted-foreground text-[10px]">Track layer</span>
            <Select.Root
              type="single"
              value={trackLayerSelectValue}
              onValueChange={(v) =>
                v && activeTrackLayerId.set(BigInt(v))}
              disabled={$drawingModeActive}
            >
              <Select.Trigger aria-label="Track layer" size="sm" class="w-full">
                {trackLayers.find(
                  (l) => String(l.id) === trackLayerSelectValue,
                )?.name ?? "Pick layer"}
              </Select.Trigger>
              <Select.Content>
                {#each trackLayers as layer (layer.id)}
                  <Select.Item value={String(layer.id)} label={layer.name}>
                    {layer.name}
                  </Select.Item>
                {/each}
              </Select.Content>
            </Select.Root>
          </label>
        {/if}
        <div class="flex flex-wrap gap-1">
          <Button variant="outline" size="xs" onclick={handleImportGpx}>
            <UploadIcon />
            GPX
          </Button>
          <Button variant="outline" size="xs" onclick={handleImportPlt}>
            <UploadIcon />
            PLT
          </Button>
        </div>
        <Button
          variant={$drawingModeActive ? "default" : "outline"}
          size="sm"
          class="w-full justify-start"
          onclick={toggleTrackDrawingMode}
          aria-label={$drawingModeActive
            ? "Finish drawing mode and keep the new track."
            : "Create an empty track and click on the map to add points."}
        >
          <PencilLineIcon />
          {$drawingModeActive
            ? `Done (${$drawingPointCount} points)`
            : "Create Track"}
        </Button>
        <Button
          variant={$tracksPanelOpen ? "secondary" : "outline"}
          size="sm"
          class="w-full justify-start"
          onclick={() => tracksPanelOpen.update((v) => !v)}
        >
          <ListIcon />
          {$tracksPanelOpen ? "Hide Tracks" : "Show Tracks"}
        </Button>
        <Button
          variant={$trackPointsPanelOpen ? "secondary" : "outline"}
          size="sm"
          class="w-full justify-start"
          onclick={() => trackPointsPanelOpen.update((v) => !v)}
        >
          <ListOrderedIcon />
          {$trackPointsPanelOpen ? "Hide Points" : "Show Points"}
        </Button>
      </section>

      <Separator />

      <section class="flex flex-col gap-1.5 px-2 py-2">
        <h2
          class="text-muted-foreground mb-0.5 text-[10px] font-semibold tracking-wider uppercase"
        >
          Waypoints
        </h2>
        {#if waypointLayers.length > 0}
          <label class="flex flex-col gap-1">
            <span class="text-muted-foreground text-[10px]">
              Waypoint layer
            </span>
            <Select.Root
              type="single"
              value={waypointLayerSelectValue}
              onValueChange={(v) =>
                v && activeWaypointLayerId.set(BigInt(v))}
            >
              <Select.Trigger
                aria-label="Waypoint layer"
                size="sm"
                class="w-full"
              >
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
          </label>
        {/if}
        <Button
          variant={$waypointsPanelOpen ? "secondary" : "outline"}
          size="sm"
          class="w-full justify-start"
          onclick={() => waypointsPanelOpen.update((v) => !v)}
        >
          <MapPinIcon />
          {$waypointsPanelOpen ? "Hide Waypoints" : "Show Waypoints"}
        </Button>
        <Button
          variant={$addWaypointMode ? "default" : "outline"}
          size="sm"
          class="w-full justify-start"
          disabled={$drawingModeActive}
          onclick={() => addWaypointMode.update((v) => !v)}
          aria-label="Click on the map to place a waypoint. Press Escape to cancel."
        >
          <MapPinPlusIcon />
          {$addWaypointMode ? "Cancel Add Waypoint" : "Add Waypoint"}
        </Button>
      </section>
    </Tooltip.Provider>
  </ScrollArea>

  <div
    class="bg-popover border-border flex min-h-6 items-center justify-between gap-2 border-t px-2 py-1 text-[10px]"
    class:text-destructive={statusIsError}
    class:text-muted-foreground={!statusIsError}
  >
    <span class="flex-1 truncate">{$status}</span>
    <Button
      variant="ghost"
      size="icon-xs"
      onclick={() => consoleOpen.update((v) => !v)}
      aria-label="Toggle console (`)"
    >
      <TerminalIcon />
    </Button>
  </div>
</aside>
